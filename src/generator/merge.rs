use {super::*, crate::config::KgAgent, std::collections::HashSet};

impl Generator {
    /// Resolve transitive inheritance chain for an agent
    /// Returns ordered list of parent names from base to most derived
    #[tracing::instrument(level = "debug", skip(self))]
    fn resolve_transitive_inheritance(
        &self,
        agent: &KgAgent,
        visited: &mut HashSet<String>,
    ) -> Result<Vec<String>> {
        if visited.contains(&agent.name) {
            return Err(crate::format_err!(
                "Circular inheritance detected: {} already in chain",
                agent.name
            ));
        }
        visited.insert(agent.name.clone());

        let mut chain = Vec::new();
        for parent_name in agent.inherits.iter() {
            let parent = self
                .resolved
                .agents
                .get(parent_name)
                .ok_or_else(|| crate::format_err!("Agent '{parent_name}' not found"))?;

            let parent_chain = self.resolve_transitive_inheritance(parent, visited)?;
            for p in parent_chain {
                if !chain.contains(&p) {
                    chain.push(p);
                }
            }
            if !chain.contains(parent_name) {
                chain.push(parent_name.clone());
            }
        }

        visited.remove(&agent.name);
        Ok(chain)
    }

    /// Merge all agents with transitive inheritance resolution
    #[tracing::instrument(level = "debug")]
    pub fn merge(&self) -> Result<Vec<KgAgent>> {
        let mut resolved_agents: HashMap<String, KgAgent> =
            HashMap::with_capacity(self.resolved.agents.len());

        for (name, agent) in &self.resolved.agents {
            let mut visited = HashSet::new();
            let parents = self.resolve_transitive_inheritance(agent, &mut visited)?;
            let span = tracing::debug_span!("agent", name = ?name, parents = ?parents.len());
            let _enter = span.enter();

            let mut merged = agent.clone();
            for parent_name in parents.iter().rev() {
                let parent =
                    self.resolved.agents.get(parent_name).ok_or_else(|| {
                        crate::format_err!("Parent agent '{parent_name}' not found")
                    })?;
                merged = merged.merge(parent.clone());
            }

            resolved_agents.insert(name.clone(), merged);
        }

        let mut agents: Vec<KgAgent> = resolved_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(agents)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::agent::hook::HookTrigger};

    #[tokio::test]
    #[test_log::test]
    async fn test_merge_inheritance_chain() -> Result<()> {
        let fs = Fs::new();
        let generator = Generator::new(
            fs,
            ConfigLocation::Local,
            crate::output::OutputFormat::Table(true),
        )?;

        let merged = generator.merge()?;
        assert_eq!(merged.len(), 7);

        // Find dependabot agent
        let dependabot = merged
            .iter()
            .find(|a| a.name == "dependabot")
            .expect("dependabot agent not found");

        // Verify inheritance chain was resolved: dependabot -> aws-test -> base
        assert_eq!(
            dependabot.description.clone().unwrap_or_default(),
            "I make life painful for developers"
        );

        // Should have prompt from aws-test
        assert_eq!(
            dependabot.prompt.clone().unwrap_or_default(),
            "you are an AWS expert".to_string()
        );

        // Should have tools from base
        let tools = &dependabot.tools;
        assert!(tools.contains("*"));

        // Should have allowed_tools merged from base and aws-test
        let allowed = &dependabot.allowed_tools;
        assert!(allowed.contains("read"));
        assert!(allowed.contains("knowledge"));
        assert!(allowed.contains("@fetch"));
        assert!(allowed.contains("@awsdocs"));

        // Should have resources from all three
        let resources = &dependabot.resources;
        assert!(resources.contains(&"file://README.md".to_string()));
        assert!(resources.contains(&"file://AGENTS.md".to_string()));
        assert!(resources.contains(&"file://.amazonq/rules/**/*.md".to_string()));

        // Should have hooks from all levels
        let hooks = dependabot.hooks();
        let spawn = hooks.get(&HookTrigger::AgentSpawn.to_string());
        assert!(spawn.is_some());
        assert!(!spawn.unwrap().is_empty());

        // Should have force permissions from dependabot overriding denies from base
        let shell = dependabot.get_tool_shell();
        let force_allow = &shell.force_allow;
        assert!(force_allow.contains("git commit .*"));
        assert!(force_allow.contains("git push .*"));

        let read = dependabot.get_tool_read();
        let force_allow = &read.force_allow;
        assert!(force_allow.contains(".*Cargo.toml.*"));

        let write = dependabot.get_tool_write();
        let force_allow = &write.force_allow;
        assert!(force_allow.contains(".*Cargo.toml.*"));

        // Should have aws tool from aws-test
        let aws = dependabot.get_tool_aws();

        assert_eq!(2, aws.allows.len());
        assert!(aws.allows.contains("ec2"));
        assert!(aws.allows.contains("s3"));

        // check try_from
        let results = generator.write_all(true).await?;
        assert!(!results.is_empty());

        Ok(())
    }
}
