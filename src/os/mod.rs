#![allow(dead_code)]

mod fs;
pub use fs::Fs;

use crate::Result;
const WINDOWS_USER_HOME: &str = "C:\\Users\\testuser";
const UNIX_USER_HOME: &str = "/home/testuser";

pub const ACTIVE_USER_HOME: &str = if cfg!(windows) {
    WINDOWS_USER_HOME
} else {
    UNIX_USER_HOME
};

/// Struct that contains the interface to every system related IO operation.
///
/// Every operation that accesses the file system, environment, or other related
/// platform primitives should be done through a [Context] as this enables
/// testing otherwise untestable code paths in unit tests.
#[derive(Clone, Debug)]
pub struct Os {
    pub fs: Fs,
}

impl Os {
    pub async fn new() -> Result<Self> {
        let fs = Fs::new();

        Ok(Self { fs })
    }
}
