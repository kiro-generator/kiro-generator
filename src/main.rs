mod agent;
mod commands;
mod config;
mod error;
mod generator;
// mod kdl;
mod os;
pub mod output;
mod schema;
mod source;

use {
    crate::{generator::Generator, os::Fs},
    clap::Parser,
    miette::{Context, IntoDiagnostic},
    std::path::Path,
    tracing::{debug, enabled},
    tracing_error::ErrorLayer,
    tracing_subscriber::prelude::*,
};
pub use {error::Error, miette::miette as format_err};
pub type Result<T> = miette::Result<T>;

pub(crate) const DOCS_URL: &str = "https://kg.cartera-mesh.com";

fn init_tracing(debug: bool, trace_agent: Option<&str>) {
    let filter = if let Some(agent) = trace_agent {
        let directive = if agent == "all" {
            "trace".to_string()
        } else {
            format!("info,[agent{{name=\"{agent}\"}}]=trace")
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
/// default configuration files: kg.kdl, default.kdl, and example.kdl.
///
/// # Arguments
/// * `fs` - Filesystem abstraction for testability
/// * `gen_dir` - Target directory path
///
/// # Errors
/// Returns an error if:
/// - kg.kdl already exists in the target directory
/// - Directory creation fails
/// - File write operations fail
async fn init(fs: &Fs, gen_dir: impl AsRef<Path>) -> Result<()> {
    let gen_dir = gen_dir.as_ref();
    let kg_config = gen_dir.join("kg.kdl");
    if fs.exists(&kg_config) {
        return Err(format_err!(
            "kg.kdl already exists at {}",
            kg_config.display()
        ));
    }

    if !fs.exists(gen_dir) {
        fs.create_dir_all(gen_dir)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to create directory {}", gen_dir.display()))?;
    }

    // Copy resource files
    let resources = [
        ("kg.kdl", include_str!("../resources/kg.kdl")),
        ("default.kdl", include_str!("../resources/default.kdl")),
        ("example.kdl", include_str!("../resources/example.kdl")),
    ];

    for (filename, content) in resources {
        let dest = gen_dir.join(filename);
        fs.write(&dest, content)
            .await
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write {}", dest.display()))?;
        println!("Created {}", dest.display());
    }

    println!("\n✓ Initialized kg configuration in {}", gen_dir.display());
    println!("\nNext steps:");
    println!("  1. Review and edit {}", kg_config.display());
    println!("  2. Run 'kg validate' to check your configuration");
    println!("  3. Run 'kg generate' to create agent files\n");
    println!("Visit {} for more info and examples", DOCS_URL);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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

    if global_mode {
        debug!(
            "changing working directory to {}",
            home_dir.as_os_str().display()
        );
        std::env::set_current_dir(&home_dir)
            .into_diagnostic()
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
    let q_generator_config: Generator = Generator::new(fs, location, format)?;
    if enabled!(tracing::Level::TRACE) {
        tracing::trace!(
            "Loaded Agent Generator Config:\n{}",
            serde_json::to_string_pretty(&q_generator_config)
                .into_diagnostic()
                .wrap_err("unable to decode to json")?
        );
    }

    match cli.command {
        commands::Command::Validate(args) | commands::Command::Generate(args) => {
            let results = q_generator_config.write_all(dry_run).await?;
            format.result(dry_run, args.show_templates, results)?;
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
        let dir = PathBuf::from("init");
        super::init(&fs, &dir).await?;
        assert!(fs.exists(&dir));
        assert!(fs.exists(dir.join("kg.kdl")));

        let result = init(&fs, &dir).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        Ok(())
    }
}
