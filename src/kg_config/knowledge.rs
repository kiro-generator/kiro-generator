use facet::Facet;

#[derive(Facet, Clone, Debug, PartialEq, Eq, Hash)]
#[facet(deny_unknown_fields, rename_all = "camelCase")]
pub struct KgKnowledge {
    #[facet(default)]
    pub source: Option<String>,
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub index_type: Option<String>,
    #[facet(default)]
    pub auto_update: Option<bool>,
}

impl KgKnowledge {
    pub fn merge(mut self, other: Self) -> Self {
        if self.source.is_none() && other.source.is_some() {
            tracing::trace!("source: merged from other");
            self.source = other.source;
        }

        if self.description.is_none() && other.description.is_some() {
            tracing::trace!("description: merged from other");
            self.description = other.description;
        }

        if self.index_type.is_none() && other.index_type.is_some() {
            tracing::trace!("index_type: merged from other");
            self.index_type = other.index_type;
        }

        if self.auto_update.is_none() && other.auto_update.is_some() {
            tracing::trace!("auto_update: merged from other");
            self.auto_update = other.auto_update;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_knowledge_minimal() -> crate::Result<()> {
        let toml = r#"
source = "file://./docs"
"#;
        let k: KgKnowledge = facet_toml::from_str(toml)?;
        assert_eq!(k.source, Some("file://./docs".to_string()));
        assert_eq!(k.description, None);
        assert_eq!(k.index_type, None);
        assert_eq!(k.auto_update, None);
        Ok(())
    }

    #[test]
    fn test_deserialize_knowledge_full() {
        let toml = r#"
source = "file://./docs"
description = "Project documentation and guides"
indexType = "best"
autoUpdate = true
"#;
        let k: KgKnowledge = facet_toml::from_str(toml).unwrap();
        assert_eq!(k.source, Some("file://./docs".to_string()));
        assert_eq!(
            k.description,
            Some("Project documentation and guides".to_string())
        );
        assert_eq!(k.index_type, Some("best".to_string()));
        assert_eq!(k.auto_update, Some(true));
    }

    #[test]
    fn test_knowledge_merge() {
        let base = KgKnowledge {
            source: Some("file://./base".to_string()),
            description: Some("Base docs".to_string()),
            index_type: None,
            auto_update: None,
        };

        let child = KgKnowledge {
            source: None,
            description: None,
            index_type: Some("fast".to_string()),
            auto_update: Some(true),
        };

        let merged = base.merge(child);
        assert_eq!(merged.source, Some("file://./base".to_string()));
        assert_eq!(merged.description, Some("Base docs".to_string()));
        assert_eq!(merged.index_type, Some("fast".to_string()));
        assert_eq!(merged.auto_update, Some(true));
    }
}
