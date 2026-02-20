use {crate::kg_config::KgCustomToolConfig, facet::Facet, std::collections::HashMap};

#[derive(Facet, Default, Clone, Debug, Eq, PartialEq)]
#[facet(default, skip_all_unless_truthy, deny_unknown_fields)]
pub struct CustomToolConfig {
    /// The URL for HTTP-based MCP server communication
    #[facet(default)]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[facet(default)]
    pub headers: HashMap<String, String>,
    /// The command string used to initialize the mcp server
    #[facet(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[facet(default)]
    pub args: Vec<String>,
    /// A list of environment variables to run the command with
    #[facet(default)]
    pub env: HashMap<String, String>,
    /// Timeout for each mcp request in ms
    #[facet(default)]
    pub timeout: Option<u64>,
    /// A boolean flag to denote whether or not to load this mcp server
    #[facet(default)]
    pub disabled: Option<bool>,
}

impl From<KgCustomToolConfig> for CustomToolConfig {
    fn from(kg: KgCustomToolConfig) -> Self {
        Self {
            url: kg.url,
            headers: kg.headers,
            command: kg.command,
            args: kg.args,
            env: kg.env,
            timeout: kg.timeout,
            disabled: kg.state.map(|s| s.is_disabled()),
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{Result, toml_parse},
    };

    #[derive(Facet, Debug)]
    struct McpDoc {
        #[facet(default, rename = "mcpServers")]
        mcp_servers: HashMap<String, CustomToolConfig>,
    }

    #[test]
    fn parse_basic_mcp() -> Result<()> {
        let raw = r#"
[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout  =1000
"#;

        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("rustdocs"));
        let mcp = doc.mcp_servers.get("rustdocs").unwrap();
        assert_eq!(mcp.command, "rust-docs-mcp");
        assert_eq!(mcp.timeout, Some(1000));
        Ok(())
    }

    #[test]
    fn parse_mcp_with_url() -> Result<()> {
        let raw = r#"
        [mcpServers.remote]
        url="http://localhost:8080"
        "#;
        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("remote"));
        let mcp = doc.mcp_servers.get("remote").unwrap();
        assert_eq!(mcp.url, "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn parse_mcp_with_env_and_headers() -> Result<()> {
        let raw = r#"
        [mcpServers.api]
        command = "api-server"
        [mcpServers.api.env]
        API_KEY= "secret123"
        DEBUG="true"
        [mcpServers.api.headers]
        Authorization= "Bearer token"
        "Content-Type"= "application/json"
        "#;
        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("api"));
        let mcp = doc.mcp_servers.get("api").unwrap();
        assert!(mcp.timeout.is_none());
        assert_eq!(mcp.env.len(), 2);
        assert_eq!(mcp.headers.len(), 2);
        let env = &mcp.env;
        assert_eq!(env.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
        let header = &mcp.headers;
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
    fn parse_mcp_with_args() -> Result<()> {
        let raw = r#"
        [mcpServers.tool]
        command = "my-tool"
        args = ["--verbose", "--output=json"]
        disabled = true
        "#;

        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("tool"));
        let mcp = doc.mcp_servers.get("tool").unwrap();
        assert_eq!(mcp.args, vec!["--verbose", "--output=json"]);
        assert_eq!(mcp.disabled, Some(true));
        Ok(())
    }
}
