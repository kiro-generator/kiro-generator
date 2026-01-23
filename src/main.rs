mod agent;
mod commands;
mod config;
mod generator;
mod os;
pub mod output;
mod schema;
mod source;

pub use color_eyre::eyre::format_err;
use {
    crate::{commands::Command, generator::Generator, os::Fs},
    clap::Parser,
    color_eyre::eyre::Context,
    std::path::Path,
    tracing::{debug, enabled},
    tracing_error::ErrorLayer,
    tracing_subscriber::prelude::*,
};
pub type Result<T> = color_eyre::Result<T>;

#[allow(dead_code)]
pub(crate) const DOCS_URL: &str = "https://kiro-generator.ai";

fn init_tracing(debug: bool, trace_agent: Option<&str>) {
    let filter = if let Some(agent) = trace_agent {
        let directive = if agent == "all" {
            "trace".to_string()
        } else {
            format!(
                "{},[agent{{name=\"{agent}\"}}]=trace",
                if debug { "debug" } else { "info" }
            )
        };
        tracing_subscriber::EnvFilter::new(directive)
    } else if debug {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
    };

    if debug {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_level(true)
                    .with_writer(std::io::stderr)
                    .with_target(true),
            )
            .with(ErrorLayer::default())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false)
                    .with_level(true)
                    .with_writer(std::io::stderr),
            )
            .with(ErrorLayer::default())
            .init();
    }
}

/// Initialize a new kg configuration directory.
///
/// Creates the specified directory (if needed) and populates it with
/// default configuration files in manifests/ and agents/ subdirectories.
///
/// # Arguments
/// * `fs` - Filesystem abstraction for testability
/// * `gen_dir` - Target directory path
///
/// # Errors
/// Returns an error if:
/// - manifests/kg.toml already exists in the target directory
/// - Directory creation fails
/// - File write operations fail
async fn init(fs: &Fs, gen_dir: impl AsRef<Path>) -> Result<()> {
    let gen_dir = gen_dir.as_ref();
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

    let local_mode = cli.is_local();
    let global_mode = cli.is_global();
    let (home_dir, home_config) = cli.config()?;
    let fs = Fs::new();

    if let commands::Command::Init(args) = &cli.command {
        let dir = match &args.location {
            Some(path) => path.clone(),
            None => home_dir.join(".kiro").join("generators"),
        };
        return init(&fs, dir).await;
    }

    if let commands::Command::Schema(schema_cmd) = &cli.command {
        use commands::SchemaCommand;
        let mut output = match schema_cmd {
            SchemaCommand::Manifest => {
                let mut s = facet_json_schema::schema_for::<config::GeneratorConfig>();
                s.description = Some("Schema for kiro-generator (kg) manifest TOML files".into());
                s
            }
            SchemaCommand::Agent => {
                let mut s = facet_json_schema::schema_for::<config::agent_file::KgAgentFileDoc>();
                s.description = Some("Schema for kiro-generator (kg) agent TOML files".into());
                s
            }
        };
        output.schema = Some("https://json-schema.org/draft/2020-12/schema".into());
        println!("{}", facet_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if global_mode {
        debug!(
            "changing working directory to {}",
            home_dir.as_os_str().display()
        );
        std::env::set_current_dir(&home_dir)
            .wrap_err(format!("failed to set CWD {}", home_dir.display()))?;
    }
    if local_mode {
        span.record("local_mode", true);
    }
    let dry_run = cli.dry_run();
    if dry_run {
        span.record("dry_run", true);
    }

    let location = if local_mode {
        generator::ConfigLocation::Local
    } else if global_mode {
        generator::ConfigLocation::Global(home_config)
    } else {
        // Default: merge both global and local
        generator::ConfigLocation::Both(home_config)
    };

    let format = cli.format_color();
    let kq_generator_config: Generator = Generator::new(fs, location, format)?;
    if enabled!(tracing::Level::TRACE) {
        tracing::trace!(
            "Loaded Agent Generator Config:\n{}",
            facet_json::to_string_pretty(&kq_generator_config)
                .wrap_err("unable to decode to json")?
        );
    }

    match cli.command {
        Command::Validate(args) | Command::Generate(args) => {
            let results = kq_generator_config.write_all(dry_run).await?;
            format.result(dry_run, args.show_templates, results)?;
        }
        Command::Diff => {
            kq_generator_config.diff()?;
        }
        _ => {}
    };

    Ok(())
}

#[cfg(test)]
mod tests {

    use {super::*, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_init_config() -> Result<()> {
        let fs = Fs::new();
        let dir = PathBuf::from("init-test");

        init(&fs, &dir).await?;
        assert!(fs.exists(dir.join("manifests")));
        assert!(fs.exists(dir.join("agents")));
        assert!(fs.exists(dir.join("manifests/kg.toml")));
        assert!(fs.exists(dir.join("agents/git.toml")));
        assert!(fs.exists(dir.join("agents/default.toml")));

        let result = init(&fs, &dir).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }
}
