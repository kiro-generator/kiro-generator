use {
    super::{KgKnowledge, Result, SubagentConfig, manifest::*, native::NativeTools},
    crate::{
        Fs,
        kiro::{CustomToolConfig, KgHook},
    },
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        path::Path,
    },
};

#[derive(Facet, Clone, Default)]
#[facet(deny_unknown_fields, skip_all_unless_truthy, default)]
pub struct KgAgentFileDoc {
    #[facet(default, rename = "$schema")]
    pub schema: Option<String>,
    pub description: Option<String>,
    pub prompt: Option<String>,
    #[facet(default)]
    pub resources: HashSet<String>,
    #[facet(default)]
    pub knowledge: HashMap<String, KgKnowledge>,
    #[facet(default, rename = "useLegacyMcpJson")]
    pub include_mcp_json: Option<bool>,
    #[facet(default)]
    pub tools: HashSet<String>,
    #[facet(default, rename = "allowedTools")]
    pub allowed_tools: HashSet<String>,
    pub model: Option<String>,
    #[facet(default)]
    pub hooks: HashMap<String, HashMap<String, KgHook>>,
    #[facet(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, CustomToolConfig>,
    #[facet(default, rename = "toolAliases")]
    pub tool_aliases: HashMap<String, String>,
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
    #[facet(default)]
    pub subagents: SubagentConfig,
}

impl Manifest {
    pub fn from_path(
        fs: &Fs,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
        template: bool,
    ) -> Option<Result<Self>> {
        if let Some(result) = super::toml_parse_path::<KgAgentFileDoc>(fs, path) {
            match result {
                Err(e) => return Some(Err(e)),
                Ok(file_source) => {
                    return Some(Ok(Self::from_file_source(name, file_source, template)));
                }
            }
        };
        None
    }

    pub fn from_file_source(
        name: impl AsRef<str>,
        file_source: KgAgentFileDoc,
        template: bool,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            description: file_source.description,
            template,
            inherits: Default::default(),
            prompt: file_source.prompt,
            resources: file_source.resources,
            knowledge: file_source.knowledge,
            include_mcp_json: file_source.include_mcp_json,
            tools: file_source.tools,
            allowed_tools: file_source.allowed_tools,
            hooks: file_source.hooks,
            model: file_source.model,
            tool_aliases: file_source.tool_aliases,
            native_tools: file_source.native_tools,
            tool_settings: file_source.tool_settings,
            mcp_servers: file_source.mcp_servers,
            keyboard_shortcut: file_source.keyboard_shortcut,
            welcome_message: file_source.welcome_message,
            subagents: file_source.subagents,
        }
    }
}
