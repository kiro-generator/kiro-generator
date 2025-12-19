mod agent;
mod commands;
mod generator;
pub(crate) mod merging_format;
mod os;
pub mod output;
mod schema;
mod source;
use {
    crate::{generator::Generator, os::Fs},
    clap::Parser,
    tracing::{debug, enabled},
    tracing_error::ErrorLayer,
    tracing_subscriber::prelude::*,
};
pub type Result<T> = color_eyre::Result<T>;

pub const DEFAULT_AGENT_RESOURCES: &[&str] = &["file://AGENTS.md", "file://README.md"];

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
    if global_mode {
        debug!(
            "changing working directory to {}",
            home_dir.as_os_str().display()
        );
        std::env::set_current_dir(&home_dir)?;
    }
    if local_mode {
        span.record("local_mode", true);
    }
    let dry_run = cli.dry_run();
    if dry_run {
        span.record("dry_run", true);
    }

    let fs = Fs::new();
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
            serde_json::to_string_pretty(&q_generator_config)?
        );
    }

    match cli.command {
        commands::Command::Validate(args) | commands::Command::Generate(args) => {
            let results = q_generator_config.write_all(dry_run).await?;
            format.result(dry_run, args.show_skeletons, results)?;
        }
        _ => {}
    };

    Ok(())
}
