mod commands;
mod generator;
mod kg_config;
mod kiro;
mod os;
pub mod output;
mod schema;
mod schema_optional;
mod source;
mod tracing_init;
mod util;

#[cfg(test)]
pub use kg_config::toml_parse;
use {
    crate::{generator::Generator, os::Fs, tracing_init::init_tracing},
    clap::Parser,
    color_eyre::eyre::Context,
    std::path::Path,
    tracing::enabled,
};
pub use {color_eyre::eyre::format_err, generator::ConfigLocation, kg_config::*};

pub type Result<T> = color_eyre::Result<T>;
#[allow(dead_code)]
pub(crate) const DOCS_URL: &str = "https://kiro-generator.io";

/// Initialize a new kg configuration directory.
///
/// Creates the specified directory (if needed) and populates it with
/// default configuration files in manifests/ and agents/ subdirectories.
///
/// # Arguments
/// * `fs` - Filesystem abstraction for testability
/// * `home_dir` - Target home directory path
///
/// # Errors
/// Returns an error if:
/// - manifests/kg.toml already exists in the target directory
/// - Directory creation fails
/// - File write operations fail
async fn init(fs: &Fs, home_dir: impl AsRef<Path>) -> Result<()> {
    let home_dir = home_dir.as_ref();
    let gen_dir = home_dir.join(".kiro").join("generators");
    let manifests_dir = gen_dir.join("manifests");
    let agents_dir = gen_dir.join("agents");
    let kg_toml = manifests_dir.join("kg.toml");

    if fs.exists(&kg_toml) {
        return Err(format_err!(
            "Configuration already exists at {}",
            kg_toml.display()
        ));
    }

    if fs.exists(&manifests_dir) {
        return Err(format_err!(
            "Directory already exists at {}",
            manifests_dir.display()
        ));
    }

    if fs.exists(&agents_dir) {
        return Err(format_err!(
            "Directory already exists at {}",
            agents_dir.display()
        ));
    }

    // Create directories
    fs.create_dir_all(&manifests_dir)
        .await
        .wrap_err(format!("Failed to create {}", manifests_dir.display()))?;
    fs.create_dir_all(&agents_dir)
        .await
        .wrap_err(format!("Failed to create {}", agents_dir.display()))?;

    // Create manifests/kg.toml
    let kg_content = include_str!("../examples/basic/manifests/kg.toml");
    fs.write(&kg_toml, kg_content)
        .await
        .wrap_err(format!("Failed to write {}", kg_toml.display()))?;

    // Create agents/git.toml
    let git_toml = agents_dir.join("git.toml");
    let git_content = include_str!("../examples/basic/agents/git.toml");
    fs.write(&git_toml, git_content)
        .await
        .wrap_err(format!("Failed to write {}", git_toml.display()))?;

    // Create agents/default.toml
    let default_toml = agents_dir.join("default.toml");
    let default_content = include_str!("../examples/basic/agents/default.toml");
    fs.write(&default_toml, default_content)
        .await
        .wrap_err(format!("Failed to write {}", default_toml.display()))?;

    println!("✓ Created {}", manifests_dir.display());
    println!("✓ Created {}", agents_dir.display());
    println!("✓ Created {}", kg_toml.display());
    println!("✓ Created {}", git_toml.display());
    println!("✓ Created {}", default_toml.display());

    // Create systemd environment files
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| home_dir.join(".config"));
    let systemd_dir = config_dir.join("kg").join("systemd");
    fs.create_dir_all(&systemd_dir)
        .await
        .wrap_err(format!("Failed to create {}", systemd_dir.display()))?;

    let global_env = systemd_dir.join("global.env");
    fs.write(
        &global_env,
        include_str!("../examples/basic/systemd/global.env"),
    )
    .await
    .wrap_err(format!("Failed to write {}", global_env.display()))?;

    let home_env = systemd_dir.join("home.env");
    fs.write(
        &home_env,
        include_str!("../examples/basic/systemd/home.env"),
    )
    .await
    .wrap_err(format!("Failed to write {}", home_env.display()))?;

    println!("✓ Created {}", global_env.display());
    println!("✓ Created {}", home_env.display());

    println!("\nInitialized kg configuration in {}", gen_dir.display());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = commands::Cli::parse();
    if matches!(cli.command, commands::Command::Version) {
        println!("{}", clap::crate_version!());
        return Ok(());
    }
    init_tracing(cli.debug, cli.trace.as_deref());
    let span = tracing::info_span!(
        "main",
        dry_run = tracing::field::Empty,
        local_mode = tracing::field::Empty
    );
    let _guard = span.enter();
    let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
    let fs = Fs::new();

    if matches!(cli.command, commands::Command::Init(..)) {
        return init(&fs, &home_dir).await;
    }

    if let commands::Command::Bootstrap(args) = &cli.command {
        return commands::bootstrap::execute(&fs, &home_dir, args.install.clone()).await;
    }

    if let commands::Command::Schema(schema_cmd) = &cli.command {
        return schema::handle_schema_command(schema_cmd);
    }

    cli.record_span(&span);
    let location = cli.config_location(home_dir)?;
    let format = cli.format_color();
    let kq_generator_config: Generator = Generator::new(fs, location, format)?;
    if enabled!(tracing::Level::TRACE) {
        tracing::trace!(
            "Loaded Agent Generator Config:\n{}",
            facet_json::to_string_pretty(&kq_generator_config)
                .wrap_err("unable to decode to json")?
        );
    }

    cli.execute(&kq_generator_config).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use {super::*, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_init_config() -> Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from("init-test");
        let gen_dir = home.join(".kiro").join("generators");

        init(&fs, &home).await?;
        assert!(fs.exists(gen_dir.join("manifests")));
        assert!(fs.exists(gen_dir.join("agents")));
        assert!(fs.exists(gen_dir.join("manifests/kg.toml")));
        assert!(fs.exists(gen_dir.join("agents/git.toml")));
        assert!(fs.exists(gen_dir.join("agents/default.toml")));

        let result = init(&fs, &home).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }
}
