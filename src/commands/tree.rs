use {
    super::TreeArgs,
    crate::{
        AgentSourceSlots,
        Manifest,
        Result,
        SourceSlot,
        generator::Generator,
        output::OutputFormatArg,
    },
    std::collections::{BTreeMap, HashMap, HashSet},
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
};

/// Main entry point — dispatches based on args.
pub(super) fn execute_tree(generator: &Generator, args: &TreeArgs) -> Result<()> {
    if args.invert {
        if args.agents.is_empty() {
            invert_all(generator, args)
        } else {
            invert_named(generator, args)
        }
    } else if args.agents.is_empty() {
        summary(generator, args)
    } else {
        detail(generator, args)
    }
}

// ---------------------------------------------------------------------------
// kg tree (no args) — summary table
// ---------------------------------------------------------------------------

fn summary(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let mut agents: Vec<&AgentSourceSlots> = generator
        .agents
        .values()
        .filter(|a| a.name != "kg-helper")
        .collect();
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    let (concrete, templates): (Vec<_>, Vec<_>) = agents.iter().partition(|a| !a.merged.template);

    match args.format {
        OutputFormatArg::Json => {
            let mut out = serde_json::Map::new();
            let to_arr = |list: &[&&AgentSourceSlots]| -> serde_json::Value {
                list.iter()
                    .map(|a| {
                        let mut m = serde_json::Map::new();
                        m.insert("name".into(), serde_json::Value::String(a.name.clone()));
                        m.insert(
                            "description".into(),
                            a.merged
                                .description
                                .as_ref()
                                .map(|d| serde_json::Value::String(d.clone()))
                                .unwrap_or(serde_json::Value::Null),
                        );
                        let inherits: Vec<serde_json::Value> = a
                            .merged
                            .inherits
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect();
                        m.insert("inherits".into(), serde_json::Value::Array(inherits));
                        serde_json::Value::Object(m)
                    })
                    .collect()
            };
            out.insert("agents".into(), to_arr(&concrete));
            if !args.no_templates {
                out.insert("templates".into(), to_arr(&templates));
            }
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Agent"),
                    Cell::new("Description"),
                    Cell::new("Inherits"),
                ]);

            for a in &concrete {
                table.add_row(summary_row(a));
            }

            if !args.no_templates && !templates.is_empty() {
                table.add_row(vec![Cell::new(""), Cell::new(""), Cell::new("")]);
                table.add_row(vec![
                    Cell::new("Templates:").fg(super_table::Color::DarkGrey),
                    Cell::new(""),
                    Cell::new(""),
                ]);
                for a in &templates {
                    table.add_row(summary_row(a));
                }
            }

            println!("{table}");
        }
    }
    Ok(())
}

fn summary_row(a: &AgentSourceSlots) -> Vec<Cell> {
    let mut inherits: Vec<&str> = a.merged.inherits.iter().map(String::as_str).collect();
    inherits.sort();
    vec![
        Cell::new(&a.name),
        Cell::new(a.merged.description.as_deref().unwrap_or("")),
        Cell::new(inherits.join(", ")),
    ]
}

// ---------------------------------------------------------------------------
// kg tree <name> — full detail (unchanged from original behavior)
// ---------------------------------------------------------------------------

fn detail(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let agents: Vec<&AgentSourceSlots> = generator
        .agents
        .values()
        .filter(|a| a.name != "kg-helper")
        .filter(|a| args.agents.iter().any(|n| n == &a.name))
        .collect();

    if agents.is_empty() {
        if matches!(args.format, OutputFormatArg::Json) {
            let empty = facet_value::Value::from(facet_value::VObject::new());
            println!("{}", facet_json::to_string_pretty(&empty)?);
        }
        return Ok(());
    }

    let value = build_json(&agents, generator)?;
    println!("{}", facet_json::to_string_pretty(&value)?);
    Ok(())
}

