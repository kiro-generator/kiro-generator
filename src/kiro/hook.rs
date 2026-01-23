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

    /// Max time the hook can run before it throws a timeout error (default:
    /// 30000ms)
    #[facet(skip_serializing_if = Option::is_none)]
    pub timeout_ms: Option<u64>,

    /// Max output size of the hook before it is truncated (default: 10240
    /// bytes)
    #[facet(skip_serializing_if = Option::is_none)]
    pub max_output_size: Option<u32>,

    /// How long the hook output is cached before it will be executed again
    /// (default: 0s)
    #[facet(skip_serializing_if = Option::is_none)]
    pub cache_ttl_seconds: Option<u64>,

    /// Optional glob matcher for hook
    /// Currently used for matching tool name of PreToolUse and PostToolUse hook
    #[facet(skip_serializing_if = Option::is_none)]
    pub matcher: Option<String>,
}

#[derive(Facet, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub struct KgHook {
    /// The command to run when the hook is triggered
    pub command: String,

    #[facet(rename = "type", skip_serializing)]
    pub hook_type: String,

    /// Max time the hook can run before it throws a timeout error (default:
    /// 30000ms)
    #[facet(skip_serializing_if = Option::is_none)]
    pub timeout_ms: Option<u64>,

    /// Max output size of the hook before it is truncated (default: 10240
    /// bytes)
    #[facet(skip_serializing_if = Option::is_none)]
    pub max_output_size: Option<u32>,

    /// How long the hook output is cached before it will be executed again
    /// (default: 0s)
    #[facet(skip_serializing_if = Option::is_none)]
    pub cache_ttl_seconds: Option<u64>,

    /// Optional glob matcher for hook
    /// Currently used for matching tool name of PreToolUse and PostToolUse hook
    #[facet(skip_serializing_if = Option::is_none)]
    pub matcher: Option<String>,
}

impl KgHook {
    pub fn merge(mut self, o: Self) -> Self {
        if self.matcher.is_none() && o.matcher.is_some() {
            tracing::trace!("matcher: merged from other");
            self.matcher = o.matcher;
        }
        if self.timeout_ms.is_none() && o.timeout_ms.is_some() {
            tracing::trace!("timeout_ms: merged from other");
            self.timeout_ms = o.timeout_ms;
        }
        if self.max_output_size.is_none() && o.max_output_size.is_some() {
            tracing::trace!("max_output_size: merged from other");
            self.max_output_size = o.max_output_size;
        }
        if self.cache_ttl_seconds.is_none() && o.cache_ttl_seconds.is_some() {
            tracing::trace!("cache_ttl_seconds: merged from other");
            self.cache_ttl_seconds = o.cache_ttl_seconds;
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
    fn hook_merge() -> crate::Result<()> {
        let parent = KgHook {
            command: "test".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: Some("*.rs".into()),
            timeout_ms: Some(5000),
            max_output_size: Some(2048),
            cache_ttl_seconds: Some(60),
        };

        let child = KgHook {
            command: "test-child".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: None,
            timeout_ms: None,
            max_output_size: None,
            cache_ttl_seconds: None,
        };

        let merged = child.merge(parent);
        assert_eq!("test-child", merged.command);
        assert_eq!(Some("*.rs".to_string()), merged.matcher);
        assert_eq!(Some(5000), merged.timeout_ms);
        assert_eq!(Some(2048), merged.max_output_size);
        assert_eq!(Some(60), merged.cache_ttl_seconds);
        Ok(())
    }

    #[test]
    fn hook_merge_child_overrides() -> crate::Result<()> {
        let parent = KgHook {
            command: "test".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: Some("*.rs".into()),
            timeout_ms: Some(5000),
            max_output_size: Some(2048),
            cache_ttl_seconds: Some(60),
        };

        let child = KgHook {
            command: "test-child".into(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: Some("*.toml".into()),
            timeout_ms: Some(10000),
            max_output_size: Some(4096),
            cache_ttl_seconds: Some(120),
        };

        let merged = child.merge(parent);
        assert_eq!("test-child", merged.command);
        assert_eq!(Some("*.toml".to_string()), merged.matcher);
        assert_eq!(Some(10000), merged.timeout_ms);
        assert_eq!(Some(4096), merged.max_output_size);
        assert_eq!(Some(120), merged.cache_ttl_seconds);
        Ok(())
    }
}
