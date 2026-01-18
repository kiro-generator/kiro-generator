use {
    super::*,
    crate::config::{ConfigResult, GeneratorConfig, KgAgent},
    std::{fmt::Display, ops::Deref, path::Path},
};

/// Load all TOML files from a manifests directory and merge them
fn load_manifests(fs: &Fs, dir: impl AsRef<Path>) -> ConfigResult<GeneratorConfig> {
    let dir_path = dir.as_ref();

    if !fs.exists(dir_path) {
        return Ok(GeneratorConfig::default());
    }

    let mut entries =
        futures::executor::block_on(async { fs.read_dir(dir_path).await }).map_err(|e| {
            crate::Error::Report(format!(
                "Failed to read directory {}: {}",
                dir_path.display(),
                e
            ))
        })?;

    let mut merged = GeneratorConfig::default();
    let mut agent_files: Vec<PathBuf> = Vec::new();

    // Collect all TOML files
    loop {
        let entry = futures::executor::block_on(async { entries.next_entry().await })
            .map_err(|e| crate::Error::Report(format!("Failed to read directory entry: {}", e)))?;

        match entry {
            None => break,
            Some(entry) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    agent_files.push(path);
                }
            }
        }
    }

    // Sort for deterministic order
    agent_files.sort();

    for path in agent_files {
        if let Some(config_result) = crate::config::toml_parse_path(fs, &path) {
            let config: GeneratorConfig = config_result?;
            let config = config.populate_names();

            // Check for duplicate agent names
            for name in config.agents.keys() {
                if merged.agents.contains_key(name) {
                    return Err(crate::Error::Report(format!(
                        "Duplicate agent '{}' found in manifests directory",
                        name
                    )));
                }
            }

            merged.agents.extend(config.agents);
        }
    }

    Ok(merged)
}

#[cfg(test)]
pub fn load_inline(fs: &Fs, path: impl AsRef<Path>) -> ConfigResult<GeneratorConfig> {
    let doc: Option<ConfigResult<GeneratorConfig>> = crate::config::toml_parse_path(fs, path);
    match doc {
        None => Ok(GeneratorConfig::default()),
        Some(d) => {
            let agents = d?;
            Ok(agents.populate_names())
        }
    }
}

fn process_local(
    fs: &Fs,
    name: impl AsRef<str>,
    location: &ConfigLocation,
    inline: Option<&KgAgent>,
    sources: &mut Vec<KdlAgentSource>,
) -> ConfigResult<KgAgent> {
    let local_agent_path = location.local_agent(fs, &name)?;

    match &local_agent_path {
        None => {
            if let Some(i) = inline {
                sources.push(KdlAgentSource::LocalInline);
                Ok(KgAgent::new(name.as_ref().to_string()).merge(i.clone()))
            } else {
                Ok(KgAgent::new(name.as_ref().to_string()))
            }
        }
        Some(path) => match KgAgent::from_path(fs, &name, path) {
            None => {
                if let Some(i) = inline {
                    sources.push(KdlAgentSource::LocalInline);
                    Ok(KgAgent::new(name.as_ref().to_string()).merge(i.clone()))
                } else {
                    Ok(KgAgent::new(name.as_ref().to_string()))
                }
            }
            Some(a) => {
                let agent = a?;
                sources.push(KdlAgentSource::LocalFile(path.clone()));
                if let Some(i) = inline {
                    sources.push(KdlAgentSource::LocalInline);
                    Ok(agent.merge(i.clone()))
                } else {
                    Ok(agent)
                }
            }
        },
    }
}

#[derive(Clone, Facet)]
#[facet(opaque)]
pub struct ResolvedAgents {
    #[facet(default)]
    pub agents: HashMap<String, KgAgent>,
    #[facet(skip, default)]
    pub sources: KdlSources,
    #[facet(skip, default)]
    pub has_local: bool,
}

impl Deref for ResolvedAgents {
    type Target = HashMap<String, KgAgent>;

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
/// * `~/.kiro/generators/agents/<agent-name>.toml`
/// * `~/.kiro/generators/manifests/*.toml`
/// * `.kiro/generators/agents/<agent-name>.toml`
/// * `.kiro/generators/manifests/*.toml`
/// ```
#[tracing::instrument(level = "info")]
pub fn discover(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> ConfigResult<ResolvedAgents> {
    // Validate no duplicate agent names
    location.validate(fs, MAX_AGENT_DIR_ENTRIES)?;

    let global_manifests_dir = location.global_manifests_dir();
    let local_manifests_dir = location.local_manifests_dir();

    let global_agents: GeneratorConfig = load_manifests(fs, global_manifests_dir)?;
    let local_agents: GeneratorConfig = load_manifests(fs, local_manifests_dir)?;

    tracing::debug!("found {} local agents", local_agents.agents.len());

    let local_names: HashSet<String> =
        HashSet::from_iter(local_agents.agents.keys().map(|k| k.to_string()));
    let global_names: HashSet<String> =
        HashSet::from_iter(global_agents.agents.keys().map(|k| k.to_string()));
    let mut all_agents_names: HashSet<String> =
        HashSet::with_capacity(global_names.len() + local_names.len());
    all_agents_names.extend(local_names.clone());
    all_agents_names.extend(global_names);

    let mut resolved_agents: HashMap<String, KgAgent> =
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
                if let Some(global_path) = location.global_agent(fs, name)?
                    && let Some(global) = KgAgent::from_path(fs, name, &global_path)
                {
                    agent_sources.push(KdlAgentSource::GlobalFile(global_path));
                    result = result.merge(global?);
                }
                resolved_agents.insert(name.to_string(), result);
            }
            ConfigLocation::Global(_) => {
                let mut global_file = KgAgent::new(name.to_string());
                if let Some(global_path) = location.global_agent(fs, name)?
                    && let Some(agent) = KgAgent::from_path(fs, name, &global_path)
                {
                    agent_sources.push(KdlAgentSource::GlobalFile(global_path));
                    global_file = agent?;
                }
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
    async fn test_discover_local_agents_toml() -> Result<()> {
        let fs = Fs::new();
        let resolved = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;
        let agents = resolved.agents;
        let sources = resolved.sources;
        assert!(!agents.is_empty());
        assert_eq!(sources.keys().len(), 7);
        assert!(sources.contains_key("base"));
        assert!(sources.contains_key("aws-test"));
        assert!(sources.contains_key("dependabot"));
        assert!(sources.contains_key("empty"));

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
    async fn test_discover_global_agents_toml() -> Result<()> {
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
            PathBuf::from(".kiro")
                .join("generators")
                .join("agents")
                .join("bad.toml"),
        );
        assert!(e.is_err());
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_both_agents_toml() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let resolved = discover(
            &fs,
            &ConfigLocation::Both(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        assert_eq!(resolved.len(), 7);

        for (n, agent_sources) in resolved.sources.iter() {
            if n == "aws-test" {
                assert_eq!(agent_sources.len(), 4);
            } else if n == "empty" {
                assert_eq!(agent_sources.len(), 2);
            } else if n == "default" {
                assert_eq!(agent_sources.len(), 3);
            }
        }

        assert!(!format!("{resolved}").is_empty());
        assert!(!format!("{resolved:?}").is_empty());
        Ok(())
    }
}
