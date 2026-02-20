use {
    super::{
        KgCustomToolConfig,
        KgKnowledge,
        SubagentConfig,
        native::{AwsTool, ExecuteShellTool, NativeTools, ReadTool, WriteTool},
    },
    crate::kiro::{KgHook, hook::AgentHook},
    color_eyre::eyre::WrapErr,
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
    },
};

#[derive(Facet, Clone, Default)]
#[facet(default, deny_unknown_fields)]
pub struct Manifest {
    /// Agent name (derived from filename if not specified)
    #[facet(default)]
    pub name: String,
    /// Whether this agent is a template. Templates are not written to disk
    /// and serve only as parent configurations for other agents to inherit
    /// from. Template status is NEVER inherited - it must be explicitly
    /// declared.
    #[facet(default)]
    pub template: bool,
    /// Human-readable description of the agent's purpose
    pub description: Option<String>,
    /// List of parent agents to inherit configuration from
    #[facet(default)]
    pub inherits: HashSet<String>,
    /// High-level context for the agent (system prompt)
    pub prompt: Option<String>,
    /// Files and URLs to include as context (supports file:// and https:// schemes)
    #[facet(default)]
    pub resources: HashSet<String>,
    /// Knowledge bases for semantic search (converted to resources array in
    /// JSON)
    #[facet(default)]
    pub knowledge: HashMap<String, KgKnowledge>,
    /// Whether to include MCP servers from mcp.json files (maps to
    /// includeMcpJson in JSON)
    #[facet(default, rename = "useLegacyMcpJson")]
    pub include_mcp_json: Option<bool>,
    /// Tools available to the agent (use ["*"] for all tools)
    #[facet(default)]
    pub tools: HashSet<String>,
    /// Tools that don't require user permission prompts (maps to allowedTools
    /// in JSON)
    #[facet(default, rename = "allowedTools")]
    pub allowed_tools: HashSet<String>,
    /// Model ID to use for this agent
    pub model: Option<String>,
    /// Lifecycle hooks (commands run at specific trigger points)
    #[facet(default)]
    pub hooks: HashMap<String, HashMap<String, KgHook>>,
    /// MCP server configurations (maps to mcpServers in JSON)
    #[facet(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, KgCustomToolConfig>,
    /// Tool name remapping for handling naming collisions (maps to toolAliases
    /// in JSON)
    #[facet(default, rename = "toolAliases")]
    pub tool_aliases: HashMap<String, String>,
    /// Native tool permissions (maps to toolsSettings in JSON)
    #[facet(default, rename = "nativeTools")]
    pub native_tools: NativeTools,
    /// Additional tool settings merged with nativeTools (maps to toolsSettings
    /// in JSON)
    #[facet(default, rename = "toolSettings")]
    pub tool_settings: HashMap<String, facet_value::Value>,

    /// Keyboard shortcut for swapping to this agent (e.g., "ctrl+shift+a",
    /// "shift+tab")
    #[facet(default, rename = "keyboardShortcut")]
    pub keyboard_shortcut: Option<String>,
    /// Welcome message displayed when switching to this agent
    #[facet(default, rename = "welcomeMessage")]
    pub welcome_message: Option<String>,
    /// Subagent permissions (maps to toolsSettings.subagent in JSON)
    #[facet(default)]
    pub subagents: SubagentConfig,
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
        for (hook_type, hooks_map) in &self.hooks {
            let mut hooks_vec = Vec::new();
            for hook in hooks_map.values() {
                hooks_vec.push(AgentHook {
                    command: hook.command.clone(),
                    matcher: hook.matcher.clone(),
                    timeout_ms: hook.timeout_ms,
                    max_output_size: hook.max_output_size,
                    cache_ttl_seconds: hook.cache_ttl_seconds,
                });
            }
            result.insert(hook_type.clone(), hooks_vec);
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
        let mut result: Vec<facet_value::Value> = Vec::new();

        for r in &self.resources {
            result.push(facet_value::value!(r));
        }

        for (name, kb) in &self.knowledge {
            let k = crate::kiro::Knowledge {
                name: name.clone(),
                knowledge_type: "knowledgeBase".to_string(),
                description: kb.description.clone(),
                source: kb.source.clone(),
                index_type: kb.index_type.clone(),
                auto_update: kb.auto_update,
            };
            result.push(
                facet_value::to_value(&k)
                    .wrap_err_with(|| format!("Failed to serialize knowledge base '{name}'"))?,
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_new() {
        let m = Manifest::new("test-agent".to_string(), true);
        assert_eq!(m.name, "test-agent");
        assert!(m.template);
    }

    #[test]
    fn manifest_resources() -> crate::Result<()> {
        let mut m = Manifest::default();
        m.resources.insert("file://README.md".to_string());
        let res = m.resources()?;
        assert_eq!(res.len(), 1);
        Ok(())
    }
}
