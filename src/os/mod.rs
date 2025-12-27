mod fs;
pub use fs::Fs;
#[cfg(test)]
const WINDOWS_USER_HOME: &str = "C:\\Users\\testuser";
#[cfg(test)]
const UNIX_USER_HOME: &str = "/home/testuser";

#[cfg(test)]
pub const ACTIVE_USER_HOME: &str = if cfg!(windows) {
    WINDOWS_USER_HOME
} else {
    UNIX_USER_HOME
};
