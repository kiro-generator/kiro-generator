use {
    crate::{
        KgAgentFileDoc,
        Result,
        commands::SchemaCommand,
        json_schema::{AdditionalProperties, JsonSchema, schema_for},
    },
    color_eyre::eyre::{Context, eyre},
    facet::Facet,
    std::collections::BTreeMap,
};

pub(crate) const KIRO_OUTPUT_SCHEMA: &str = include_str!("../schemas/kiro-agent.json");
const JSON_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
const KG_MAPPING_DELIM: &str = kg_macro::kg_mapping_delim!();

/// The Manifest and KgAgentFileDoc schemas are mostly the same. Manifest adds
/// `name`, `inherits`, and `template`.
const MANIFEST_PARTIAL_SCHEMA: &str = r#"
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "$schema": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "null"
        }
      ]
    },
    "agents": {
      "type": "object",
      "additionalProperties": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Agent name (derived from filename if not specified)"
          },
          "inherits": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "List of parent agents to inherit configuration from"
          },
          "template": {
            "type": "boolean",
            "description": "Whether this agent is a template. Templates are not written to disk\n and serve only as parent configurations for other agents to inherit\n from. Template status is NEVER inherited - it must be explicitly\n declared."
          }
        },
        "additionalProperties": false,
        "title": "Manifest"
      }
    }
  },
  "additionalProperties": false,
  "description": "Agent manifests which control inheritance",
  "title": "Agent Manifest"
}

"#;

/// The Manifest schema is built manually because the `kg_mapping` macro adds
/// Rust docs for `build_mappings`. JsonSchema injects Rust docs into the
/// `description` field of JSON Schema, so this keeps the description in
/// KgAgentFileDoc in sync with Manifest.
#[tracing::instrument]
fn build_manifest_schema() -> Result<JsonSchema> {
    let agent = schema_for::<KgAgentFileDoc>();
    let agent_props = agent
        .properties
        .ok_or_else(|| eyre!("KgAgentFileDoc schema is missing properties"))?;

    let mut manifest: JsonSchema = facet_json::from_str(MANIFEST_PARTIAL_SCHEMA)
        .wrap_err("failed to parse MANIFEST_PARTIAL_SCHEMA")?;
    let mut manifest_props = manifest
        .properties
        .take()
        .ok_or_else(|| eyre!("manifest schema is missing properties"))?;
    let agents_prop = manifest_props
        .get_mut("agents")
        .ok_or_else(|| eyre!("manifest schema is missing the agents property"))?;
    let additional_props = agents_prop
        .additional_properties
        .as_mut()
        .ok_or_else(|| eyre!("manifest schema is missing agents.additionalProperties"))?;

    let merged_props = match additional_props {
        AdditionalProperties::Bool(_) => {
            return Err(eyre!(
                "manifest schema expects agents.additionalProperties to be a schema"
            ));
        }
        AdditionalProperties::Schema(schema) => {
            schema.properties.get_or_insert_with(Default::default)
        }
    };

    merged_props.extend(agent_props);
    manifest.properties = Some(manifest_props);
    Ok(manifest)
}

#[tracing::instrument(skip(cmd), fields(command = %cmd))]
pub(crate) fn build_schema(cmd: &SchemaCommand) -> Result<String> {
    let mut schema = match cmd {
        SchemaCommand::Manifest => build_manifest_schema()?,
        SchemaCommand::Agent(_) => schema_for::<KgAgentFileDoc>(),
        SchemaCommand::KiroAgent => return Ok(KIRO_OUTPUT_SCHEMA.to_string()),
    };
    schema.schema = Some(JSON_SCHEMA_DIALECT.into());
    let schema_str = facet_json::to_string_pretty(&schema)
        .wrap_err_with(|| format!("unable to generate schema for {cmd}"))?;
    tracing::debug!(command = %cmd, "schema built");
    Ok(schema_str)
}

#[derive(Facet, Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Mapping {
    #[facet(default, skip_serializing_if = Option::is_none)]
    kiro_schema_path: Option<String>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    kg_schema_paths: Option<Vec<String>>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    description: Option<String>,
}

