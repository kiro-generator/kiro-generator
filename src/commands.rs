use {
    crate::output::{ColorOverride, OutputFormat, OutputFormatArg},
    clap::{
        Parser,
        Subcommand,
        builder::{Styles, styling::AnsiColor},
    },
    color_eyre::eyre::eyre,
    std::{io::IsTerminal, path::PathBuf},
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
    #[arg(long, global = true, short = 'd' , short_aliases = ['v'], aliases = ["verbose", "debug"], default_value = "false")]
    pub debug: bool,
    /// Enable trace level debug for an agent. Use keyword 'all' to debug all
    /// agents. Note, this is very verbose
    #[arg(long, short = 't', global = true, value_name = "AGENT_NAME")]
    pub trace: Option<String>,
    /// When to show color.
    #[arg(long = "color", short = 'c',  global = true, default_value_t = ColorOverride::default(), value_name = "WHEN")]
    pub color_override: ColorOverride,
    /// Format of the console output
    #[arg(short = 'f', long,  global = true , default_value_t = OutputFormatArg::default())]
    pub format: OutputFormatArg,
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
    /// Show skeleton agents in output
    #[arg(long, default_value = "false")]
    pub show_skeletons: bool,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Validate the agent configuration files but do not generate kiro agents
    #[command(alias = "v")]
    Validate(Args),
    /// Generate agent configuration JSON files, if local config found only
    /// local agents are generated. Use --global to generate $HOME agents
    #[command(alias = "g")]
    Generate(Args),
    /// Not implemented yet
    #[command(alias = "m")]
    Migrate,
    /// Display version information
    Version,
}

impl Default for Command {
    fn default() -> Self {
        Command::Validate(Args::default())
    }
}

impl Cli {
    pub fn format_color(&self) -> OutputFormat {
        match &self.format {
            OutputFormatArg::Table => OutputFormat::Table(self.color()),
            OutputFormatArg::Json => OutputFormat::Json,
        }
    }

    pub fn color(&self) -> bool {
        match &self.color_override {
            ColorOverride::Auto => {
                std::io::stdout().is_terminal()
                    && std::env::var("NO_COLOR").is_err()
                    && std::env::var("CI").is_err()
            }
            ColorOverride::Never => false,
            ColorOverride::Always => true,
        }
    }

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
