use {
    super::{
        KgCustomToolConfig,
        KgKnowledge,
        Result,
        SubagentConfig,
        manifest::*,
        native::NativeTools,
    },
    crate::{Fs, kiro::KgHook},
    facet::Facet,
    kg_macro::kg_schema,
    std::{
        collections::{HashMap, HashSet},
        path::Path,
    },
};

/// Definition for kg agents, .kiro/generators/agents, excluding inheritance
#[kg_schema]
pub struct KgAgentFileDoc {
    #[facet(rename = "$schema")]
    pub schema: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/description",
        description = "This field is not model facing and is mostly here for users to discern \
                       between agents"
    )]
    pub description: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/prompt",
        description = "The intention for this field is to provide high level context to the \
                       agent. This should be seen as the same category of context as a system \
                       prompt."
    )]
    pub prompt: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/resources",
        description = "Files to include in the agent's context"
    )]
    #[facet(default)]
    pub resources: HashSet<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/resources",
        description = "Knowledge bases to include in the agent's context"
    )]
    #[facet(default)]
    pub knowledge: HashMap<String, KgKnowledge>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/useLegacyMcpJson",
        description = "Whether or not to include the legacy global MCP configuration in the \
                       agent. You can reference tools brought in by these servers just as you \
                       would with the servers you configure in the mcpServers field in this \
                       config."
    )]
    #[facet(rename = "useLegacyMcpJson")]
    pub include_mcp_json: Option<bool>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/tools",
        description = "List of tools the agent can see. Use \"@{MCP_SERVER_NAME}/tool_name\" to \
                       specify tools from mcp servers. To include all tools from a server, use \
                       \"@{MCP_SERVER_NAME}\""
    )]
    #[facet(default)]
    pub tools: HashSet<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/allowedTools",
        description = "List of tools the agent is explicitly allowed to use"
    )]
    #[facet(default, rename = "allowedTools")]
    pub allowed_tools: HashSet<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/model",
        description = "The model ID to use for this agent. If not specified, uses the default \
                       model."
    )]
    pub model: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/hooks",
        description = "Commands to run when a chat session is created"
    )]
    #[facet(default)]
    pub hooks: HashMap<String, HashMap<String, KgHook>>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/mcpServers",
        description = "Configuration for Model Context Protocol (MCP) servers"
    )]
    #[facet(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, KgCustomToolConfig>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/toolAliases",
        description = "Tool aliases for remapping tool names"
    )]
    #[facet(default, rename = "toolAliases")]
    pub tool_aliases: HashMap<String, String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/toolsSettings",
        description = "Native tool settings (shell, read, write, etc.) merged into toolsSettings."
    )]
    #[facet(default, rename = "nativeTools")]
    pub native_tools: NativeTools,
    #[kg_mapping(
        kiro_schema_path = "#/properties/toolsSettings",
        description = "Additional toolsSettings entries merged into toolsSettings."
    )]
    #[facet(default, rename = "toolSettings")]
    pub tool_settings: HashMap<String, facet_value::Value>,

    #[kg_mapping(
        kiro_schema_path = "#/properties/keyboardShortcut",
        description = "Keyboard shortcut for swapping to this agent (e.g., \"ctrl+shift+a\", \
                       \"shift+tab\")"
    )]
    #[facet(rename = "keyboardShortcut")]
    pub keyboard_shortcut: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/welcomeMessage",
        description = "Welcome message displayed when switching to this agent"
    )]
    #[facet(rename = "welcomeMessage")]
    pub welcome_message: Option<String>,
    #[kg_mapping(
        kiro_schema_path = "#/properties/toolsSettings",
        description = "Allow/deny lists for subagents, emitted into toolsSettings.subagent."
    )]
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
