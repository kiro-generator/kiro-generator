use {
    crate::agent::hook::{Hook, HookTrigger},
    facet::Facet,
    facet_kdl as kdl,
    std::collections::HashMap,
};

macro_rules! define_hook_doc {
    ($name:ident) => {
        #[derive(Facet, Default, Clone, Debug, PartialEq, Eq)]
        #[facet(default, rename_all = "kebab-case")]
        pub struct $name {
            #[facet(kdl::argument)]
            pub name: String,
            #[facet(kdl::child, default)]
            command: String,
            #[facet(kdl::child, default, rename = "timeout-ms")]
            timeout_ms: u64,
            #[facet(kdl::child, default, rename = "max-output-size")]
            max_output_size: u64,
            #[facet(kdl::child, default, rename = "cache-ttl")]
            cache_ttl_seconds: u64,
            #[facet(kdl::child, default)]
            matcher: Option<String>,
        }
        impl From<$name> for Hook {
            fn from(value: $name) -> Hook {
                Hook {
                    command: value.command,
                    timeout_ms: value.timeout_ms,
                    max_output_size: value.max_output_size,
                    cache_ttl_seconds: value.cache_ttl_seconds,
                    matcher: value.matcher,
                }
            }
        }
    };
}

#[derive(Facet, Clone, Default, Debug, PartialEq, Eq)]
#[facet(default)]
struct GenericValue {
    #[facet(kdl::argument)]
    value: String,
}

define_hook_doc!(HookAgentSpawnDoc);
define_hook_doc!(HookUserPromptSubmitDoc);
define_hook_doc!(HookPreToolUseDoc);
define_hook_doc!(HookPostToolUseDoc);
define_hook_doc!(HookStopDoc);

#[derive(Facet, Clone, Default, Debug, PartialEq, Eq)]
pub struct HookDoc {
    #[facet(kdl::children, default, rename = "agent-spawn")]
    pub agent_spawn: Vec<HookAgentSpawnDoc>,
    #[facet(kdl::children, default, rename = "user-prompt-submit")]
    pub user_prompt_submit: Vec<HookUserPromptSubmitDoc>,
    #[facet(kdl::children, default, rename = "pre-tool-use")]
    pub pre_tool_use: Vec<HookPreToolUseDoc>,
    #[facet(kdl::children, default, rename = "post-tool-use")]
    pub post_tool_use: Vec<HookPostToolUseDoc>,
    #[facet(kdl::children, default)]
    pub stop: Vec<HookStopDoc>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct HookPart {
    pub agent_spawn: HashMap<String, Hook>,
    pub user_prompt_submit: HashMap<String, Hook>,
    pub pre_tool_use: HashMap<String, Hook>,
    pub post_tool_use: HashMap<String, Hook>,
    pub stop: HashMap<String, Hook>,
}

impl From<HookDoc> for HookPart {
    fn from(value: HookDoc) -> Self {
        Self {
            agent_spawn: value
                .agent_spawn
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            user_prompt_submit: value
                .user_prompt_submit
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            pre_tool_use: value
                .pre_tool_use
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            post_tool_use: value
                .post_tool_use
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
            stop: value
                .stop
                .into_iter()
                .map(|h| (h.name.clone(), Hook::from(h)))
                .collect(),
        }
    }
}

impl HookPart {
    pub fn hooks(&self, trigger: &HookTrigger) -> Vec<Hook> {
        match trigger {
            HookTrigger::AgentSpawn => self.agent_spawn.values().cloned().collect(),
            HookTrigger::UserPromptSubmit => self.user_prompt_submit.values().cloned().collect(),
            HookTrigger::PreToolUse => self.pre_tool_use.values().cloned().collect(),
            HookTrigger::PostToolUse => self.post_tool_use.values().cloned().collect(),
            HookTrigger::Stop => self.stop.values().cloned().collect(),
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.agent_spawn = merge_hooks(self.agent_spawn, other.agent_spawn);
        self.user_prompt_submit = merge_hooks(self.user_prompt_submit, other.user_prompt_submit);
        self.pre_tool_use = merge_hooks(self.pre_tool_use, other.pre_tool_use);
        self.post_tool_use = merge_hooks(self.post_tool_use, other.post_tool_use);
        self.stop = merge_hooks(self.stop, other.stop);
        self
    }
}

fn merge_hooks(
    mut base: HashMap<String, Hook>,
    other: HashMap<String, Hook>,
) -> HashMap<String, Hook> {
    if base.is_empty() {
        return other;
    }
    if other.is_empty() {
        return base;
    }

    for (key, other_hook) in other {
        base.entry(key)
            .and_modify(|h| *h = h.clone().merge(other_hook.clone()))
            .or_insert(other_hook);
    }
    base
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{Result, config::kdl_parse},
        std::time::Duration,
    };

