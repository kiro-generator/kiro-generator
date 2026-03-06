use {
    crate::{
        Result,
        commands::{TreeCommand, TreeFormatArg, TreeSummaryArgs},
        generator::Generator,
        tree::summary::{
            SummaryEntry,
            SummaryOptions,
            SummaryReport,
            summarize,
            summarize_concrete,
            summarize_templates,
        },
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
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Agent"),
            Cell::new("Description"),
            Cell::new("Inherits"),
        ]);

    for (name, entry) in &report.agents {
        table.add_row(summary_row(name, entry));
    }

    if !report.templates.is_empty() {
        table.add_row(vec![Cell::new(""), Cell::new(""), Cell::new("")]);
        table.add_row(vec![
            Cell::new("Templates:").fg(super_table::Color::DarkGrey),
            Cell::new(""),
            Cell::new(""),
        ]);

        for (name, entry) in &report.templates {
            table.add_row(summary_row(name, entry));
        }
    }

    println!("{table}");
}

fn summary_row(name: &str, entry: &SummaryEntry) -> Vec<Cell> {
    let inherits: Vec<&str> = entry.inherits.iter().map(String::as_str).collect();
    vec![
        Cell::new(name),
        Cell::new(&entry.description),
        Cell::new(inherits.join(", ")),
    ]
}
