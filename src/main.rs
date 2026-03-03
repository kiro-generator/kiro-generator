mod commands;
mod generator;
mod json_schema;
mod kg_config;
mod kiro;
mod os;
pub mod output;
mod schema;
mod source;
mod tracing_init;

#[cfg(test)]
pub use kg_config::toml_parse;
use {
    crate::{generator::Generator, os::Fs, tracing_init::init_tracing},
    clap::Parser,
    color_eyre::eyre::{Context, bail},
    std::path::Path,
    tracing::enabled,
};
pub use {
    color_eyre::eyre::format_err,
    generator::ConfigLocation,
    kg_config::*,
    source::{AgentSourceSlots, SourceSlot},
};

pub type Result<T> = color_eyre::Result<T>;
#[allow(dead_code)]
pub(crate) const DOCS_URL: &str = "https://kiro-generator.io";

/// Embedded kg-helper agent JSON, used when the system package path is not
/// available (e.g. `cargo install` users).
const KG_HELPER_AGENT_JSON: &str = include_str!("../resources/agents/kg-helper.json");

/// System package path for the kg-helper agent JSON on Linux.
#[cfg(target_os = "linux")]
const KG_HELPER_SYSTEM_PATH: &str = "/usr/share/doc/kiro-generator/agents/kg-helper.json";

/// System package path for the kg-helper agent JSON on macOS (Homebrew).
#[cfg(target_os = "macos")]
const KG_HELPER_SYSTEM_PATH: &str = "/opt/homebrew/share/kiro-generator/agents/kg-helper.json";

/// Return the system package path for kg-helper.json if it exists on disk.
fn kg_helper_system_path(fs: &Fs) -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(KG_HELPER_SYSTEM_PATH);
    if fs.exists(p) {
        return Some(p.to_path_buf());
    }
    None
}

