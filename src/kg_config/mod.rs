mod agent_file;
mod knowledge;
mod manifest;
mod merge;
mod native;
mod subagent;

use {
    crate::{Fs, Result},
    facet::Facet,
    facet_toml as toml,
    std::{collections::HashMap, fmt::Debug, path::Path},
};
pub use {
    agent_file::KgAgentFileDoc,
    knowledge::KgKnowledge,
    manifest::Manifest,
    subagent::SubagentConfig,
};

pub fn toml_parse_path<T>(fs: &Fs, path: impl AsRef<Path>) -> Option<Result<T>>
where
    T: for<'a> facet::Facet<'a>,
{
    if !fs.exists(&path) {
        return None;
    }
    match fs.read_to_string_sync(&path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(r) => Some(Ok(r)),
            Err(e) => Some(Err(e.into())), // facet_toml error auto-converts to eyre::Report
        },
        Err(e) => Some(Err(e)), // io error from Fs already is eyre::Report
    }
}

#[cfg(test)]
pub fn toml_parse<T>(content: &str) -> Result<T>
where
    T: for<'a> facet::Facet<'a>,
{
    Ok(toml::from_str::<T>(content)?)
}

#[derive(Default, Facet)]
#[facet(deny_unknown_fields)]
pub struct GeneratorConfig {
    #[facet(default, rename = "$schema")]
    pub schema: Option<String>,
    #[facet(default, rename = "agents")]
    pub agents: HashMap<String, Manifest>,
}
impl GeneratorConfig {
    pub fn populate_names(mut self) -> Self {
        for (k, v) in self.agents.iter_mut() {
            v.name = k.clone();
        }
        self
    }
}
impl Debug for GeneratorConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "agents={}", self.agents.len())
    }
}

impl GeneratorConfig {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Manifest> {
        self.agents.get(name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::KgAgentFileDoc};

    #[test_log::test]
    fn test_agent_decoding() -> Result<()> {
        let toml_agents = include_str!("../../data/test-decoding.toml");

        let config: GeneratorConfig = toml_parse(toml_agents)?;
        assert_eq!(config.agents.len(), 1);
        let agent = config.agents.get("test");
        assert!(agent.is_some());
        let agent = agent.unwrap().clone();
        assert!(agent.model.is_none());
        assert!(!agent.template);
        let inherits = &agent.inherits;
        assert_eq!(inherits.len(), 1);
        assert_eq!(inherits.iter().next().unwrap(), "parent");
        assert!(agent.description.is_some());
        assert!(agent.prompt.is_some());
        assert!(agent.include_mcp_json.unwrap_or_default());
        let tools = &agent.tools;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools.iter().next().unwrap(), "*");
        let resources = &agent.resources;
        assert_eq!(resources.len(), 2);
        assert!(resources.contains(&"file://resource.md".to_string()));
        assert!(resources.contains(&"file://README.md".to_string()));

        let hooks = &agent.hooks;
        let agent_spawn_hooks = hooks.get("agentSpawn");
        assert!(agent_spawn_hooks.is_some());
        let agent_spawn_hooks = agent_spawn_hooks.unwrap();
        let spawn_hook = agent_spawn_hooks.get("spawn");
        assert!(spawn_hook.is_some());
        assert_eq!(spawn_hook.unwrap().command, "echo i have spawned");

        assert!(hooks.contains_key("preToolUse"));
        assert!(hooks.contains_key("stop"));
        assert!(hooks.contains_key("postToolUse"));
        assert!(hooks.contains_key("userPromptSubmit"));

        let allowed = &agent.allowed_tools;
        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains("@awsdocs"));

        let subagents = &agent.subagents;
        assert_eq!(subagents.allow.len(), 1);
        assert!(subagents.allow.contains("backend"));
        assert!(subagents.deny.is_empty());

        let mcp = &agent.mcp_servers;
        assert_eq!(mcp.len(), 1);
        assert!(mcp.contains_key("awsdocs"));
        let aws_docs = mcp.get("awsdocs").unwrap();
        assert_eq!(aws_docs.command, "aws-docs");
        assert_eq!(aws_docs.args, vec!["--verbose", "--config=/path"]);
        assert!(!aws_docs.disabled);
        assert_eq!(aws_docs.headers.len(), 1);
        assert_eq!(aws_docs.env.len(), 2);
        assert_eq!(aws_docs.timeout, Some(5000));
        assert_eq!(agent.alias.len(), 1);

        assert_eq!(1, agent.tool_settings.len());
        assert!(agent.tool_settings.contains_key("whoami"));
        let raw = r#"{"env":{"LOG_LEVEL":"debug"}}"#;
        let result = facet_value::format_value(agent.tool_settings.get("whoami").unwrap());
        assert_eq!(raw, result.replace("\n", "").replace(" ", ""));

        Ok(())
    }

    #[test_log::test]
    fn test_agent_empty() -> Result<()> {
        let toml_agents = r#"
            [agents.test]
            template=true
        "#;

        let config: GeneratorConfig = toml_parse(toml_agents)?;
        assert!(!format!("{config:?}").is_empty());
        assert_eq!(config.agents.len(), 1);
        let agent = config.agents.get("test").unwrap();
        assert!(agent.model.is_none());
        assert!(agent.template);

        Ok(())
    }

    #[test_log::test]
    fn test_agent_file_source() -> Result<()> {
        let agent_str = include_str!("../../data/test-file-source.toml");
        let agent: KgAgentFileDoc = toml_parse(agent_str)?;
        assert_eq!(
            agent.description.unwrap_or_default().to_string(),
            "agent from file"
        );

        let subagents = &agent.subagents;
        assert_eq!(subagents.allow.len(), 1);
        assert!(subagents.allow.contains("pr-review"));
        assert!(subagents.deny.is_empty());

        Ok(())
    }
}
