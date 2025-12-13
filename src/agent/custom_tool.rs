use {
    crate::merging_format::MergedSet,
    serde::{Deserialize, Serialize},
    std::collections::{HashMap, HashSet},
};

#[derive(Clone, Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransportType {
    /// Standard input/output transport (default)
    #[default]
    Stdio,
    /// HTTP transport for web-based communication
    Http,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConfig {
    /// Custom redirect URI for OAuth flow (e.g., "127.0.0.1:7778")
    /// If not specified, a random available port will be assigned by the OS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MergingCustomToolConfig {
    /// The transport type to use for communication with the MCP server
    #[serde(default)]
    pub r#type: TransportType,
    /// The URL for HTTP-based MCP server communication
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// OAuth configuration for this server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,
    /// The command string used to initialize the mcp server
    #[serde(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub args: MergedSet,
    /// A list of environment variables to run the command with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// Timeout for each mcp request in ms
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// A boolean flag to denote whether or not to load this mcp server
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolConfig {
    /// The transport type to use for communication with the MCP server
    #[serde(default)]
    pub r#type: TransportType,
    /// The URL for HTTP-based MCP server communication
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// OAuth configuration for this server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,
    /// The command string used to initialize the mcp server
    #[serde(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub args: HashSet<String>,
    /// A list of environment variables to run the command with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// Timeout for each mcp request in ms
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// A boolean flag to denote whether or not to load this mcp server
    #[serde(default)]
    pub disabled: bool,
}

impl From<MergingCustomToolConfig> for CustomToolConfig {
    fn from(config: MergingCustomToolConfig) -> Self {
        CustomToolConfig {
            r#type: config.r#type,
            url: config.url,
            headers: config.headers,
            oauth: config.oauth,
            command: config.command,
            args: config.args.into(),
            env: config.env,
            timeout: config.timeout,
            disabled: config.disabled,
        }
    }
}

pub fn default_timeout() -> u64 {
    120 * 1000
}
