use {super::*, std::fmt::Display};

pub enum AgentSource {
    Raw(String),
    LocalFile(PathBuf),
    LocalInline(String),
    GlobalFile(PathBuf),
    GlobalInline(String),
}

impl Display for AgentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSource::Raw(_) => write!(f, "raw"),
            AgentSource::GlobalInline(_) => write!(f, "global-inline",),
            AgentSource::LocalFile(path) => write!(f, "file://{}", path.display()),
            AgentSource::LocalInline(_) => write!(f, "local-inline",),
            AgentSource::GlobalFile(path) => write!(f, "file://{}", path.display()),
        }
    }
}

impl AgentSource {
    pub(super) fn to_cell(&self, fs: &Fs) -> Cell {
        match self {
            AgentSource::Raw(_) => Cell::new(format!("{self}")),
            AgentSource::GlobalInline(content) | AgentSource::LocalInline(content) => {
                if content.is_empty() {
                    Cell::new(format!("{self} [empty]"))
                        .fg(Color::Red)
                        .add_attributes(vec![Attribute::Bold, Attribute::SlowBlink]) // I am having fun
                } else {
                    Cell::new(format!("{self}"))
                }
            }
            AgentSource::GlobalFile(path) | AgentSource::LocalFile(path) => {
                match (fs.exists(path), fs.read_to_string_sync(path)) {
                    (true, Ok(_)) => Cell::new(format!("{self}")),
                    _ => Cell::new(format!("{self} [empty]")).fg(Color::Red),
                }
            }
        }
    }

    pub(super) fn new_local_agent(name: &str, fs: &Fs) -> (Self, bool) {
        let location = PathBuf::from(".kiro")
            .join("generators")
            .join(format!("{name}.toml"));
        let content = fs.read_to_string_sync(&location).ok().unwrap_or_default();
        (Self::LocalFile(location), content.is_empty())
    }

    pub(super) fn to_source(&self, fs: &Fs) -> config::File<FileSourceString, MergingTomlFormat> {
        match self {
            AgentSource::GlobalInline(content)
            | AgentSource::LocalInline(content)
            | AgentSource::Raw(content) => config::File::from_str(content, MergingTomlFormat),
            AgentSource::GlobalFile(path) | AgentSource::LocalFile(path) => {
                match (fs.exists(path), fs.read_to_string_sync(path)) {
                    (false, _) => config::File::from_str("", MergingTomlFormat),
                    (true, Ok(content)) => config::File::from_str(&content, MergingTomlFormat),
                    (true, Err(e)) => {
                        tracing::debug!("failed to read from file {}: {e}", path.display());
                        config::File::from_str("", MergingTomlFormat)
                    }
                }
            }
        }
    }
}