    fn rando() -> HashMap<String, Hook> {
        let value = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let name = format!("$name-{value}");
        let mut hooks = HashMap::new();
        hooks.insert(name, Hook {
            command: format!("{value}"),
            timeout_ms: value,
            max_output_size: value,
            cache_ttl_seconds: value,
            matcher: Some(format!("{value}")),
        });
        hooks
    }

    impl HookPart {
        pub fn randomize() -> Self {
            Self {
                agent_spawn: rando(),
                user_prompt_submit: rando(),
                pre_tool_use: rando(),
                post_tool_use: rando(),
                stop: rando(),
            }
        }
    }
    #[test_log::test]
    pub fn test_hooks_kdl() -> Result<()> {
        let kdl = r#"
            agent-spawn "test" {
                command "echo"
                timeout-ms  1231
                max-output-size 69
                cache-ttl-seconds 666
                matcher "blah"
            }
            user-prompt-submit "prompt-hook" {
                command "validate-prompt"
                timeout-ms 500
            }
            pre-tool-use "pre-hook" {
                command "pre-check"
                matcher "git*"
            }
            post-tool-use "post-hook" {
                command "post-check"
            }
            stop "cleanup" {
                command "cleanup-script"
                cache-ttl-seconds 0
            }
        "#;
        let doc: HookDoc = kdl_parse(kdl)?;
        let doc = HookPart::from(doc);

        assert_eq!(1, doc.agent_spawn.len());
        let hook = doc.agent_spawn.get("test");
        assert!(hook.is_some());
        let hook = hook.unwrap();
        assert_eq!(hook.command, "echo");
        assert_eq!(hook.timeout_ms, 1231);
        assert_eq!(hook.max_output_size, 69);

        assert_eq!(1, doc.user_prompt_submit.len());
        assert!(doc.user_prompt_submit.contains_key("prompt-hook"));

        assert_eq!(1, doc.pre_tool_use.len());
        let pre = doc.pre_tool_use.get("pre-hook").unwrap();
        assert_eq!(pre.matcher, Some("git*".to_string()));

        assert_eq!(1, doc.post_tool_use.len());
        assert!(doc.post_tool_use.contains_key("post-hook"));

        assert_eq!(1, doc.stop.len());
        assert!(doc.stop.contains_key("cleanup"));

        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_empty() -> Result<()> {
        let child = HookPart::default();
        let parent = HookPart::default();
        let merged = child.merge(parent);

        assert!(merged.agent_spawn.is_empty());
        assert!(merged.user_prompt_submit.is_empty());
        assert!(merged.pre_tool_use.is_empty());
        assert!(merged.post_tool_use.is_empty());
        assert!(merged.stop.is_empty());
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_empty_child() -> Result<()> {
        let child = HookPart::default();
        let parent = HookPart::randomize();
        let before = parent.clone();
        let merged = child.merge(parent);

        assert_eq!(merged, before);
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_no_merge() -> Result<()> {
        let child = HookPart::randomize();
        let parent = HookPart::randomize();
        let before = child.clone();
        let merged = child.merge(parent);
        assert_eq!(merged, before);
        Ok(())
    }

    #[test_log::test]
    pub fn test_hooks_merge_parent() -> Result<()> {
        let child = HookPart::randomize();
        std::thread::sleep(Duration::from_millis(1300));
        let parent = HookPart::randomize();
        let merged = child.merge(parent);
        assert_eq!(merged.agent_spawn.len(), 2);
        assert_eq!(merged.user_prompt_submit.len(), 2);
        assert_eq!(merged.pre_tool_use.len(), 2);
        assert_eq!(merged.post_tool_use.len(), 2);
        assert_eq!(merged.stop.len(), 2);
        Ok(())
    }
}
