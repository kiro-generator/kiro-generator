use {facet::Facet, std::fmt::Display};

#[allow(dead_code)]
pub enum HookTrigger {
    /// Triggered during agent spawn
    AgentSpawn,
    /// Triggered per user message submission
    UserPromptSubmit,
    /// Triggered before tool execution
    PreToolUse,
    /// Triggered after tool execution
    PostToolUse,
    /// Triggered when the assistant finishes responding
    Stop,
}

impl Display for HookTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookTrigger::AgentSpawn => write!(f, "agentSpawn"),
            HookTrigger::UserPromptSubmit => write!(f, "userPromptSubmit"),
            HookTrigger::PreToolUse => write!(f, "preToolUse"),
            HookTrigger::PostToolUse => write!(f, "postToolUse"),
            HookTrigger::Stop => write!(f, "stop"),
        }
    }
}

#[derive(Facet, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct AgentHook {
    /// The command to run when the hook is triggered
    pub command: String,

    /// Optional glob matcher for hook
    /// Currently used for matching tool name of PreToolUse and PostToolUse hook
    #[facet(skip_serializing_if = Option::is_none)]
    pub matcher: Option<String>,
}

#[derive(Facet, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Hook {
    /// The command to run when the hook is triggered
    pub command: String,

    #[facet(rename = "type", skip_serializing)]
    pub hook_type: String,

    /// Optional glob matcher for hook
    /// Currently used for matching tool name of PreToolUse and PostToolUse hook
    #[facet(skip_serializing_if = Option::is_none)]
    pub matcher: Option<String>,
}

impl Hook {
    pub fn merge(mut self, o: Self) -> Self {
        if self.matcher.is_none() && o.matcher.is_some() {
            tracing::trace!("matcher: merged from other");
            self.matcher = o.matcher;
        }
        if self.command.is_empty() {
            tracing::trace!("command: merged from other");
            self.command = o.command;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hook_merge() -> crate::config::ConfigResult<()> {
        let parent = Hook {
            command: "test".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: Some("*.rs".into()),
        };

        let child = Hook {
            command: "test-child".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: None,
        };

        let merged = child.merge(parent);
        assert_eq!("test-child", merged.command);
        assert_eq!(Some("*.rs".to_string()), merged.matcher);
        Ok(())
    }
}
