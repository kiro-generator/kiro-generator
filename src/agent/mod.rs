mod custom_tool;
pub mod hook;
mod kg_agent;
mod mcp_config;
pub mod tools;
mod wrapper_types;
pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://README.md", "file://AGENTS.md"];
pub const DEFAULT_APPROVE: [&str; 0] = [];
use {
    super::agent::hook::{Hook, HookTrigger},
    crate::Result,
    color_eyre::eyre::eyre,
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fmt::Display,
    },
};
pub use {
    kg_agent::KgAgent,
    mcp_config::McpServerConfig,
    tools::*,
    wrapper_types::OriginalToolName,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Agent {
    /// Name of the agent
    #[serde(default)]
    pub name: String,
    /// This field is not model facing and is mostly here for users to discern
    /// between agents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The intention for this field is to provide high level context to the
    /// agent. This should be seen as the same category of context as a system
    /// prompt.
    #[serde(default)]
    pub prompt: Option<String>,
    /// Configuration for Model Context Protocol (MCP) servers
    #[serde(default)]
    pub mcp_servers: McpServerConfig,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[serde(default)]
    pub tools: HashSet<String>,
    /// Tool aliases for remapping tool names
    #[serde(default)]
    pub tool_aliases: HashMap<OriginalToolName, String>,
    /// List of tools the agent is explicitly allowed to use
    #[serde(default)]
    pub allowed_tools: HashSet<String>,
    /// Files to include in the agent's context
    #[serde(default)]
    pub resources: HashSet<String>,
    /// Commands to run when a chat session is created
    #[serde(default)]
    pub hooks: HashMap<HookTrigger, Vec<Hook>>,
    /// Settings for specific tools. These are mostly for native tools. The
    /// actual schema differs by tools and is documented in detail in our
    /// documentation
    #[serde(default)]
    pub tools_settings: HashMap<ToolTarget, serde_json::Value>,
    /// The model ID to use for this agent. If not specified, uses the default
    /// model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, rename = "includeMcpJson")]
    pub include_mcp_json: bool,
}

impl Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Agent {
    #[cfg(test)]
    pub(crate) fn get_tool<T: serde::de::DeserializeOwned + Default>(&self, tool: ToolTarget) -> T {
        match self.tools_settings.get(&tool) {
            Some(value) => match serde_json::from_value(value.clone()) {
                Ok(settings) => settings,
                Err(e) => {
                    tracing::debug!("Failed to deserialize tool settings for {tool}: {e}");
                    T::default()
                }
            },
            None => {
                tracing::debug!("No tool settings found for {tool}");
                T::default()
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        let schema: serde_json::Value = serde_json::from_str(crate::schema::SCHEMA)?;
        let validator = jsonschema::validator_for(&schema)?;
        let instance = serde_json::to_value(self)?;

        if let Err(e) = validator.validate(&instance) {
            return Err(eyre!(
                "Validation error: {}\n{}",
                e,
                serde_json::to_string(&instance)?
            ));
        }
        Ok(())
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            name: "kiro_default".to_string(),
            description: Some("Default agent".to_string()),
            tools: {
                let mut set = HashSet::new();
                set.insert("*".to_string());
                set
            },
            prompt: Default::default(),
            mcp_servers: Default::default(),
            tool_aliases: Default::default(),
            allowed_tools: {
                let mut set = HashSet::<String>::new();
                let default_approve = DEFAULT_APPROVE.iter().copied().map(str::to_string);
                set.extend(default_approve);
                set
            },
            resources: {
                let mut resources = HashSet::new();
                resources.extend(DEFAULT_AGENT_RESOURCES.iter().map(|&s| s.into()));
                //                resources.insert(format!("file://{}", RULES_PATTERN).into());
                resources
            },
            hooks: Default::default(),
            tools_settings: Default::default(),
            include_mcp_json: true,
            model: None,
        }
    }
}
