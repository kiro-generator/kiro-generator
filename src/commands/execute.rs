use {
    super::{Cli, Command, GenerateArgs, ValidateArgs},
    crate::{Result, generator::Generator},
    color_eyre::eyre::Context,
    tracing::debug,
};

impl Cli {
    /// Execute the CLI command
    pub async fn execute(&self, generator: &Generator) -> Result<()> {
        match &self.command {
            Command::Validate(args) => self.execute_validate(generator, args).await,
            Command::Generate(args) => self.execute_generate(generator, args).await,
            Command::Diff(_) => generator.diff(),
            _ => Ok(()),
        }
    }

    async fn execute_validate(&self, generator: &Generator, args: &ValidateArgs) -> Result<()> {
        let results = generator.write_all(self.dry_run(), false).await?;
        self.format_color()
            .result(self.dry_run(), args.show_templates, results)
    }

    async fn execute_generate(&self, generator: &Generator, args: &GenerateArgs) -> Result<()> {
        let result = generator.write_all(self.dry_run(), args.force).await;

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
