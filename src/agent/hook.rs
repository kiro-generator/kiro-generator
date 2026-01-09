use {
    serde::{Deserialize, Serialize},
    std::fmt::Display,
};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_OUTPUT_SIZE: u64 = 1024 * 10;
const DEFAULT_CACHE_TTL_SECONDS: u64 = 0;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, enum_iterator::Sequence,
)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Hook {
    /// The command to run when the hook is triggered
    pub command: String,

    /// Max time the hook can run before it throws a timeout error
    #[serde(default = "Hook::default_timeout_ms")]
    pub timeout_ms: u64,

    /// Max output size of the hook before it is truncated
    #[serde(default = "Hook::default_max_output_size")]
    pub max_output_size: u64,

    /// How long the hook output is cached before it will be executed again
    #[serde(default = "Hook::default_cache_ttl_seconds")]
    pub cache_ttl_seconds: u64,

    /// Optional glob matcher for hook
    /// Currently used for matching tool name of PreToolUse and PostToolUse hook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
}

impl Hook {
    pub fn merge(mut self, o: Self) -> Self {
        if self.cache_ttl_seconds == 0 {
            self.cache_ttl_seconds = if o.cache_ttl_seconds == 0 {
                DEFAULT_CACHE_TTL_SECONDS
            } else {
                o.cache_ttl_seconds
            };
        }
        if self.command.is_empty() {
            self.command = o.command;
        }
        if self.max_output_size == 0 {
            self.max_output_size = if o.max_output_size == 0 {
                DEFAULT_MAX_OUTPUT_SIZE
            } else {
                o.max_output_size
            };
        }
        if self.timeout_ms == 0 {
            self.timeout_ms = if o.timeout_ms == 0 {
                DEFAULT_TIMEOUT_MS
            } else {
                o.timeout_ms
            };
        }
        if self.matcher.is_none() && o.matcher.is_some() {
            self.matcher = o.matcher;
        }
        self
    }

    fn default_timeout_ms() -> u64 {
        DEFAULT_TIMEOUT_MS
    }

    fn default_max_output_size() -> u64 {
        DEFAULT_MAX_OUTPUT_SIZE
    }

    fn default_cache_ttl_seconds() -> u64 {
        DEFAULT_CACHE_TTL_SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_trigger_display() {
        assert_eq!(HookTrigger::AgentSpawn.to_string(), "agentSpawn");
        assert_eq!(HookTrigger::Stop.to_string(), "stop");
    }

    #[test]
    fn hook_defaults() {
        assert_eq!(Hook::default_timeout_ms(), 30_000);
        assert_eq!(Hook::default_max_output_size(), 10_240);
        assert_eq!(Hook::default_cache_ttl_seconds(), 0);
    }

    #[test]
    fn hook_serde() {
        let hook = Hook {
            command: "test".into(),
            timeout_ms: 1000,
            max_output_size: 500,
            cache_ttl_seconds: 10,
            matcher: Some("*.rs".into()),
        };
        let json = serde_json::to_string(&hook).unwrap();
        let deserialized: Hook = serde_json::from_str(&json).unwrap();
        assert_eq!(hook, deserialized);
    }
}
