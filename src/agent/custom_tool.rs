use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolConfig {
    /// The URL for HTTP-based MCP server communication
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// The command string used to initialize the mcp server
    #[serde(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// A list of environment variables to run the command with
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// Timeout for each mcp request in ms
    #[serde(default = "tool_default_timeout")]
    pub timeout: u64,
    /// A boolean flag to denote whether or not to load this mcp server
    #[serde(default)]
    pub disabled: bool,
}

pub fn tool_default_timeout() -> u64 {
    120 * 1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_defaultttl() {
        assert_eq!(tool_default_timeout(), 120 * 1000);
    }

    #[test]
    fn custom_tool_config_serde() {
        let config = CustomToolConfig {
            url: "http://test".into(),
            headers: HashMap::new(),
            command: "cmd".into(),
            args: vec!["arg1".into()],
            env: HashMap::new(),
            timeout: 5000,
            disabled: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CustomToolConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