fn build_json(agents: &[&AgentSourceSlots], generator: &Generator) -> Result<facet_value::Value> {
    let mut obj = facet_value::VObject::new();
    for agent_slots in agents {
        let name = &agent_slots.name;
        let manifest = &agent_slots.merged;
        let mut agent = facet_value::VObject::new();
        agent.insert("template", facet_value::Value::from(manifest.template));
        if !manifest.template {
            let output = generator.destination_dir(name).join(format!("{name}.json"));
            agent.insert(
                "output",
                facet_value::Value::from(output.to_string_lossy().as_ref()),
            );
        }
        if let Some(ref desc) = manifest.description {
            agent.insert("description", facet_value::Value::from(desc.as_str()));
        }
        let src_arr = sources_to_json(agent_slots);
        agent.insert("sources", facet_value::Value::from(src_arr));
        let inherits: facet_value::VArray = manifest
            .inherits
            .iter()
            .map(|s| facet_value::Value::from(s.as_str()))
            .collect();
        agent.insert("inherits", facet_value::Value::from(inherits));
        if let Ok(chain) = generator.inheritance_chain(name) {
            let chain_arr: facet_value::VArray = chain
                .iter()
                .map(|s| facet_value::Value::from(s.as_str()))
                .collect();
            agent.insert("resolved_chain", facet_value::Value::from(chain_arr));
        }
        obj.insert(name.as_str(), facet_value::Value::from(agent));
    }
    Ok(facet_value::Value::from(obj))
}

fn sources_to_json(agent_slots: &AgentSourceSlots) -> facet_value::VArray {
    let mut sources = facet_value::VArray::new();
    push_source(&mut sources, &agent_slots.local_agent_file);
    push_source(&mut sources, &agent_slots.local_manifest);
    push_source(&mut sources, &agent_slots.global_manifest);
    push_source(&mut sources, &agent_slots.global_agent_file);
    sources
}

fn push_source(sources: &mut facet_value::VArray, slot: &SourceSlot) {
    let Some(path) = &slot.path else {
        return;
    };
    let mut o = facet_value::VObject::new();
    o.insert("type", slot.source_type().unwrap_or_default());
    o.insert(
        "path",
        facet_value::Value::from(path.path().to_string_lossy().as_ref()),
    );
    let fields_arr: facet_value::VArray = manifest_fields(&slot.manifest)
        .into_iter()
        .map(facet_value::Value::from)
        .collect();
    o.insert("modified_fields", facet_value::Value::from(fields_arr));
    sources.push(facet_value::Value::from(o));
}

// ---------------------------------------------------------------------------
// kg tree --invert <name> — reverse dependency chain
// ---------------------------------------------------------------------------

fn invert_named(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let reverse = build_reverse_map(generator);

    match args.format {
        OutputFormatArg::Json => {
            let out = build_invert_named_json(&reverse, &args.agents);
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            for target in &args.agents {
                let paths = find_dependents(&reverse, target);
                println!("{target}");
                for path in &paths {
                    let chain = path[1..].join(" -> ");
                    let label = if path.len() == 2 { " (direct)" } else { "" };
                    println!("  -> {chain}{label}");
                }
                if paths.is_empty() {
                    println!("  (no dependents)");
                }
            }
        }
    }
    Ok(())
}

fn build_invert_named_json<'a>(
    reverse: &HashMap<&'a str, Vec<&'a str>>,
    targets: &[String],
) -> serde_json::Map<String, serde_json::Value> {
    let mut out = serde_json::Map::new();
    for target in targets {
        let paths = find_dependents(reverse, target);
        let dependents: Vec<serde_json::Value> = paths
            .iter()
            .map(|path| {
                let mut m = serde_json::Map::new();
                m.insert(
                    "agent".into(),
                    serde_json::Value::String(path.last().unwrap().clone()),
                );
                m.insert(
                    "path".into(),
                    path.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                );
                serde_json::Value::Object(m)
            })
            .collect();

        let mut inner = serde_json::Map::new();
        inner.insert("dependents".into(), serde_json::Value::Array(dependents));
        out.insert(target.clone(), serde_json::Value::Object(inner));
    }
    out
}

// ---------------------------------------------------------------------------
// kg tree --invert (no name) — used/unused template report
// ---------------------------------------------------------------------------

