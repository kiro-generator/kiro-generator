use super::define_location_resource;

define_location_resource!(KgFileResource, "file");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_file_resource() -> crate::Result<()> {
        let toml = r#"
enabled = true
optional = true
locations = ["README.md", "/tmp/design.md"]
"#;
        let r: KgFileResource = facet_toml::from_str(toml)?;
        assert_eq!(r.enabled, Some(true));
        assert_eq!(r.optional, Some(true));
        assert!(r.locations.contains("README.md"));
        assert!(r.locations.contains("/tmp/design.md"));
        Ok(())
    }

    #[test]
    fn test_merge_file_locations_union() {
        let base = KgFileResource {
            enabled: Some(true),
            optional: None,
            locations: ["README.md".to_string()].into_iter().collect(),
        };

        let child = KgFileResource {
            enabled: Some(true),
            optional: None,
            locations: ["RUST.md".to_string()].into_iter().collect(),
        };

        let merged = child.merge(base);
        assert!(merged.locations.contains("README.md"));
        assert!(merged.locations.contains("RUST.md"));
    }
}
