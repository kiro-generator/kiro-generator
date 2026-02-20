use {facet::Facet, std::collections::HashMap};

/// The operational state of an MCP server.
#[derive(Facet, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum McpServerState {
    #[facet(rename = "enabled")]
    Enabled,
    #[facet(rename = "disabled")]
    Disabled,
}

impl McpServerState {
    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

/// MCP server configuration for agent definitions.
#[derive(Facet, Default, Clone, Debug, Eq, PartialEq)]
#[facet(default, deny_unknown_fields)]
pub struct KgCustomToolConfig {
    /// The URL for HTTP-based MCP server communication
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    pub headers: HashMap<String, String>,
    /// The command string used to initialize the MCP server
    pub command: String,
    /// A list of arguments to be used to run the command with
    pub args: Vec<String>,
    /// A list of environment variables to run the command with
    pub env: HashMap<String, String>,
    /// Timeout for each MCP request in milliseconds
    pub timeout: Option<u64>,
    /// MCP server operational state ("enabled" or "disabled", maps to disabled
    /// boolean in JSON)
    pub state: Option<McpServerState>,
}

impl KgCustomToolConfig {
    /// Merge child (self) with parent (other). Child wins for explicit values.
    pub fn merge(mut self, other: Self) -> Self {
        if self.timeout.is_none() && other.timeout.is_some() {
            tracing::trace!("timeout: merged from other");
            self.timeout = other.timeout;
        }
        if self.state.is_none() {
            if other.state.is_some() {
                tracing::trace!("mcpState: setting to {:?}", other.state);
            }
            self.state = other.state;
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
        mcp_servers: HashMap<String, KgCustomToolConfig>,
    }

    #[test_log::test]
    fn state_disabled_from_toml() -> Result<()> {
        let raw = r#"
[mcpServers.ctx]
command = "ctx-mcp"
state = "disabled"
"#;
        let doc: McpDoc = toml_parse(raw)?;
        let mcp = doc.mcp_servers.get("ctx").unwrap();
        assert_eq!(mcp.state, Some(McpServerState::Disabled));
        assert!(mcp.state.as_ref().unwrap().is_disabled());
        Ok(())
    }

    #[test_log::test]
    fn state_enabled_from_toml() -> Result<()> {
        let raw = r#"
[mcpServers.ctx]
command = "ctx-mcp"
state = "enabled"
"#;
        let doc: McpDoc = toml_parse(raw)?;
        let mcp = doc.mcp_servers.get("ctx").unwrap();
        assert_eq!(mcp.state, Some(McpServerState::Enabled));
        assert!(!mcp.state.as_ref().unwrap().is_disabled());
        Ok(())
    }

    #[test_log::test]
    fn state_absent_from_toml() -> Result<()> {
        let raw = r#"
[mcpServers.ctx]
command = "ctx-mcp"
"#;
        let doc: McpDoc = toml_parse(raw)?;
        let mcp = doc.mcp_servers.get("ctx").unwrap();
        assert_eq!(mcp.state, None);
        Ok(())
    }

    #[test_log::test]
    fn merge_child_state_wins() {
        let parent = KgCustomToolConfig {
            command: "parent-cmd".into(),
            state: Some(McpServerState::Disabled),
            ..Default::default()
        };
        let child = KgCustomToolConfig {
            state: Some(McpServerState::Enabled),
            ..Default::default()
        };
        let merged = child.merge(parent);
        assert_eq!(merged.command, "parent-cmd");
        assert_eq!(merged.state, Some(McpServerState::Enabled));
    }

    #[test_log::test]
    fn merge_child_inherits_state_when_absent() {
        let parent = KgCustomToolConfig {
            state: Some(McpServerState::Disabled),
            ..Default::default()
        };
        let child = KgCustomToolConfig::default();
        let merged = child.merge(parent);
        assert_eq!(merged.state, Some(McpServerState::Disabled));
    }
}
