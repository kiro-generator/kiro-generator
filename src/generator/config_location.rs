use {
    super::*,
    color_eyre::eyre::{WrapErr, bail},
    std::{fmt::Display, path::Path},
};

/// Recursively search for an agent TOML file in a directory tree
fn find_agent_file(
    fs: &Fs,
    dir: &Path,
    agent_name: &str,
    current_depth: usize,
    max_depth: usize,
) -> crate::Result<Option<PathBuf>> {
    if current_depth > max_depth || !fs.exists(dir) {
        return Ok(None);
    }

    let entries = fs.read_dir_sync(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .wrap_err_with(|| format!("Failed to read metadata for {}", path.display()))?;

        if metadata.is_dir() {
            // Recurse into subdirectory
            if let Some(found) =
                find_agent_file(fs, &path, agent_name, current_depth + 1, max_depth)?
            {
                return Ok(Some(found));
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && stem == agent_name
        {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

/// Represents where configuration files are located
pub enum ConfigLocation {
    /// Only global ~/.kiro/generators
    Global(PathBuf),
    /// Only local ./.kiro/generators
    Local,
    /// Both global and local configs (local overrides global)
    Both(PathBuf),
}

impl ConfigLocation {
    pub fn global_path(&self) -> PathBuf {
        match self {
            ConfigLocation::Both(p) | Self::Global(p) => p.clone(),
            #[cfg(not(test))]
            ConfigLocation::Local => PathBuf::default(),
            #[cfg(test)]
            ConfigLocation::Local => PathBuf::from("dev").join("null"),
        }
    }

    /// Validate that there are no duplicate agent names in the agent
    /// directories
    #[allow(clippy::too_many_arguments)]
    pub fn validate(&self, fs: &Fs, max_entities: usize) -> crate::Result<()> {
        fn scan_for_duplicates(
            fs: &Fs,
            dir: &Path,
            current_depth: usize,
            max_depth: usize,
            seen: &mut HashMap<String, PathBuf>,
            scope: &str,
            total_entities: &mut usize,
            max_entities: usize,
        ) -> crate::Result<()> {
            if current_depth > max_depth || !fs.exists(dir) {
                return Ok(());
            }

            let entries = fs.read_dir_sync(dir)?;
            for entry in entries {
                let entry = entry?;
                *total_entities += 1;
                if *total_entities > max_entities {
                    let path = entry.path();
                    bail!(
                        "Directory {} has too many files or directories",
                        path.display()
                    );
                }
                let path = entry.path();
                let metadata = entry.metadata().wrap_err("Failed to read metadata")?;

                if metadata.is_dir() {
                    scan_for_duplicates(
                        fs,
                        &path,
                        current_depth + 1,
                        max_depth,
                        seen,
                        scope,
                        total_entities,
                        max_entities,
                    )?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("toml")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    let agent_name = stem.to_string();
                    if let Some(existing_path) = seen.get(&agent_name) {
                        bail!(
                            "Duplicate agent '{}' found in {}:\n  - {}\n  - {}",
                            agent_name,
                            scope,
                            existing_path.display(),
                            path.display()
                        );
                    }
                    seen.insert(agent_name, path);
                }
            }

            Ok(())
        }

        // Validate global agents if applicable
        let mut total_entities = 0;

        if !matches!(self, ConfigLocation::Local) {
            let global_agents_dir = match self {
                ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("agents"),
                ConfigLocation::Local => unreachable!(),
            };
            let mut global_seen = HashMap::new();
            scan_for_duplicates(
                fs,
                &global_agents_dir,
                0,
                super::MAX_AGENT_DIR_DEPTH,
                &mut global_seen,
                "global agents",
                &mut total_entities,
                max_entities,
            )?;
        }

        // Validate local agents if applicable (separate scope, can override global)
        if !matches!(self, ConfigLocation::Global(_)) {
            let local_agents_dir = PathBuf::from(".kiro/generators/agents");
            let mut local_seen = HashMap::new();
            scan_for_duplicates(
                fs,
                &local_agents_dir,
                0,
                super::MAX_AGENT_DIR_DEPTH,
                &mut local_seen,
                "local agents",
                &mut total_entities,
                max_entities,
            )?;
        }

        Ok(())
    }

    /// Get path to agent definition file in agents/ directory (searches
    /// recursively)
    pub fn global_agent(&self, fs: &Fs, name: impl AsRef<str>) -> crate::Result<Option<PathBuf>> {
        let agents_dir = match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("agents"),
            ConfigLocation::Local => return Ok(None),
        };

        find_agent_file(
            fs,
            &agents_dir,
            name.as_ref(),
            0,
            super::MAX_AGENT_DIR_DEPTH,
        )
    }

    /// Get path to agent definition file in agents/ directory (searches
    /// recursively)
    pub fn local_agent(&self, fs: &Fs, name: impl AsRef<str>) -> crate::Result<Option<PathBuf>> {
        let agents_dir = match self {
            Self::Local | Self::Both(_) => PathBuf::from(".kiro/generators/agents"),
            Self::Global(_) => return Ok(None),
        };

        find_agent_file(
            fs,
            &agents_dir,
            name.as_ref(),
            0,
            super::MAX_AGENT_DIR_DEPTH,
        )
    }

    /// Get path to global manifests directory
    pub fn global_manifests_dir(&self) -> PathBuf {
        match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("manifests"),
            #[cfg(not(test))]
            ConfigLocation::Local => PathBuf::default(),
            #[cfg(test)]
            ConfigLocation::Local => PathBuf::from("dev").join("null"),
        }
    }

    /// Get path to local manifests directory
    pub fn local_manifests_dir(&self) -> PathBuf {
        match self {
            Self::Local | Self::Both(_) => {
                PathBuf::from(".kiro").join("generators").join("manifests")
            }
            #[cfg(not(test))]
            Self::Global(_) => PathBuf::default(),
            #[cfg(test)]
            Self::Global(_) => PathBuf::from("dev").join("null"),
        }
    }
}

impl Debug for ConfigLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLocation::Global(_) => write!(f, "[global]"),
            ConfigLocation::Local => write!(f, "[local]"),
            ConfigLocation::Both(_) => {
                write!(f, "[global,local]")
            }
        }
    }
}

