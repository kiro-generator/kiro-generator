use {
    crate::{
        Result,
        commands::{TreeCommand, TreeFormatArg, TreeSummaryArgs},
        generator::Generator,
        tree::{SummaryEntry, SummaryReport, summarize_concrete, summarize_templates},
    },
    color_eyre::eyre::bail,
    facet::Facet,
    std::collections::BTreeMap,
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
};

#[derive(Facet)]
struct SummaryJsonWithTemplates {
    agents: BTreeMap<String, SummaryEntry>,
    templates: BTreeMap<String, SummaryEntry>,
}

#[derive(Facet)]
struct SummaryJsonAgentsOnly {
    agents: BTreeMap<String, SummaryEntry>,
}

pub(super) fn execute_tree(generator: &Generator, cmd: &TreeCommand) -> Result<()> {
    match cmd {
        TreeCommand::Summary(args) => summary(generator, args),
        TreeCommand::Detail(_) => {
            bail!("`kg tree detail` is still being refactored; use `kg tree summary` for now")
        }
        TreeCommand::Invert(_) => {
            bail!("`kg tree invert` is still being refactored; use `kg tree summary` for now")
        }
    }
}

fn summary(generator: &Generator, args: &TreeSummaryArgs) -> Result<()> {
    let report = SummaryReport {
        agents: summarize_concrete(generator),
        templates: if args.no_templates {
            Default::default()
        } else {
            summarize_templates(generator)
        },
    };
    match args.format {
        TreeFormatArg::Json => println!("{}", facet_json::to_string_pretty(&report)?),
        TreeFormatArg::Table => print_summary_tables(generator, &report, args.locations),
    };
    Ok(())
}

fn print_summary_tables(
    generator: &Generator,
    report: &crate::tree::summary::SummaryReport,
    show_locations: bool,
) {
    println!("{}", build_summary_table("Agents", &report.agents));

    if !report.templates.is_empty() {
        println!();
        println!("{}", build_summary_table("Templates", &report.templates));
    }

    if show_locations {
        println!();
        println!("{}", build_locations_table(generator));
    }
}

fn file_locations(generator: &Generator) -> Vec<Row> {
    let mut agents: Vec<_> = generator.agents.values().collect();
    agents.sort_by(|left, right| left.name.cmp(&right.name));

    agents
        .into_iter()
        .map(|a| {
            Row::from(vec![
                a.name.clone(),
                a.global_manifest
                    .location()
                    .unwrap_or_default()
                    .display()
                    .to_string(),
                a.global_agent_file
                    .location()
                    .unwrap_or_default()
                    .display()
                    .to_string(),
                a.local_manifest
                    .location()
                    .unwrap_or_default()
                    .display()
                    .to_string(),
                a.local_agent_file
                    .location()
                    .unwrap_or_default()
                    .display()
                    .to_string(),
            ])
        })
        .collect()
}

fn build_titled_table(name: &str, columns: u16) -> Table {
    let mut table = Table::new();
    let header_title = vec![
        Cell::new(name)
            .set_colspan(columns)
            .set_alignment(CellAlignment::Center),
    ];
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header_title);

    table
}

fn build_summary_table(name: &str, entries: &BTreeMap<String, SummaryEntry>) -> Table {
    let mut table = build_titled_table(name, 3);
    table.add_row(vec![
        Cell::new("Name"),
        Cell::new("Description"),
        Cell::new("Inherits"),
    ]);
    let rows: Vec<Row> = entries
        .values()
        .map(|a| {
            vec![
                Cell::new(a.name.clone()),
                Cell::new(a.description.clone()),
                Cell::new(a.inherits_join().unwrap_or_else(|e| {
                    tracing::warn!("failed to create inherits list {e}");
                    String::from("Failed to serialize inheritance structure")
                })),
            ]
            .into()
        })
        .collect();
    table.add_rows(rows);

    table
}

fn build_locations_table(generator: &Generator) -> Table {
    let mut table = build_titled_table("Locations", 5);
    table.add_row(vec![
        Cell::new("Name"),
        Cell::new("Global Manifest"),
        Cell::new("Global File"),
        Cell::new("Local Manifest"),
        Cell::new("Local File"),
    ]);
    table.add_rows(file_locations(generator));

    table
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            AgentSourceSlots,
            ConfigLocation,
            Manifest,
            SourceSlot,
            os::Fs,
            output::OutputFormat,
            source::KgAgentSource,
        },
        std::{collections::HashMap, path::PathBuf},
    };

    fn source_slot(path: KgAgentSource) -> SourceSlot {
        SourceSlot {
            path: Some(path),
            manifest: Manifest::default(),
        }
    }

    fn fixture_generator() -> Result<Generator> {
        let mut generator = Generator::new(Fs::new(), ConfigLocation::Local, OutputFormat::Json)?;
        generator.agents = HashMap::from([
            (String::from("rust"), AgentSourceSlots {
                name: String::from("rust"),
                global_manifest: source_slot(KgAgentSource::GlobalManifest(PathBuf::from(
                    "/tmp/base.toml",
                ))),
                local_agent_file: source_slot(KgAgentSource::LocalFile(PathBuf::from(
                    ".kiro/generators/agents/rust.toml",
                ))),
                merged: Manifest::default(),
                ..Default::default()
            }),
            (String::from("aws"), AgentSourceSlots {
                name: String::from("aws"),
                global_agent_file: source_slot(KgAgentSource::GlobalFile(PathBuf::from(
                    "/tmp/aws.toml",
                ))),
                local_manifest: source_slot(KgAgentSource::LocalManifest(PathBuf::from(
                    ".kiro/generators/manifests/aws.toml",
                ))),
                merged: Manifest::default(),
                ..Default::default()
            }),
        ]);

        Ok(generator)
    }

    #[tokio::test]
    #[test_log::test]
    async fn file_locations_are_sorted_and_include_agent_name() -> Result<()> {
        let generator = fixture_generator()?;
        let rows = file_locations(&generator);
        let rendered = build_locations_table(&generator).to_string();

        assert_eq!(rows.len(), 2);
        assert!(rendered.contains("/tmp/aws.toml"));
        assert!(rendered.contains(".kiro/generators/manifests/aws.toml"));
        assert!(rendered.contains("/tmp/base.toml"));
        assert!(rendered.contains(".kiro/generators/agents/rust.toml"));
        assert!(rendered.find("aws").unwrap() < rendered.find("rust").unwrap());

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn locations_table_has_lookup_headers() -> Result<()> {
        let generator = fixture_generator()?;
        let rendered = build_locations_table(&generator).to_string();

        assert!(rendered.contains("Locations"));
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("Global Manifest"));
        assert!(rendered.contains("Global File"));
        assert!(rendered.contains("Local Manifest"));
        assert!(rendered.contains("Local File"));
        assert!(rendered.contains("aws"));
        assert!(rendered.contains("rust"));

        Ok(())
    }
}
