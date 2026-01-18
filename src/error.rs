#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TomlDeserializeError(#[from] facet_toml::DeserializeError<facet_toml::TomlError>),
    #[error(transparent)]
    JsonDeserializeError(#[from] facet_json::DeserializeError<facet_json::JsonError>),
    #[error(transparent)]
    JsonSerializeError(#[from] facet_format::SerializeError<facet_json::JsonSerializeError>),
    #[error(transparent)]
    FileIoError(#[from] std::io::Error),
    #[error("Invalid syntax in file {0}")]
    FileDeserializeError(String),
    #[error("Duplicate agent '{name}' found in {scope}:\n  - {first}\n  - {second}")]
    DuplicateAgent {
        name: String,
        scope: String,
        first: String,
        second: String,
    },
    #[error("error report {0}")]
    Report(String),

    #[error("Directory {0} has too many files or directories")]
    MaxEntities(String),
}
