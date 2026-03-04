use {
    super::TreeArgs,
    crate::{AgentSourceSlots, Manifest, Result, generator::Generator, output::OutputFormatArg},
    facet::Facet,
    std::collections::{BTreeSet, HashMap, HashSet},
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
};

#[derive(Facet)]
struct TreeSummary {
    name: String,
    description: String,
    inherits: BTreeSet<String>,
}

#[derive(Facet)]
struct TreeDependent {
    agent: String,
    chain: Vec<String>,
}

#[derive(Facet)]
struct TreeInvert {
    dependents: Vec<TreeDependent>,
}

#[derive(Facet)]
struct TreeSource {
    source_type: String,
    path: String,
    modified_fields: BTreeSet<String>,
}

#[derive(Facet)]
struct TreeDetail {
    template: bool,
    output: String,
    description: String,
    inherits: BTreeSet<String>,
    resolved_chain: Vec<String>,
    sources: Vec<TreeSource>,
}

#[derive(Facet)]
struct TemplateUsage {
    name: String,
    dependent_count: usize,
}

#[derive(Facet)]
struct TemplateReport {
    used: Vec<TemplateUsage>,
    unused: Vec<String>,
}
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
    let to_summary = |a: &AgentSourceSlots| TreeSummary {
        name: a.name.clone(),
        description: a.merged.description.clone().unwrap_or_default(),
        inherits: a.merged.inherits.iter().cloned().collect(),
    };

    let agents: Vec<TreeSummary> = generator.agents.values().map(to_summary).collect();

    let mut concrete: HashMap<String, TreeSummary> = HashMap::new();
    let mut templates: HashMap<String, TreeSummary> = HashMap::new();

    for agent in agents {
        let is_template = generator.agents[&agent.name].merged.template;
        if is_template {
            templates.insert(agent.name.clone(), agent);
        } else {
            concrete.insert(agent.name.clone(), agent);
        }
    }

    match args.format {
        OutputFormatArg::Json => {
            let mut out = HashMap::new();
            out.insert("agents", concrete);
            if !args.no_templates {
                out.insert("templates", templates);
            }
            println!("{}", facet_json::to_string_pretty(&out)?);
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

            let mut concrete_sorted: Vec<_> = concrete.values().collect();
            concrete_sorted.sort_by(|a, b| a.name.cmp(&b.name));
            for a in concrete_sorted {
                table.add_row(summary_row(a));
            }

            if !args.no_templates && !templates.is_empty() {
                table.add_row(vec![Cell::new(""), Cell::new(""), Cell::new("")]);
                table.add_row(vec![
                    Cell::new("Templates:").fg(super_table::Color::DarkGrey),
                    Cell::new(""),
                    Cell::new(""),
                ]);
                let mut templates_sorted: Vec<_> = templates.values().collect();
                templates_sorted.sort_by(|a, b| a.name.cmp(&b.name));
                for a in templates_sorted {
                    table.add_row(summary_row(a));
                }
            }

            println!("{table}");
        }
    }
    Ok(())
}

fn summary_row(a: &TreeSummary) -> Vec<Cell> {
    let mut inherits: Vec<&str> = a.inherits.iter().map(String::as_str).collect();
    inherits.sort();
    vec![
        Cell::new(&a.name),
        Cell::new(&a.description),
        Cell::new(inherits.join(", ")),
    ]
}

// ---------------------------------------------------------------------------
// kg tree <name> — full detail
// ---------------------------------------------------------------------------

fn detail(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let mut out: HashMap<String, TreeDetail> = HashMap::new();

    for slots in generator.agents.values() {
        if !args.agents.iter().any(|n| n == &slots.name) {
            continue;
        }
        let manifest = &slots.merged;
        let output = if manifest.template {
            String::new()
        } else {
            generator
                .destination_dir(&slots.name)
                .join(format!("{}.json", slots.name))
                .to_string_lossy()
                .into_owned()
        };
        let resolved_chain = generator.inheritance_chain(&slots.name).unwrap_or_default();

        let sources = collect_sources(slots);

        out.insert(slots.name.clone(), TreeDetail {
            template: manifest.template,
            output,
            description: manifest.description.clone().unwrap_or_default(),
            inherits: manifest.inherits.iter().cloned().collect(),
            resolved_chain,
            sources,
        });
    }

    if out.is_empty() {
        if matches!(args.format, OutputFormatArg::Json) {
            println!("{}", facet_json::to_string_pretty(&out)?);
        }
        return Ok(());
    }

    println!("{}", facet_json::to_string_pretty(&out)?);
    Ok(())
}

