use {
    crate::{
        Result,
        generator::AgentResult,
        kiro::{KiroAgent, ToolTarget},
        source::KdlSources,
    },
    color_eyre::eyre::Context,
    colored::Colorize,
    std::fmt::Display,
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
    tracing::enabled,
};

/// Override the color setting. Default is [`ColorOverride::Auto`].
#[derive(Copy, Clone, Debug, Default, clap::ValueEnum)]
pub enum ColorOverride {
    /// Always display color (i.e. force it).
    Always,
    /// Automatically determine if color should be used or not.
    #[default]
    Auto,
    /// Never display color.
    Never,
}

impl Display for ColorOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ColorOverride::Always => "always",
            ColorOverride::Auto => "auto",
            ColorOverride::Never => "never",
        };

        write!(f, "{s}")
    }
}
#[derive(Copy, Clone, Default, Debug, clap::ValueEnum)]
pub enum OutputFormatArg {
    #[default]
    Table,
    Json,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputFormat {
    Table(bool),
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table(true)
    }
}

impl Display for OutputFormatArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Table => write!(f, "table"),
            Self::Json => write!(f, "json"),
        }
    }
}

pub(crate) fn agent_header() -> Cell {
    Cell::new(format!("Agent {}", emojis_rs::EMOJI_ROBOT))
}

fn serialize_yaml(label: &str, values: &[String]) -> Option<Cell> {
    if values.is_empty() {
        return None;
    }
    serde_yaml2::to_string(values)
        .inspect_err(|e| tracing::warn!("Failed to serialize {}: {}", label, e))
        .ok()
        .map(|l| Cell::new(format!("{}{}", label, l)))
}

impl OutputFormat {
    pub fn sources(&self, sources: &KdlSources) -> Result<()> {
        // Always trace per agent
        for (name, agent_sources) in sources.iter() {
            let span = tracing::trace_span!("agent", name = name.as_str());
            let _enter = span.enter();
            tracing::trace!(
                sources = ?agent_sources.iter().map(ToString::to_string).collect::<Vec<_>>(),
                "agent sources"
            );
        }

        // Only show table in debug mode
        if !enabled!(tracing::Level::DEBUG) {
            return Ok(());
        }
        match self {
            Self::Table(_color) => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        agent_header(),
                        Cell::new("Sources")
                            .set_colspan(4)
                            .set_alignment(CellAlignment::Center),
                    ]);
                for (name, agent_sources) in sources.iter() {
                    let mut row: Vec<Cell> = vec![Cell::new(name.to_string())];
                    row.extend(agent_sources.iter().map(|s| s.to_cell()));
                    table.add_row(row);
                }
                eprintln!("{table}");
                Ok(())
            }
            Self::Json => Ok(()),
        }
    }

    fn agent_result_to_row(&self, result: &AgentResult) -> Row {
        let mut row = Row::new();

        // Agent name with template indicator
        let name_cell = if result.agent.template {
            Cell::new(format!("{} {}", result.agent.name, "💀"))
        } else {
            Cell::new(&result.agent.name)
        };
        row.add_cell(name_cell);

        // Location: 🏠 for global, 📁 for local
        let location = if result.agent.template {
            Cell::new("")
        } else if result.destination.is_absolute() {
            Cell::new("🏠")
        } else {
            Cell::new("📁")
        };
        row.add_cell(location);

        // MCP servers (only enabled ones)
        let mut servers = Vec::new();
        for (k, v) in &result.agent.mcp_servers {
            if !v.disabled {
                servers.push(k.clone());
            }
        }
        servers.sort();
        row.add_cell(Cell::new(servers.join(", ")));

        // Override permissions (security-critical)
        let sh = result.force_allow(&ToolTarget::Shell);
        let read = result.force_allow(&ToolTarget::Read);
        let write = result.force_allow(&ToolTarget::Write);

        let mut forced = vec![];
        if let Some(c) = serialize_yaml("cmds:\n", &sh) {
            forced.push(c);
        }
        if let Some(c) = serialize_yaml("read:\n", &read) {
            forced.push(c);
        }
        if let Some(c) = serialize_yaml("write:\n", &write) {
            forced.push(c);
        }

        match forced.len() {
            0 => {
                row.add_cell(Cell::new("").set_colspan(3));
            }
            1 => {
                row.add_cell(forced[0].clone().set_colspan(3));
            }
            2 => {
                row.add_cell(forced[0].clone());
                row.add_cell(forced[1].clone().set_colspan(2));
            }
            _ => {
                for c in forced {
                    row.add_cell(c);
                }
            }
        }

        row
    }

    fn maybe_color(&self, mut cell: Cell, c: Color) -> Cell {
        match self {
            Self::Table(color) if *color => {
                cell = cell.fg(c);
            }
            _ => {}
        };
        cell
    }

    pub fn result(
        &self,
        dry_run: bool,
        show_templates: bool,
        results: Vec<AgentResult>,
    ) -> Result<()> {
        match self {
            Self::Table(_) => {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                // Different header styling for dry-run vs actual generation
                if dry_run {
                    table.set_header(vec![
                        self.maybe_color(
                            Cell::new(format!("Agent {} (PREVIEW)", emojis_rs::EMOJI_ROBOT)),
                            Color::Yellow,
                        ),
                        self.maybe_color(Cell::new("Loc"), Color::Yellow),
                        self.maybe_color(
                            Cell::new(format!("MCP {}", emojis_rs::EMOJI_COMPUTER)),
                            Color::Yellow,
                        ),
                        self.maybe_color(
                            Cell::new("Override (Allowed) Permissions")
                                .set_colspan(3)
                                .set_alignment(CellAlignment::Center),
                            Color::Yellow,
                        ),
                    ]);
                } else {
                    table.set_header(vec![
                        agent_header(),
                        Cell::new("Loc"),
                        Cell::new(format!("MCP {}", emojis_rs::EMOJI_COMPUTER)),
                        Cell::new("Override (Allowed) Permissions")
                            .set_colspan(3)
                            .set_alignment(CellAlignment::Center),
                    ]);
                }

                for result in &results {
                    if show_templates || !result.agent.template {
                        table.add_row(self.agent_result_to_row(result));
                    }
                }

                println!("{table}");
                if dry_run {
                    println!("\n{} Config is valid", emojis_rs::EMOJI_SUCCESS);
                    println!(
                        "{} Run {} to generate agent files",
                        "→".yellow().bold(),
                        "kg generate".green().bold()
                    );
                } else {
                    println!("\n{} Generated agent files", emojis_rs::EMOJI_CHECK);
                }
                Ok(())
            }
            Self::Json => {
                let kiro_agents: Vec<KiroAgent> =
                    results.into_iter().map(|a| a.kiro_agent).collect();
                println!(
                    "{}",
                    facet_json::to_string_pretty(&kiro_agents)
                        .wrap_err("Failed to serialize agents to JSON")?
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_override_display() {
        assert_eq!(ColorOverride::Always.to_string(), "always");
        assert_eq!(ColorOverride::Auto.to_string(), "auto");
        assert_eq!(ColorOverride::Never.to_string(), "never");
    }

    #[test]
    fn output_format_arg_display() {
        assert_eq!(OutputFormatArg::Table.to_string(), "table");
        assert_eq!(OutputFormatArg::Json.to_string(), "json");
    }

    #[test]
    fn output_format_default() {
        let fmt = OutputFormat::default();
        assert!(matches!(fmt, OutputFormat::Table(true)));
    }
}
