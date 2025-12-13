use {
    super::custom_tool::{CustomToolConfig, MergingCustomToolConfig},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

// This is to mirror claude's config set up
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "camelCase", transparent)]
pub struct MergingMcpServerConfig {
    pub mcp_servers: HashMap<String, MergingCustomToolConfig>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "camelCase", transparent)]
pub struct McpServerConfig {
    pub mcp_servers: HashMap<String, CustomToolConfig>,
}

impl From<MergingMcpServerConfig> for McpServerConfig {
    fn from(value: MergingMcpServerConfig) -> Self {
        Self {
            mcp_servers: value
                .mcp_servers
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
