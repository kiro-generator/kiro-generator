#[cfg(not(test))]
use color_eyre::eyre::Context;
use {super::Cli, std::path::PathBuf};

impl Cli {
    /// Determine the configuration location based on CLI flags and current
    /// directory
    pub fn config_location(&self, home_dir: PathBuf) -> crate::Result<crate::ConfigLocation> {
        use crate::ConfigLocation;

        if self.is_local() {
            return Ok(ConfigLocation::Local);
        }
        let home_config = home_dir.join(".kiro").join("generators");
        if self.is_global() {
            tracing::debug!("changing working directory to {}", home_dir.display());
            // Don't change CWD in tests - it affects other tests
            #[cfg(not(test))]
            std::env::set_current_dir(&home_dir)
                .wrap_err(format!("failed to set CWD {}", home_dir.display()))?;

            return Ok(ConfigLocation::Global(home_config));
        }

        // Check if we're in the home directory - if so, only use global config
        // to avoid treating ~/.kiro/generators as both local and global
        let current_dir = std::env::current_dir()
            .map_err(|e| crate::format_err!("Failed to get current directory: {}", e))?;

        if current_dir == home_dir {
            tracing::debug!("Current directory is home directory, using global config only");
            Ok(ConfigLocation::Global(home_config))
        } else {
            // Default: merge both global and local
            Ok(ConfigLocation::Both(home_config))
        }
    }

    /// Record CLI state to the tracing span
    pub fn record_span(&self, span: &tracing::Span) -> bool {
        if self.is_local() {
            span.record("local_mode", true);
        }
        if self.dry_run() {
            span.record("dry_run", true);
        }
        self.dry_run()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            commands::{Command, GenerateArgs, ValidateArgs},
            output::ColorOverride,
        },
    };

    #[test_log::test]
    fn test_config_location_local() -> crate::Result<()> {
        let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Validate(ValidateArgs {
                local: true,
                ..Default::default()
            }),
        };
        let location = cli.config_location(home_dir)?;
        assert!(matches!(location, crate::ConfigLocation::Local));
        Ok(())
    }

    #[test_log::test]
    fn test_config_location_global() -> crate::Result<()> {
        let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Generate(GenerateArgs {
                global: true,
                ..Default::default()
            }),
        };
        let location = cli.config_location(home_dir)?;
        assert!(matches!(location, crate::ConfigLocation::Global(_)));
        Ok(())
    }

    #[test_log::test]
    fn test_config_location_both() -> crate::Result<()> {
        let home_dir = dirs::home_dir().ok_or(crate::format_err!("unable to find HOME dir"))?;
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Validate(ValidateArgs::default()),
        };
        let location = cli.config_location(home_dir)?;
        // Will be Both or Global depending on if we're in home dir
        assert!(matches!(
            location,
            crate::ConfigLocation::Both(_) | crate::ConfigLocation::Global(_)
        ));
        Ok(())
    }

    #[test_log::test]
    fn test_record_span() {
        let span = tracing::info_span!(
            "test",
            dry_run = tracing::field::Empty,
            local_mode = tracing::field::Empty
        );
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Auto,
            command: Command::Validate(ValidateArgs {
                local: true,
                ..Default::default()
            }),
        };
        let dry_run = cli.record_span(&span);
        assert!(dry_run);
    }
}
