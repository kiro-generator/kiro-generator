use {facet::Facet, std::collections::HashMap};

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

impl CustomToolConfig {
    pub fn merge(mut self, other: Self) -> Self {
        // Child wins for explicit values
        if self.timeout.is_none() && other.timeout.is_some() {
            tracing::trace!("timeout: merged from other");
            self.timeout = other.timeout;
        }

        if other.disabled.is_some_and(|d| d) {
            tracing::trace!("disabled: set from other");
            self.disabled = Some(true);
        } else if self.disabled.is_none() {
            self.disabled = other.disabled;
        }

        if self.url.is_empty() && !other.url.is_empty() {
            tracing::trace!("url: merged from other");
            self.url = other.url;
        }

        if self.command.is_empty() && !other.command.is_empty() {
            tracing::trace!("command: merged from other");
            self.command = other.command;
        }

        if !other.args.is_empty() {
            tracing::trace!(count = other.args.len(), "args: extended");
            self.args.extend(other.args);
        }

        let parent_env_count = other.env.len();
        let child_env_count = self.env.len();
        let mut merged = other.env; // Start with parent
        merged.extend(self.env); // Child overwrites parent
        if merged.len() != child_env_count {
            tracing::trace!(
                parent_count = parent_env_count,
                child_count = child_env_count,
                "env: merged"
            );
        }
        self.env = merged;

        let parent_headers_count = other.headers.len();
        let child_headers_count = self.headers.len();
        let mut merged = other.headers; // Start with parent
        merged.extend(self.headers); // Child overwrites parent
        if merged.len() != child_headers_count {
            tracing::trace!(
                parent_count = parent_headers_count,
                child_count = child_headers_count,
                "headers: merged"
            );
        }
        self.headers = merged;

        self
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

    #[test]
    fn test_merge_child_wins() {
        let parent = CustomToolConfig {
            command: "parent-cmd".into(),
            timeout: Some(1000),
            env: [("KEY".into(), "parent".into())].into(),
            headers: [("Auth".into(), "parent-token".into())].into(),
            ..Default::default()
        };

        let child = CustomToolConfig {
            timeout: Some(2000),
            env: [("KEY".into(), "child".into())].into(),
            headers: [("Auth".into(), "child-token".into())].into(),
            ..Default::default()
        };

        let merged = child.merge(parent);
        assert_eq!(merged.command, "parent-cmd");
        assert_eq!(merged.timeout, Some(2000));
        assert_eq!(merged.env.get("KEY"), Some(&"child".into()));
        assert_eq!(merged.headers.get("Auth"), Some(&"child-token".into()));
    }

    #[test]
    fn test_merge_extends_collections() {
        let parent = CustomToolConfig {
            env: [("PARENT_KEY".into(), "parent".into())].into(),
            headers: [("X-Parent".into(), "value".into())].into(),
            args: vec!["--parent".into()],
            ..Default::default()
        };

        let child = CustomToolConfig {
            env: [("CHILD_KEY".into(), "child".into())].into(),
            headers: [("X-Child".into(), "value".into())].into(),
            args: vec!["--child".into()],
            ..Default::default()
        };

        let merged = child.merge(parent);
        assert_eq!(merged.env.len(), 2);
        assert_eq!(merged.headers.len(), 2);
        assert_eq!(merged.args.len(), 2);
        assert!(merged.args.contains(&"--parent".into()));
        assert!(merged.args.contains(&"--child".into()));
    }

    #[test]
    fn test_merge_empty_strings() {
        let parent = CustomToolConfig {
            command: "parent-cmd".into(),
            url: "http://parent".into(),
            ..Default::default()
        };

        let child = CustomToolConfig {
            command: "child-cmd".into(),
            ..Default::default()
        };

        let merged = child.merge(parent);
        assert_eq!(merged.command, "child-cmd");
        assert_eq!(merged.url, "http://parent");
    }
}
