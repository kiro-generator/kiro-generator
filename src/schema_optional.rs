use {
    facet::Facet,
    facet_json_schema::{AdditionalProperties, SchemaType},
    std::collections::BTreeMap,
};

/// Additional properties can be a boolean or a schema
#[derive(Debug, Clone, Facet)]
#[facet(untagged)]
#[repr(C)]
pub enum AdditionalPropertiesOptional {
    #[allow(dead_code)]
    Bool(bool),
    #[allow(dead_code)]
    Schema(Box<JsonSchemaOptional>),
}

impl From<AdditionalProperties> for AdditionalPropertiesOptional {
    fn from(value: AdditionalProperties) -> Self {
        match value {
            AdditionalProperties::Bool(b) => AdditionalPropertiesOptional::Bool(b),
            AdditionalProperties::Schema(s) => {
                AdditionalPropertiesOptional::Schema(Box::new((*s).into()))
            }
        }
    }
}

#[derive(Facet, Clone, Debug, Default)]
pub struct JsonSchemaOptional {
    /// The JSON Schema dialect
    #[facet(rename = "$schema", skip_serializing_if = Option::is_none)]
    pub schema: Option<String>,

    /// Reference to another schema definition
    #[facet(rename = "$ref", skip_serializing_if = Option::is_none)]
    pub ref_: Option<String>,

    /// Schema definitions for reuse
    #[facet(rename = "$defs", skip_serializing_if = Option::is_none)]
    pub defs: Option<BTreeMap<String, JsonSchemaOptional>>,

    /// The type of the schema
    #[facet(rename = "type", skip_serializing_if = Option::is_none)]
    pub type_: Option<SchemaType>,

    /// For objects: the properties
    #[facet(skip_serializing_if = Option::is_none)]
    pub properties: Option<BTreeMap<String, JsonSchemaOptional>>,

    /// For objects: required property names
    #[facet(skip_serializing_if = Option::is_none)]
    pub required: Option<Vec<String>>,

    /// For objects: additional properties schema or false
    #[facet(rename = "additionalProperties", skip_serializing_if = Option::is_none)]
    pub additional_properties: Option<AdditionalPropertiesOptional>,

    /// For arrays: the items schema
    #[facet(skip_serializing_if = Option::is_none)]
    pub items: Option<Box<JsonSchemaOptional>>,

    /// For strings: enumerated values
    #[facet(rename = "enum", skip_serializing_if = Option::is_none)]
    pub enum_: Option<Vec<String>>,

    /// For numbers: minimum value
    #[facet(skip_serializing_if = Option::is_none)]
    pub minimum: Option<f64>,

    /// For numbers: maximum value
    #[facet(skip_serializing_if = Option::is_none)]
    pub maximum: Option<f64>,

    /// For oneOf/anyOf/allOf
    #[facet(rename = "oneOf", skip_serializing_if = Option::is_none)]
    pub one_of: Option<Vec<JsonSchemaOptional>>,

    #[facet(rename = "anyOf", skip_serializing_if = Option::is_none)]
    pub any_of: Option<Vec<JsonSchemaOptional>>,

    #[facet(rename = "allOf", skip_serializing_if = Option::is_none)]
    pub all_of: Option<Vec<JsonSchemaOptional>>,

    /// Description from doc comments
    #[facet(skip_serializing_if = Option::is_none)]
    pub description: Option<String>,

    /// Title (type name)
    #[facet(skip_serializing_if = Option::is_none)]
    pub title: Option<String>,

    /// Constant value
    #[facet(rename = "const", skip_serializing_if = Option::is_none)]
    pub const_: Option<String>,

    /// For objects: schema for property names
    #[facet(rename = "propertyNames", skip_serializing_if = Option::is_none)]
    pub property_names: Option<Box<JsonSchemaOptional>>,
}

