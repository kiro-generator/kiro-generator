use {
    super::native::{AwsTool, ExecuteShellTool, NativeTools, ReadTool, WriteTool},
    crate::{
        agent::{CustomToolConfig, KgHook, hook::AgentHook},
        config::Knowledge,
    },
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
    },
};

#[derive(Facet, Clone, Default)]
#[facet(default, deny_unknown_fields)]
pub struct Manifest {
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
    #[facet(default)]
    pub knowledge: HashMap<String, Knowledge>,
    #[facet(default, rename = "useLegacyMcpJson")]
    pub include_mcp_json: Option<bool>,
    #[facet(default)]
    pub tools: HashSet<String>,
    #[facet(default, rename = "allowedTools")]
    pub allowed_tools: HashSet<String>,
    pub model: Option<String>,
    #[facet(default)]
    pub hooks: HashMap<String, KgHook>,
    #[facet(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, CustomToolConfig>,
    #[facet(default, rename = "toolAliases")]
    pub alias: HashMap<String, String>,
    #[facet(default, rename = "nativeTools")]
    pub native_tools: NativeTools,
    #[facet(default, rename = "toolSettings")]
    pub tool_settings: HashMap<String, facet_value::Value>,

    /// Keyboard shortcut for swapping to this agent (e.g., "ctrl+shift+a",
    /// "shift+tab")
    #[facet(default, rename = "keyboardShortcut")]
    pub keyboard_shortcut: Option<String>,
    /// Welcome message displayed when switching to this agent
    #[facet(default, rename = "welcomeMessage")]
    pub welcome_message: Option<String>,
}

impl Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Manifest {
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
                        timeout_ms: h.timeout_ms,
                        max_output_size: h.max_output_size,
                        cache_ttl_seconds: h.cache_ttl_seconds,
                    })
                })
                .or_insert(vec![AgentHook {
                    command: h.command.clone(),
                    matcher: h.matcher.clone(),
                    timeout_ms: h.timeout_ms,
                    max_output_size: h.max_output_size,
                    cache_ttl_seconds: h.cache_ttl_seconds,
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

    pub fn resources(&self) -> crate::Result<Vec<facet_value::Value>> {
        let mut result = Vec::new();

        for r in &self.resources {
            result.push(facet_value::Value::from(r.as_str()));
        }

        for (name, kb) in &self.knowledge {
            let source = kb.source.as_ref().ok_or_else(|| {
                color_eyre::eyre::eyre!("Knowledge '{}' missing required 'source' field", name)
            })?;

            let mut obj = facet_value::VObject::new();
            obj.insert(
                facet_value::VString::from("type"),
                facet_value::Value::from("knowledgeBase"),
            );
            obj.insert(
                facet_value::VString::from("name"),
                facet_value::Value::from(name.as_str()),
            );
            obj.insert(
                facet_value::VString::from("source"),
                facet_value::Value::from(source.as_str()),
            );

            if let Some(desc) = &kb.description {
                obj.insert(
                    facet_value::VString::from("description"),
                    facet_value::Value::from(desc.as_str()),
                );
            }
            if let Some(idx) = &kb.index_type {
                obj.insert(
                    facet_value::VString::from("indexType"),
                    facet_value::Value::from(idx.as_str()),
                );
            }
            if let Some(auto) = kb.auto_update {
                obj.insert(
                    facet_value::VString::from("autoUpdate"),
                    facet_value::Value::from(auto),
                );
            }

            result.push(facet_value::Value::from(obj));
        }

        Ok(result)
    }
}
