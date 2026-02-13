pub(crate) mod bootstrap;
mod execute;
mod runtime;
mod tree;
#[cfg(target_os = "linux")]
mod watch_linux;
#[cfg(not(target_os = "linux"))]
mod watch_peasants;

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
    #[arg(long, global = true, short = 'd' , short_aliases = ['v'], aliases = ["verbose", "debug"], default_value = "false", env = "KG_DEBUG")]
    pub debug: bool,
    /// Enable trace level debug for an agent. Use keyword 'all' to debug all
    /// agents. Note, this is very verbose
    #[arg(long, short = 't', global = true, value_name = "AGENT_NAME")]
    pub trace: Option<String>,
    /// When to show color.
    #[arg(long = "color", short = 'c',  global = true, default_value_t = ColorOverride::default(), value_name = "WHEN", env = "KG_COLOR")]
    pub color_override: ColorOverride,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Clone, Default)]
pub struct InitArgs {}

#[derive(clap::Args, Clone, Default)]
pub struct BootstrapArgs {
    /// Install SKILL.md from a reviewed file
    #[arg(long, value_name = "PATH")]
    pub install: Option<PathBuf>,
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
    #[arg(short = 'f', long,  default_value_t = OutputFormatArg::default(), env = "KG_FORMAT")]
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
    /// Always write agent config even if nothing has changed
    #[arg(long, default_value = "false", env = "KG_FORCE")]
    pub force: bool,
    /// Show diff of changes before writing
    #[arg(long, default_value = "false", env = "KG_DIFF")]
    pub diff: bool,
    /// Format of the console output
    #[arg(short = 'f', long,  default_value_t = OutputFormatArg::default(), env = "KG_FORMAT")]
    pub format: OutputFormatArg,
    /// Display desktop notification when generation completes or errors
    #[arg(long, default_value = "false", env = "KG_NOTIFY")]
    #[cfg(target_os = "linux")]
    pub notify: bool,
}

#[derive(clap::Args, Clone, Default, Debug)]
pub struct DiffArgs {
    /// Use only global configuration (ignore local .kiro/generators/)
    #[arg(short = 'g', long)]
    pub global: bool,
    /// Use compact output (collapse unchanged items)
    #[arg(long)]
    pub compact: bool,
    /// Disable colored output
    #[arg(long)]
    pub plain: bool,
}

#[derive(clap::Args, Clone)]
pub struct WatchArgs {
    /// Disable the watcher instead of enabling it
    #[arg(long)]
    pub disable: bool,
    /// List active watchers
    #[arg(long, short = 'l', conflicts_with_all = ["disable", "path"])]
    pub list: bool,
    /// Project path to watch (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

#[derive(clap::Args, Clone, Default)]
pub struct TreeArgs {
    /// Show specific agents and their inheritance chains
    pub agents: Vec<String>,
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
    #[command(
        after_help = "To init in a custom location, override HOME:\n  HOME=$(mktemp -d) kg init"
    )]
    Init(InitArgs),
    /// Display version information
    Version,
    /// Compare generator files with Kiro agent files
    Diff(DiffArgs),
    /// Output JSON schema for configuration files
    #[command(subcommand)]
    Schema(SchemaCommand),
    /// Enable/disable systemd path watcher for automatic regeneration on config
    /// changes
    #[command(alias = "w")]
    Watch(WatchArgs),
    /// Display agent hierarchy and configuration sources as a tree
    #[command(alias = "t")]
    Tree(TreeArgs),
    /// Scan existing .kiro/agents/*.json files and install the kg-helper skill
    #[command(
        alias = "b",
        after_help = "After bootstrap, start kiro-cli and ask:\n  \"Help me set up kg for my \
                      project\""
    )]
    Bootstrap(BootstrapArgs),
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
            OutputFormatArg::Plain => OutputFormat::Plain,
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
