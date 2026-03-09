use {
    super::*,
    crate::{GeneratorConfig, Manifest},
    color_eyre::eyre::bail,
    std::path::Path,
};

/// Load all TOML files from a manifests directory and combine them.
/// Returns the merged config and a mapping of agent name → manifest file path.
#[tracing::instrument(level = "info", skip(fs), fields(dir = %dir.as_ref().display()))]
fn load_manifests(
    fs: &Fs,
    is_local: bool,
    dir: impl AsRef<Path>,
) -> crate::Result<HashMap<String, SourceSlot>> {
    let dir_path = dir.as_ref();

    if !fs.exists(dir_path) {
        tracing::debug!("dir does not exist");
        return Ok(HashMap::default());
    }

    let entries = fs.read_dir_sync(dir_path)?;
    let mut merged: HashMap<String, SourceSlot> = HashMap::new();
    let mut manifest_files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            tracing::debug!("found file {}", path.display());
            manifest_files.push(path);
        }
    }
    // Sort for deterministic order
    manifest_files.sort();
    for path in manifest_files {
        let _span = tracing::info_span!("parse_manifest", path = %path.display()).entered();
        if let Some(config_result) = crate::toml_parse_path(fs, &path) {
            let config: GeneratorConfig = config_result?;
            let config = config.populate_names();
            // Check for duplicate agent names
            for name in config.agents.keys() {
                if let Some(a) = merged.get(name) {
                    bail!(
                        "Duplicate agent '{name}' found in manifests file {} ({})",
                        path.display(),
                        a.path
                            .as_ref()
                            .map(|p| p.path().display().to_string())
                            .unwrap_or_default()
                    );
                }
            }
            tracing::debug!("adding {} agents to manifest list", config.agents.len());
            let sources: HashMap<String, SourceSlot> = config
                .agents
                .iter()
                .map(|(k, v)| {
                    (k.clone(), SourceSlot {
                        path: Some(KgAgentSource::manifest(path.clone(), is_local)),
                        manifest: v.clone(),
                    })
                })
                .collect();
            merged.extend(sources);
        }
    }

    Ok(merged)
}

#[cfg(test)]
pub fn load_inline(fs: &Fs, path: impl AsRef<Path>) -> crate::Result<GeneratorConfig> {
    let doc = crate::toml_parse_path(fs, path);
    match doc {
        None => Ok(GeneratorConfig::default()),
        Some(d) => {
            let agents: GeneratorConfig = d?;
            Ok(agents.populate_names())
        }
    }
}

fn merge_manifests(
    name: &str,
    global_manifest: &SourceSlot,
    local_manifest: &SourceSlot,
    global_agent_file: &SourceSlot,
    local_agent_file: &SourceSlot,
    location: &ConfigLocation,
) -> Manifest {
    // Highest-precedence source must be first: Manifest::merge keeps existing
    // values and only fills gaps from `other`.
    // For `Both`, precedence is:
    // local agent file > local manifest > global agent file > global manifest.
    let mut ordered: Vec<&Manifest> = Vec::with_capacity(4);

    match location {
        ConfigLocation::Global(_) => {
            if global_agent_file.path.is_some() {
                ordered.push(&global_agent_file.manifest);
            }
            if global_manifest.path.is_some() {
                ordered.push(&global_manifest.manifest);
            }
        }
        ConfigLocation::Local => {
            if local_agent_file.path.is_some() {
                ordered.push(&local_agent_file.manifest);
            }
            if local_manifest.path.is_some() {
                ordered.push(&local_manifest.manifest);
            }
        }
        ConfigLocation::Both(_) => {
            if local_agent_file.path.is_some() {
                ordered.push(&local_agent_file.manifest);
            }
            if local_manifest.path.is_some() {
                ordered.push(&local_manifest.manifest);
            }
            if global_agent_file.path.is_some() {
                ordered.push(&global_agent_file.manifest);
            }
            if global_manifest.path.is_some() {
                ordered.push(&global_manifest.manifest);
            }
        }
    }

    let mut iter = ordered.into_iter();
    let mut merged = match iter.next() {
        Some(first) => first.clone(),
        None => Manifest {
            name: String::from(name),
            ..Default::default()
        },
    };

    for other in iter {
        merged = merged.merge(other.clone());
    }

    if merged.name.is_empty() {
        merged.name = String::from(name)
    }

    merged
}

#[tracing::instrument(level = "info", skip(fs), fields(location = %location))]
pub fn load_sources(fs: &Fs, location: &ConfigLocation) -> crate::Result<Vec<AgentSourceSlots>> {
    let global_manifests_dir = location.global_manifests_dir();
    let local_manifests_dir = location.local_manifests_dir();
    let mut global_agents = load_manifests(fs, false, global_manifests_dir)?;
    let mut local_agents = load_manifests(fs, true, local_manifests_dir)?;
    tracing::debug!(
        "found {} global manifests and {} local manifests",
        global_agents.len(),
        local_agents.len()
    );
    let all_agents_names: HashSet<String> = global_agents
        .keys()
        .chain(local_agents.keys())
        .cloned()
        .collect();

    let mut slots: Vec<AgentSourceSlots> = Vec::with_capacity(all_agents_names.len());

    for name in all_agents_names {
        let _span = tracing::info_span!("merge_manifest", agent = name).entered();
        let global_manifest: SourceSlot = global_agents.remove(&name).unwrap_or_default();
        let local_manifest: SourceSlot = local_agents.remove(&name).unwrap_or_default();
        let global_agent_file = SourceSlot::from_agent_path(
            fs,
            &name,
            location,
            true,
            global_manifest.manifest.template,
        )?;
        let local_agent_file = SourceSlot::from_agent_path(
            fs,
            &name,
            location,
            false,
            local_manifest.manifest.template,
        )?;
        let merged = merge_manifests(
            &name,
            &global_manifest,
            &local_manifest,
            &global_agent_file,
            &local_agent_file,
            location,
        );
        slots.push(AgentSourceSlots {
            name,
            global_manifest,
            local_manifest,
            global_agent_file,
            local_agent_file,
            merged,
        });
    }

    slots.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(slots)
}

