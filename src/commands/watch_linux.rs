use {
    super::WatchArgs,
    crate::{Result, os::systemd::escape_path},
    color_eyre::eyre::Context,
    std::path::{Path, PathBuf},
    tracing::debug,
    zbus::{Connection, proxy, zvariant::OwnedObjectPath},
};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait SystemdManager {
    #[allow(clippy::type_complexity)]
    fn enable_unit_files(
        &self,
        files: &[&str],
        runtime: bool,
        force: bool,
    ) -> zbus::Result<(bool, Vec<(String, String, String)>)>;

    fn disable_unit_files(
        &self,
        files: &[&str],
        runtime: bool,
    ) -> zbus::Result<Vec<(String, String, String)>>;

    fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    #[allow(clippy::type_complexity)]
    fn list_units_by_patterns(
        &self,
        states: &[&str],
        patterns: &[&str],
    ) -> zbus::Result<
        Vec<(
            String,
            String,
            String,
            String,
            String,
            String,
            OwnedObjectPath,
            u32,
            String,
            OwnedObjectPath,
        )>,
    >;
}

fn resolve_path(args: &WatchArgs) -> Result<PathBuf> {
    match &args.path {
        Some(p) => std::fs::canonicalize(p)
            .wrap_err_with(|| format!("Failed to resolve path '{}'", p.display())),
        None => std::env::current_dir().wrap_err("Failed to get current directory"),
    }
}

fn unit_name(path: impl AsRef<Path>) -> String {
    let escaped = escape_path(&path.as_ref().to_string_lossy());
    format!("kiro-generator-local@{escaped}.path")
}

pub async fn execute_watch(args: &WatchArgs) -> Result<()> {
    if args.list {
        return list_watchers().await;
    }

    let path = resolve_path(args)?;
    let unit = unit_name(&path);

    let conn = Connection::session()
        .await
        .wrap_err("Failed to connect to session D-Bus")?;
    let manager = SystemdManagerProxy::new(&conn)
        .await
        .wrap_err("Failed to create systemd manager proxy")?;

    if args.disable {
        debug!("Stopping and disabling {unit}");
        manager
            .stop_unit(&unit, "replace")
            .await
            .wrap_err_with(|| format!("Failed to stop {unit}"))?;
        manager
            .disable_unit_files(&[&unit], false)
            .await
            .wrap_err_with(|| format!("Failed to disable {unit}"))?;
        println!("Disabled watcher for {}", path.display());
    } else {
        debug!("Enabling and starting {unit}");
        manager
            .enable_unit_files(&[&unit], false, false)
            .await
            .wrap_err_with(|| format!("Failed to enable {unit}"))
            .wrap_err("Unit files not found in ~/.config/systemd/user/\n  Install them from: https://github.com/dougEfresh/kiro-generator/tree/main/resources/systemd")?;
        manager
            .start_unit(&unit, "replace")
            .await
            .wrap_err_with(|| format!("Failed to start {unit}"))?;
        println!("Watching {}", path.display());
    }

    Ok(())
}

async fn list_watchers() -> Result<()> {
    let conn = Connection::session()
        .await
        .wrap_err("Failed to connect to session D-Bus")?;
    let manager = SystemdManagerProxy::new(&conn)
        .await
        .wrap_err("Failed to create systemd manager proxy")?;

    let units = manager
        .list_units_by_patterns(&[], &["kiro-generator-local@*.path"])
        .await
        .wrap_err("Failed to list watcher units")?;

    if units.is_empty() {
        println!("No active watchers");
    } else {
        for (name, _, _, _, state, ..) in &units {
            println!("{state}\t{name}");
        }
    }

    Ok(())
}
