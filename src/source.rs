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
        let path = if global {
            location.global_agent(fs, name)?
        } else {
            location.local_agent(fs, name)?
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
    pub fn new(
        name: String,
        global_manifest: SourceSlot,
        local_manifest: SourceSlot,
        global_agent_file: SourceSlot,
        local_agent_file: SourceSlot,
        merged: Manifest,
    ) -> Self {
        Self {
            name,
            global_manifest,
            local_manifest,
            global_agent_file,
            local_agent_file,
            merged,
        }
    }

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
        let mut sources = Vec::with_capacity(4);
        let gm = value.global_manifest.path.clone();
        let lm = value.local_manifest.path.clone();
        let ag = value.global_agent_file.path.clone();
        let al = value.local_agent_file.path.clone();

        if let Some(s) = gm {
            sources.push(s)
        }
        if let Some(s) = lm {
            sources.push(s)
        }
        if let Some(s) = ag {
            sources.push(s)
        }
        if let Some(s) = al {
            sources.push(s)
        }
        sources
    }
}

#[derive(Clone)]
pub enum KgAgentSource {
    LocalFile(PathBuf),
    LocalManifest(PathBuf),
    GlobalFile(PathBuf),
    GlobalManifest(PathBuf),
}

impl Default for KgAgentSource {
    fn default() -> Self {
        Self::LocalManifest(PathBuf::new())
    }
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
    use super::*;

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
}