fn invert_all(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let reverse = build_reverse_map(generator);

    // Count unique transitive dependents for each agent/template.
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for name in generator.agents.keys() {
        if name == "kg-helper" {
            continue;
        }
        let dependents = find_unique_dependents(&reverse, name);
        counts.insert(name, dependents.len());
    }

    let templates: Vec<(&str, usize)> = generator
        .agents
        .values()
        .filter(|a| a.merged.template && a.name != "kg-helper")
        .map(|a| (a.name.as_str(), *counts.get(a.name.as_str()).unwrap_or(&0)))
        .collect();

    let (mut used, mut unused): (Vec<_>, Vec<_>) = templates.into_iter().partition(|(_, c)| *c > 0);
    used.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    unused.sort_by(|a, b| a.0.cmp(b.0));

    match args.format {
        OutputFormatArg::Json => {
            let used_json: Vec<serde_json::Value> = used
                .iter()
                .map(|(name, count)| {
                    let mut m = serde_json::Map::new();
                    m.insert("name".into(), serde_json::Value::String((*name).into()));
                    m.insert(
                        "dependent_count".into(),
                        serde_json::Value::Number((*count).into()),
                    );
                    serde_json::Value::Object(m)
                })
                .collect();
            let unused_json: Vec<serde_json::Value> = unused
                .iter()
                .map(|(name, _)| serde_json::Value::String((*name).into()))
                .collect();
            let mut out = serde_json::Map::new();
            out.insert("used".into(), serde_json::Value::Array(used_json));
            out.insert("unused".into(), serde_json::Value::Array(unused_json));
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        _ => {
            if !used.is_empty() {
                println!("Used templates:");
                for (name, count) in &used {
                    println!(
                        "  {:<20} used by {} agent{}",
                        name,
                        count,
                        if *count == 1 { "" } else { "s" }
                    );
                }
            }
            if !unused.is_empty() {
                if !used.is_empty() {
                    println!();
                }
                println!("Unused templates:");
                for (name, _) in &unused {
                    println!("  {name}");
                }
            }
            if used.is_empty() && unused.is_empty() {
                println!("No templates defined");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers — reverse dependency graph
// ---------------------------------------------------------------------------

/// Build a map of parent_name -> Vec<child_name> (direct inheritors only).
fn build_reverse_map(generator: &Generator) -> HashMap<&str, Vec<&str>> {
    let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, slots) in &generator.agents {
        if name == "kg-helper" {
            continue;
        }
        for parent in &slots.merged.inherits {
            reverse
                .entry(parent.as_str())
                .or_default()
                .push(name.as_str());
        }
    }
    // Sort children for deterministic output
    for children in reverse.values_mut() {
        children.sort();
    }
    reverse
}

/// Find all transitive dependents of `target`, returning each as a full path
/// from target to leaf.
fn find_dependents<'a>(
    reverse: &HashMap<&'a str, Vec<&'a str>>,
    target: &'a str,
) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    let mut stack: Vec<Vec<&str>> = vec![vec![target]];

    while let Some(path) = stack.pop() {
        let current = *path.last().unwrap();
        if let Some(children) = reverse.get(current) {
            for child in children {
                if path.contains(child) {
                    tracing::warn!(
                        target = %target,
                        current = %current,
                        child = %child,
                        chain = ?path,
                        "Cycle detected in inheritance graph while computing reverse dependencies; skipping edge"
                    );
                    continue;
                }
                let mut new_path = path.clone();
                new_path.push(child);
                // Record this path as a dependent
                results.push(new_path.iter().map(|s| (*s).to_string()).collect());
                // Continue DFS
                stack.push({
                    let mut p = path.clone();
                    p.push(child);
                    p
                });
            }
        }
    }

    results.sort();
    results
}

/// Find all unique transitive dependents of `target`.
fn find_unique_dependents<'a>(
    reverse: &HashMap<&'a str, Vec<&'a str>>,
    target: &'a str,
) -> HashSet<&'a str> {
    let mut seen = HashSet::new();
    let mut stack = vec![target];

    while let Some(current) = stack.pop() {
        if let Some(children) = reverse.get(current) {
            for &child in children {
                if seen.insert(child) {
                    stack.push(child);
                }
            }
        }
    }

    seen.remove(target);
    seen
}

