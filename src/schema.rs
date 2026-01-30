use {
    crate::{GeneratorConfig, KgAgentFileDoc, Result, commands::SchemaCommand},
    color_eyre::eyre::Context,
};

pub(crate) const SCHEMA: &str = include_str!("../schemas/kiro-agent.json");

fn get_schema_description(cmd: &SchemaCommand) -> &'static str {
    match cmd {
        SchemaCommand::Manifest => "Schema for kiro-generator (kg) manifest TOML files",
        SchemaCommand::Agent => "Schema for kiro-generator (kg) agent TOML files",
    }
}

fn build_schema(cmd: &SchemaCommand) -> Result<String> {
    let mut output = match cmd {
        SchemaCommand::Manifest => facet_json_schema::schema_for::<GeneratorConfig>(),
        SchemaCommand::Agent => facet_json_schema::schema_for::<KgAgentFileDoc>(),
    };
    output.description = Some(get_schema_description(cmd).into());
    output.schema = Some("https://json-schema.org/draft/2020-12/schema".into());
    facet_json::to_string_pretty(&output).wrap_err(format!("unable to generate schema for {cmd}"))
}

pub(crate) fn handle_schema_command(cmd: &SchemaCommand) -> Result<()> {
    let output = build_schema(cmd)?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_schema_description() -> Result<()> {
        let desc = build_schema(&SchemaCommand::Manifest)?;
        assert!(desc.contains("manifest TOML files"));
        Ok(())
    }

    #[test]
    fn test_agent_schema_description() -> Result<()> {
        let desc = build_schema(&SchemaCommand::Agent)?;
        assert!(desc.contains("agent TOML files"));
        Ok(())
    }
}
