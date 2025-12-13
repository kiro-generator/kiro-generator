use {
    serde::{Deserialize, Serialize},
    std::{borrow::Borrow, hash::Hash, ops::Deref},
};

/// Subject of the tool name change. For tools in mcp servers, you would need to
/// prefix them with their server names
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct OriginalToolName(String);

impl Deref for OriginalToolName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<str> for OriginalToolName {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}
