use {
    super::*,
    crate::config::{GeneratorConfig, KgAgent},
    color_eyre::eyre::bail,
    std::{fmt::Display, ops::Deref, path::Path},
};

/// Load all TOML files from a manifests directory and merge them
fn load_manifests(fs: &Fs, dir: impl AsRef<Path>) -> crate::Result<GeneratorConfig> {
    let dir_path = dir.as_ref();

    if !fs.exists(dir_path) {
        return Ok(GeneratorConfig::default());
    }

    let entries = fs.read_dir_sync(dir_path)?;
    let mut merged = GeneratorConfig::default();
    let mut agent_files: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            agent_files.push(path);
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
                    bail!("Duplicate agent '{}' found in manifests directory", name);
                }
            }

            merged.agents.extend(config.agents);
        }
    }

    Ok(merged)
}

#[cfg(test)]
pub fn load_inline(fs: &Fs, path: impl AsRef<Path>) -> crate::Result<GeneratorConfig> {
    let doc = crate::config::toml_parse_path(fs, path);
    match doc {
        None => Ok(GeneratorConfig::default()),
        Some(d) => {
            let agents: GeneratorConfig = d?;
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
) -> crate::Result<Option<KgAgent>> {
    let local_agent_path = location.local_agent(fs, &name)?;
    let template = inline.map(|i| i.template).unwrap_or(false);

    match &local_agent_path {
        None => Ok(inline.cloned()),
        Some(path) => {
            // Template status is only defined in manifests, passed via inline config
            match KgAgent::from_path(fs, &name, path, template) {
                None => Ok(None),
                Some(a) => {
                    let agent = a?;
                    sources.push(KdlAgentSource::LocalFile(path.clone()));
                    if let Some(i) = inline {
                        sources.push(KdlAgentSource::LocalInline);
                        Ok(Some(agent.merge(i.clone())))
                    } else {
                        Ok(Some(agent))
                    }
                }
            }
        }
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
#[tracing::instrument(level = "info", skip(format))]
pub fn discover(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> crate::Result<ResolvedAgents> {
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
    let all_agents_names: HashSet<String> =
        local_names.iter().chain(&global_names).cloned().collect();

    let mut resolved_agents: HashMap<String, KgAgent> =
        HashMap::with_capacity(all_agents_names.len());
    let mut sources: KdlSources = KdlSources::from(&all_agents_names);

    for (name, agent_sources) in sources.iter_mut() {
        let span = tracing::debug_span!("agent", name = ?name);
        let _enter = span.enter();
        tracing::trace!("matching location");

        let template = global_agents.get(name).map(|a| a.template).unwrap_or(false);
        let mut global_path_buf: Option<PathBuf> = None;
        let file_source: Option<KgAgent> = if let Some(global_path) =
            location.global_agent(fs, name)?
            && let Some(agent) = KgAgent::from_path(fs, name, &global_path, template)
        {
            global_path_buf = Some(global_path);
            Some(agent?)
        } else {
            None
        };
        match location {
            ConfigLocation::Local => {
                if let Some(a) =
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?
                {
                    resolved_agents.insert(name.to_string(), a);
                }
            }
            ConfigLocation::Both(_) => {
                // Match all possible combinations of agent sources
                // Tuple: (local_file, global_inline, global_file)
                // Merge order: local <- global_file <- global_inline (rightmost wins)
                match (
                    process_local(fs, name, location, local_agents.get(name), agent_sources)?,
                    global_agents.get(name),
                    file_source,
                ) {
                    // Invariant violation - at least one source must exist
                    (None, None, None) => panic!("agent definitions are invalid"),

                    // All three sources present: merge local <- global_file <- global_inline
                    (Some(l), Some(g), Some(gf)) => {
                        tracing::trace!("found local file, inline global and global file source");
                        agent_sources.push(KdlAgentSource::GlobalInline);
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), l.merge(gf.merge(g.clone())));
                    }

                    // Global sources only: merge global_file <- global_inline
                    (None, Some(g), Some(gf)) => {
                        tracing::trace!("found inline global and global file source");
                        agent_sources.push(KdlAgentSource::GlobalInline);
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), gf.merge(g.clone()));
                    }

                    // Global file only
                    (None, None, Some(gf)) => {
                        tracing::trace!("found global file source");
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), gf);
                    }

                    // Local + global inline
                    (Some(l), Some(g), None) => {
                        tracing::trace!("found local and inline global");
                        resolved_agents.insert(name.to_string(), l.merge(g.clone()));
                    }

                    // Local + global file
                    (Some(l), None, Some(gf)) => {
                        tracing::trace!("found local and global file");
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), l.merge(gf));
                    }

                    // Local only
                    (Some(l), None, None) => {
                        tracing::trace!("found only local");
                        resolved_agents.insert(name.to_string(), l);
                    }

                    // Global inline only
                    (None, Some(g), None) => {
                        tracing::trace!("found only global manifest file");
                        agent_sources.push(KdlAgentSource::GlobalInline);
                        resolved_agents.insert(name.to_string(), g.clone());
                    }
                };
            }
            ConfigLocation::Global(_) => {
                match (global_agents.get(name), file_source) {
                    (Some(i), None) => {
                        agent_sources.push(KdlAgentSource::GlobalInline);
                        resolved_agents.insert(name.to_string(), i.clone());
                    }
                    (None, Some(a)) => {
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), a);
                    }
                    (Some(i), Some(a)) => {
                        agent_sources.push(KdlAgentSource::GlobalInline);
                        agent_sources.push(KdlAgentSource::GlobalFile(global_path_buf.unwrap()));
                        resolved_agents.insert(name.to_string(), a.merge(i.clone()));
                    }
                    (None, None) => {
                        tracing::debug!("no global agent definition found");
                    }
                };
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

    #[tokio::test]
    #[test_log::test]
    async fn test_template_not_inherited() -> Result<()> {
        let fs = Fs::new();
        let resolved = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;

        // base is a template
        let base = resolved.agents.get("base").unwrap();
        assert!(base.template);

        // aws-test inherits from base but is NOT a template
        let aws_test = resolved.agents.get("aws-test").unwrap();
        assert!(!aws_test.template);

        Ok(())
    }
}
