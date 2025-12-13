use {
    config::{FileStoredFormat, Format, Map, Value},
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    std::{
        collections::HashSet,
        error::Error,
        ops::{Deref, DerefMut},
    },
};

/// A TOML format that converts arrays of strings into maps for merging
/// behavior. This tricks the config crate into merging arrays (as map keys)
/// instead of overwriting them.
#[derive(Debug, Clone)]
pub struct MergingTomlFormat;

/// Wrapper type that deserializes from a map (created by MergingTomlFormat)
/// back to a HashSet. Use this for fields like `allowedTools`,
/// `allowedCommands`, etc.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MergedSet(pub HashSet<String>);

impl<'de> Deserialize<'de> for MergedSet {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use {
            serde::de::{self, Visitor},
            std::fmt,
        };

        struct MergedSetVisitor;

        impl<'de> Visitor<'de> for MergedSetVisitor {
            type Value = MergedSet;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of strings or an array of strings")
            }

            fn visit_map<M>(self, mut map: M) -> std::result::Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut set = HashSet::new();
                while let Some((key, _value)) = map.next_entry::<String, String>()? {
                    set.insert(key);
                }
                Ok(MergedSet(set))
            }

            fn visit_seq<S>(self, mut seq: S) -> std::result::Result<Self::Value, S::Error>
            where
                S: de::SeqAccess<'de>,
            {
                let mut set = HashSet::new();
                while let Some(value) = seq.next_element::<String>()? {
                    set.insert(value);
                }
                Ok(MergedSet(set))
            }
        }

        deserializer.deserialize_any(MergedSetVisitor)
    }
}

impl Serialize for MergedSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a regular array (Vec) for output
        let vec: Vec<&String> = self.0.iter().collect();
        vec.serialize(serializer)
    }
}

impl Deref for MergedSet {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MergedSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashSet<String>> for MergedSet {
    fn from(set: HashSet<String>) -> Self {
        MergedSet(set)
    }
}

impl From<MergedSet> for HashSet<String> {
    fn from(merged: MergedSet) -> Self {
        merged.0
    }
}

impl MergedSet {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Format for MergingTomlFormat {
    fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> std::result::Result<Map<String, Value>, Box<dyn Error + Send + Sync>> {
        let table = from_toml_table(uri, toml::from_str(text)?);
        Ok(table)
    }
}

impl FileStoredFormat for MergingTomlFormat {
    fn file_extensions(&self) -> &'static [&'static str] {
        &["toml"]
    }
}

fn from_toml_table(uri: Option<&String>, table: toml::Table) -> Map<String, Value> {
    let mut m = Map::new();
    for (key, value) in table {
        m.insert(key, from_toml_value(uri, value));
    }
    m
}

fn from_toml_value(uri: Option<&String>, value: toml::Value) -> Value {
    match value {
        toml::Value::String(value) => Value::new(uri, value),
        toml::Value::Float(value) => Value::new(uri, value),
        toml::Value::Integer(value) => Value::new(uri, value),
        toml::Value::Boolean(value) => Value::new(uri, value),
        toml::Value::Datetime(datetime) => Value::new(uri, datetime.to_string()),

        toml::Value::Table(table) => {
            let m = from_toml_table(uri, table);
            Value::new(uri, m)
        }

        toml::Value::Array(array) => {
            // Check if this is an array of strings - if so, convert to map for merging
            if is_string_array(&array) {
                let mut m = Map::new();
                for value in array {
                    if let toml::Value::String(s) = value {
                        // Use the string itself as the key, value is empty string as marker
                        m.insert(s, Value::new(uri, ""));
                    }
                }
                Value::new(uri, m)
            } else {
                // For non-string arrays, keep default behavior
                let mut l = Vec::new();
                for value in array {
                    l.push(from_toml_value(uri, value));
                }
                Value::new(uri, l)
            }
        }
    }
}

/// Check if a toml array contains only strings
fn is_string_array(array: &[toml::Value]) -> bool {
    !array.is_empty() && array.iter().all(|v| matches!(v, toml::Value::String(_)))
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::Result,
        color_eyre::eyre,
        config::{Config, File},
    };

    #[test_log::test]
    fn test_string_array_becomes_map() -> Result<()> {
        let toml = r#"
            allowedTools = ["read", "write", "execute"]
        "#;

        let format = MergingTomlFormat;
        let parsed = format
            .parse(None, toml)
            .map_err(|e| eyre::eyre!("Parse error: {}", e))?;

        // Should be a map, not an array
        let allowed_tools = parsed.get("allowedTools").unwrap();
        assert!(matches!(allowed_tools.kind, config::ValueKind::Table(_)));

        Ok(())
    }

