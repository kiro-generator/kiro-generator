use {super::WatchArgs, crate::Result, color_eyre::eyre::eyre};

pub async fn execute_watch(_args: &WatchArgs) -> Result<()> {
    Err(eyre!(
        "watch is only supported on Linux with systemd. Pull requests welcome!"
    ))
}
