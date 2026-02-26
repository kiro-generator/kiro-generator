use {
    super::TreeArgs,
    crate::{Manifest, Result, generator::Generator, source::KgSources},
};

pub(super) fn execute_tree(generator: &Generator, args: &TreeArgs) -> Result<facet_value::Value> {
    let resolved = &generator.resolved;

    let agents: Vec<(&String, &Manifest)> = resolved
        .agents
        .iter()
        .filter(|(name, _)| args.agents.is_empty() || args.agents.iter().any(|a| a == *name))
        .collect();

    if agents.is_empty() {
        return Ok(facet_value::Value::from(facet_value::VObject::new()));
    }

    build_json(&agents, &resolved.sources, generator)
}

fn build_json(
    agents: &[(&String, &Manifest)],
    sources: &KgSources,
    generator: &Generator,
) -> Result<facet_value::Value> {
    let mut obj = facet_value::VObject::new();
    for (name, manifest) in agents {
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
        let src_arr: facet_value::VArray = match sources.get(name.as_str()) {
            Some(s) => s
                .iter()
                .map(|s| {
                    let mut o = facet_value::VObject::new();
                    o.insert("type", facet_value::Value::from(s.source_type()));
                    o.insert(
                        "path",
                        facet_value::Value::from(s.path().to_string_lossy().as_ref()),
                    );
                    facet_value::Value::from(o)
                })
                .collect(),
            None => facet_value::VArray::new(),
        };
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
