use {super::*, std::fmt::Display};

/// Represents where configuration files are located
pub enum ConfigLocation {
    /// Only global ~/.kiro/generators/kg.toml
    Global(PathBuf),
    /// Only local .kiro/generators/kg.toml
    Local,
    /// Both global and local configs (local overrides global)
    Both(PathBuf),
}

impl ConfigLocation {
    #[cfg(test)]
    pub fn global(&self) -> PathBuf {
        match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.clone(),
            ConfigLocation::Local => PathBuf::from("dev").join("null"),
        }
    }

    /// Returns the global config path, or /dev/null in tests if not set
    #[cfg(not(test))]
    pub fn global(&self) -> PathBuf {
        match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.clone(),
            ConfigLocation::Local => PathBuf::default(),
        }
    }

    pub fn local(&self) -> PathBuf {
        match self {
            Self::Local | Self::Both(_) => {
                PathBuf::from(".kiro").join("generators").join("kg.toml")
            }
            Self::Global(_) => PathBuf::default(),
        }
    }

    /// Validates that at least one config file exists
    pub fn is_valid(&self, fs: &Fs) -> Result<()> {
        let global_exists = fs.exists(self.global());
        let local_exists = fs.exists(self.local());

        if !global_exists && !local_exists {
            return Err(eyre!(
                "no kg.toml found at global ({}) or local ({})",
                self.global().display(),
                self.local().display()
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
