use {super::*, std::collections::HashSet, tracing::enabled};

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
            return Err(color_eyre::eyre::eyre!(
                "Circular inheritance detected: {} already in chain",
                agent.name
            ));
        }
        visited.insert(agent.name.clone());

        let mut chain = Vec::new();
        for parent_name in &agent.inherits.0 {
            let parent = self
                .agents
                .get(parent_name)
                .ok_or_else(|| color_eyre::eyre::eyre!("Agent '{parent_name}' not found"))?;

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
        let fs = &self.fs;
        let mut resolved_agents: HashMap<String, KgAgent> =
            HashMap::with_capacity(self.agents.len());

        let mut cached_serialized_agents: HashMap<String, AgentSource> =
            HashMap::with_capacity(self.agents.len());
        for (k, v) in self.agents.iter() {
            let value = serde_json::to_value(v)?;
            if value.is_null() {
                tracing::warn!("agent {k} is empty");
                continue;
            }
            cached_serialized_agents.insert(
                k.clone(),
                AgentSource::Raw(
                    toml::to_string(&v)
                        .wrap_err_with(|| format!("could not serialize agent {k} to toml"))?,
                ),
            );
        }

        for (name, inline_agent) in &self.agents {
            let mut visited = HashSet::new();
            let parents = self.resolve_transitive_inheritance(inline_agent, &mut visited)?;
            let span = tracing::debug_span!("agent", name = ?name, parents = ?parents.len());
            let _enter = span.enter();
            let mut builder = Config::builder();
            if !cached_serialized_agents.contains_key(name) {
                return Err(color_eyre::eyre::eyre!(
                    "Cached source for agent '{name}' not found",
                ));
            }

            for parent in &parents {
                if !cached_serialized_agents.contains_key(parent) {
                    return Err(color_eyre::eyre::eyre!(
                        "Cached source for parent agent '{parent}' not found",
                    ));
                }
                let parent_source = cached_serialized_agents.get(parent).unwrap().to_source(fs);
                builder = builder.add_source(parent_source);
            }

            let source = cached_serialized_agents.get(name).unwrap().to_source(fs);
            builder = builder.add_source(source);

            let mut agent: KgAgent = builder.build()?.try_deserialize().context(format!(
                "failed to merge agent {name} with parents {:?}",
                parents
            ))?;

            for parent in &parents {
                if !self.agents.contains_key(parent) {
                    return Err(color_eyre::eyre::eyre!(
                        "[{name}] Parent agent definition '{parent}' not found",
                    ));
                }
                agent = self.merge_tools(self.agents.get(parent).unwrap(), agent)?;
            }

            agent.name = name.clone();
            if enabled!(tracing::Level::TRACE)
                && let Err(e) = self.format.trace_agent(&agent)
            {
                tracing::error!("Failed to trace agent: {e}");
            }
            resolved_agents.insert(name.clone(), agent);
        }

        // Filter out skeletons and return QgAgent instances
        let mut agents: Vec<KgAgent> = resolved_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(agents)
    }

    /// Merge parent into child (child takes precedence)
    #[tracing::instrument(level = "debug")]
    fn merge_tools(&self, parent: &KgAgent, mut child: KgAgent) -> Result<KgAgent> {
        let agent_aws = child.get_tool_aws();
        let parent_aws = parent.get_tool_aws();
        let agent_exec = child.get_tool_shell();
        let parent_exec = parent.get_tool_shell();
        let agent_fs_read = child.get_tool_read();
        let parent_fs_read = parent.get_tool_read();
        let agent_fs_write = child.get_tool_write();
        let parent_fs_write = parent.get_tool_write();

        child.set_tool(ToolTarget::Shell, agent_exec.merge(parent_exec));
        child.set_tool(ToolTarget::Aws, agent_aws.merge(parent_aws));
        child.set_tool(ToolTarget::Read, agent_fs_read.merge(parent_fs_read));
        child.set_tool(ToolTarget::Write, agent_fs_write.merge(parent_fs_write));

        Ok(child)
    }
}
