use {
    crate::{ConfigLocation, Manifest, os::Fs},
    std::{
        fmt::{Debug, Display},
        path::{Path, PathBuf},
    },
    super_table::Cell,
};

#[derive(Clone, Default)]
pub struct SourceSlot {
    pub path: Option<KgAgentSource>,
    pub manifest: Manifest,
}

impl SourceSlot {
    pub fn source_type(&self) -> Option<String> {
        self.path.as_ref().map(|p| p.source_type().to_string())
    }

    pub fn from_agent_path(
        fs: &Fs,
        name: &str,
        location: &ConfigLocation,
        global: bool,
        template: bool,
    ) -> crate::Result<Self> {
        let (path, path_type) = if global {
            (location.global_agent(fs, name)?, "global_path")
        } else {
            (location.local_agent(fs, name)?, "local_path")
        };

        match &path {
            None => tracing::debug!("{path_type}=not found"),
            Some(p) => tracing::debug!("{path_type}={}", p.display()),
        };
        match path {
            None => Ok(Self::default()),
            Some(path) => match Manifest::from_path(fs, name, &path, template) {
                None => Ok(Self::default()),
                Some(result) => {
                    let manifest = result?;
                    Ok(Self {
                        path: Some(if global {
                            KgAgentSource::GlobalFile(path)
                        } else {
                            KgAgentSource::LocalFile(path)
                        }),
                        manifest,
                    })
                }
            },
        }
    }
}

#[derive(Clone, Default)]
pub struct AgentSourceSlots {
    pub name: String,
    pub global_manifest: SourceSlot,
    pub local_manifest: SourceSlot,
    pub global_agent_file: SourceSlot,
    pub local_agent_file: SourceSlot,
    pub merged: Manifest,
}

impl AgentSourceSlots {
    pub fn has_local(&self) -> bool {
        self.local_manifest.path.is_some() || self.local_agent_file.path.is_some()
    }
}

impl Debug for AgentSourceSlots {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}][local={}]", self.name, self.has_local())
    }
}

impl From<&AgentSourceSlots> for Vec<KgAgentSource> {
    fn from(value: &AgentSourceSlots) -> Self {
        [
            &value.global_manifest.path,
            &value.local_manifest.path,
            &value.global_agent_file.path,
            &value.local_agent_file.path,
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect()
    }
}

#[derive(Clone)]
pub enum KgAgentSource {
    LocalFile(PathBuf),
    LocalManifest(PathBuf),
    GlobalFile(PathBuf),
    GlobalManifest(PathBuf),
}

impl Display for KgAgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KgAgentSource::GlobalManifest(p) => write!(f, "global-manifest:{}", p.display()),
            KgAgentSource::GlobalFile(p) => write!(f, "global-file:{}", p.display()),
            KgAgentSource::LocalManifest(p) => write!(f, "local-manifest:{}", p.display()),
            KgAgentSource::LocalFile(p) => write!(f, "local-file:{}", p.display()),
        }
    }
}

impl Debug for KgAgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KgAgentSource::GlobalManifest(p) => write!(f, "{}", p.display()),
            KgAgentSource::GlobalFile(p) => write!(f, "{}", p.display()),
            KgAgentSource::LocalManifest(p) => write!(f, "{}", p.display()),
            KgAgentSource::LocalFile(p) => write!(f, "{}", p.display()),
        }
    }
}

impl AsRef<Path> for KgAgentSource {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

impl KgAgentSource {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::LocalFile(_) | Self::LocalManifest(_))
    }

    pub fn manifest(path: PathBuf, local: bool) -> Self {
        if local {
            Self::LocalManifest(path)
        } else {
            Self::GlobalManifest(path)
        }
    }

    pub fn source_type(&self) -> &str {
        match self {
            Self::GlobalManifest(_) => "global-manifest",
            Self::GlobalFile(_) => "global-file",
            Self::LocalManifest(_) => "local-manifest",
            Self::LocalFile(_) => "local-file",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::GlobalManifest(p)
            | Self::GlobalFile(p)
            | Self::LocalManifest(p)
            | Self::LocalFile(p) => p,
        }
    }

    pub fn to_cell(&self) -> Cell {
        Cell::new(format!("{}:{}", self.source_type(), self.path().display()))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{ConfigLocation, Fs},
    };

    #[test]
    fn kg_agent_source_display() {
        assert_eq!(
            KgAgentSource::GlobalManifest(PathBuf::from("kg.toml")).to_string(),
            "global-manifest:kg.toml"
        );
        assert_eq!(
            KgAgentSource::LocalManifest(PathBuf::from("kg.toml")).to_string(),
            "local-manifest:kg.toml"
        );
        assert_eq!(
            KgAgentSource::GlobalFile(PathBuf::from("/foo")).to_string(),
            "global-file:/foo"
        );
        assert_eq!(
            KgAgentSource::LocalFile(PathBuf::from("bar")).to_string(),
            "local-file:bar"
        );
    }

    #[test]
    fn kg_agent_source_to_cell() {
        assert_eq!(
            KgAgentSource::GlobalManifest(PathBuf::from("kg.toml"))
                .to_cell()
                .content(),
            "global-manifest:kg.toml"
        );
        assert_eq!(
            KgAgentSource::LocalManifest(PathBuf::from("kg.toml"))
                .to_cell()
                .content(),
            "local-manifest:kg.toml"
        );
        assert_eq!(
            KgAgentSource::GlobalFile(PathBuf::from("/foo"))
                .to_cell()
                .content(),
            "global-file:/foo"
        );
        assert_eq!(
            KgAgentSource::LocalFile(PathBuf::from("bar"))
                .to_cell()
                .content(),
            "local-file:bar"
        );
    }

    #[test]
    fn kg_agent_source_helpers() {
        let local_manifest = KgAgentSource::manifest(PathBuf::from("a.toml"), true);
        let global_manifest = KgAgentSource::manifest(PathBuf::from("b.toml"), false);
        let local_file = KgAgentSource::LocalFile(PathBuf::from("local-file.toml"));

        assert!(local_manifest.is_local());
        assert!(!global_manifest.is_local());
        assert!(local_file.is_local());

        assert_eq!(local_manifest.source_type(), "local-manifest");
        assert_eq!(global_manifest.source_type(), "global-manifest");
        assert_eq!(local_file.source_type(), "local-file");

        assert_eq!(local_manifest.path(), Path::new("a.toml"));
        assert_eq!(global_manifest.as_ref(), Path::new("b.toml"));
        assert_eq!(format!("{local_file:?}"), "local-file.toml");
    }

    #[test]
    fn agent_source_slots_has_local() {
        let mut slots = AgentSourceSlots::default();
        assert!(!slots.has_local());

        slots.local_manifest.path = Some(KgAgentSource::LocalManifest(PathBuf::from("kg.toml")));
        assert!(slots.has_local());
    }

    #[tokio::test]
    #[test_log::test]
    async fn source_slot_from_agent_path_returns_default_when_missing() -> crate::Result<()> {
        let fs = Fs::new();
        let slot = SourceSlot::from_agent_path(
            &fs,
            "agent-name-that-does-not-exist",
            &ConfigLocation::Local,
            false,
            false,
        )?;
        assert!(slot.path.is_none());
        Ok(())
    }
}
