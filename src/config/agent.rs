use {
    super::native::{AwsTool, ExecuteShellTool, NativeTools, ReadTool, WriteTool},
    crate::agent::{CustomToolConfig, Hook, hook::AgentHook},
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
    },
};

#[derive(Facet, Clone, Default)]
#[facet(default, deny_unknown_fields)]
pub struct KgAgent {
    #[facet(default)]
    pub name: String,
    /// Whether this agent is a template. Templates are not written to disk
    /// and serve only as parent configurations for other agents to inherit
    /// from. Template status is NEVER inherited - it must be explicitly
    /// declared.
    #[facet(default)]
    pub template: bool,
    pub description: Option<String>,
    #[facet(default)]
    pub inherits: HashSet<String>,
    pub prompt: Option<String>,
    #[facet(default)]
    pub resources: HashSet<String>,
    #[facet(default, rename = "includeMcpJson")]
    pub include_mcp_json: Option<bool>,
    #[facet(default)]
    pub tools: HashSet<String>,
    #[facet(default, rename = "allowedTools")]
    pub allowed_tools: HashSet<String>,
    pub model: Option<String>,
    #[facet(default)]
    pub hooks: HashMap<String, Hook>,
    #[facet(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, CustomToolConfig>,
    #[facet(default, rename = "toolAliases")]
    pub alias: HashMap<String, String>,
    #[facet(default, rename = "nativeTools")]
    pub native_tools: NativeTools,
    #[facet(default, rename = "toolSettings")]
    pub tool_settings: HashMap<String, facet_value::Value>,
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
    pub fn new(name: String, template: bool) -> Self {
        Self {
            name,
            template,
            ..Default::default()
        }
    }

    pub fn hooks(&self) -> HashMap<String, Vec<AgentHook>> {
        let mut result: HashMap<String, Vec<AgentHook>> = HashMap::new();
        for h in self.hooks.values() {
            result
                .entry(h.hook_type.clone())
                .and_modify(|e| {
                    e.push(AgentHook {
                        command: h.command.clone(),
                        matcher: h.matcher.clone(),
                    })
                })
                .or_insert(vec![AgentHook {
                    command: h.command.clone(),
                    matcher: h.matcher.clone(),
                }]);
        }
        result
    }

    pub fn get_tool_aws(&self) -> &AwsTool {
        &self.native_tools.aws
    }

    pub fn get_tool_read(&self) -> &ReadTool {
        &self.native_tools.read
    }

    pub fn get_tool_write(&self) -> &WriteTool {
        &self.native_tools.write
    }

    pub fn get_tool_shell(&self) -> &ExecuteShellTool {
        &self.native_tools.shell
    }
}
