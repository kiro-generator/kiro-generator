use {
    crate::{agent::CustomToolConfig, config::GenericVec},
    facet::Facet,
    facet_kdl as kdl,
};

#[derive(Facet, Clone, Debug)]
struct KeyVal {
    #[facet(kdl::argument)]
    key: String,
    #[facet(kdl::argument)]
    value: String,
}

#[derive(Facet, Default, Clone, Debug)]
#[facet(rename_all = "kebab-case", default)]
pub struct CustomToolConfigDoc {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child, default)]
    pub url: String,

    #[facet(kdl::child, default)]
    pub command: String,

    #[facet(kdl::child, default)]
    args: GenericVec,

    #[facet(kdl::child, default)]
    env: GenericVec,

    #[facet(kdl::child, default)]
    header: GenericVec,

    #[facet(kdl::child, default)]
    pub(super) timeout: u64,

    #[facet(kdl::property, default)]
    pub disabled: bool,
}

impl From<CustomToolConfigDoc> for CustomToolConfig {
    fn from(value: CustomToolConfigDoc) -> Self {
        Self {
            url: value.url,
            command: value.command,
            args: value.args.item.into_iter().collect(),
            timeout: if value.timeout == 0 {
                crate::agent::tool_default_timeout()
            } else {
                value.timeout
            },
            disabled: value.disabled,
            headers: value.header.into(),
            env: value.env.into(),
        }
    }
}

impl From<&CustomToolConfigDoc> for CustomToolConfig {
    fn from(value: &CustomToolConfigDoc) -> Self {
        value.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::config::{ConfigResult, kdl_parse},
        indoc::indoc,
        std::collections::HashMap,
    };

    #[derive(Facet, Debug)]
    struct McpDoc {
        #[facet(kdl::child)]
        mcp: CustomToolConfigDoc,
    }

    #[test]
    fn parse_basic_mcp() -> ConfigResult<()> {
        let kdl = indoc! {
            r#"mcp "rustdocs" {
              command "rust-docs-mcp"
              timeout 1000
        }"#
        };

        let doc: McpDoc = kdl_parse(kdl)?;
        assert_eq!(doc.mcp.name, "rustdocs");
        assert_eq!(doc.mcp.command, "rust-docs-mcp");
        assert_eq!(doc.mcp.timeout, 1000);
        Ok(())
    }

    #[test]
    fn parse_mcp_with_url() -> ConfigResult<()> {
        let kdl = r#"mcp "remote" {
            url "http://localhost:8080"
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl)?;
        assert_eq!(doc.mcp.name, "remote");
        assert_eq!(doc.mcp.url, "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn parse_mcp_with_env_and_headers() -> ConfigResult<()> {
        let kdl = r#"mcp "api" {
            command "api-server"
            env "API_KEY" "secret123"
            env "DEBUG" "true"
            header "Authorization" "Bearer token"
            header "Content-Type" "application/json"
        }"#;
        let doc: McpDoc = facet_kdl::from_str(kdl)?;
        assert_eq!(doc.mcp.env.len(), 4);
        assert_eq!(doc.mcp.header.len(), 4);

        let env: HashMap<String, String> = doc.mcp.env.into();
        assert_eq!(env.len(), 2);
        assert_eq!(env.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));

        let header: HashMap<String, String> = doc.mcp.header.into();
        assert_eq!(header.len(), 2);
        assert_eq!(
            header.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(
            header.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        Ok(())
    }

    #[test]
    fn parse_mcp_with_args() -> ConfigResult<()> {
        let kdl = indoc! { r#"mcp "tool" {
            command "my-tool"
            args "--verbose" "--output=json"
        }"#
        };

        let doc: McpDoc = kdl_parse(kdl)?;
        assert_eq!(doc.mcp.args.item, vec!["--verbose", "--output=json"]);
        Ok(())
    }

    #[test]
    fn convert_to_custom_tool_config() -> ConfigResult<()> {
        let kdl = r#"mcp "test" disabled=#true {
            command "test-cmd"
            timeout 5000
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl)?;
        let config: CustomToolConfig = doc.mcp.into();

        assert_eq!(config.command, "test-cmd");
        assert_eq!(config.timeout, 5000);
        assert!(config.disabled);
        Ok(())
    }

    #[test]
    fn default_timeout_when_zero() -> ConfigResult<()> {
        let kdl = r#"mcp "test" {
            command "test-cmd"
            timeout 0
        }"#;

        let doc: McpDoc = facet_kdl::from_str(kdl)?;
        let config: CustomToolConfig = doc.mcp.into();

        assert_eq!(config.timeout, crate::agent::tool_default_timeout());
        Ok(())
    }
}
