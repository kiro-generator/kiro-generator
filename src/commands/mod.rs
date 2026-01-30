mod execute;
mod runtime;

use {
    crate::output::{ColorOverride, OutputFormat, OutputFormatArg},
    clap::{
        Parser,
        Subcommand,
        builder::{Styles, styling::AnsiColor},
    },
    std::{fmt::Display, io::IsTerminal, path::PathBuf},
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
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Clone, Default)]
pub struct InitArgs {
    /// Directory where configuration will be created.
    /// Defaults to $HOME/.kiro/generators if not specified.
    pub location: Option<PathBuf>,
}

#[derive(clap::Args, Clone, Default)]
pub struct ValidateArgs {
    /// Use only local configuration (ignore global ~/.kiro/generators/)
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
    /// Use only global configuration (ignore local .kiro/generators/)
    #[arg(short = 'g', long, conflicts_with = "local")]
    pub global: bool,
    /// Show template agents in output
    #[arg(long, default_value = "false")]
    pub show_templates: bool,
    /// Format of the console output
    #[arg(short = 'f', long,  default_value_t = OutputFormatArg::default())]
    pub format: OutputFormatArg,
}

#[derive(clap::Args, Clone, Default)]
pub struct GenerateArgs {
    /// Use only local configuration (ignore global ~/.kiro/generators/)
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
    /// Use only global configuration (ignore local .kiro/generators/)
    #[arg(short = 'g', long, conflicts_with = "local")]
    pub global: bool,
    /// Show template agents in output
    #[arg(long, default_value = "false")]
    pub show_templates: bool,
    /// Format of the console output
    #[arg(short = 'f', long,  default_value_t = OutputFormatArg::default())]
    pub format: OutputFormatArg,
    /// Display desktop notification when generation completes or errors
    #[arg(long, default_value = "false")]
    #[cfg(target_os = "linux")]
    pub notify: bool,
}

#[derive(clap::Args, Clone, Default)]
pub struct DiffArgs {
    /// Use only global configuration (ignore local .kiro/generators/)
    #[arg(short = 'g', long)]
    pub global: bool,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Validate the agent configuration files but do not generate kiro agents
    #[command(alias = "v")]
    Validate(ValidateArgs),
    /// Generate agent configuration JSON files, if local config found only
    /// local agents are generated. Use --global to generate $HOME agents
    #[command(alias = "g")]
    Generate(GenerateArgs),
    /// Create default configuration in directory ~/.kiro/generators
    #[command()]
    Init(InitArgs),
    /// Display version information
    Version,
    /// Compare generator files with Kiro agent files
    Diff(DiffArgs),
    /// Output JSON schema for configuration files
    #[command(subcommand)]
    Schema(SchemaCommand),
}

#[derive(Subcommand, Clone)]
pub enum SchemaCommand {
    /// Output JSON schema for manifest files (kg.toml)
    Manifest,
    /// Output JSON schema for agent definition files
    Agent,
}

impl Display for SchemaCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Manifest => "manifest",
            Self::Agent => "agent",
        })
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Validate(ValidateArgs::default())
    }
}

impl Cli {
    pub fn format_color(&self) -> OutputFormat {
        let format = match &self.command {
            Command::Validate(a) => &a.format,
            Command::Generate(a) => &a.format,
            _ => return OutputFormat::Table(self.color()),
        };

        match format {
            OutputFormatArg::Table => OutputFormat::Table(self.color()),
            OutputFormatArg::Json => OutputFormat::Json,
        }
    }

    fn color(&self) -> bool {
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

    pub(super) fn dry_run(&self) -> bool {
        matches!(self.command, Command::Validate(_))
    }

    pub(super) fn is_local(&self) -> bool {
        match &self.command {
            Command::Generate(args) => args.local,
            Command::Validate(args) => args.local,
            _ => false,
        }
    }

    pub(super) fn is_global(&self) -> bool {
        match &self.command {
            Command::Generate(args) => args.global,
            Command::Validate(args) => args.global,
            Command::Diff(args) => args.global,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn test_dry_run() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Validate(ValidateArgs::default()),
        };
        assert!(cli.dry_run());

        let cli = Cli {
            command: Command::Generate(GenerateArgs::default()),
            ..cli
        };
        assert!(!cli.dry_run());
    }

    #[test_log::test]
    fn test_is_local() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Validate(ValidateArgs {
                local: true,
                ..Default::default()
            }),
        };
        assert!(cli.is_local());
        assert!(!cli.is_global());
    }

    #[test_log::test]
    fn test_is_global() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Generate(GenerateArgs {
                global: true,
                ..Default::default()
            }),
        };
        assert!(cli.is_global());
        assert!(!cli.is_local());
    }

    #[test_log::test]
    fn test_color_auto() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::default(),
        };
        // Color depends on terminal and env vars, just verify it doesn't panic
        let _ = cli.color();
    }

    #[test_log::test]
    fn test_color_always() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Always,
            command: Command::default(),
        };
        assert!(cli.color());
    }

    #[test_log::test]
    fn test_color_never() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Never,
            command: Command::default(),
        };
        assert!(!cli.color());
    }

    #[test_log::test]
    fn test_format_color() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Always,
            command: Command::default(),
        };
        assert!(matches!(cli.format_color(), OutputFormat::Table(true)));

        let cli = Cli {
            command: Command::Validate(ValidateArgs {
                format: OutputFormatArg::Json,
                ..Default::default()
            }),
            ..cli
        };
        assert!(matches!(cli.format_color(), OutputFormat::Json));
    }
}
