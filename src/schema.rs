use {
    crate::{GeneratorConfig, KgAgentFileDoc, Result, commands::SchemaCommand},
    color_eyre::eyre::Context,
};

pub(crate) const KIRO_OUTPUT_SCHEMA: &str = include_str!("../schemas/kiro-agent.json");

pub(crate) fn build_schema(cmd: &SchemaCommand) -> Result<String> {
    if matches!(*cmd, SchemaCommand::KiroAgent) {
        return Ok(KIRO_OUTPUT_SCHEMA.to_string());
    }
    let mut schema = match cmd {
        SchemaCommand::Manifest => crate::json_schema::schema_for::<GeneratorConfig>(),
        SchemaCommand::Agent => crate::json_schema::schema_for::<KgAgentFileDoc>(),
        SchemaCommand::KiroAgent => unreachable!(),
    };
    schema.schema = Some("https://json-schema.org/draft/2020-12/schema".into());
    facet_json::to_string_pretty(&schema).wrap_err(format!("unable to generate schema for {cmd}"))
}

pub(crate) fn handle_schema_command(cmd: &SchemaCommand) -> Result<()> {
    let output = build_schema(cmd)?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, color_eyre::eyre::eyre};

    #[test]
    fn test_manifest_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Manifest)?;
        let schema: serde_json::Value = serde_json::from_str(&schema_str)
            .wrap_err("manifest schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("manifest schema failed meta-validation: {e}"))?;
        Ok(())
    }

    #[test]
    fn test_agent_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Agent)?;
        let schema: serde_json::Value = serde_json::from_str(&schema_str)
            .wrap_err("agent schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("agent schema failed meta-validation: {e}"))?;
        Ok(())
    }
}
