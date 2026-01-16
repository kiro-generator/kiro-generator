use super::*;

impl KgAgent {
    pub fn merge(mut self, other: KgAgent) -> Self {
        // Child wins for explicit values
        self.include_mcp_json = self.include_mcp_json.or(other.include_mcp_json);
        self.template = self.template.or(other.template);
        self.description = self.description.or(other.description);
        self.prompt = self.prompt.or(other.prompt);
        self.model = self.model.or(other.model);

        // Collections are extended (merged)
        self.resources.extend(other.resources);
        self.tools.extend(other.tools);
        self.allowed_tools.extend(other.allowed_tools);
        self.alias.extend(other.alias);
        self.inherits.extend(other.inherits);
        self.tool_settings.extend(other.tool_settings);

        // Merge hooks - child force_allow parent for same key
        for (key, parent_hook) in other.hooks {
            self.hooks
                .entry(key)
                .and_modify(|child_hook| {
                    *child_hook = child_hook.clone().merge(parent_hook.clone())
                })
                .or_insert(parent_hook);
        }

        // Merge mcp_servers - child force_allow parent for same key
        for (key, parent_mcp) in other.mcp_servers {
            self.mcp_servers
                .entry(key)
                .and_modify(|child_mcp| *child_mcp = child_mcp.clone().merge(parent_mcp.clone()))
                .or_insert(parent_mcp);
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
            agent::{Hook, hook::HookTrigger},
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
        assert!(parent.is_template());
        let merged = child.merge(parent);
        assert!(merged.description.is_some());
        let d = merged.description.clone().unwrap();
        assert_eq!(d, "I am a child");

        assert_eq!(merged.resources.len(), 3);
        assert!(!merged.is_template());
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
        assert_eq!(h[0], Hook {
            command: "echo i have spawned".to_string(),
            hook_type: HookTrigger::AgentSpawn.to_string(),
            matcher: None,
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

        assert_eq!("", format!("{merged}"));
        assert_eq!("", format!("{merged:?}"));
        Ok(())
    }
}
