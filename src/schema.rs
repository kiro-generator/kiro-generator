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