fn manifest_fields(manifest: &Manifest) -> Vec<String> {
    let mut fields = Vec::new();
    if manifest.template {
        fields.push("template".to_string());
    }
    if manifest.description.is_some() {
        fields.push("description".to_string());
    }
    if !manifest.inherits.is_empty() {
        fields.push("inherits".to_string());
    }
    if manifest.prompt.is_some() {
        fields.push("prompt".to_string());
    }
    if !manifest.resources.is_empty() {
        for k in manifest.resources.keys() {
            fields.push(format!("resources.{k}"));
        }
    }
    if !manifest.skills.is_empty() {
        for k in manifest.skills.keys() {
            fields.push(format!("skills.{k}"));
        }
    }
    if !manifest.knowledge.is_empty() {
        for k in manifest.knowledge.keys() {
            fields.push(format!("knowledge.{k}"));
        }
    }
    if manifest.include_mcp_json.is_some() {
        fields.push("useLegacyMcpJson".to_string());
    }
    if !manifest.tools.is_empty() {
        fields.push("tools".to_string());
    }
    if !manifest.allowed_tools.is_empty() {
        fields.push("allowedTools".to_string());
    }
    if manifest.model.is_some() {
        fields.push("model".to_string());
    }
    if !manifest.hooks.is_empty() {
        for k in manifest.hooks.keys() {
            fields.push(format!("hooks.{k}"));
        }
    }
    if !manifest.mcp_servers.is_empty() {
        for k in manifest.mcp_servers.keys() {
            fields.push(format!("mcpServers.{k}"));
        }
    }
    if !manifest.tool_aliases.is_empty() {
        for k in manifest.tool_aliases.keys() {
            fields.push(format!("toolAliases.{k}"));
        }
    }
    if manifest.native_tools != Default::default() {
        if manifest.native_tools.shell != Default::default() {
            fields.push("nativeTools.shell".to_string());
        }

        if manifest.native_tools.aws != Default::default() {
            fields.push("nativeTools.aws".to_string());
        }

        if manifest.native_tools.read != Default::default() {
            fields.push("nativeTools.read".to_string());
        }

        if manifest.native_tools.write != Default::default() {
            fields.push("nativeTools.write".to_string());
        }

        if manifest.native_tools.web_fetch != Default::default() {
            fields.push("nativeTools.web_fetch".to_string());
        }

        if manifest.native_tools.glob != Default::default() {
            fields.push("nativeTools.glob".to_string());
        }
        if manifest.native_tools.grep != Default::default() {
            fields.push("nativeTools.grep".to_string());
        }
    }
    if !manifest.tool_settings.is_empty() {
        for k in manifest.tool_settings.keys() {
            fields.push(format!("toolSettings.{k}"));
        }
    }
    if manifest.keyboard_shortcut.is_some() {
        fields.push("keyboardShortcut".to_string());
    }
    if manifest.welcome_message.is_some() {
        fields.push("welcomeMessage".to_string());
    }
    if manifest.subagents != Default::default() {
        fields.push("subagents".to_string());
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn unique_dependents_dedupes_diamond_paths() -> Result<()> {
        let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
        reverse.insert("base", vec!["a", "b"]);
        reverse.insert("a", vec!["c"]);
        reverse.insert("b", vec!["c"]);

        let dependents = find_unique_dependents(&reverse, "base");
        assert_eq!(dependents.len(), 3);
        assert!(dependents.contains("a"));
        assert!(dependents.contains("b"));
        assert!(dependents.contains("c"));
        Ok(())
    }

    #[test_log::test]
    fn unique_dependents_cycle_does_not_include_target() -> Result<()> {
        let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
        reverse.insert("base", vec!["a"]);
        reverse.insert("a", vec!["base"]);

        let dependents = find_unique_dependents(&reverse, "base");
        assert_eq!(dependents.len(), 1);
        assert!(dependents.contains("a"));
        assert!(!dependents.contains("base"));
        Ok(())
    }

    #[test_log::test]
    fn find_dependents_skips_cycle_edges() -> Result<()> {
        let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
        reverse.insert("base", vec!["a"]);
        reverse.insert("a", vec!["base", "c"]);

        let paths = find_dependents(&reverse, "base");
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&vec!["base".into(), "a".into()]));
        assert!(paths.contains(&vec!["base".into(), "a".into(), "c".into()]));
        Ok(())
    }

    #[test_log::test]
    fn invert_named_json_combines_all_targets_into_one_object() -> Result<()> {
        let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
        reverse.insert("base", vec!["a"]);
        reverse.insert("template", vec!["b"]);

        let out = build_invert_named_json(&reverse, &["base".into(), "template".into()]);
        assert_eq!(out.len(), 2);
        assert!(out.contains_key("base"));
        assert!(out.contains_key("template"));
        Ok(())
    }
}
