mod custom_tool;
pub mod diff;
pub mod hook;
pub mod tools;
pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://README.md", "file://AGENTS.md"];
pub const DEFAULT_APPROVE: [&str; 0] = [];
use {
    crate::{Manifest, Result, kiro::hook::AgentHook},
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        fmt::Display,
    },
};
pub use {custom_tool::CustomToolConfig, hook::KgHook, tools::*};

#[derive(Facet, Debug, Clone, Eq, PartialEq)]
pub struct KiroAgent {
    /// Name of the agent
    pub name: String,
    /// This field is not model facing and is mostly here for users to discern
    /// between agents
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub description: Option<String>,
    /// The intention for this field is to provide high level context to the
    /// agent. This should be seen as the same category of context as a system
    /// prompt.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub prompt: Option<String>,
    /// Configuration for Model Context Protocol (MCP) servers
    #[facet(default, rename = "mcpServers", skip_serializing_if = HashMap::is_empty)]
    pub mcp_servers: HashMap<String, CustomToolConfig>,
    /// List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to
    /// specify tools from mcp servers. To include all tools from a server,
    /// use \"@{MCP_SERVER_NAME}\"
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub tools: HashSet<String>,
    /// Tool aliases for remapping tool names
    #[facet(default, rename = "toolAliases", skip_serializing_if = HashMap::is_empty)]
    pub tool_aliases: HashMap<String, String>,
    /// List of tools the agent is explicitly allowed to use
    #[facet(default, rename = "allowedTools", skip_serializing_if = HashSet::is_empty)]
    pub allowed_tools: HashSet<String>,
    /// Files to include in the agent's context
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub resources: Vec<facet_value::Value>,
    /// Commands to run when a chat session is created
    #[facet(default, skip_serializing_if = HashMap::is_empty)]
    pub hooks: HashMap<String, Vec<AgentHook>>,
    /// Settings for specific tools. These are mostly for native tools. The
    /// actual schema differs by tools and is documented in detail in our
    /// documentation
    #[facet(default, rename = "toolsSettings", skip_serializing_if = HashMap::is_empty)]
    pub tools_settings: HashMap<String, facet_value::Value>,
    /// The model ID to use for this agent. If not specified, uses the default
    /// model.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub model: Option<String>,
    #[facet(default, rename = "useLegacyMcpJson")]
    pub include_mcp_json: bool,
}

impl Display for KiroAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl KiroAgent {
    pub fn validate(&self) -> Result<()> {
        let schema: serde_json::Value = serde_json::from_str(crate::schema::SCHEMA)?;
        let validator = jsonschema::validator_for(&schema)?;
        let instance = serde_json::from_str(&facet_json::to_string(&self)?)?;

        if let Err(e) = validator.validate(&instance) {
            return Err(crate::format_err!(
                "Validation error: {}\n{}",
                e,
                facet_json::to_string(&self).unwrap_or_default()
            ));
        }
        Ok(())
    }
}

impl TryFrom<&Manifest> for KiroAgent {
    type Error = color_eyre::Report;

    fn try_from(value: &Manifest) -> std::result::Result<Self, Self::Error> {
        let native_tools = &value.native_tools;
        let mut tools_settings = HashMap::new();

        let tool: AwsTool = native_tools.into();
        let tool_name = ToolTarget::Aws.to_string();
        if tool != AwsTool::default() {
            let v: facet_value::Value = facet_json::from_str(&facet_json::to_string(&tool)?)?;
            tools_settings.insert(tool_name.to_string(), v);
        }
        let tool: ReadTool = native_tools.into();
        let tool_name = ToolTarget::Read.to_string();
        if tool != ReadTool::default() {
            let v: facet_value::Value = facet_json::from_str(&facet_json::to_string(&tool)?)?;
            tools_settings.insert(tool_name.to_string(), v);
        }
        let tool: WriteTool = native_tools.into();
        let tool_name = ToolTarget::Write.to_string();
        if tool != WriteTool::default() {
            let v: facet_value::Value = facet_json::from_str(&facet_json::to_string(&tool)?)?;
            tools_settings.insert(tool_name.to_string(), v);
        }
        let tool: ExecuteShellTool = native_tools.into();
        let tool_name = ToolTarget::Shell.to_string();
        if tool != ExecuteShellTool::default() {
            let v: facet_value::Value = facet_json::from_str(&facet_json::to_string(&tool)?)?;
            tools_settings.insert(tool_name.to_string(), v);
        }
        let default_agent = Self::default();
        let tools = value.tools.clone();
        let allowed_tools = value.allowed_tools.clone();
        let resources = value.resources()?;

        let extra_tool_settings = value.tool_settings.clone();
        tools_settings.extend(extra_tool_settings);

        Ok(Self {
            name: value.name.clone(),
            description: value.description.clone(),
            prompt: value.prompt.clone(),
            mcp_servers: value.mcp_servers.clone(),
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
            hooks: value.hooks(),
            tools_settings,
            model: value.model.clone(),
            include_mcp_json: value.include_mcp_json.is_some_and(|f| f),
        })
    }
}

impl Default for KiroAgent {
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
                let mut resources = Vec::new();
                for r in DEFAULT_AGENT_RESOURCES {
                    resources.push(facet_value::Value::from(*r));
                }
                resources
            },
            hooks: Default::default(),
            tools_settings: Default::default(),
            include_mcp_json: true,
            model: None,
        }
    }
}

#[derive(Facet, Clone, Debug, PartialEq, Eq, Hash)]
#[facet(deny_unknown_fields, rename_all = "camelCase")]
pub struct Knowledge {
    pub name: String,
    #[facet(rename = "type")]
    pub knowledge_type: String,
    #[facet(default)]
    pub source: Option<String>,
    #[facet(default)]
    pub description: Option<String>,
    #[facet(default)]
    pub index_type: Option<String>,
    #[facet(default)]
    pub auto_update: Option<bool>,
}

impl Display for Knowledge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name={},description={:?}", self.name, self.description)
    }
}