    #[test_log::test]
    fn test_merging_behavior() -> Result<()> {
        let base_toml = r#"
            allowedTools = ["read", "write"]
        "#;

        let child_toml = r#"
            allowedTools = ["execute", "write"]
        "#;

        let config = Config::builder()
            .add_source(File::from_str(base_toml, MergingTomlFormat))
            .add_source(File::from_str(child_toml, MergingTomlFormat))
            .build()?;

        // Deserialize as a map to see the merged keys
        let result: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
            config.try_deserialize()?;

        let tools = result.get("allowedTools").unwrap();
        assert_eq!(3, tools.len());
        assert!(tools.contains_key("read"));
        assert!(tools.contains_key("write"));
        assert!(tools.contains_key("execute"));

        Ok(())
    }

    #[test_log::test]
    fn test_non_string_arrays_unchanged() -> Result<()> {
        let toml = r#"
            numbers = [1, 2, 3]
            mixed = [[1, 2], [3, 4]]
        "#;

        let format = MergingTomlFormat;
        let parsed = format
            .parse(None, toml)
            .map_err(|e| eyre::eyre!("Parse error: {}", e))?;

        // Should still be arrays
        let numbers = parsed.get("numbers").unwrap();
        assert!(matches!(numbers.kind, config::ValueKind::Array(_)));

        let mixed = parsed.get("mixed").unwrap();
        assert!(matches!(mixed.kind, config::ValueKind::Array(_)));

        Ok(())
    }

    #[test_log::test]
    fn test_merged_set_deserialization() -> Result<()> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct TestConfig {
            #[serde(rename = "allowedTools")]
            allowed_tools: MergedSet,
        }

        let base_toml = r#"
            allowedTools = ["read", "write"]
        "#;

        let child_toml = r#"
            allowedTools = ["execute", "write"]
        "#;

        let config = Config::builder()
            .add_source(File::from_str(base_toml, MergingTomlFormat))
            .add_source(File::from_str(child_toml, MergingTomlFormat))
            .build()?;

        let result: TestConfig = config.try_deserialize()?;

        assert_eq!(3, result.allowed_tools.0.len());
        assert!(result.allowed_tools.0.contains("read"));
        assert!(result.allowed_tools.0.contains("write"));
        assert!(result.allowed_tools.0.contains("execute"));

        Ok(())
    }

    #[test_log::test]
    fn test_merged_set_serialization() -> Result<()> {
        let mut set = HashSet::new();
        set.insert("read".to_string());
        set.insert("write".to_string());
        set.insert("execute".to_string());

        let merged = MergedSet(set);
        let json = serde_json::to_string(&merged)?;

        // Should serialize as an array
        let deserialized: Vec<String> = serde_json::from_str(&json)?;
        assert_eq!(3, deserialized.len());
        assert!(deserialized.contains(&"read".to_string()));
        assert!(deserialized.contains(&"write".to_string()));
        assert!(deserialized.contains(&"execute".to_string()));

        Ok(())
    }

    #[test_log::test]
    fn test_empty_merged_set_inheritance() -> Result<()> {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct TestConfig {
            #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
            tools: MergedSet,
            #[serde(
                default,
                rename = "allowedTools",
                skip_serializing_if = "MergedSet::is_empty"
            )]
            allowed_tools: MergedSet,
        }

        // Parent has tools defined
        let parent = TestConfig {
            tools: MergedSet(["*".to_string()].into_iter().collect()),
            allowed_tools: MergedSet(
                ["read".to_string(), "write".to_string()]
                    .into_iter()
                    .collect(),
            ),
        };

        // Child has empty tools (should inherit from parent)
        let child = TestConfig {
            tools: MergedSet::default(),
            allowed_tools: MergedSet(["execute".to_string()].into_iter().collect()),
        };

        // Serialize both to TOML
        let parent_toml = toml::to_string(&parent)?;
        let child_toml = toml::to_string(&child)?;

        // Child should NOT have tools field (empty set skipped)
        assert!(!child_toml.contains("tools"));
        // Child should have allowedTools
        assert!(child_toml.contains("allowedTools"));

        // Merge using config crate
        let config = Config::builder()
            .add_source(File::from_str(&parent_toml, MergingTomlFormat))
            .add_source(File::from_str(&child_toml, MergingTomlFormat))
            .build()?;

        let result: TestConfig = config.try_deserialize()?;

        // tools should be inherited from parent (not overwritten by empty child)
        assert_eq!(1, result.tools.len());
        assert!(result.tools.contains("*"));

        // allowedTools should be merged
        assert_eq!(3, result.allowed_tools.len());
        assert!(result.allowed_tools.contains("read"));
        assert!(result.allowed_tools.contains("write"));
        assert!(result.allowed_tools.contains("execute"));

        Ok(())
    }
}
