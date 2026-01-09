use {
    super::*,
    crate::config::{GeneratorConfig, GeneratorConfigDoc, KdlAgent, KdlAgentDoc},
    std::{fmt::Display, ops::Deref, path::Path},
};

pub fn load_inline(fs: &Fs, path: impl AsRef<Path>) -> Result<GeneratorConfig> {
    let doc: Option<Result<GeneratorConfigDoc>> = crate::config::kdl_parse_path(fs, path);
    match doc {
        None => Ok(GeneratorConfig::default()),
        Some(d) => {
            let d = d?;
            Ok(d.into())
        }
    }
}

fn process_local(
    fs: &Fs,
    name: impl AsRef<str>,
    location: &ConfigLocation,
    inline: Option<&KdlAgent>,
    sources: &mut Vec<KdlAgentSource>,
) -> Result<KdlAgent> {
    let local_agent_path = location.local(&name);
    let result = KdlAgentDoc::from_path(fs, &name, &local_agent_path);
    match result {
        None => Ok(KdlAgent::new(name.as_ref().to_string())),
        Some(a) => {
            let agent = KdlAgent::from(a?.clone());
            sources.push(KdlAgentSource::LocalFile(local_agent_path));
            if let Some(i) = inline {
                sources.push(KdlAgentSource::LocalInline);
                Ok(agent.merge(i.clone()))
            } else {
                Ok(agent)
            }
        }
    }
}

#[derive(Clone, Serialize)]
pub struct ResolvedAgents {
    #[serde(skip)]
    pub agents: HashMap<String, KdlAgent>,
    pub sources: KdlSources,
    #[serde(skip)]
    pub has_local: bool,
}

impl Deref for ResolvedAgents {
    type Target = HashMap<String, KdlAgent>;

    fn deref(&self) -> &Self::Target {
        &self.agents
    }
}

impl Debug for ResolvedAgents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolved={}", self.agents.len())
    }
}

impl Display for ResolvedAgents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "resolved={}", self.agents.len())
    }
}

/// First pass: Discover all agents from configuration files
///
/// merge agent config from lowest precedence to higher precedence:
/// ```text
/// * `~/.kiro/generators/<agent-name>.kdl`
/// * `~/.kiro/generators/kg.kdl`
/// * `.kiro/generators/<agent-name>.kdl`
/// * `.kiro/generators/kg.kdl`
/// ```
#[tracing::instrument(level = "info")]
pub fn discover(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> Result<ResolvedAgents> {
    location.is_valid(fs)?;

    let global_path = location.global_kg();
    let local_path = location.local_kg();
    let global_agents: GeneratorConfig = load_inline(fs, global_path)?;
    let local_agents: GeneratorConfig = load_inline(fs, local_path)?;
    tracing::debug!("found {} local agents", local_agents.agents.len());

    let local_names: HashSet<String> =
        HashSet::from_iter(local_agents.agents.keys().map(|k| k.to_string()));
    let global_names: HashSet<String> =
        HashSet::from_iter(global_agents.agents.keys().map(|k| k.to_string()));
    let mut all_agents_names: HashSet<String> =
        HashSet::with_capacity(global_names.len() + local_names.len());
    all_agents_names.extend(local_names.clone());
    all_agents_names.extend(global_names);

    let mut resolved_agents: HashMap<String, KdlAgent> =
        HashMap::with_capacity(all_agents_names.len());
    let mut sources: KdlSources = KdlSources::from(&all_agents_names);

    for (name, agent_sources) in sources.iter_mut() {
        let span = tracing::debug_span!("agent", name = ?name);
        let _enter = span.enter();
        tracing::trace!("matching location");

        match location {
            ConfigLocation::Local => {
                resolved_agents.insert(
                    name.to_string(),
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?,
                );
            }
            ConfigLocation::Both(_) => {
                let mut result =
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?;
                if let Some(a) = global_agents.get(name) {
                    agent_sources.push(KdlAgentSource::GlobalInline);
                    result = result.merge(a.clone());
                }
                let maybe_global_file = KdlAgentDoc::from_path(fs, name, location.global(name));
                if let Some(global) = maybe_global_file {
                    agent_sources.push(KdlAgentSource::GlobalFile(location.global(name)));
                    result = result.merge(KdlAgent::from(global?.clone()));
                }
                resolved_agents.insert(name.to_string(), result);
            }
            ConfigLocation::Global(_) => {
                let mut global_file = match KdlAgentDoc::from_path(fs, name, location.global(name))
                {
                    None => KdlAgent::new(name.to_string()),
                    Some(a) => {
                        agent_sources.push(KdlAgentSource::GlobalFile(location.global(name)));
                        KdlAgent::from(a?)
                    }
                };
                if let Some(inline) = global_agents.get(name) {
                    agent_sources.push(KdlAgentSource::GlobalInline);
                    global_file = global_file.merge(inline.clone());
                }
                resolved_agents.insert(name.to_string(), global_file);
            }
        };
    }
    if let Err(e) = format.sources(&sources) {
        tracing::error!("Failed to format sources: {}", e);
    }
    let has_local = sources.values().flatten().any(|s| {
        matches!(
            s,
            KdlAgentSource::LocalFile(_) | KdlAgentSource::LocalInline
        )
    });
    Ok(ResolvedAgents {
        agents: resolved_agents,
        sources,
        has_local,
    })
}

#[cfg(test)]
mod tests {
    use {super::*, crate::os::ACTIVE_USER_HOME, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_local_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let resolved = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;
        let agents = resolved.agents;
        let sources = resolved.sources;
        assert!(!agents.is_empty());
        assert_eq!(sources.keys().len(), 3);
        assert!(sources.contains_key("base"));
        assert!(sources.contains_key("aws-test"));
        assert!(sources.contains_key("dependabot"));

        let source = sources.get("base").unwrap();
        assert_eq!(source.len(), 2);

        let source = sources.get("aws-test").unwrap();
        assert_eq!(source.len(), 2);

        let source = sources.get("dependabot").unwrap();
        assert_eq!(source.len(), 2);

        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        discover(
            &fs,
            &ConfigLocation::Global(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_global_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let resolved = discover(
            &fs,
            &ConfigLocation::Global(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;
        assert_eq!(resolved.len(), 3);
        for agent_sources in resolved.sources.values() {
            for s in agent_sources {
                assert!(
                    matches!(
                        s,
                        KdlAgentSource::GlobalInline | KdlAgentSource::GlobalFile(_)
                    ),
                    "agent is not global"
                )
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_errors() {
        let fs = Fs::new();
        let e = load_inline(
            &fs,
            PathBuf::from(".kiro").join("generators").join("bad.kdl"),
        );
        assert!(e.is_err());
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_both_agents_kdl() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let resolved = discover(
            &fs,
            &ConfigLocation::Both(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        assert_eq!(resolved.len(), 3);

        for (n, agent_sources) in resolved.sources.iter() {
            if n == "aws-test" {
                assert_eq!(agent_sources.len(), 4);
            } else {
                assert_eq!(agent_sources.len(), 3);
            }
        }

        assert!(!format!("{resolved}").is_empty());
        assert!(!format!("{resolved:?}").is_empty());
        Ok(())
    }
}
