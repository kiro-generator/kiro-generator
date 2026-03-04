use super::define_location_resource;

define_location_resource!(KgSkillResource, "skill");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_skill_resource() -> crate::Result<()> {
        let toml = r#"
enabled = true
optional = false
locations = [".kiro/skills/**/SKILL.md"]
"#;
        let r: KgSkillResource = facet_toml::from_str(toml)?;
        assert_eq!(r.enabled, Some(true));
        assert_eq!(r.optional, Some(false));
        assert!(r.locations.contains(".kiro/skills/**/SKILL.md"));
        Ok(())
    }

    #[test]
    fn test_merge_skill_locations_union() {
        let base = KgSkillResource {
            enabled: Some(true),
            optional: None,
            locations: ["a/SKILL.md".to_string()].into_iter().collect(),
        };

        let child = KgSkillResource {
            enabled: Some(true),
            optional: None,
            locations: ["b/SKILL.md".to_string()].into_iter().collect(),
        };

        let merged = child.merge(base);
        assert!(merged.locations.contains("a/SKILL.md"));
        assert!(merged.locations.contains("b/SKILL.md"));
    }
}
