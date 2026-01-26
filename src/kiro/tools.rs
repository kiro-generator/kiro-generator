use {
    facet::Facet,
    std::{collections::HashSet, fmt::Display},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, enum_iterator::Sequence)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Facet, PartialEq, Eq)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct AwsTool {
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_services: HashSet<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub denied_services: HashSet<String>,
    #[facet(default)]
    pub auto_allow_readonly: bool,
}

impl Default for AwsTool {
    fn default() -> Self {
        Self {
            allowed_services: Default::default(),
            denied_services: Default::default(),
            auto_allow_readonly: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Facet, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteShellTool {
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_commands: HashSet<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub denied_commands: HashSet<String>,
    #[facet(default)]
    pub deny_by_default: bool,
    pub auto_allow_readonly: bool,
}

impl Default for ExecuteShellTool {
    fn default() -> Self {
        Self {
            allowed_commands: Default::default(),
            denied_commands: Default::default(),
            deny_by_default: false,
            auto_allow_readonly: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Facet, Default, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadTool {
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_paths: HashSet<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub denied_paths: HashSet<String>,
}

#[allow(dead_code)]
#[derive(Debug, Facet, Default, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteTool {
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_paths: HashSet<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub denied_paths: HashSet<String>,
}

#[derive(Debug, Clone, Facet, Default, PartialEq, Eq)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubagentTool {
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_agents: HashSet<String>,
}

// Normalized variants for stable diffing (Vec instead of HashSet)

#[derive(Debug, Clone, Facet, PartialEq, Eq)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedAwsTool {
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_services: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub denied_services: Vec<String>,
    #[facet(default)]
    pub auto_allow_readonly: bool,
}

impl From<AwsTool> for NormalizedAwsTool {
    fn from(tool: AwsTool) -> Self {
        let mut allowed: Vec<_> = tool.allowed_services.into_iter().collect();
        let mut denied: Vec<_> = tool.denied_services.into_iter().collect();
        allowed.sort();
        denied.sort();
        Self {
            allowed_services: allowed,
            denied_services: denied,
            auto_allow_readonly: tool.auto_allow_readonly,
        }
    }
}

#[derive(Debug, Facet, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedExecuteShellTool {
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_commands: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub denied_commands: Vec<String>,
    #[facet(default)]
    pub deny_by_default: bool,
    pub auto_allow_readonly: bool,
}

impl From<ExecuteShellTool> for NormalizedExecuteShellTool {
    fn from(tool: ExecuteShellTool) -> Self {
        let mut allowed: Vec<_> = tool.allowed_commands.into_iter().collect();
        let mut denied: Vec<_> = tool.denied_commands.into_iter().collect();
        allowed.sort();
        denied.sort();
        Self {
            allowed_commands: allowed,
            denied_commands: denied,
            deny_by_default: tool.deny_by_default,
            auto_allow_readonly: tool.auto_allow_readonly,
        }
    }
}

#[derive(Debug, Facet, Default, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedReadTool {
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_paths: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub denied_paths: Vec<String>,
}

impl From<ReadTool> for NormalizedReadTool {
    fn from(tool: ReadTool) -> Self {
        let mut allowed: Vec<_> = tool.allowed_paths.into_iter().collect();
        let mut denied: Vec<_> = tool.denied_paths.into_iter().collect();
        allowed.sort();
        denied.sort();
        Self {
            allowed_paths: allowed,
            denied_paths: denied,
        }
    }
}

#[derive(Debug, Facet, Default, PartialEq, Eq, Clone)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedWriteTool {
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_paths: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub denied_paths: Vec<String>,
}

impl From<WriteTool> for NormalizedWriteTool {
    fn from(tool: WriteTool) -> Self {
        let mut allowed: Vec<_> = tool.allowed_paths.into_iter().collect();
        let mut denied: Vec<_> = tool.denied_paths.into_iter().collect();
        allowed.sort();
        denied.sort();
        Self {
            allowed_paths: allowed,
            denied_paths: denied,
        }
    }
}

#[derive(Debug, Clone, Facet, Default, PartialEq, Eq)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedSubagentTool {
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_agents: Vec<String>,
}

impl From<SubagentTool> for NormalizedSubagentTool {
    fn from(tool: SubagentTool) -> Self {
        let mut allowed: Vec<_> = tool.allowed_agents.into_iter().collect();
        allowed.sort();
        Self {
            allowed_agents: allowed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_target_display() {
        assert_eq!(ToolTarget::Aws.to_string(), "aws");
        assert_eq!(ToolTarget::Shell.to_string(), "shell");
    }

    #[test]
    fn tool_target_as_ref() {
        assert_eq!(ToolTarget::Read.as_ref(), "read");
        assert_eq!(ToolTarget::Write.as_ref(), "write");
    }

    #[test]
    fn aws_tool_default() {
        let tool = AwsTool::default();
        assert!(tool.auto_allow_readonly);
        assert!(tool.allowed_services.is_empty());
    }

    #[test]
    fn execute_shell_tool_default() {
        let tool = ExecuteShellTool::default();
        assert!(!tool.deny_by_default);
        assert!(tool.auto_allow_readonly);
    }

    #[test]
    fn read_tool_facet() {
        let tool = ReadTool::default();
        let json = facet_json::to_string(&tool).unwrap();
        let deserialized: ReadTool = facet_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn write_tool_facet() {
        let tool = WriteTool::default();
        let json = facet_json::to_string(&tool).unwrap();
        let deserialized: WriteTool = facet_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }
}
