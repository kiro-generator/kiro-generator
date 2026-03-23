use {
    crate::{
        Manifest,
        Result,
        kiro::{KiroAgent, ToolTarget},
        os::Fs,
    },
    color_eyre::eyre::Context,
    facet::Facet,
    rediff::FacetDiff,
    std::{
        collections::{HashMap, HashSet},
        fmt::{self, Debug, Display},
        path::PathBuf,
    },
};

pub(super) const MAX_AGENT_DIR_DEPTH: usize = 5;
/// max number of files or directories in a given Path.
/// 1000 should be more than enough to handle templates and real agents
pub(super) const MAX_AGENT_DIR_ENTRIES: usize = 1000;

mod config_location;
pub(crate) mod discover;
mod merge;

pub use config_location::*;

use crate::source::*;

#[derive(Debug, Clone)]
pub enum AgentDiff {
    New,
    Same,
    Changed(String),
}
impl Display for AgentDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "new agent"),
            Self::Same => write!(f, "no changes"),
            Self::Changed(s) => write!(f, "{s}"),
        }
    }
}

pub struct AgentResult {
    pub kiro_agent: KiroAgent,
    pub agent: Manifest,
    pub writable: bool,
    pub destination: PathBuf,
}

impl AgentResult {
    pub fn is_template(&self) -> bool {
        self.agent.template
    }

    pub fn force_allow(&self, target: &ToolTarget) -> Vec<String> {
        match target {
            ToolTarget::Read => self
                .agent
                .native_tools
                .read
                .force_allow
                .iter()
                .map(|f| f.to_string())
                .collect(),
            ToolTarget::Write => self
                .agent
                .native_tools
                .write
                .force_allow
                .iter()
                .map(|f| f.to_string())
                .collect(),
            ToolTarget::Shell => self
                .agent
                .native_tools
                .shell
                .force_allow
                .iter()
                .map(|f| f.to_string())
                .collect(),
            _ => vec![],
        }
    }
}

/// Main generator that orchestrates agent discovery and merging
#[derive(Facet)]
#[facet(opaque)]
pub struct Generator {
    global_path: PathBuf,
    pub(crate) agents: HashMap<String, AgentSourceSlots>,
    #[facet(skip, default)]
    fs: Fs,
    #[facet(skip, default)]
    #[allow(unused)]
    format: crate::output::OutputFormat,
}

impl Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "global_path={} exists={}",
            self.global_path.display(),
            self.fs.exists(&self.global_path),
        )
    }
}

impl Generator {
    /// Create a new Generator with explicit configuration location
    pub fn new(
        fs: Fs,
        location: ConfigLocation,
        format: crate::output::OutputFormat,
    ) -> Result<Self> {
        let global_path = location.global_path();
        let agents = discover::discover(&fs, &location, &format)?;
        Ok(Self {
            global_path,
            agents,
            fs,
            format,
        })
    }

    pub fn concrete_agents(&self) -> Vec<&AgentSourceSlots> {
        self.agents
            .values()
            .filter(|a| !a.merged.template)
            .collect()
    }

    pub fn template_agents(&self) -> Vec<&AgentSourceSlots> {
        self.agents.values().filter(|a| a.merged.template).collect()
    }

    pub fn contains_local_agents(&self) -> bool {
        self.agents.values().any(|s| s.has_local())
    }

    /// Check if an agent is defined in local kg.toml
    pub fn is_local(&self, agent_name: impl AsRef<str>) -> bool {
        self.agents
            .get(agent_name.as_ref())
            .map(|s| s.has_local())
            .unwrap_or(false)
    }

    /// Get the destination directory for an agent (global or local)
    pub fn destination_dir(&self, agent_name: impl AsRef<str>) -> PathBuf {
        if self.is_local(agent_name) {
            PathBuf::from(".kiro").join("agents")
        } else {
            dirs::home_dir()
                .map(|h| h.join(".kiro").join("agents"))
                .unwrap_or_else(|| PathBuf::from(".kiro").join("agents"))
        }
    }

