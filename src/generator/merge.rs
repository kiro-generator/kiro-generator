use {super::*, std::collections::HashSet};

impl Generator {
    /// Resolve transitive inheritance chain for an agent
    /// Returns ordered list of parent names from base to most derived
    #[tracing::instrument(level = "info", skip(self))]
    fn resolve_transitive_inheritance(
        &self,
        agent: &Manifest,
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
                .agents
                .get(parent_name)
                .ok_or_else(|| crate::format_err!("Agent '{parent_name}' not found"))?;

            let parent_chain = self.resolve_transitive_inheritance(&parent.merged, visited)?;
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

    /// Public accessor for the resolved inheritance chain of a named agent.
    pub fn inheritance_chain(&self, name: &str) -> Result<Vec<String>> {
        let agent = self
            .agents
            .get(name)
            .ok_or_else(|| crate::format_err!("Agent '{name}' not found"))?;
        self.resolve_transitive_inheritance(&agent.merged, &mut HashSet::new())
    }

    pub fn inheritance_chain_safe(&self, name: &str) -> Vec<String> {
        self.inheritance_chain(name).unwrap_or_else(|err| {
            tracing::warn!(agent = name, error = %err, "failed to resolve inheritance chain");
            Vec::new()
        })
    }

    /// Merge all agents with transitive inheritance resolution
    #[tracing::instrument(level = "info", skip(self))]
    pub fn merge(&self) -> Result<Vec<Manifest>> {
        let mut resolved_agents: HashMap<String, Manifest> =
            HashMap::with_capacity(self.agents.len());

        for (name, agent_slots) in &self.agents {
            let agent = &agent_slots.merged;
            let mut visited = HashSet::new();
            let parents = self.resolve_transitive_inheritance(agent, &mut visited)?;
            let span = tracing::info_span!(
                "agent",
                name = name.as_str(),
                parents = parents.len(),
                template = agent.template
            );
            let _enter = span.enter();

            let mut merged = agent.clone();
            for parent_name in parents.iter().rev() {
                let parent = &self
                    .agents
                    .get(parent_name)
                    .ok_or_else(|| crate::format_err!("Parent agent '{parent_name}' not found"))?
                    .merged;
                tracing::trace!(parent = %parent_name, parent_template = parent.template, "merging parent");
                merged = merged.merge(parent.clone());
            }

            resolved_agents.insert(name.clone(), merged);
        }

        let mut agents: Vec<Manifest> = resolved_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(agents)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::kiro::hook::HookTrigger};

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
        assert_eq!(resources.len(), 1);
        let default = resources.get("default");
        assert!(default.is_some());
        let default = default.unwrap();
        assert!(default.locations.contains("README.md"));
        assert!(default.locations.contains("AGENTS.md"));
        assert!(default.locations.contains(".amazonq/rules/**/*.md"));

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
        let results = generator.write_all(true, false).await?;
        assert!(!results.is_empty());

        Ok(())
    }
}
