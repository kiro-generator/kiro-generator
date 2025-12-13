use super::*;

impl Generator {
    /// Merge all agents with their inheritance chains
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
            let span = tracing::debug_span!("merge", parents = ?inline_agent.inherits.0.len(), child = ?name);
            let _enter = span.enter();
            let mut builder = Config::builder();
            if !cached_serialized_agents.contains_key(name) {
                return Err(color_eyre::eyre::eyre!(
                    "Cached source for agent '{name}' not found",
                ));
            }
            for parent in &inline_agent.inherits.0 {
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
                inline_agent.inherits.0
            ))?;

            for parent in &inline_agent.inherits.0 {
                if !self.agents.contains_key(parent) {
                    return Err(color_eyre::eyre::eyre!(
                        "[{name}] Parent agent definition '{parent}' not found",
                    ));
                }
                agent = self.merge_tools(self.agents.get(parent).unwrap(), agent)?;
            }
            agent.name = name.clone();
            resolved_agents.insert(name.clone(), agent);
        }

        // Filter out skeletons and return QgAgent instances
        let mut agents: Vec<KgAgent> = resolved_agents.values().cloned().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(agents)
    }

    /// Merge parent into child (child takes precedence)
    fn merge_tools(&self, parent: &KgAgent, mut child: KgAgent) -> Result<KgAgent> {
        let span = tracing::debug_span!("merge-tools", parent = ?parent, child = ?child);
        let _enter = span.enter();

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
