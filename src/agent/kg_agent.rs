use {
    super::hook::{Hook, HookTrigger},
    crate::{
        agent::{
            Agent,
            AwsTool,
            ExecuteShellTool,
            MergingAwsTool,
            MergingExecuteShellTool,
            MergingReadTool,
            MergingWriteTool,
            OriginalToolName,
            ReadTool,
            ToolTarget,
            WriteTool,
            mcp_config::MergingMcpServerConfig,
        },
        merging_format::MergedSet,
    },
    serde::{Deserialize, Serialize, de::DeserializeOwned},
    std::{
        collections::HashMap,
        fmt::{Debug, Display},
    },
    tracing::debug,
};
#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KgAgent {
    /// Name of the agent
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// This field is not model facing and is mostly here for users to discern
    /// between agents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The intention for this field is to provide high level context to the
    /// agent. This should be seen as the same category of context as a system
    /// prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Configuration for Model Context Protocol (MCP) servers
    #[serde(default)]
    pub mcp_servers: MergingMcpServerConfig,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub tools: MergedSet,
    /// Tool aliases for remapping tool names
    #[serde(default)]
    pub tool_aliases: HashMap<OriginalToolName, String>,
    /// List of tools the agent is explicitly allowed to use
    #[serde(
        default,
        rename = "allowedTools",
        skip_serializing_if = "MergedSet::is_empty"
    )]
    pub allowed_tools: MergedSet,
    /// Files to include in the agent's context
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub resources: MergedSet,
    /// Commands to run when a chat session is created
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub hooks: HashMap<HookTrigger, HashMap<String, Hook>>,
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
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub inherits: MergedSet,
    /// Whether to generate a real agent config file or not.
    #[serde(default)]
    pub skeleton: bool,
}

impl Default for KgAgent {
    fn default() -> Self {
        let default_agent = Agent::default();
        Self {
            name: "kiro_default".to_string(),
            description: None,
            allowed_tools: default_agent.allowed_tools.into(),
            tools_settings: Default::default(),
            model: None,
            include_mcp_json: false,
            inherits: MergedSet::default(),
            skeleton: false,
            mcp_servers: MergingMcpServerConfig::default(),
            hooks: Default::default(),
            prompt: default_agent.prompt,
            tool_aliases: default_agent.tool_aliases,
            tools: default_agent.tools.into(),
            resources: default_agent.resources.into(),
        }
    }
}

impl Debug for KgAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for KgAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl KgAgent {
    pub fn skeleton(&self) -> bool {
        self.skeleton
    }

    pub fn get_tool_aws(&self) -> MergingAwsTool {
        self.get_tool(ToolTarget::Aws)
    }

    pub fn get_tool_read(&self) -> MergingReadTool {
        self.get_tool(ToolTarget::Read)
    }

    pub fn get_tool_write(&self) -> MergingWriteTool {
        self.get_tool(ToolTarget::Write)
    }

    pub fn get_tool_shell(&self) -> MergingExecuteShellTool {
        self.get_tool(ToolTarget::Shell)
    }

    pub fn get_tool<T: DeserializeOwned + Default>(&self, tool: ToolTarget) -> T {
        match self.tools_settings.get(tool.as_ref()) {
            Some(value) => match serde_json::from_value(value.clone()) {
                Ok(settings) => settings,
                Err(e) => {
                    debug!("Failed to deserialize tool settings for {tool}: {e}");
                    T::default()
                }
            },
            None => {
                tracing::trace!("No tool settings found for {tool}");
                T::default()
            }
        }
    }

    pub fn set_tool<T: Serialize>(&mut self, tool: ToolTarget, settings: T) {
        match serde_json::to_value(settings) {
            Ok(value) => {
                self.tools_settings.insert(tool.to_string(), value);
            }
            Err(e) => {
                tracing::warn!("Failed to serialize tool settings for agent {self} {tool}: {e}");
            }
        };
    }
}

impl From<KgAgent> for Agent {
    fn from(me: KgAgent) -> Self {
        Agent::from(&me)
    }
}

impl From<&KgAgent> for Agent {
    fn from(me: &KgAgent) -> Self {
        let mut kiro_tool_settings: HashMap<ToolTarget, serde_json::Value> = HashMap::new();
        let tools: Vec<ToolTarget> = enum_iterator::all::<ToolTarget>().collect();
        for tool in tools {
            if me.tools_settings.contains_key(tool.as_ref()) {
                let result: Option<serde_json::Value> = match tool {
                    ToolTarget::Aws => {
                        let t: MergingAwsTool = me.get_tool(tool);
                        let t: AwsTool = t.into();
                        serde_json::to_value(&t).ok()
                    }
                    ToolTarget::Shell => {
                        let t: MergingExecuteShellTool = me.get_tool(tool);
                        let t: ExecuteShellTool = t.into();
                        serde_json::to_value(&t).ok()
                    }
                    ToolTarget::Read => {
                        let t: MergingReadTool = me.get_tool(tool);
                        let t: ReadTool = t.into();
                        serde_json::to_value(&t).ok()
                    }
                    ToolTarget::Write => {
                        let t: MergingWriteTool = me.get_tool(tool);
                        let t: WriteTool = t.into();
                        serde_json::to_value(&t).ok()
                    }
                };
                if let Some(value) = result {
                    kiro_tool_settings.insert(tool, value);
                } else {
                    tracing::warn!("failed to serialize tool {tool}")
                }
            }
        }
        let mut hooks: HashMap<HookTrigger, Vec<Hook>> = HashMap::new();
        for (k, v) in me.hooks.iter() {
            hooks.insert(*k, v.values().cloned().collect());
        }
        Self {
            name: me.name.clone(),
            description: me.description.clone(),
            tools: me.tools.clone().into(),
            prompt: me.prompt.clone(),
            mcp_servers: me.mcp_servers.clone().into(),
            tool_aliases: me.tool_aliases.clone(),
            allowed_tools: me.allowed_tools.clone().into(),
            resources: me.resources.clone().into(),
            hooks,
            tools_settings: kiro_tool_settings,
            include_mcp_json: me.include_mcp_json,
            model: me.model.clone(),
        }
    }
}
