use {
    super::{Cli, Command, GenerateArgs, ValidateArgs, tree::execute_tree},
    crate::{Result, generator::Generator},
    color_eyre::eyre::Context,
    tracing::debug,
};

#[cfg(target_os = "linux")]
use super::watch_linux::execute_watch;
#[cfg(not(target_os = "linux"))]
use super::watch_peasants::execute_watch;

impl Cli {
    /// Execute the CLI command
    pub async fn execute(&self, generator: &Generator) -> Result<()> {
        match &self.command {
            Command::Validate(args) => self.execute_validate(generator, args).await,
            Command::Generate(args) => self.execute_generate(generator, args).await,
            Command::Diff(args) => generator.diff(args),
            Command::Watch(args) => execute_watch(args).await,
            Command::Tree(args) => {
                let value = execute_tree(generator, args)?;
                println!("{}", facet_json::to_string_pretty(&value)?);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn execute_validate(&self, generator: &Generator, args: &ValidateArgs) -> Result<()> {
        let results = generator.write_all(self.dry_run(), false).await?;
        self.format_color()
            .result(self.dry_run(), args.show_templates, results)
    }

    async fn execute_generate(&self, generator: &Generator, args: &GenerateArgs) -> Result<()> {
        if args.diff {
            generator.generate_diff()?;
        }

        let result = generator
            .write_all(self.dry_run(), args.skip_unchanged)
            .await;

        #[cfg(target_os = "linux")]
        if args.notify {
            self.send_notification(&result)?;
        }

        self.format_color()
            .result(self.dry_run(), args.show_templates, result?)
    }

    #[cfg(target_os = "linux")]
    fn send_notification(&self, result: &Result<Vec<crate::generator::AgentResult>>) -> Result<()> {
        use notify_rust::Notification;

        let (summary, body, icon) = match result {
            Ok(results) => {
                let generated = results.iter().filter(|a| !a.is_template()).count();
                (
                    "kg generate",
                    format!("✓ Generated {} agents", generated),
                    "dialog-information",
                )
            }
            Err(e) => ("kg generate", format!("Error: {e}"), "dialog-error"),
        };

        debug!("Sending desktop notification {icon}");
        Notification::new()
            .summary(summary)
            .body(&body)
            .icon(icon)
            .show()
            .wrap_err("Failed to send desktop notification")
            .wrap_err("Ensure notification daemon (e.g. mako, dunst) is running")?;

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            commands::DiffArgs,
            os::{ACTIVE_USER_HOME, Fs},
            output::ColorOverride,
        },
    };

    #[tokio::test]
    #[test_log::test]
    async fn test_exec() -> Result<()> {
        let fs = Fs::new();
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Never,
            command: Command::Validate(ValidateArgs {
                local: true,
                ..Default::default()
            }),
        };

        let generator: Generator = Generator::new(
            fs,
            cli.config_location(ACTIVE_USER_HOME.into())?,
            crate::output::OutputFormat::Json,
        )?;

        cli.execute(&generator).await?;

        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Never,
            command: Command::Generate(GenerateArgs {
                local: true,
                ..Default::default()
            }),
        };

        cli.execute(&generator).await?;
        let cli = Cli {
            debug: false,
            trace: None,
            color_override: ColorOverride::Never,
            command: Command::Diff(DiffArgs::default()),
        };

        cli.execute(&generator).await?;
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_tree_nonexistent_returns_empty() -> Result<()> {
        let fs = Fs::new();
        let generator = Generator::new(
            fs,
            crate::ConfigLocation::Local,
            crate::output::OutputFormat::Json,
        )?;
        let args = super::super::TreeArgs {
            agents: vec!["nonexistent".to_string()],
        };
        let value = execute_tree(&generator, &args)?;
        assert_eq!(value, facet_value::Value::from(facet_value::VObject::new()));
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_tree_known_agents() -> Result<()> {
        let fs = Fs::new();
        let generator = Generator::new(
            fs,
            crate::ConfigLocation::Local,
            crate::output::OutputFormat::Json,
        )?;
        let args = super::super::TreeArgs {
            agents: vec!["base".to_string(), "dependabot".to_string()],
        };
        let value = execute_tree(&generator, &args)?;
        let obj = value.as_object().expect("expected object");
        assert!(obj.get("base").is_some(), "base agent missing");
        assert!(obj.get("dependabot").is_some(), "dependabot agent missing");
        Ok(())
    }
}