impl Display for ConfigLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLocation::Global(p) => write!(f, "global={}", p.display()),
            ConfigLocation::Local => write!(f, "local"),
            ConfigLocation::Both(p) => {
                write!(f, "global={},local", p.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::os::ACTIVE_USER_HOME};

    #[tokio::test]
    async fn test_validate_local_no_duplicates() -> crate::Result<()> {
        let fs = Fs::new();
        let location = ConfigLocation::Local;
        location.validate(&fs, 1000)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_global_no_duplicates() -> crate::Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let location = ConfigLocation::Global(g_path);
        location.validate(&fs, 1000)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_both_no_duplicates() -> crate::Result<()> {
        let fs = Fs::new();
        let g_path = PathBuf::from(ACTIVE_USER_HOME)
            .join(".kiro")
            .join("generators");
        let location = ConfigLocation::Both(g_path);
        location.validate(&fs, 1000)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_max_entities_exceeded() {
        let fs = Fs::new();
        let location = ConfigLocation::Local;
        let result = location.validate(&fs, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too many"));
    }

    #[tokio::test]
    async fn test_validate_with_duplicate_agents() -> crate::Result<()> {
        let fs = Fs::new();
        let agents_dir = PathBuf::from(".kiro/generators/agents");
        fs.create_dir_all(&agents_dir).await?;

        // Create duplicate agent files
        let dup1 = agents_dir.join("duplicate.toml");
        let subdir = agents_dir.join("subdir");
        fs.create_dir_all(&subdir).await?;
        let dup2 = subdir.join("duplicate.toml");

        fs.write(&dup1, b"description = \"First\"").await?;
        fs.write(&dup2, b"description = \"Second\"").await?;

        let location = ConfigLocation::Local;
        let result = location.validate(&fs, 1000);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"),);
        Ok(())
    }

    #[tokio::test]
    async fn test_find_agent_file_in_subdirectory() -> crate::Result<()> {
        let fs = Fs::new();
        let agents_dir = PathBuf::from(".kiro/generators/agents");
        fs.create_dir_all(&agents_dir).await?;

        // Create agent in subdirectory
        let subdir = agents_dir.join("aws-mcps");
        fs.create_dir_all(&subdir).await?;
        let agent_file = subdir.join("eks.toml");
        fs.write(&agent_file, b"description = \"EKS agent\"")
            .await?;

        let result = find_agent_file(&fs, &agents_dir, "eks", 0, 5)?;
        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.ends_with("aws-mcps/eks.toml"));
        Ok(())
    }

    #[tokio::test]
    async fn test_find_agent_file_respects_max_depth() -> crate::Result<()> {
        let fs = Fs::new();
        let agents_dir = PathBuf::from(".kiro/generators/agents");
        fs.create_dir_all(&agents_dir).await?;

        // Create deeply nested agent (depth 3)
        let deep_dir = agents_dir.join("a").join("b").join("c");
        fs.create_dir_all(&deep_dir).await?;
        let agent_file = deep_dir.join("deep.toml");
        fs.write(&agent_file, b"description = \"Deep agent\"")
            .await?;

        // Should find with max_depth=5
        let result = find_agent_file(&fs, &agents_dir, "deep", 0, 5)?;
        assert!(result.is_some());

        // Should NOT find with max_depth=2
        let result = find_agent_file(&fs, &agents_dir, "deep", 0, 2)?;
        assert!(result.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_find_agent_file_returns_none_when_not_found() -> crate::Result<()> {
        let fs = Fs::new();
        let agents_dir = PathBuf::from(".kiro/generators/agents");
        fs.create_dir_all(&agents_dir).await?;

        let result = find_agent_file(&fs, &agents_dir, "nonexistent", 0, 5)?;
        assert!(result.is_none());
        Ok(())
    }
}
