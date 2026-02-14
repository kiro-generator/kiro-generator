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

    pub fn resources(&self) -> HashSet<String> {
        self.agent.resources.clone()
    }
}

/// Main generator that orchestrates agent discovery and merging
#[derive(Facet)]
#[facet(opaque)]
pub struct Generator {
    global_path: PathBuf,
    pub(crate) resolved: discover::ResolvedAgents,
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
            "global_path={} exists={} local_agents={}",
            self.global_path.display(),
            self.fs.exists(&self.global_path),
            self.resolved.has_local
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
        let resolved = discover::discover(&fs, &location, &format)?;
        Ok(Self {
            global_path,
            resolved,
            fs,
            format,
        })
    }

    /// Check if an agent is defined in local kg.toml
    pub fn is_local(&self, agent_name: impl AsRef<str>) -> bool {
        self.resolved.sources.is_local(agent_name)
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
        self.diff_agents(args.format, &[])
    }

    /// Diff for generate command — always compact, no filter
    pub fn generate_diff(&self) -> Result<()> {
        self.diff_agents(crate::output::DiffFormatArg::Compact, &[])
    }

    fn diff_agents(&self, format: crate::output::DiffFormatArg, _filter: &[String]) -> Result<()> {
        let agents: Vec<Manifest> = self.merge()?.into_iter().filter(|a| !a.template).collect();
        let all_agents = !self.resolved.has_local;
        let mut changed = 0;
        let mut unchanged = 0;

        for a in agents {
            if all_agents || self.is_local(&a.name) {
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
        }

        if changed == 0 {
            println!("No changes ({} agents checked)", unchanged);
        } else {
            println!("{} changed, {} unchanged", changed, unchanged);
        }

        Ok(())
    }

    #[tracing::instrument(skip(dry_run, skip_unchanged), level = "info")]
    pub async fn write_all(&self, dry_run: bool, skip_unchanged: bool) -> Result<Vec<AgentResult>> {
        let agents = self.merge()?;
        let mut results = Vec::with_capacity(agents.len());
        // If no local agents defined, write all (global) agents
        // If local agents exist, only write those
        let write_all_agents = !self.resolved.has_local;
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
