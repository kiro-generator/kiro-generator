use {
    super::{
        GenericItem,
        hook::{HookDoc, HookPart},
        mcp::CustomToolConfigDoc,
        native::{AwsTool, ExecuteShellTool, NativeTools, NativeToolsDoc, ReadTool, WriteTool},
    },
    crate::{
        agent::CustomToolConfig,
        config::{GenericSet, GenericVec, split_newline},
    },
    facet::Facet,
    facet_kdl as kdl,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        hash::Hash,
    },
};

#[derive(Facet, Clone, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ToolAliasKdl {
    #[facet(default)]
    #[facet(kdl::argument)]
    from: String,
    #[facet(kdl::argument)]
    to: String,
}

#[derive(Facet, Clone, Debug)]
pub struct ToolSetting {
    #[facet(kdl::argument)]
    name: String,
    #[facet(kdl::child)]
    json: Json,
}

#[derive(Facet, Clone, Debug)]
struct Json {
    #[facet(kdl::argument)]
    value: String,
}

impl ToolSetting {
    #[allow(dead_code)]
    fn to_value(&self) -> crate::Result<(String, serde_json::Value)> {
        todo!()
        // let v: serde_json::Value = serde_json::from_str(&self.json.value)
        //     .wrap_err_with(|| format!("Failed to parse JSON for tool-setting
        // '{}'", self.name))?;
        //
        // if !v.is_object() {
        //     return Err(crate::format_err!(
        //         "tool-setting '{}' must be a JSON object, got: {}",
        //         self.name,
        //         v
        //     ));
        // }
        //
        // Ok((self.name.clone(), v))
    }
}

#[derive(Clone, Default)]
pub struct KdlAgent {
    pub name: String,
    pub template: Option<bool>,
    pub description: Option<String>,
    pub inherits: HashSet<String>,
    pub prompt: Option<String>,
    pub resources: HashSet<String>,
    pub include_mcp_json: Option<bool>,
    pub tools: HashSet<String>,
    pub allowed_tools: HashSet<String>,
    pub model: Option<String>,
    pub hook: HookPart,
    pub mcp: HashMap<String, CustomToolConfig>,
    pub alias: HashMap<String, String>,
    pub native_tool: NativeTools,
    pub tool_setting: Vec<ToolSetting>,
}

#[derive(Facet, Clone, Default)]
#[facet(rename_all = "kebab-case", default)]
pub struct KdlAgentDoc {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::property, default)]
    pub template: Option<bool>,

    #[facet(kdl::child, default)]
    pub(super) description: Option<String>,

    #[facet(kdl::child, default)]
    pub(super) inherits: GenericSet,

    #[facet(kdl::child, default)]
    pub(super) prompt: Option<String>,

    #[facet(kdl::children, default)]
    pub(super) resources: Vec<GenericItem>,

    #[facet(kdl::property, default)]
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

impl Debug for KdlAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for KdlAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<KdlAgentDoc> for KdlAgent {
    fn from(value: KdlAgentDoc) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
            prompt: value.prompt.clone(),
            alias: value.tool_aliases(),
            allowed_tools: value.allowed_tools(),
            inherits: value.inherits(),
            template: value.template,
            include_mcp_json: value.include_mcp_json,
            hook: value.hooks(),
            resources: value.resources(),
            model: value.model.clone(),
            mcp: value.mcp_servers(),
            tools: value.tools(),
            tool_setting: Default::default(), // TODO use facet::Value
            native_tool: value.native_tool.into(),
        }
    }
}

impl KdlAgent {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn is_template(&self) -> bool {
        self.template.is_some_and(|f| f)
    }

    pub fn get_tool_aws(&self) -> &AwsTool {
        &self.native_tool.aws
    }

    pub fn get_tool_read(&self) -> &ReadTool {
        &self.native_tool.read
    }

    pub fn get_tool_write(&self) -> &WriteTool {
        &self.native_tool.write
    }

    pub fn get_tool_shell(&self) -> &ExecuteShellTool {
        &self.native_tool.shell
    }
}

impl KdlAgentDoc {
    pub fn tool_aliases(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        for a in &self.alias {
            let m = HashMap::from(a.clone());
            map.extend(m);
        }
        map
    }

    pub fn hooks(&self) -> HookPart {
        HookPart::from(self.hook.clone().unwrap_or_default())
    }

    pub fn allowed_tools(&self) -> HashSet<String> {
        self.allowed_tools.item.clone()
    }

    pub fn tools(&self) -> HashSet<String> {
        self.tools.item.clone()
    }

    pub fn inherits(&self) -> HashSet<String> {
        self.inherits.item.clone()
    }

    pub fn resources(&self) -> HashSet<String> {
        split_newline(self.resources.clone())
    }

    pub fn mcp_servers(&self) -> HashMap<String, CustomToolConfig> {
        self.mcp
            .iter()
            .map(|m| (m.name.clone(), m.into()))
            .collect()
    }

    pub fn extra_tool_settings(&self) -> crate::Result<HashMap<String, serde_json::Value>> {
        Ok(HashMap::new())
        // for setting in &self.tool_setting {
        //     let (name, value) = setting.to_value()?;
        //     if result.contains_key(&name) {
        //         return Err(color_eyre::eyre::eyre!(
        //             "[{self}] - Duplicate tool-setting '{}' found. Each
        // tool-setting name must be \              unique.",
        //             name
        //         ));
        //     }
        //     result.insert(name, value);
        // }
    }
}