    /// Compute diff between existing agent file and generated agent
    fn compute_diff(
        &self,
        agent_name: &str,
        generated: &KiroAgent,
        format: crate::output::DiffFormatArg,
    ) -> Result<AgentDiff> {
        let destination = self
            .destination_dir(agent_name)
            .join(format!("{}.json", agent_name));

        if !self.fs.exists(&destination) {
            return Ok(AgentDiff::New);
        }

        let existing = self.fs.read_to_string_sync(&destination)?;
        let existing_agent = facet_json::from_str::<KiroAgent>(&existing).wrap_err_with(|| {
            format!(
                "Failed to parse existing agent file {}",
                destination.display()
            )
        })?;

        let normalized_existing = existing_agent.normalize();
        let normalized_generated = generated.clone().normalize();
        let diff = normalized_existing.diff(&normalized_generated);

        if diff.is_equal() {
            Ok(AgentDiff::Same)
        } else {
            // Choose formatting based on args
            let formatted = match format {
                crate::output::DiffFormatArg::Agent => rediff::format_diff_compact_plain(&diff),
                crate::output::DiffFormatArg::Compact => rediff::format_diff_compact(&diff),
                crate::output::DiffFormatArg::Plain => {
                    let config = rediff::DiffFormat {
                        colors: false,
                        max_inline_changes: 10,
                        prefer_compact: false,
                    };
                    rediff::format_diff(&diff, &config)
                }
                crate::output::DiffFormatArg::Full => {
                    let config = rediff::DiffFormat {
                        colors: true,
                        max_inline_changes: 10,
                        prefer_compact: false,
                    };
                    rediff::format_diff(&diff, &config)
                }
            };

            Ok(AgentDiff::Changed(formatted))
        }
    }

    #[tracing::instrument(level = "info")]
    pub fn diff(&self, args: &crate::commands::DiffArgs) -> Result<()> {
        self.diff_agents(args.format, &args.agents)
    }

    /// Diff for generate command — always compact, no filter
    pub fn generate_diff(&self) -> Result<()> {
        self.diff_agents(crate::output::DiffFormatArg::Compact, &[])
    }

    fn diff_agents(&self, format: crate::output::DiffFormatArg, filter: &[String]) -> Result<()> {
        let agents: Vec<Manifest> = self.merge()?.into_iter().filter(|a| !a.template).collect();
        let all_agents = !self.contains_local_agents();
        let mut changed = 0;
        let mut unchanged = 0;
        let visible_agents = agents
            .into_iter()
            .filter(|agent| all_agents || self.is_local(&agent.name))
            .collect::<Vec<_>>();
        let missing_agents = missing_agents(&visible_agents, filter);
        let agents = filter_agents(visible_agents, filter);

        for a in agents {
            let destination = self
                .destination_dir(&a.name)
                .join(format!("{}.json", a.name));
            let generated_agent = KiroAgent::try_from(&a)?;

            match self.compute_diff(&a.name, &generated_agent, format)? {
                AgentDiff::New => {
                    println!("{}: (new agent)", destination.display());
                    println!();
                    changed += 1;
                }
                AgentDiff::Changed(diff_output) => {
                    println!("{}:", destination.display());
                    println!("{}", diff_output);
                    println!();
                    changed += 1;
                }
                AgentDiff::Same => {
                    unchanged += 1;
                }
            }
        }

        println!("{}", diff_summary(changed, unchanged, &missing_agents));

        Ok(())
    }

    #[tracing::instrument(skip(dry_run, skip_unchanged), level = "info")]
    pub async fn write_all(&self, dry_run: bool, skip_unchanged: bool) -> Result<Vec<AgentResult>> {
        let agents = self.merge()?;
        let mut results = Vec::with_capacity(agents.len());
        // If no local agents defined, write all (global) agents
        // If local agents exist, only write those
        let write_all_agents = !self.contains_local_agents();
        for agent in agents {
            let span = tracing::info_span!("agent", name = ?agent.name, local = self.is_local(&agent.name));
            let _enter = span.enter();
            if write_all_agents || self.is_local(&agent.name) {
                results.push(self.write(agent, dry_run, skip_unchanged).await?);
            }
        }
        Ok(results)
    }

    #[tracing::instrument(skip(dry_run,skip_unchanged), level = "info", fields(out = tracing::field::Empty))]
    pub(crate) async fn write(
        &self,
        agent: Manifest,
        dry_run: bool,
        skip_unchanged: bool,
    ) -> Result<AgentResult> {
        let destination = self.destination_dir(&agent.name);
        let result = AgentResult {
            kiro_agent: KiroAgent::try_from(&agent)?,
            writable: !agent.template,
            destination,
            agent,
        };

        if let Ok(j) = facet_json::to_string_pretty(&result.kiro_agent) {
            tracing::trace!("{j}");
        }
        result.kiro_agent.validate()?;
        if dry_run {
            return Ok(result);
        }
        if !self.fs.exists(&result.destination) {
            self.fs
                .create_dir_all(&result.destination)
                .await
                .wrap_err_with(|| {
                    format!(
                        "failed to create directory {}",
                        result.destination.display()
                    )
                })?;
        }
        if result.writable {
            let out = result
                .destination
                .join(format!("{}.json", result.agent.name));

            tracing::Span::current().record("out", tracing::field::display(&out.display()));

            if !skip_unchanged {
                // Default: always write
                self.fs
                    .write(&out, facet_json::to_string_pretty(&result.kiro_agent)?)
                    .await
                    .wrap_err_with(|| format!("failed to write file {}", out.display()))?;
                return Ok(result);
            }

            // --skip-unchanged: compute diff and skip if unchanged
            let diff = self.compute_diff(
                &result.agent.name,
                &result.kiro_agent,
                crate::output::DiffFormatArg::Compact,
            )?;
            tracing::debug!("{diff}");
            match diff {
                AgentDiff::New => {
                    self.fs
                        .write(&out, facet_json::to_string_pretty(&result.kiro_agent)?)
                        .await
                        .wrap_err_with(|| format!("failed to write file {}", out.display()))?;
                }
                AgentDiff::Changed(_) => {
                    self.fs
                        .write(&out, facet_json::to_string_pretty(&result.kiro_agent)?)
                        .await
                        .wrap_err_with(|| format!("failed to write file {}", out.display()))?;
                }
                AgentDiff::Same => {}
            }
        }
        Ok(result)
    }
}

