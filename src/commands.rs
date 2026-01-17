use {
    crate::output::{ColorOverride, OutputFormat, OutputFormatArg},
    clap::{
        Parser,
        Subcommand,
        builder::{Styles, styling::AnsiColor},
    },
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
pub struct InitArgs {
    /// Directory where configuration will be created.
    /// Defaults to $HOME/.kiro/generators if not specified.
    pub location: Option<PathBuf>,
}

#[derive(clap::Args, Clone, Default)]
pub struct Args {
    /// Use only local configuration (ignore global ~/.kiro/generators/)
    #[arg(long, conflicts_with = "global")]
    pub local: bool,
    /// Use only global configuration (ignore local .kiro/generators/)
    #[arg(short = 'g', long, conflicts_with = "local")]
    pub global: bool,
    /// Show template agents in output
    #[arg(long, default_value = "false")]
    pub show_templates: bool,
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
    /// Create default configuration in directory ~/.kiro/generators
    #[command()]
    Init(InitArgs),
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
        let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
        let cfg = home_dir.join(".kiro").join("generators");
        Ok((home_dir, cfg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            format: OutputFormatArg::Table,
            command: Command::Validate(Args::default()),
        };
        assert!(cli.dry_run());

        let cli = Cli {
            command: Command::Generate(Args::default()),
            ..cli
        };
        assert!(!cli.dry_run());
    }

    #[test]
    fn test_is_local() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            format: OutputFormatArg::Table,
            command: Command::Validate(Args {
                local: true,
                ..Args::default()
            }),
        };
        assert!(cli.is_local());
        assert!(!cli.is_global());
    }

    #[test]
    fn test_is_global() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            format: OutputFormatArg::Table,
            command: Command::Generate(Args {
                global: true,
                ..Args::default()
            }),
        };
        assert!(cli.is_global());
        assert!(!cli.is_local());
    }

    #[test]
    fn test_color_auto() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            format: OutputFormatArg::Table,
            command: Command::default(),
        };
        // Color depends on terminal and env vars, just verify it doesn't panic
        let _ = cli.color();
    }

    #[test]
    fn test_color_always() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Always,
            format: OutputFormatArg::Table,
            command: Command::default(),
        };
        assert!(cli.color());
    }

    #[test]
    fn test_color_never() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Never,
            format: OutputFormatArg::Table,
            command: Command::default(),
        };
        assert!(!cli.color());
    }

    #[test]
    fn test_format_color() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Always,
            format: OutputFormatArg::Table,
            command: Command::default(),
        };
        assert!(matches!(cli.format_color(), OutputFormat::Table(true)));

        let cli = Cli {
            format: OutputFormatArg::Json,
            ..cli
        };
        assert!(matches!(cli.format_color(), OutputFormat::Json));
    }

    #[test]
    fn test_config() {
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            format: OutputFormatArg::Table,
            command: Command::default(),
        };
        let result = cli.config();
        assert!(result.is_ok());
        let (home, cfg) = result.unwrap();
        assert!(cfg.ends_with(".kiro/generators"));
        assert!(cfg.starts_with(&home));
    }
}
