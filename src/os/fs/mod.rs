use {
    color_eyre::eyre::WrapErr,
    std::{
        collections::HashMap,
        fmt::Debug,
        io,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    },
    tempfile::TempDir,
    tokio::fs,
};

pub const WINDOWS_USER_HOME: &str = "C:\\Users\\testuser";
pub const UNIX_USER_HOME: &str = "/home/testuser";

pub const ACTIVE_USER_HOME: &str = if cfg!(windows) {
    WINDOWS_USER_HOME
} else {
    UNIX_USER_HOME
};

// Import platform-specific modules
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use tracing::info;
// Use platform-specific functions
#[cfg(unix)]
use unix::append as platform_append;
#[cfg(windows)]
use windows::append as platform_append;

/// Rust path handling is hard coded to work specific ways depending on the
/// OS that is being executed on. Because of this, if Unix paths are provided,
/// they aren't recognized. For example a leading prefix of '/' isn't considered
/// an absolute path. To fix this, all test paths would need to have windows
/// equivalents which is tedious and can lead to errors and missed test cases.
/// To make writing tests easier, path normalization happens on Windows systems
/// implicitly during test runtime.
#[cfg(test)]
fn normalize_test_path(path: impl AsRef<Path>) -> PathBuf {
    #[cfg(windows)]
    {
        use typed_path::Utf8TypedPath;
        let path_ref = path.as_ref();

        // Only process string paths with forward slashes
        let typed_path = Utf8TypedPath::derive(path_ref.to_str().unwrap());
        if typed_path.is_unix() {
            let windows_path = typed_path.with_windows_encoding().to_string();

            // If path is absolute (starts with /) and doesn't already have a drive letter
            if PathBuf::from(&windows_path).has_root() {
                // Prepend C: drive letter to make it truly absolute on Windows
                return PathBuf::from(format!("C:{}", windows_path));
            }

            return PathBuf::from(windows_path);
        }
    }
    path.as_ref().to_path_buf()
}

/// Cross-platform path append that handles test paths consistently
fn append(base: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    #[cfg(test)]
    {
        // Normalize the path for tests, then use the platform-specific append
        platform_append(normalize_test_path(base), normalize_test_path(path))
    }

    #[cfg(not(test))]
    {
        // In non-test code, just use the platform-specific append directly
        platform_append(base, path)
    }
}

#[derive(Clone, Default)]
pub enum Fs {
    #[default]
    Real,
    /// Uses the real filesystem except acts as if the process has
    /// a different root directory by using [TempDir]
    Chroot(Arc<TempDir>),
    Fake(Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>),
}

impl Debug for Fs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fs::Chroot(dir) => write!(f, "chroot({})", dir.path().display()),
            _ => Ok(()), // only Chroot is useful, in testing
        }
    }
}

impl Fs {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        match cfg!(test) {
            true => {
                info!("using CHROOT filesystem");
                let tempdir = tempfile::tempdir().expect("failed creating temporary directory");
                let fs = Self::Chroot(tempdir.into());
                futures::executor::block_on(async {
                    fs.create_dir_all(ACTIVE_USER_HOME)
                        .await
                        .expect("failed to create test user home");
                    let user_path = PathBuf::from(ACTIVE_USER_HOME)
                        .join(".kiro")
                        .join("generators");
                    fs.create_dir_all(&user_path)
                        .await
                        .expect("failed to create test user home");
                    fs.create_dir_all("./.kiro").await.ok();
                    fs.create_dir_all("./.kiro/generators").await.ok();
                    fs.create_dir_all("./.kiro/generators/manifests").await.ok();
                    fs.create_dir_all("./.kiro/generators/agents").await.ok();
                    fs.create_dir_all("./.kiro/agents").await.ok();

                    // Copy local manifests
                    if let Ok(entries) = std::fs::read_dir("./.kiro/generators/manifests") {
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type()
                                && file_type.is_file()
                                && let Ok(config) = std::fs::read_to_string(entry.path())
                                && let Some(name) = entry.file_name().to_str()
                            {
                                fs.write(format!("./.kiro/generators/manifests/{}", name), config)
                                    .await
                                    .ok();
                            }
                        }
                    }

                    // Copy local agents
                    if let Ok(entries) = std::fs::read_dir("./.kiro/generators/agents") {
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type()
                                && file_type.is_file()
                                && let Ok(config) = std::fs::read_to_string(entry.path())
                                && let Some(name) = entry.file_name().to_str()
                            {
                                fs.write(format!("./.kiro/generators/agents/{}", name), config)
                                    .await
                                    .ok();
                            }
                        }
                    }

                    // Copy global manifests
                    if let Ok(entries) = std::fs::read_dir("./.kiro/global/manifests") {
                        fs.create_dir_all(user_path.join("manifests")).await.ok();
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type()
                                && file_type.is_file()
                                && let Ok(config) = std::fs::read_to_string(entry.path())
                                && let Some(name) = entry.file_name().to_str()
                            {
                                fs.write(user_path.join("manifests").join(name), config)
                                    .await
                                    .ok();
                            }
                        }
                    }

