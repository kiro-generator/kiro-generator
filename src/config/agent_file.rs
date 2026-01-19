use {
    super::agent::*,
    crate::{
        Fs,
        agent::{CustomToolConfig, Hook},
        config::{ConfigResult, native::NativeTools},
    },
    facet::Facet,
    std::{
        collections::{HashMap, HashSet},
        path::Path,
    },
};

#[derive(Facet, Clone, Default)]
#[facet(deny_unknown_fields, default)]
pub struct KgAgentFileDoc {
    pub description: Option<String>,
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

impl KgAgent {
    pub fn from_path(
        fs: &Fs,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
        template: bool,
    ) -> Option<ConfigResult<Self>> {
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
            include_mcp_json: file_source.include_mcp_json,
            tools: file_source.tools,
            allowed_tools: file_source.allowed_tools,
            hooks: file_source.hooks,
            model: file_source.model,
            alias: file_source.alias,
            native_tools: file_source.native_tools,
            tool_settings: file_source.tool_settings,
            mcp_servers: file_source.mcp_servers,
        }
    }
}
