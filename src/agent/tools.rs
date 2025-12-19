use {
    crate::merging_format::MergedSet,
    serde::{Deserialize, Serialize},
    std::{collections::HashSet, fmt::Display},
};

pub trait ToolMerge<T: Serialize + Default> {
    fn merge(self, other: T) -> Self;
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, enum_iterator::Sequence,
)]
#[serde(rename_all = "camelCase")]
pub enum ToolTarget {
    Aws,
    Shell,
    Read,
    Write,
}

impl Display for ToolTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolTarget::Aws => write!(f, "aws"),
            ToolTarget::Shell => write!(f, "shell"),
            ToolTarget::Read => write!(f, "read"),
            ToolTarget::Write => write!(f, "write"),
        }
    }
}

impl AsRef<str> for ToolTarget {
    fn as_ref(&self) -> &str {
        match self {
            ToolTarget::Aws => "aws",
            ToolTarget::Shell => "shell",
            ToolTarget::Read => "read",
            ToolTarget::Write => "write",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MergingAwsTool {
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub allowed_services: MergedSet,
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub denied_services: MergedSet,
    #[serde(default)]
    pub auto_allow_readonly: bool,
}

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AwsTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_services: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_services: HashSet<String>,
    #[serde(default)]
    pub auto_allow_readonly: bool,
}

impl From<MergingAwsTool> for AwsTool {
    fn from(tool: MergingAwsTool) -> Self {
        Self {
            allowed_services: tool.allowed_services.into(),
            denied_services: tool.denied_services.into(),
            auto_allow_readonly: tool.auto_allow_readonly,
        }
    }
}

impl ToolMerge<MergingAwsTool> for MergingAwsTool {
    fn merge(mut self, parent: MergingAwsTool) -> Self {
        // MergedSet already handles merging via MergingTomlFormat
        // Just handle boolean flag
        self.auto_allow_readonly |= parent.auto_allow_readonly;
        self
    }
}

impl Default for MergingAwsTool {
    fn default() -> Self {
        Self {
            allowed_services: MergedSet::default(),
            denied_services: MergedSet::default(),
            auto_allow_readonly: default_allow_read_only(),
        }
    }
}

fn default_allow_read_only() -> bool {
    false
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MergingExecuteShellTool {
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub allowed_commands: MergedSet,
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub denied_commands: MergedSet,
    #[serde(default)]
    pub deny_by_default: bool,
    #[serde(default = "default_allow_read_only")]
    pub auto_allow_readonly: bool,

    #[serde(default)]
    pub force_allowed_commands: MergedSet,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteShellTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_commands: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_commands: HashSet<String>,
    #[serde(default)]
    pub deny_by_default: bool,
    #[serde(default = "default_allow_read_only")]
    pub auto_allow_readonly: bool,
}

impl From<MergingExecuteShellTool> for ExecuteShellTool {
    fn from(tool: MergingExecuteShellTool) -> Self {
        ExecuteShellTool {
            allowed_commands: tool.allowed_commands.into(),
            denied_commands: tool.denied_commands.into(),
            deny_by_default: tool.deny_by_default,
            auto_allow_readonly: tool.auto_allow_readonly,
        }
    }
}

impl ToolMerge<MergingExecuteShellTool> for MergingExecuteShellTool {
    fn merge(mut self, parent: MergingExecuteShellTool) -> Self {
        // MergedSet already handles merging via MergingTomlFormat
        // Just handle force_allowed_commands override
        if !self.force_allowed_commands.is_empty() {
            tracing::trace!(
                "Applying execute_shell.forceAllowedCommands: {:?}",
                self.force_allowed_commands.iter().collect::<Vec<_>>()
            );
            for cmd in self.force_allowed_commands.iter() {
                self.allowed_commands.insert(cmd.clone());
                if !self.denied_commands.remove(cmd) {
                    tracing::trace!("Removed command from denied_commands: {}", cmd);
                }
            }
        }

        // Handle boolean flags
        self.auto_allow_readonly |= parent.auto_allow_readonly;
        self.deny_by_default |= parent.deny_by_default;
        self
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MergingReadTool {
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub allowed_paths: MergedSet,
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub denied_paths: MergedSet,
    #[serde(default)]
    pub force_allowed_paths: MergedSet,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_paths: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_paths: HashSet<String>,
}

impl From<MergingReadTool> for ReadTool {
    fn from(tool: MergingReadTool) -> Self {
        Self {
            allowed_paths: tool.allowed_paths.into(),
            denied_paths: tool.denied_paths.into(),
        }
    }
}

impl ToolMerge<MergingReadTool> for MergingReadTool {
    fn merge(mut self, _parent: MergingReadTool) -> Self {
        // MergedSet already handles merging via MergingTomlFormat
        // Just handle force_allowed_paths override
        if !self.force_allowed_paths.is_empty() {
            tracing::trace!(
                "Applying fs_read.forceAllowedPaths: {:?}",
                self.force_allowed_paths.iter().collect::<Vec<_>>()
            );
            for path in self.force_allowed_paths.iter() {
                self.allowed_paths.insert(path.clone());
                self.denied_paths.remove(path);
            }
        }
        self
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MergingWriteTool {
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub allowed_paths: MergedSet,
    #[serde(default, skip_serializing_if = "MergedSet::is_empty")]
    pub denied_paths: MergedSet,
    #[serde(default, skip_serializing)]
    pub force_allowed_paths: MergedSet,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteTool {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub allowed_paths: HashSet<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub denied_paths: HashSet<String>,
}

impl From<MergingWriteTool> for WriteTool {
    fn from(tool: MergingWriteTool) -> Self {
        Self {
            allowed_paths: tool.allowed_paths.into(),
            denied_paths: tool.denied_paths.into(),
        }
    }
}

impl ToolMerge<MergingWriteTool> for MergingWriteTool {
    fn merge(mut self, _parent: MergingWriteTool) -> Self {
        // MergedSet already handles merging via MergingTomlFormat
        // Just handle force_allowed_paths override
        if !self.force_allowed_paths.is_empty() {
            tracing::trace!(
                "Applying fs_write.forceAllowedPaths: {:?}",
                self.force_allowed_paths.iter().collect::<Vec<_>>()
            );
            for path in self.force_allowed_paths.iter() {
                self.allowed_paths.insert(path.clone());
                self.denied_paths.remove(path);
            }
        }
        self
    }
}
