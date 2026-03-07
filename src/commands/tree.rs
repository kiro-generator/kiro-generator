use {
    crate::{
        Result,
        commands::{TreeCommand, TreeFormatArg, TreeSummaryArgs},
        generator::Generator,
        tree::{SummaryEntry, SummaryReport, summarize, summarize_concrete, summarize_templates},
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
        TreeFormatArg::Table => print_summary_table(&report),
    };
    Ok(())
}

fn print_summary_table(report: &crate::tree::summary::SummaryReport) {
    println!("Agents:");
    println!("{}", build_summary_table("Agents", &report.agents));

    if !report.templates.is_empty() {
        println!();
        println!("Templates:");
        println!("{}", build_summary_table("Templates", &report.templates));
    }
}

fn build_summary_table(name: &str, entries: &BTreeMap<String, SummaryEntry>) -> Table {
    let mut table = Table::new();
    let header_title = vec![
        Cell::new(name)
            .set_colspan(4)
            .set_alignment(CellAlignment::Center),
    ];
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(header_title);

    table.add_row(vec![
        Cell::new(name),
        Cell::new("Description"),
        Cell::new("Inherits"),
        Cell::new("locations"),
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
                Cell::new(a.locations.join("\n")),
            ]
            .into()
        })
        .collect();
    table.add_rows(rows);

    table
}
