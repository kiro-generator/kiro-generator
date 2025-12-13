use {
    clap::{
        Parser,
        Subcommand,
        builder::{Styles, styling::AnsiColor},
    },
    color_eyre::eyre::eyre,
    std::path::PathBuf,
};

/// Get the color styles for the CLI help menu.
fn __cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(name = "kg", version, about, long_about = "", styles=__cli_styles())]
pub struct Cli {
    #[arg(long, global = true, short = 'v', short_alias = 'd', aliases = ["verbose", "debug"], default_value = "false")]
    pub debug: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Clone, Default)]
pub struct Args {
    #[arg(long, conflicts_with = "global")]
    /// Ignore global $HOME kg.toml and all global agent definitions
    pub local: bool,
    #[arg(short = 'g', long, conflicts_with = "local")]
    /// Ignore local .kiro/generators/kg.toml config agent definitions
    pub global: bool,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Validate the agent configuration files but do not generate kiro agents
    #[command(alias = "v")]
    Validate(Args),
    #[command(alias = "g")]
    Generate(Args),
    #[command(alias = "m")]
    Migrate,
    Version,
}

impl Default for Command {
    fn default() -> Self {
        Command::Validate(Args::default())
    }
}

impl Cli {
    pub fn dry_run(&self) -> bool {
        matches!(self.command, Command::Validate(_))
    }

    pub fn is_local(&self) -> bool {
        match &self.command {
            Command::Generate(args) => args.local,
            Command::Validate(args) => args.local,
            _ => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match &self.command {
            Command::Generate(args) => args.global,
            Command::Validate(args) => args.global,
            _ => false,
        }
    }

    /// Return home dir and ~/.kiro/generators/kg.toml
    pub fn config(&self) -> crate::Result<(PathBuf, PathBuf)> {
        let home_dir = dirs::home_dir().ok_or(eyre!("cannot locate home directory"))?;
        let cfg = home_dir.join(".kiro").join("generators").join("kg.toml");
        Ok((home_dir, cfg))
    }
}
