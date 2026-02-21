use {
    crate::{
        GeneratorConfig,
        KgAgentFileDoc,
        Result,
        commands::SchemaCommand,
        json_schema::{JsonSchema, schema_for},
    },
    color_eyre::eyre::{Context, eyre},
    facet::Facet,
    std::collections::HashMap,
};

pub(crate) const KIRO_OUTPUT_SCHEMA: &str = include_str!("../schemas/kiro-agent.json");
const KG_MAPPING_DELIM: &str = kg_macro::kg_mapping_delim!();

pub(crate) fn build_schema(cmd: &SchemaCommand) -> Result<String> {
    if matches!(*cmd, SchemaCommand::KiroAgent) {
        return Ok(KIRO_OUTPUT_SCHEMA.to_string());
    }
    let mut schema = match cmd {
        SchemaCommand::Manifest => schema_for::<GeneratorConfig>(),
        SchemaCommand::Agent(_) => schema_for::<KgAgentFileDoc>(),
        SchemaCommand::KiroAgent => unreachable!(),
    };
    schema.schema = Some("https://json-schema.org/draft/2020-12/schema".into());
    facet_json::to_string_pretty(&schema).wrap_err(format!("unable to generate schema for {cmd}"))
}

#[derive(Facet, Debug)]
pub(crate) struct Mapping {
    #[facet(default, skip_serializing_if = Option::is_none)]
    kiro_schema_path: Option<String>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    kg_schema_paths: Option<Vec<String>>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    description: Option<String>,
}

type MappingGroups = HashMap<String, HashMap<String, Mapping>>;

pub(crate) fn build_mappings(agent: JsonSchema) -> Result<MappingGroups> {
    let props = agent.properties.ok_or_else(|| eyre!("no props for you"))?;
    let mut kg_to_kiro: HashMap<String, Mapping> = HashMap::with_capacity(props.len());
    for (k, v) in props {
        let key = format!("#/properties/{k}");
        let agent_prop_desc_schema_path = v.description.unwrap_or_default();
        let (description, kiro_schema_path) = agent_prop_desc_schema_path
            .split_once(KG_MAPPING_DELIM)
            .ok_or_else(|| eyre!("failed to find delim for {agent_prop_desc_schema_path}"))?;
        let kiro_schema_path = kiro_schema_path.trim().to_string();
        let description = description.trim().to_string();
        kg_to_kiro.insert(key.clone(), Mapping {
            kiro_schema_path: Some(kiro_schema_path.clone()),
            kg_schema_paths: None,
            description: Some(description.clone()),
        });
    }
    let mut mappings: MappingGroups = HashMap::with_capacity(1);
    mappings.insert("kg_to_kiro".to_string(), kg_to_kiro);
    Ok(mappings)
}

pub(crate) fn handle_schema_mappings() -> Result<()> {
    let sch = schema_for::<KgAgentFileDoc>();
    let result = build_mappings(sch)?;
    let json_str = facet_json::to_string_pretty(&result)?;
    println!("{json_str}");
    Ok(())
}

pub(crate) fn handle_schema_command(cmd: &SchemaCommand) -> Result<()> {
    let output = build_schema(cmd)?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{commands::SchemaAgentArgs, json_schema::schema_for},
        color_eyre::eyre::eyre,
    };

    #[test]
    fn test_mappings() -> Result<()> {
        let sch = schema_for::<KgAgentFileDoc>();
        let result = build_mappings(sch)?;
        let json_str = facet_json::to_string_pretty(&result)?;
        assert!(json_str.contains("kg_to_kiro"));
        Ok(())
    }

    #[test]
    fn test_manifest_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Manifest)?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema_str).wrap_err("manifest schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("manifest schema failed meta-validation: {e}"))?;
        Ok(())
    }

    #[test]
    fn test_agent_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Agent(SchemaAgentArgs::default()))?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema_str).wrap_err("agent schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("agent schema failed meta-validation: {e}"))?;
        Ok(())
    }
}
