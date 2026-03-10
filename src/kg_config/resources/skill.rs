use super::define_location_resource;

define_location_resource!(KgSkillResource, "skill");

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::kg_config::{SearchQuery, Searchable},
    };

    #[test]
    fn test_deserialize_skill_resource() -> crate::Result<()> {
        let toml = r#"
disabled = true
optional = false
locations = [".kiro/skills/**/SKILL.md"]
"#;
        let r: KgSkillResource = facet_toml::from_str(toml)?;
        assert_eq!(r.disabled, Some(true));
        assert_eq!(r.optional, Some(false));
        assert!(r.locations.contains(".kiro/skills/**/SKILL.md"));
        Ok(())
    }

    #[test]
    fn test_merge_skill_locations_union() {
        let base = KgSkillResource {
            disabled: Some(true),
            optional: None,
            locations: ["a/SKILL.md".to_string()].into_iter().collect(),
        };

        let child = KgSkillResource {
            disabled: Some(true),
            optional: None,
            locations: ["b/SKILL.md".to_string()].into_iter().collect(),
        };

        let merged = child.merge(base);
        assert!(merged.locations.contains("a/SKILL.md"));
        assert!(merged.locations.contains("b/SKILL.md"));
    }

    #[test]
    fn search_matches_locations() {
        let skill = KgSkillResource {
            locations: ["foo/SKILL.md".to_string(), "bar/notes.md".to_string()]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        assert!(skill.search(&"skill".into()));
        assert!(!skill.search(&SearchQuery::from("skill").case_sensitive()));
        assert!(!skill.search(&"missing".into()));
    }
}