/// First pass: Discover all agents from configuration files
///
/// Precedence (highest → lowest):
/// 1. `.kiro/generators/agents/<name>.toml`      (local agent file)
/// 2. `.kiro/generators/manifests/*.toml`         (local manifest)
/// 3. `~/.kiro/generators/agents/<name>.toml`     (global agent file)
/// 4. `~/.kiro/generators/manifests/*.toml`       (global manifest)
#[tracing::instrument(level = "info", skip(format))]
pub fn discover(
    fs: &Fs,
    location: &ConfigLocation,
    format: &crate::output::OutputFormat,
) -> crate::Result<HashMap<String, AgentSourceSlots>> {
    location.validate(fs, MAX_AGENT_DIR_ENTRIES)?;
    let agents = load_sources(fs, location)?;
    if let Err(e) = format.sources(&agents) {
        tracing::error!("Failed to format sources: {}", e);
    }
    Ok(agents.into_iter().map(|s| (s.name.clone(), s)).collect())
}

#[cfg(test)]
mod tests {
    use {super::*, crate::os::ACTIVE_USER_HOME, color_eyre::eyre::eyre, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_local_agents_toml() -> Result<()> {
        let fs = Fs::new();
        let agents = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;
        let sources: HashMap<String, Vec<KgAgentSource>> =
            agents.iter().map(|(k, v)| (k.clone(), v.into())).collect();
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

        for agent_sources in sources.values() {
            for s in agent_sources {
                assert!(
                    matches!(
                        s,
                        KgAgentSource::LocalFile(_) | KgAgentSource::LocalManifest(_)
                    ),
                    "agent is not local"
                );
            }
        }
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_global_agents_toml() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let agents = discover(
            &fs,
            &ConfigLocation::Global(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;
        let sources: HashMap<String, Vec<KgAgentSource>> =
            agents.iter().map(|(k, v)| (k.clone(), v.into())).collect();
        assert_eq!(agents.len(), 3);
        for agent_sources in sources.values() {
            for s in agent_sources {
                assert!(
                    matches!(
                        s,
                        KgAgentSource::GlobalManifest(_) | KgAgentSource::GlobalFile(_)
                    ),
                    "agent is not global"
                );
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
        let agents = discover(
            &fs,
            &ConfigLocation::Both(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;
        let sources: HashMap<String, Vec<KgAgentSource>> =
            agents.iter().map(|(k, v)| (k.clone(), v.into())).collect();

        assert_eq!(agents.len(), 7);

        for (n, agent_sources) in sources.iter() {
            if n == "aws-test" {
                assert_eq!(agent_sources.len(), 4);
            } else if n == "empty" {
                assert_eq!(agent_sources.len(), 2);
            } else if n == "default" {
                assert_eq!(agent_sources.len(), 3);
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_load_sources_both() -> Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let sources = load_sources(&fs, &ConfigLocation::Both(g_path))?;

        let aws_test = sources
            .iter()
            .find(|slot| slot.name == "aws-test")
            .ok_or_else(|| eyre!("missing aws-test manifest slots"))?;
        assert!(aws_test.local_manifest.path.is_some());
        assert!(aws_test.global_manifest.path.is_some());
        assert!(aws_test.local_agent_file.path.is_some());
        assert!(aws_test.global_agent_file.path.is_some());

        let empty = sources
            .iter()
            .find(|slot| slot.name == "empty")
            .ok_or_else(|| eyre!("missing empty manifest slots"))?;
        assert!(empty.local_manifest.path.is_some());
        assert!(empty.global_manifest.path.is_none());
        assert!(empty.local_agent_file.path.is_some());
        assert!(empty.global_agent_file.path.is_none());

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_template_not_inherited() -> Result<()> {
        let fs = Fs::new();
        let agents = discover(
            &fs,
            &ConfigLocation::Local,
            &crate::output::OutputFormat::Table(true),
        )?;

        // base is a template
        let base = &agents.get("base").unwrap().merged;
        assert!(base.template);

        // aws-test inherits from base but is NOT a template
        let aws_test = &agents.get("aws-test").unwrap().merged;
        assert!(!aws_test.template);

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_is_template() -> Result<()> {
        let fs = Fs::new();

        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let agents = discover(
            &fs,
            &ConfigLocation::Both(g_path.clone()),
            &crate::output::OutputFormat::Table(true),
        )?;

        let generator = super::Generator {
            global_path: g_path,
            agents,
            fs,
            format: crate::output::OutputFormat::Plain,
        };

        let templates = generator
            .write_all(true, false)
            .await?
            .iter()
            .filter(|a| a.is_template())
            .count();
        assert_eq!(3, templates);

        Ok(())
    }
}