impl From<facet_json_schema::JsonSchema> for JsonSchemaOptional {
    fn from(value: facet_json_schema::JsonSchema) -> Self {
        let mut result = Self {
            schema: value.schema,
            ref_: value.ref_,
            defs: value
                .defs
                .map(|m| m.into_iter().map(|(k, v)| (k, v.into())).collect()),
            type_: value.type_,
            properties: value
                .properties
                .map(|m| m.into_iter().map(|(k, v)| (k, v.into())).collect()),
            required: value.required,
            additional_properties: value.additional_properties.map(Into::into),
            items: value.items.map(|b| Box::new((*b).into())),
            enum_: value.enum_,
            // Skip meaningless type bounds (i128::MIN, u128::MAX, etc)
            minimum: value
                .minimum
                .filter(|&v| v > i16::MIN as i128)
                .map(|v| v as f64),
            maximum: value
                .maximum
                .filter(|&v| v < u16::MAX as u128)
                .map(|v| v as f64),
            one_of: value
                .one_of
                .map(|v| v.into_iter().map(Into::into).collect()),
            any_of: value
                .any_of
                .map(|v| v.into_iter().map(Into::into).collect()),
            all_of: value
                .all_of
                .map(|v| v.into_iter().map(Into::into).collect()),
            description: value.description,
            title: value.title,
            const_: value.const_,
            property_names: None,
        };

        // TEMPORARY: Restrict hooks field keys to HookTrigger enum values
        // This will be removed once facet supports HashMap with enum keys
        if let Some(ref mut props) = result.properties
            && let Some(hooks_schema) = props.get_mut("hooks")
        {
            hooks_schema.property_names = Some(Box::new(JsonSchemaOptional {
                enum_: Some(vec![
                    "agentSpawn".to_string(),
                    "userPromptSubmit".to_string(),
                    "preToolUse".to_string(),
                    "postToolUse".to_string(),
                    "stop".to_string(),
                ]),
                ..Default::default()
            }));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{GeneratorConfig, KgAgentFileDoc},
        color_eyre::eyre::WrapErr,
    };

    #[test]
    fn test_manifest_schema_no_nulls() -> crate::Result<()> {
        let schema = facet_json_schema::schema_for::<GeneratorConfig>();
        let output: JsonSchemaOptional = schema.into();
        let json = facet_json::to_string_pretty(&output)
            .wrap_err("Failed to serialize manifest schema to JSON")?;
        assert!(!json.contains(": null"));
        Ok(())
    }

    #[test]
    fn test_agent_schema_no_nulls() -> crate::Result<()> {
        let schema = facet_json_schema::schema_for::<KgAgentFileDoc>();
        let output: JsonSchemaOptional = schema.into();
        let json = facet_json::to_string_pretty(&output)
            .wrap_err("Failed to serialize agent schema to JSON")?;
        assert!(!json.contains(": null"));
        Ok(())
    }

    #[test]
    fn test_agent_schema_is_valid() -> crate::Result<()> {
        let schema = facet_json_schema::schema_for::<KgAgentFileDoc>();
        let output: JsonSchemaOptional = schema.into();
        let json =
            facet_json::to_string(&output).wrap_err("Failed to serialize agent schema to JSON")?;
        let schema_value: serde_json::Value =
            serde_json::from_str(&json).wrap_err("Failed to parse generated schema as JSON")?;

        jsonschema::validator_for(&schema_value)
            .wrap_err("Generated agent schema is not valid JSON Schema")?;
        Ok(())
    }

    #[test]
    fn test_manifest_schema_is_valid() -> crate::Result<()> {
        let schema = facet_json_schema::schema_for::<GeneratorConfig>();
        let output: JsonSchemaOptional = schema.into();
        let json = facet_json::to_string(&output)
            .wrap_err("Failed to serialize manifest schema to JSON")?;
        let schema_value: serde_json::Value =
            serde_json::from_str(&json).wrap_err("Failed to parse generated schema as JSON")?;

        jsonschema::validator_for(&schema_value)
            .wrap_err("Generated manifest schema is not valid JSON Schema")?;
        Ok(())
    }

    #[test]
    fn test_hooks_property_names_restricted() {
        let schema = facet_json_schema::schema_for::<KgAgentFileDoc>();
        let output: JsonSchemaOptional = schema.into();

        let hooks_schema = output
            .properties
            .as_ref()
            .and_then(|p| p.get("hooks"))
            .expect("hooks property should exist");

        let property_names = hooks_schema
            .property_names
            .as_ref()
            .expect("hooks should have propertyNames constraint");

        let enum_values = property_names
            .enum_
            .as_ref()
            .expect("propertyNames should have enum");

        assert_eq!(enum_values.len(), 5);
        assert!(enum_values.contains(&"agentSpawn".to_string()));
        assert!(enum_values.contains(&"userPromptSubmit".to_string()));
        assert!(enum_values.contains(&"preToolUse".to_string()));
        assert!(enum_values.contains(&"postToolUse".to_string()));
        assert!(enum_values.contains(&"stop".to_string()));
    }

    #[test]
    fn test_manifest_hooks_property_names_restricted() {
        let schema = facet_json_schema::schema_for::<GeneratorConfig>();
        let output: JsonSchemaOptional = schema.into();

        // Navigate to agents -> additionalProperties (Manifest schema) -> properties ->
        // hooks
        let agents_schema = output
            .properties
            .as_ref()
            .and_then(|p| p.get("agents"))
            .expect("agents property should exist");

        let manifest_schema = match agents_schema.additional_properties.as_ref() {
            Some(AdditionalPropertiesOptional::Schema(s)) => s.as_ref(),
            _ => panic!("agents should have additionalProperties schema"),
        };

        let hooks_schema = manifest_schema
            .properties
            .as_ref()
            .and_then(|p| p.get("hooks"))
            .expect("hooks property should exist in Manifest");

        let property_names = hooks_schema
            .property_names
            .as_ref()
            .expect("hooks should have propertyNames constraint");

        let enum_values = property_names
            .enum_
            .as_ref()
            .expect("propertyNames should have enum");

        assert_eq!(enum_values.len(), 5);
        assert!(enum_values.contains(&"agentSpawn".to_string()));
        assert!(enum_values.contains(&"userPromptSubmit".to_string()));
        assert!(enum_values.contains(&"preToolUse".to_string()));
        assert!(enum_values.contains(&"postToolUse".to_string()));
        assert!(enum_values.contains(&"stop".to_string()));
    }
}
