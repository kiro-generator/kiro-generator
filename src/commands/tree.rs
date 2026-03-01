use {
    super::TreeArgs,
    crate::{AgentSourceSlots, Manifest, Result, SourceSlot, generator::Generator},
};

pub(super) fn execute_tree(generator: &Generator, args: &TreeArgs) -> Result<facet_value::Value> {
    let agents: Vec<&AgentSourceSlots> = generator
        .agents
        .values()
        .filter(|a| args.agents.is_empty() || args.agents.iter().any(|n| n == &a.name))
        .collect();

    if agents.is_empty() {
        return Ok(facet_value::Value::from(facet_value::VObject::new()));
    }

    build_json(&agents, generator)
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
        fields.push("resources".to_string());
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