fn collect_sources(slots: &AgentSourceSlots) -> Vec<TreeSource> {
    [
        &slots.local_agent_file,
        &slots.local_manifest,
        &slots.global_manifest,
        &slots.global_agent_file,
    ]
    .into_iter()
    .filter_map(|slot| {
        let src = slot.path.as_ref()?;
        Some(TreeSource {
            source_type: src.source_type().to_string(),
            path: src.path().to_string_lossy().into_owned(),
            modified_fields: manifest_fields(&slot.manifest).into_iter().collect(),
        })
    })
    .collect()
}

// ---------------------------------------------------------------------------
// kg tree --invert <name> — reverse dependency chain
// ---------------------------------------------------------------------------

fn invert_named(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let reverse = build_reverse_map(generator);

    match args.format {
        OutputFormatArg::Json => {
            let out = build_invert_named(&reverse, &args.agents);
            println!("{}", facet_json::to_string_pretty(&out)?);
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

fn build_invert_named<'a>(
    reverse: &HashMap<&'a str, Vec<&'a str>>,
    targets: &[String],
) -> HashMap<String, TreeInvert> {
    let mut out = HashMap::new();
    for target in targets {
        let paths = find_dependents(reverse, target);
        let dependents = paths
            .into_iter()
            .map(|path| TreeDependent {
                agent: path
                    .last()
                    .expect("path is non-empty by construction")
                    .clone(),
                chain: path,
            })
            .collect();
        out.insert(target.clone(), TreeInvert { dependents });
    }
    out
}

// ---------------------------------------------------------------------------
// kg tree --invert (no name) — used/unused template report
// ---------------------------------------------------------------------------

fn invert_all(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let reverse = build_reverse_map(generator);

    let mut used = Vec::new();
    let mut unused = Vec::new();

    for a in generator.agents.values().filter(|a| a.merged.template) {
        let count = find_unique_dependents(&reverse, &a.name).len();
        if count > 0 {
            used.push(TemplateUsage {
                name: a.name.clone(),
                dependent_count: count,
            });
        } else {
            unused.push(a.name.clone());
        }
    }

    used.sort_by(|a, b| {
        b.dependent_count
            .cmp(&a.dependent_count)
            .then(a.name.cmp(&b.name))
    });
    unused.sort();

    let report = TemplateReport { used, unused };

    match args.format {
        OutputFormatArg::Json => {
            println!("{}", facet_json::to_string_pretty(&report)?);
        }
        _ => {
            if !report.used.is_empty() {
                println!("Used templates:");
                for t in &report.used {
                    println!(
                        "  {:<20} used by {} agent{}",
                        t.name,
                        t.dependent_count,
                        if t.dependent_count == 1 { "" } else { "s" }
                    );
                }
            }
            if !report.unused.is_empty() {
                if !report.used.is_empty() {
                    println!();
                }
                println!("Unused templates:");
                for name in &report.unused {
                    println!("  {name}");
                }
            }
            if report.used.is_empty() && report.unused.is_empty() {
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
                stack.push(new_path);
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
    use {
        super::*,
        crate::{GeneratorConfig, os::Fs, source::KgAgentSource, toml_parse},
        std::path::PathBuf,
    };

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

        let out = build_invert_named(&reverse, &["base".into(), "template".into()]);
        assert_eq!(out.len(), 2);
        assert!(out.contains_key("base"));
        assert!(out.contains_key("template"));
        Ok(())
    }

    #[test_log::test]
    fn test_invert_all_with_fixtures() -> Result<()> {
        let raw = include_str!("../../fixtures/manifest-test/test-merge-agent.toml");
        let fs = Fs::new();
        let mut generator = Generator::new(
            fs,
            crate::ConfigLocation::Local,
            crate::output::OutputFormat::Json,
        )?;
        let agents: GeneratorConfig = toml_parse(raw)?;
        let agents = agents.populate_names();
        generator.agents = agents
            .agents
            .iter()
            .map(|(k, v)| {
                (k.clone(), AgentSourceSlots {
                    name: k.clone(),
                    merged: v.clone(),
                    global_manifest: Default::default(),
                    local_manifest: crate::SourceSlot {
                        path: Some(KgAgentSource::LocalManifest(PathBuf::new().join("test"))),
                        manifest: v.clone(),
                    },
                    global_agent_file: Default::default(),
                    local_agent_file: Default::default(),
                })
            })
            .collect();
        let args = TreeArgs {
            agents: vec![],
            invert: true,
            no_templates: false,
            format: OutputFormatArg::Json,
            trace: None,
        };
        execute_tree(&generator, &args)?;
        Ok(())
    }
}