                    // Copy global agents
                    if let Ok(entries) = std::fs::read_dir("./.kiro/global/agents") {
                        fs.create_dir_all(user_path.join("agents")).await.ok();
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type()
                                && file_type.is_file()
                                && let Ok(config) = std::fs::read_to_string(entry.path())
                                && let Some(name) = entry.file_name().to_str()
                            {
                                fs.write(user_path.join("agents").join(name), config)
                                    .await
                                    .ok();
                            }
                        }
                    }
                });

                fs
            }
            false => Self::Real,
        }
    }

    pub fn is_chroot(&self) -> bool {
        matches!(self, Self::Chroot(_))
    }

    pub fn from_slice(vars: &[(&str, &str)]) -> Self {
        let map: HashMap<_, _> = vars
            .iter()
            .map(|(k, v)| (PathBuf::from(k), v.as_bytes().to_vec()))
            .collect();

        Self::Fake(Arc::new(Mutex::new(map)))
    }

    pub async fn create_new(&self, path: impl AsRef<Path>) -> crate::Result<fs::File> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::File::create_new(path).await,
            Self::Chroot(root) => fs::File::create_new(append(root.path(), path)).await,
            Self::Fake(_) => Err(io::Error::other("unimplemented")),
        }
        .wrap_err_with(|| format!("Failed to create file: {}", path.display()))
    }

    pub async fn create_dir(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::create_dir(path).await,
            Self::Chroot(root) => fs::create_dir(append(root.path(), path)).await,
            Self::Fake(_) => Err(io::Error::other("unimplemented")),
        }
        .wrap_err_with(|| format!("Failed to create directory: {}", path.display()))
    }

    pub async fn create_dir_all(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::create_dir_all(path).await,
            Self::Chroot(root) => fs::create_dir_all(append(root.path(), path)).await,
            Self::Fake(_) => Err(io::Error::other("unimplemented")),
        }
        .wrap_err_with(|| format!("Failed to create directory tree: {}", path.display()))
    }

    /// Attempts to open a file in read-only mode.
    ///
    /// This is a proxy to [`tokio::fs::File::open`].
    pub async fn open(&self, path: impl AsRef<Path>) -> crate::Result<fs::File> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::File::open(path).await,
            Self::Chroot(root) => fs::File::open(append(root.path(), path)).await,
            Self::Fake(_) => Err(io::Error::other("unimplemented")),
        }
        .wrap_err_with(|| format!("Failed to open file: {}", path.display()))
    }

    pub async fn read(&self, path: impl AsRef<Path>) -> crate::Result<Vec<u8>> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::read(path).await,
            Self::Chroot(root) => fs::read(append(root.path(), path)).await,
            Self::Fake(map) => {
                let Ok(lock) = map.lock() else {
                    return Err(io::Error::other("poisoned lock").into());
                };
                let Some(data) = lock.get(path) else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "not found").into());
                };
                Ok(data.clone())
            }
        }
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))
    }

    pub async fn read_to_string(&self, path: impl AsRef<Path>) -> crate::Result<String> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::read_to_string(path).await,
            Self::Chroot(root) => fs::read_to_string(append(root.path(), path)).await,
            Self::Fake(map) => {
                let Ok(lock) = map.lock() else {
                    return Err(io::Error::other("poisoned lock").into());
                };
                let Some(data) = lock.get(path) else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "not found").into());
                };
                match String::from_utf8(data.clone()) {
                    Ok(string) => Ok(string),
                    Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
                }
            }
        }
        .wrap_err_with(|| format!("Failed to read file to string: {}", path.display()))
    }

    pub fn read_to_string_sync(&self, path: impl AsRef<Path>) -> crate::Result<String> {
        let path = path.as_ref();
        match self {
            Self::Real => std::fs::read_to_string(path),
            Self::Chroot(root) => std::fs::read_to_string(append(root.path(), path)),
            Self::Fake(map) => {
                let Ok(lock) = map.lock() else {
                    return Err(io::Error::other("poisoned lock").into());
                };
                let Some(data) = lock.get(path) else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "not found").into());
                };
                match String::from_utf8(data.clone()) {
                    Ok(string) => Ok(string),
                    Err(err) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
                }
            }
        }
        .wrap_err_with(|| format!("Failed to read file to string: {}", path.display()))
    }

    /// Creates a future that will open a file for writing and write the entire
    /// contents of `contents` to it.
    ///
    /// This is a proxy to [`tokio::fs::write`].
    pub async fn write(
        &self,
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> crate::Result<()> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::write(path, contents).await,
            Self::Chroot(root) => fs::write(append(root.path(), path), contents).await,
            Self::Fake(map) => {
                let Ok(mut lock) = map.lock() else {
                    return Err(io::Error::other("poisoned lock").into());
                };
                lock.insert(path.to_owned(), contents.as_ref().to_owned());
                Ok(())
            }
        }
        .wrap_err_with(|| format!("Failed to write file: {}", path.display()))
    }

    /// Removes a file from the filesystem.
    ///
    /// Note that there is no guarantee that the file is immediately deleted
    /// (e.g. depending on platform, other open file descriptors may prevent
    /// immediate removal).
    ///
    /// This is a proxy to [`tokio::fs::remove_file`].
    pub async fn remove_file(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::remove_file(path).await,
            Self::Chroot(root) => fs::remove_file(append(root.path(), path)).await,
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to remove file: {}", path.display()))
    }

    /// Removes a directory at this path, after removing all its contents. Use
    /// carefully!
    ///
    /// This is a proxy to [`tokio::fs::remove_dir_all`].
    pub async fn remove_dir_all(&self, path: impl AsRef<Path>) -> crate::Result<()> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::remove_dir_all(path).await,
            Self::Chroot(root) => fs::remove_dir_all(append(root.path(), path)).await,
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to remove directory: {}", path.display()))
    }

    /// Renames a file or directory to a new name, replacing the original file
    /// if `to` already exists.
    ///
    /// This will not work if the new name is on a different mount point.
    ///
    /// This is a proxy to [`tokio::fs::rename`].
    pub async fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        match self {
            Self::Real => fs::rename(from, to).await,
            Self::Chroot(root) => {
                fs::rename(append(root.path(), from), append(root.path(), to)).await
            }
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to rename {} to {}", from.display(), to.display()))
    }

    /// Copies the contents of one file to another. This function will also copy
    /// the permission bits of the original file to the destination file.
    /// This function will overwrite the contents of to.
    ///
    /// This is a proxy to [`tokio::fs::copy`].
    pub async fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<u64> {
        let from = from.as_ref();
        let to = to.as_ref();
        match self {
            Self::Real => fs::copy(from, to).await,
            Self::Chroot(root) => {
                fs::copy(append(root.path(), from), append(root.path(), to)).await
            }
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to copy {} to {}", from.display(), to.display()))
    }

    /// Returns `Ok(true)` if the path points at an existing entity.
    ///
    /// This function will traverse symbolic links to query information about
    /// the destination file. In case of broken symbolic links this will
    /// return `Ok(false)`.
    ///
    /// This is a proxy to [`tokio::fs::try_exists`].
    pub async fn try_exists(&self, path: impl AsRef<Path>) -> crate::Result<bool> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::try_exists(path).await,
            Self::Chroot(root) => fs::try_exists(append(root.path(), path)).await,
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to check if path exists: {}", path.display()))
    }

    /// Returns `true` if the path points at an existing entity.
    ///
    /// This is a proxy to [std::path::Path::exists]. See the related doc
    /// comment in std on the pitfalls of using this versus
    /// [std::path::Path::try_exists].
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        match self {
            Self::Real => path.as_ref().exists(),
            Self::Chroot(root) => append(root.path(), path).exists(),
            Self::Fake(_) => panic!("unimplemented"),
        }
    }

    /// This is a proxy to [`tokio::fs::read_dir`].
    pub async fn read_dir(&self, path: impl AsRef<Path>) -> crate::Result<fs::ReadDir> {
        let path = path.as_ref();
        match self {
            Self::Real => fs::read_dir(path).await,
            Self::Chroot(root) => fs::read_dir(append(root.path(), path)).await,
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to read directory: {}", path.display()))
    }

    /// Returns an iterator over the entries within a directory (synchronous).
    ///
    /// This is a proxy to [`std::fs::read_dir`].
    pub fn read_dir_sync(&self, path: impl AsRef<Path>) -> crate::Result<std::fs::ReadDir> {
        let path = path.as_ref();
        match self {
            Self::Real => std::fs::read_dir(path),
            Self::Chroot(root) => std::fs::read_dir(append(root.path(), path)),
            Self::Fake(_) => panic!("unimplemented"),
        }
        .wrap_err_with(|| format!("Failed to read directory: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake() {
        let dir = PathBuf::from("/dir");
        let fs = Fs::from_slice(&[("/test", "test")]);

        let _ = fs.create_dir(dir.join("create_dir")).await.unwrap_err();
        let _ = fs
            .create_dir_all(dir.join("create/dir/all"))
            .await
            .unwrap_err();
        fs.write(dir.join("write"), b"write").await.unwrap();
        assert_eq!(fs.read(dir.join("write")).await.unwrap(), b"write");
        assert_eq!(fs.read_to_string(dir.join("write")).await.unwrap(), "write");
    }

    #[tokio::test]
    async fn test_real() {
        let dir = tempfile::tempdir().unwrap();
        let fs = Fs::Real;

        fs.create_dir(dir.path().join("create_dir")).await.unwrap();
        fs.create_dir_all(dir.path().join("create/dir/all"))
            .await
            .unwrap();
        fs.write(dir.path().join("write"), b"write").await.unwrap();
        assert_eq!(fs.read(dir.path().join("write")).await.unwrap(), b"write");
        assert_eq!(
            fs.read_to_string(dir.path().join("write")).await.unwrap(),
            "write"
        );
    }

    macro_rules! test_append_cases {
    ($(
        $name:ident: ($a:expr, $b:expr) => $expected:expr
    ),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(append($a, $b), normalize_test_path($expected));
            }
        )*
    };
}

    test_append_cases!(
            append_test_path_to_dir: ("/abc/test", "/test") => "/abc/test/test",
            append_absolute_to_tmp_dir: ("/tmp/.dir", "/tmp/.dir/home/myuser") =>
    "/tmp/.dir/home/myuser",         append_different_tmp_path: ("/tmp/.dir",
    "/tmp/hello") => "/tmp/.dir/tmp/hello",         append_nested_path_to_tmpdir:
    ("/tmp/.dir", "/tmp/.dir/tmp/.dir/home/user") => "/tmp/.dir/home/user",
        );

    #[tokio::test]
    async fn test_read_to_string() {
        let fs = Fs::new();
        fs.write("fake", "contents").await.unwrap();
        fs.write("invalid_utf8", &[255]).await.unwrap();

        // async tests
        assert_eq!(
            fs.read_to_string("fake").await.unwrap(),
            "contents",
            "should read fake file"
        );
        let err = fs.read_to_string("unknown").await;
        assert!(err.is_err(), "unknown path should error");
        let err_msg = err.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to read"),
            "error should indicate read failure, got: {}",
            err_msg
        );
        assert!(
            fs.read_to_string("invalid_utf8")
                .await
                .is_err_and(|err| err.to_string().contains("Failed to read")),
            "invalid utf8 should return error"
        );

        // sync tests
        assert_eq!(
            fs.read_to_string_sync("fake").unwrap(),
            "contents",
            "should read fake file"
        );
        assert!(
            fs.read_to_string_sync("unknown")
                .is_err_and(|err| err.to_string().contains("Failed to read")),
            "unknown path should return error"
        );
        assert!(
            fs.read_to_string_sync("invalid_utf8")
                .is_err_and(|err| err.to_string().contains("Failed to read")),
            "invalid utf8 should return error"
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_chroot_file_operations_for_unix() {
        if nix::unistd::Uid::effective().is_root() {
            println!("currently running as root, skipping.");
            return;
        }

        let fs = Fs::new();
        assert!(fs.is_chroot());

        fs.write("/fake", "contents").await.unwrap();
        assert_eq!(fs.read_to_string("/fake").await.unwrap(), "contents");
        assert_eq!(fs.read_to_string_sync("/fake").unwrap(), "contents");

        assert!(!fs.try_exists("/etc").await.unwrap());

        fs.create_dir_all("/etc/b/c").await.unwrap();
        assert!(fs.try_exists("/etc").await.unwrap());
        let mut read_dir = fs.read_dir("/etc").await.unwrap();
        let e = read_dir.next_entry().await.unwrap();
        assert!(e.unwrap().metadata().await.unwrap().is_dir());
        assert!(read_dir.next_entry().await.unwrap().is_none());

        fs.remove_dir_all("/etc").await.unwrap();
        assert!(!fs.try_exists("/etc").await.unwrap());

        fs.copy("/fake", "/fake_copy").await.unwrap();
        assert_eq!(fs.read_to_string("/fake_copy").await.unwrap(), "contents");
        assert_eq!(fs.read_to_string_sync("/fake_copy").unwrap(), "contents");

        fs.remove_file("/fake_copy").await.unwrap();
        assert!(!fs.try_exists("/fake_copy").await.unwrap());

        fs.write("/rename_1", "abc").await.unwrap();
        fs.write("/rename_2", "123").await.unwrap();
        fs.rename("/rename_2", "/rename_1").await.unwrap();
        assert_eq!(fs.read_to_string("/rename_1").await.unwrap(), "123");

        // Checking open
        assert!(fs.open("/does_not_exist").await.is_err());
        assert!(fs.open("/rename_1").await.is_ok());
    }

    #[tokio::test]
    async fn test_create_new() {
        let fs = Fs::new();
        fs.create_new("my_file.txt").await.unwrap();
        assert!(fs.create_new("my_file.txt").await.is_err());
    }
}
