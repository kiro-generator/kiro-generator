#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("syntax error in file {0}")]
    DeserializeError(String, facet_kdl::KdlDeserializeError),
}
