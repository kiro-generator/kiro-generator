use {
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        ops::{Deref, DerefMut},
        path::{Path, PathBuf},
    },
    super_table::Cell,
};

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
impl KgAgentSource {
    fn is_local(&self) -> bool {
        matches!(self, Self::LocalFile(_) | Self::LocalManifest(_))
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

#[derive(Clone, Default)]
pub struct KgSources(pub HashMap<String, Vec<KgAgentSource>>);

impl Debug for KgSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sources={}", self.0.len())
    }
}
impl From<&HashSet<String>> for KgSources {
    fn from(value: &HashSet<String>) -> Self {
        let mut sources = Self(HashMap::with_capacity(value.len()));
        value.iter().for_each(|n| sources.add(n));
        sources
    }
}
impl KgSources {
    pub fn is_local(&self, name: impl AsRef<str>) -> bool {
        if let Some(a) = self.get(name.as_ref()) {
            return a.iter().any(|p| p.is_local());
        }
        false
    }

    fn add(&mut self, name: &str) {
        self.0.insert(name.to_string(), Vec::with_capacity(4));
    }
}

impl Deref for KgSources {
    type Target = HashMap<String, Vec<KgAgentSource>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KgSources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

    #[test]
    fn kg_sources_debug() {
        let mut sources = KgSources::default();
        sources.add("agent1");
        sources.add("agent2");
        assert_eq!(format!("{:?}", sources), "sources=2");
    }
}
