use {
    super::{
        GenericItem,
        GenericSet,
        agent::*,
        hook::HookDoc,
        mcp::CustomToolConfigDoc,
        native::NativeToolsDoc,
    },
    crate::{
        Fs,
        config::{ConfigResult, GenericVec},
    },
    facet::Facet,
    facet_kdl as kdl,
    std::path::Path,
};

#[derive(Facet, Copy, Default, Clone, Debug, PartialEq, Eq)]
#[facet(default)]
pub(super) struct BoolDoc {
    #[facet(kdl::argument)]
    pub value: bool,
}
#[derive(Facet, Clone, Default)]
#[facet(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct KdlAgentFileDoc {
    #[facet(kdl::child, default)]
    pub(super) description: Option<String>,

    #[facet(kdl::child, default)]
    pub(super) inherits: GenericSet,

    #[facet(kdl::child, default)]
    pub(super) prompt: Option<String>,

    #[facet(kdl::children, default)]
    pub(super) resources: Vec<GenericItem>,

    #[facet(kdl::child, default)]
    pub include_mcp_json: Option<bool>,

    #[facet(kdl::child, rename = "tools", default)]
    pub(super) tools: GenericSet,

    #[facet(kdl::child, default)]
    pub(super) allowed_tools: GenericSet,

    #[facet(kdl::child, default)]
    pub(super) model: Option<String>,

    #[facet(kdl::child, default)]
    pub(super) hook: Option<HookDoc>,

    #[facet(kdl::children, default)]
    pub(super) mcp: Vec<CustomToolConfigDoc>,

    #[facet(kdl::children, default)]
    pub(super) alias: Vec<GenericVec>,

    #[facet(kdl::child, default)]
    pub native_tool: NativeToolsDoc,

    #[facet(kdl::children, default)]
    pub(super) tool_setting: Vec<ToolSetting>,
}

impl KdlAgentDoc {
    pub fn from_path(
        fs: &Fs,
        name: impl AsRef<str>,
        path: impl AsRef<Path>,
    ) -> Option<ConfigResult<Self>> {
        if let Some(result) = super::kdl_parse_path::<KdlAgentFileDoc>(fs, path) {
            match result {
                Err(e) => return Some(Err(e)),
                Ok(file_source) => return Some(Ok(Self::from_file_source(name, file_source))),
            }
        };
        None
    }

    pub fn from_file_source(name: impl AsRef<str>, file_source: KdlAgentFileDoc) -> Self {
        Self {
            name: name.as_ref().to_string(),
            description: file_source.description,
            template: None,
            inherits: file_source.inherits,
            prompt: file_source.prompt,
            resources: file_source.resources,
            include_mcp_json: file_source.include_mcp_json,
            tools: file_source.tools,
            allowed_tools: file_source.allowed_tools,
            model: file_source.model,
            hook: file_source.hook,
            mcp: file_source.mcp,
            alias: file_source.alias,
            native_tool: file_source.native_tool,
            tool_setting: file_source.tool_setting,
        }
    }
}