type MappingGroups = BTreeMap<String, BTreeMap<String, Mapping>>;

#[tracing::instrument(skip(agent))]
pub(crate) fn build_mappings(agent: JsonSchema) -> Result<MappingGroups> {
    let props = agent
        .properties
        .ok_or_else(|| eyre!("agent schema is missing properties"))?;
    tracing::debug!(property_count = props.len(), "building schema mappings");
    let mut kg_to_kiro: BTreeMap<String, Mapping> = BTreeMap::new();
    for (prop_name, prop_schema) in props {
        let key = format!("#/properties/{prop_name}");
        let description = prop_schema.description.unwrap_or_default();
        if description.is_empty() {
            tracing::debug!(field = %prop_name, "skipping field with no description");
            continue;
        }
        let (description, kiro_schema_path) =
            description.split_once(KG_MAPPING_DELIM).ok_or_else(|| {
                eyre!("missing mapping delimiter {KG_MAPPING_DELIM} in {key} description")
            })?;
        kg_to_kiro.insert(key, Mapping {
            kiro_schema_path: Some(kiro_schema_path.trim().to_string()),
            kg_schema_paths: None,
            description: Some(description.trim().to_string()),
        });
    }
    let mut mappings: MappingGroups = BTreeMap::new();
    mappings.insert("kg_to_kiro".to_string(), kg_to_kiro);
    Ok(mappings)
}

#[tracing::instrument]
pub(crate) fn handle_schema_mappings() -> Result<()> {
    let sch = schema_for::<KgAgentFileDoc>();
    let result = build_mappings(sch)?;
    let json_str = facet_json::to_string_pretty(&result)?;
    println!("{json_str}");
    Ok(())
}

#[tracing::instrument(skip(cmd), fields(command = %cmd))]
pub(crate) fn handle_schema_command(cmd: &SchemaCommand) -> Result<()> {
    let output = build_schema(cmd)?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, crate::json_schema::schema_for, color_eyre::eyre::eyre};

    #[test_log::test]
    fn test_mappings() -> Result<()> {
        let sch = schema_for::<KgAgentFileDoc>();
        let result = build_mappings(sch)?;
        let json_str = facet_json::to_string_pretty(&result)?;
        assert!(json_str.contains("kg_to_kiro"));
        handle_schema_mappings()?;
        Ok(())
    }

    #[test_log::test]
    fn test_manifest_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Manifest)?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema_str).wrap_err("manifest schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("manifest schema failed meta-validation: {e}"))?;
        handle_schema_command(&SchemaCommand::Manifest)?;
        Ok(())
    }

    #[test_log::test]
    fn test_manifest_schema_drift_guard() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Manifest)?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema_str).wrap_err("manifest schema is not valid JSON")?;

        let properties = schema
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| eyre!("manifest schema is missing properties"))?;
        let agents = properties
            .get("agents")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| eyre!("manifest schema is missing properties.agents"))?;
        let additional_properties = agents
            .get("additionalProperties")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| eyre!("manifest schema is missing agents.additionalProperties"))?;
        let agent_properties = additional_properties
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                eyre!("manifest schema is missing agents.additionalProperties.properties")
            })?;

        for field in ["name", "inherits", "template", "description"] {
            assert!(
                agent_properties.contains_key(field),
                "manifest schema missing {field}"
            );
        }

        Ok(())
    }

    #[test_log::test]
    fn test_agent_schema_is_valid() -> Result<()> {
        let schema_str = build_schema(&SchemaCommand::Agent(Default::default()))?;
        let schema: serde_json::Value =
            serde_json::from_str(&schema_str).wrap_err("agent schema is not valid JSON")?;
        jsonschema::meta::validate(&schema)
            .map_err(|e| eyre!("agent schema failed meta-validation: {e}"))?;

        handle_schema_command(&SchemaCommand::Agent(Default::default()))?;
        handle_schema_command(&SchemaCommand::KiroAgent)?;
        Ok(())
    }
}