/// Install the kg-helper agent to `~/.kiro/agents/kg-helper.json`.
///
/// Copies from the system package path when available (signed artifact),
/// otherwise writes the embedded fallback. Always overwrites — this file
/// is owned by kg and safe to refresh on every `kg init`.
async fn install_kg_helper_agent(fs: &Fs, home_dir: impl AsRef<Path>, force: bool) -> Result<()> {
    let kiro_agents_dir = home_dir.as_ref().join(".kiro").join("agents");
    let dest = kiro_agents_dir.join("kg-helper.json");

    let (content, src_label) = if let Some(src) = kg_helper_system_path(fs) {
        let content = fs
            .read_to_string(&src)
            .await
            .wrap_err_with(|| format!("Failed to read {}", src.display()))?;
        let label = src.display().to_string();
        (content, label)
    } else {
        (KG_HELPER_AGENT_JSON.to_string(), "embedded".to_string())
    };

    println!("Source : {src_label}");
    println!("Install: {}", dest.display());
    if !force {
        use std::io::IsTerminal;
        // Refuse to prompt on non-interactive stdin unless --force is used.
        if !std::io::stdin().is_terminal() {
            bail!(
                "Refusing to prompt because stdin is not a TTY; rerun with --force to skip  \
                 confirmation"
            );
        }
        let proceed = tokio::task::spawn_blocking(|| -> std::io::Result<bool> {
            use std::io::{self, Write};
            print!("Proceed? [y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().eq_ignore_ascii_case("y"))
        })
        .await
        .wrap_err("interactive prompt task panicked")??;
        if !proceed {
            println!("Operation canceled");
            return Ok(());
        }
    }

    fs.create_dir_all(&kiro_agents_dir)
        .await
        .wrap_err_with(|| format!("Failed to create {}", kiro_agents_dir.display()))?;
    fs.write(&dest, &content)
        .await
        .wrap_err_with(|| format!("Failed to write {}", dest.display()))?;

    println!("✓ Installed {}", dest.display());
    println!("\nStart the kg-helper agent:");
    println!("  kiro-cli --agent kg-helper");

    Ok(())
}

/// Create the skeleton kg configuration directory structure.
///
/// Idempotent: skips files/dirs that already exist rather than erroring.
async fn init_skeleton(fs: &Fs, home_dir: impl AsRef<Path>) -> Result<()> {
    let gen_dir = home_dir.as_ref().join(".kiro").join("generators");
    let manifests_dir = gen_dir.join("manifests");
    let agents_dir = gen_dir.join("agents");
    let kg_toml = manifests_dir.join("kg.toml");

    fs.create_dir_all(&manifests_dir)
        .await
        .wrap_err_with(|| format!("Failed to create {}", manifests_dir.display()))?;
    fs.create_dir_all(&agents_dir)
        .await
        .wrap_err_with(|| format!("Failed to create {}", agents_dir.display()))?;

    if !fs.exists(&kg_toml) {
        let kg_content = include_str!("../examples/basic/manifests/kg.toml");
        fs.write(&kg_toml, kg_content)
            .await
            .wrap_err_with(|| format!("Failed to write {}", kg_toml.display()))?;
        println!("✓ Created {}", kg_toml.display());
    }

    let git_toml = agents_dir.join("git.toml");
    if !fs.exists(&git_toml) {
        let git_content = include_str!("../examples/basic/agents/git.toml");
        fs.write(&git_toml, git_content)
            .await
            .wrap_err_with(|| format!("Failed to write {}", git_toml.display()))?;
        println!("✓ Created {}", git_toml.display());
    }

    let default_toml = agents_dir.join("default.toml");
    if !fs.exists(&default_toml) {
        let default_content = include_str!("../examples/basic/agents/default.toml");
        fs.write(&default_toml, default_content)
            .await
            .wrap_err_with(|| format!("Failed to write {}", default_toml.display()))?;
        println!("✓ Created {}", default_toml.display());
    }

    println!("\nInitialized kg configuration in {}", gen_dir.display());
    Ok(())
}

async fn init(fs: &Fs, home_dir: impl AsRef<Path>, skeleton: bool, force: bool) -> Result<()> {
    if skeleton {
        init_skeleton(fs, home_dir).await
    } else {
        install_kg_helper_agent(fs, home_dir, force).await
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

    // Extract trace option from commands that support it
    let trace = match &cli.command {
        commands::Command::Validate(args) => args.trace.as_deref(),
        commands::Command::Generate(args) => args.trace.as_deref(),
        commands::Command::Diff(args) => args.trace.as_deref(),
        commands::Command::Tree(args) => args.trace.as_deref(),
        _ => None,
    };

    init_tracing(cli.debug, trace);
    let span = tracing::info_span!(
        "main",
        dry_run = tracing::field::Empty,
        local_mode = tracing::field::Empty
    );
    let _guard = span.enter();
    let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
    let fs = Fs::new();

    if let commands::Command::Init(args) = &cli.command {
        return init(&fs, &home_dir, args.skeleton, args.force).await;
    }

    if let commands::Command::Schema(schema_cmd) = &cli.command {
        if let commands::SchemaCommand::Agent(args) = &schema_cmd
            && args.mappings
        {
            return schema::handle_schema_mappings();
        }
        return schema::handle_schema_command(schema_cmd);
    }

    cli.record_span(&span);
    let location = cli.config_location(home_dir)?;
    let format = cli.format_color();
    let kg_generator_config: Generator = Generator::new(fs, location, format)?;
    if enabled!(tracing::Level::TRACE) {
        tracing::trace!(
            "Loaded Agent Generator Config:\n{}",
            facet_json::to_string_pretty(&kg_generator_config)
                .wrap_err("unable to decode to json")?
        );
    }

    cli.execute(&kg_generator_config).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use {super::*, std::path::PathBuf};

    #[tokio::test]
    #[test_log::test]
    async fn test_init_skeleton() -> Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from("init-test");
        let gen_dir = home.join(".kiro").join("generators");

        init(&fs, &home, true, false).await?;
        assert!(fs.exists(gen_dir.join("manifests")));
        assert!(fs.exists(gen_dir.join("agents")));
        assert!(fs.exists(gen_dir.join("manifests/kg.toml")));
        assert!(fs.exists(gen_dir.join("agents/git.toml")));
        assert!(fs.exists(gen_dir.join("agents/default.toml")));

        // Idempotent: second run should not error
        init(&fs, &home, true, false).await?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_init_installs_kg_helper_agent() -> Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from("init-agent-test");
        let dest = home.join(".kiro").join("agents").join("kg-helper.json");

        init(&fs, &home, false, true).await?;

        assert!(fs.exists(&dest), "kg-helper.json not created");
        let content = fs.read_to_string(&dest).await?;
        let v: serde_json::Value = serde_json::from_str(&content)?;
        assert_eq!(v["name"], "kg-helper");

        Ok(())
    }
}
