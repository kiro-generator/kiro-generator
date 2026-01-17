use {super::*, std::fmt::Display};

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
    /// Get path to agent definition file in agents/ directory
    pub fn global_agent(&self, name: impl AsRef<str>) -> PathBuf {
        let n = format!("{}.toml", name.as_ref());
        match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => {
                path.join("agents").join(n)
            }
            #[cfg(not(test))]
            ConfigLocation::Local => PathBuf::default(),
            #[cfg(test)]
            ConfigLocation::Local => PathBuf::from("dev").join("null").join(n),
        }
    }

    /// Get path to agent definition file in agents/ directory
    pub fn local_agent(&self, name: impl AsRef<str>) -> PathBuf {
        match self {
            Self::Local | Self::Both(_) => PathBuf::from(".kiro")
                .join("generators")
                .join("agents")
                .join(format!("{}.toml", name.as_ref())),
            #[cfg(not(test))]
            Self::Global(_) => PathBuf::default(),
            #[cfg(test)]
            Self::Global(_) => PathBuf::from("dev").join("null"),
        }
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

    /// Get path to global kg.toml manifest
    pub fn global_kg(&self) -> PathBuf {
        self.global_manifests_dir().join("kg.toml")
    }

    /// Get path to local kg.toml manifest
    pub fn local_kg(&self) -> PathBuf {
        self.local_manifests_dir().join("kg.toml")
    }

    /// Validates that at least one config file exists
    pub fn is_valid(&self, fs: &Fs) -> Result<()> {
        let global_exists = fs.exists(self.global_kg());
        let local_exists = fs.exists(self.local_kg());

        if !global_exists && !local_exists {
            return Err(crate::format_err!(
                "no kg.toml found at global ({}) or local ({})",
                self.global_kg().display(),
                self.local_kg().display()
            ));
        }
        Ok(())
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