fn filter_agents(agents: Vec<Manifest>, filter: &[String]) -> Vec<Manifest> {
    if filter.is_empty() {
        agents
    } else {
        agents
            .into_iter()
            .filter(|agent| filter.contains(&agent.name))
            .collect()
    }
}

fn missing_agents(agents: &[Manifest], filter: &[String]) -> Vec<String> {
    if filter.is_empty() {
        return Vec::new();
    }

    let available = agents
        .iter()
        .map(|agent| agent.name.as_str())
        .collect::<Vec<_>>();
    let mut missing = Vec::new();
    for name in filter {
        if !available.contains(&name.as_str()) && !missing.contains(name) {
            missing.push(name.clone());
        }
    }
    missing
}

fn diff_summary(changed: usize, unchanged: usize, missing: &[String]) -> String {
    let mut summary = if changed == 0 {
        format!("No changes ({} agents checked)", unchanged)
    } else {
        format!("{} changed, {} unchanged", changed, unchanged)
    };

    if !missing.is_empty() {
        let label = if missing.len() == 1 {
            "agent not found in current scope"
        } else {
            "agents not found in current scope"
        };
        summary.push_str(&format!("; {label}: {}", missing.join(", ")));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn source_slots_returns_agent_sources() -> Result<()> {
        let generator = crate::tree::fixture_generator()?;
        let agent_source_slots = generator.agents.get("parent").expect("parent should exist");
        let slots = agent_source_slots.source_slots();

        assert_eq!(slots.len(), 4);
        assert_eq!(
            slots
                .iter()
                .filter(|slot| slot.location().is_some())
                .count(),
            1
        );
        assert_eq!(
            slots[2].location().map(|path| path.display().to_string()),
            Some(String::from("test"))
        );
        assert!(!generator.agents.contains_key("missing"));

        Ok(())
    }

    #[test]
    fn filter_agents_returns_all_agents_when_filter_empty() {
        let agents = vec![
            Manifest::new(String::from("alpha"), false),
            Manifest::new(String::from("beta"), false),
        ];

        let filtered = filter_agents(agents, &[]);

        assert_eq!(
            filtered
                .into_iter()
                .map(|agent| agent.name)
                .collect::<Vec<_>>(),
            vec![String::from("alpha"), String::from("beta")]
        );
    }

    #[test]
    fn filter_agents_keeps_requested_names_in_existing_order() {
        let agents = vec![
            Manifest::new(String::from("alpha"), false),
            Manifest::new(String::from("beta"), false),
            Manifest::new(String::from("gamma"), false),
        ];
        let filter = vec![String::from("gamma"), String::from("alpha")];

        let filtered = filter_agents(agents, &filter);

        assert_eq!(
            filtered
                .into_iter()
                .map(|agent| agent.name)
                .collect::<Vec<_>>(),
            vec![String::from("alpha"), String::from("gamma")]
        );
    }

    #[test]
    fn missing_agents_reports_unknown_names_once_in_filter_order() {
        let agents = vec![
            Manifest::new(String::from("alpha"), false),
            Manifest::new(String::from("beta"), false),
        ];
        let filter = vec![
            String::from("gamma"),
            String::from("alpha"),
            String::from("gamma"),
            String::from("delta"),
        ];

        assert_eq!(missing_agents(&agents, &filter), vec![
            String::from("gamma"),
            String::from("delta")
        ]);
    }

    #[test]
    fn diff_summary_mentions_missing_agents_in_current_scope() {
        assert_eq!(
            diff_summary(0, 0, &[String::from("missing")]),
            String::from(
                "No changes (0 agents checked); agent not found in current scope: missing"
            )
        );
    }
}
