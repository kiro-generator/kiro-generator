mod custom_tool;
pub mod hook;
pub mod tools;
pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://README.md", "file://AGENTS.md"];
pub const DEFAULT_APPROVE: [&str; 0] = [];
use {
    super::agent::hook::{Hook, HookTrigger},
    crate::{Result, config::KdlAgent},
    miette::IntoDiagnostic,
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fmt::Display,
    },
};
pub use {
    custom_tool::{CustomToolConfig, tool_default_timeout},
    tools::*,
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
    pub mcp_servers: HashMap<String, CustomToolConfig>,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[serde(default)]
    pub tools: HashSet<String>,
    /// Tool aliases for remapping tool names
    #[serde(default)]
    pub tool_aliases: HashMap<String, String>,
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
    pub tools_settings: HashMap<String, serde_json::Value>,
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
    pub fn validate(&self) -> Result<()> {
        // TODO cache this
        let schema: serde_json::Value =
            serde_json::from_str(crate::schema::SCHEMA).into_diagnostic()?;
        let validator = jsonschema::validator_for(&schema).into_diagnostic()?;
        let instance = serde_json::to_value(self).into_diagnostic()?;

        if let Err(e) = validator.validate(&instance) {
            return Err(crate::format_err!(
                "Validation error: {}\n{}",
                e,
                serde_json::to_string(&instance).unwrap_or_default()
            ));
        }
        Ok(())
    }
}

impl TryFrom<&KdlAgent> for Agent {
    type Error = miette::Report;

    fn try_from(value: &KdlAgent) -> std::result::Result<Self, Self::Error> {
        let native_tools = &value.native_tool;
        let mut tools_settings = HashMap::new();

        let tool: AwsTool = native_tools.into();
        let tool_name = ToolTarget::Aws.to_string();
        if tool != AwsTool::default() {
            tools_settings.insert(
                tool_name.to_string(),
                serde_json::to_value(&tool).map_err(|e| {
                    crate::format_err!(
                        "Failed to serialize {tool_name} tool
    configuration {e}"
                    )
                })?,
            );
        }
        let tool: ReadTool = native_tools.into();
        let tool_name = ToolTarget::Read.to_string();
        if tool != ReadTool::default() {
            tools_settings.insert(
                tool_name.to_string(),
                serde_json::to_value(&tool).map_err(|e| {
                    crate::format_err!(
                        "Failed to serialize {tool_name} tool
    configuration {e}"
                    )
                })?,
            );
        }
        let tool: WriteTool = native_tools.into();
        let tool_name = ToolTarget::Write.to_string();
        if tool != WriteTool::default() {
            tools_settings.insert(
                tool_name.to_string(),
                serde_json::to_value(&tool).map_err(|e| {
                    crate::format_err!(
                        "Failed to serialize {tool_name} tool
    configuration {e}"
                    )
                })?,
            );
        }
        let tool: ExecuteShellTool = native_tools.into();
        let tool_name = ToolTarget::Shell.to_string();
        if tool != ExecuteShellTool::default() {
            tools_settings.insert(
                tool_name.to_string(),
                serde_json::to_value(&tool).map_err(|e| {
                    crate::format_err!(
                        "Failed to serialize {tool_name} tool
    configuration {e}"
                    )
                })?,
            );
        }
        let default_agent = Self::default();
        let tools = value.tools.clone();
        let allowed_tools = value.allowed_tools.clone();
        let resources: HashSet<String> = value.resources.clone();

        // Extra tool settings override native tools
        // let extra_tool_settings = value.extra_tool_settings()?;
        // tools_settings.extend(extra_tool_settings);

        let mut hooks: HashMap<HookTrigger, Vec<Hook>> = HashMap::new();
        let triggers: Vec<HookTrigger> = enum_iterator::all::<HookTrigger>().collect();
        for t in triggers {
            hooks.insert(t, value.hook.hooks(&t));
        }
        Ok(Self {
            name: value.name.clone(),
            description: value.description.clone(),
            prompt: value.prompt.clone(),
            mcp_servers: value.mcp.clone(),
            tools: if tools.is_empty() {
                default_agent.tools
            } else {
                tools
            },
            tool_aliases: value.alias.clone(),
            allowed_tools: if allowed_tools.is_empty() {
                default_agent.allowed_tools
            } else {
                allowed_tools
            },
            resources: if resources.is_empty() {
                default_agent.resources
            } else {
                resources
            },
            hooks,
            tools_settings,
            model: value.model.clone(),
            include_mcp_json: value.include_mcp_json.is_some_and(|f| f),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_agent() -> crate::Result<()> {
        let agent = Agent {
            name: "test".to_string(),
            ..Default::default()
        };
        assert_eq!("test", format!("{agent}"));

        let kg_agent = KdlAgent::default();
        let agent = Agent::try_from(&kg_agent)?;
        assert_eq!(agent.tools, Agent::default().tools);

        Ok(())
    }
}
