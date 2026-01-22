use super::*;

impl Manifest {
    pub fn merge(mut self, other: Manifest) -> Self {
        tracing::trace!(
            self_name = %self.name,
            self_template = self.template,
            other_name = %other.name,
            other_template = other.template,
            "merge start"
        );

        // Child wins for explicit values
        if self.include_mcp_json.is_none() && other.include_mcp_json.is_some() {
            tracing::trace!("include_mcp_json: merged from other");
            self.include_mcp_json = other.include_mcp_json;
        }

        // template is never merged - only the original declaration matters

        if self.description.is_none() && other.description.is_some() {
            tracing::trace!("description: merged from other");
            self.description = other.description;
        }

        if self.prompt.is_none() && other.prompt.is_some() {
            tracing::trace!("prompt: merged from other");
            self.prompt = other.prompt;
        }

        if self.model.is_none() && other.model.is_some() {
            tracing::trace!("model: merged from other");
            self.model = other.model;
        }

        if self.keyboard_shortcut.is_none() && other.keyboard_shortcut.is_some() {
            tracing::trace!("keyboardShortcut: merged from other");
            self.keyboard_shortcut = other.keyboard_shortcut;
        }

        if self.welcome_message.is_none() && other.welcome_message.is_some() {
            tracing::trace!("welcomeMessage: merged from other");
            self.welcome_message = other.welcome_message;
        }

        // Collections are extended (merged)
        if !other.resources.is_empty() {
            tracing::trace!(count = other.resources.len(), "resources: extended");
            self.resources.extend(other.resources);
        }

        if !other.knowledge.is_empty() {
            tracing::trace!(count = other.knowledge.len(), "knowledge: merging");
            for (key, other_kb) in other.knowledge {
                self.knowledge
                    .entry(key.clone())
                    .and_modify(|self_kb| {
                        tracing::trace!(name = %key, "knowledge: merged");
                        *self_kb = self_kb.clone().merge(other_kb.clone());
                    })
                    .or_insert_with(|| {
                        tracing::trace!(name = %key, "knowledge: inserted");
                        other_kb
                    });
            }
        }

        if !other.tools.is_empty() {
            tracing::trace!(count = other.tools.len(), "tools: extended");
            self.tools.extend(other.tools);
        }

        if !other.allowed_tools.is_empty() {
            tracing::trace!(count = other.allowed_tools.len(), "allowed_tools: extended");
            self.allowed_tools.extend(other.allowed_tools);
        }

        if !other.alias.is_empty() {
            tracing::trace!(count = other.alias.len(), "alias: extended");
            self.alias.extend(other.alias);
        }

        if !other.inherits.is_empty() {
            tracing::trace!(count = other.inherits.len(), "inherits: extended");
            self.inherits.extend(other.inherits);
        }

        if !other.tool_settings.is_empty() {
            tracing::trace!(count = other.tool_settings.len(), "tool_settings: extended");
            self.tool_settings.extend(other.tool_settings);
        }

        // Merge hooks - child force_allow parent for same key
        for (key, parent_hook) in other.hooks {
            self.hooks
                .entry(key.clone())
                .and_modify(|child_hook| {
                    tracing::trace!(hook = %key, "hook: merged");
                    *child_hook = child_hook.clone().merge(parent_hook.clone())
                })
                .or_insert_with(|| {
                    tracing::trace!(hook = %key, "hook: inserted");
                    parent_hook
                });
        }

        // Merge mcp_servers - child force_allow parent for same key
        for (key, parent_mcp) in other.mcp_servers {
            self.mcp_servers
                .entry(key.clone())
                .and_modify(|child_mcp| {
                    tracing::trace!(server = %key, "mcp_server: merged");
                    *child_mcp = child_mcp.clone().merge(parent_mcp.clone())
                })
                .or_insert_with(|| {
                    tracing::trace!(server = %key, "mcp_server: inserted");
                    parent_mcp
                });
        }

        self.native_tools = self.native_tools.merge(other.native_tools);

        self
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            agent::hook::{AgentHook, HookTrigger},
            config,
        },
    };

    const CONFIG: &str = include_str!("../../data/test-merge-agent.toml");

    #[test_log::test]
    fn test_agent_merge() -> config::ConfigResult<()> {
        let config: GeneratorConfig = config::toml_parse(CONFIG)?;
        assert_eq!(config.agents.len(), 2);
        let child = config.agents.get("child");
        let parent = config.agents.get("parent");
        assert!(child.is_some());
        assert!(parent.is_some());
        let child = child.unwrap().clone();
        let parent = parent.unwrap().clone();
        assert!(!child.tools.is_empty());
        assert!(!parent.tools.is_empty());
        assert!(parent.template);
        let merged = child.merge(parent);
        assert!(merged.description.is_some());
        let d = merged.description.clone().unwrap();
        assert_eq!(d, "I am a child");

        assert_eq!(merged.resources.len(), 3);
        assert!(!merged.template);
        assert!(merged.include_mcp_json.unwrap_or_default());

        assert_eq!(merged.inherits.len(), 1);
        assert!(merged.inherits.contains("parent"));

        assert_eq!(merged.prompt, Some("i tell you what to do".to_string()));
        let tools = &merged.tools;
        assert_eq!(tools.len(), 3);
        assert!(tools.contains("@awsdocs"));
        assert!(tools.contains("shell"));
        assert!(tools.contains("web_search"));

        assert_eq!(merged.model, Some("claude".to_string()));

        let allowed_tools = &merged.allowed_tools;
        assert_eq!(allowed_tools.len(), 1);
        assert!(allowed_tools.contains("write"));

        let hooks = merged.hooks();
        assert!(!hooks.is_empty());
        let h = hooks.get(&HookTrigger::AgentSpawn.to_string());
        assert!(h.is_some());
        let h = h.unwrap();
        assert!(!h.is_empty());
        assert_eq!(h[0], AgentHook {
            command: "echo i have spawned".to_string(),
            matcher: None,
            timeout_ms: None,
            max_output_size: None,
            cache_ttl_seconds: None,
        });

        let h = hooks.get(&HookTrigger::UserPromptSubmit.to_string());
        assert!(h.is_some());

        let alias = &merged.alias;
        assert_eq!(alias.len(), 2);
        assert!(alias.contains_key("fs_read"));
        assert!(alias.contains_key("execute_bash"));

        let tool = merged.get_tool_write();
        assert!(tool.force_allow.contains("Cargo.lock"));
        assert_eq!(tool.allows.len(), 2);
        assert_eq!(tool.force_allow.len(), 1);
        assert_eq!(tool.denies.len(), 1);

        let tool = merged.get_tool_read();
        assert_eq!(tool.allows.len(), 2);
        assert_eq!(tool.force_allow.len(), 0);
        assert_eq!(tool.denies.len(), 1);

        let tool = merged.get_tool_shell();
        assert_eq!(tool.allows.len(), 2);
        assert_eq!(tool.force_allow.len(), 1);
        assert_eq!(tool.denies.len(), 1);

        let tool = merged.get_tool_aws();
        assert!(tool.allows.is_empty());
        assert!(tool.denies.is_empty());

        // Knowledge merge tests
        assert_eq!(merged.knowledge.len(), 2);
        let docs = merged.knowledge.get("docs");
        assert!(docs.is_some());
        let docs = docs.unwrap();
        assert_eq!(docs.source, Some("file://./parent-docs".to_string()));
        assert_eq!(docs.description, Some("Parent documentation".to_string()));
        assert_eq!(docs.index_type, Some("best".to_string()));

        let api = merged.knowledge.get("api");
        assert!(api.is_some());
        let api = api.unwrap();
        assert_eq!(api.source, Some("file://./api-docs".to_string()));
        assert_eq!(api.description, Some("API documentation".to_string()));

        assert_eq!("", format!("{merged}"));
        assert_eq!("", format!("{merged:?}"));

        assert_eq!(
            merged.welcome_message,
            Some("children are better than parents".to_string())
        );

        assert_eq!(merged.keyboard_shortcut, Some("ctrl+shift+a".to_string()));
        Ok(())
    }
}
