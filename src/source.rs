use {
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        ops::{Deref, DerefMut},
        path::PathBuf,
    },
    super_table::Cell,
};

#[derive(Clone, Default)]
pub enum KdlAgentSource {
    LocalFile(PathBuf),
    #[default]
    LocalInline,
    GlobalFile(PathBuf),
    GlobalInline,
}
impl Display for KdlAgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KdlAgentSource::GlobalInline => write!(f, "global-inline"),
            KdlAgentSource::GlobalFile(p) => write!(f, "{}", p.display()),
            KdlAgentSource::LocalInline => write!(f, "local-inline"),
            KdlAgentSource::LocalFile(p) => write!(f, "{}", p.display()),
        }
    }
}
impl KdlAgentSource {
    fn is_local(&self) -> bool {
        matches!(self, Self::LocalFile(_) | Self::LocalInline)
    }

    pub fn to_cell(&self) -> Cell {
        match self {
            KdlAgentSource::GlobalInline => Cell::new("global-inline"),
            KdlAgentSource::GlobalFile(p) => Cell::new(format!("{}", p.display())),
            KdlAgentSource::LocalInline => Cell::new("local-inline"),
            KdlAgentSource::LocalFile(p) => Cell::new(format!("{}", p.display())),
        }
    }
}

#[derive(Clone, Default)]
pub struct KdlSources(pub HashMap<String, Vec<KdlAgentSource>>);

impl Debug for KdlSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sources={}", self.0.len())
    }
}
impl From<&HashSet<String>> for KdlSources {
    fn from(value: &HashSet<String>) -> Self {
        let mut sources = Self(HashMap::with_capacity(value.len()));
        value.iter().for_each(|n| sources.add(n));
        sources
    }
}
impl KdlSources {
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

impl Deref for KdlSources {
    type Target = HashMap<String, Vec<KdlAgentSource>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KdlSources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
