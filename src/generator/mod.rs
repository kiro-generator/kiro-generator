use {
    crate::{
        Result,
        agent::{Agent, ToolTarget},
        config::Manifest,
        os::Fs,
    },
    color_eyre::eyre::Context,
    facet::Facet,
    facet_diff::FacetDiff,
    std::{
        collections::{HashMap, HashSet},
        fmt::{self, Debug},
        path::PathBuf,
    },
};

pub(super) const MAX_AGENT_DIR_DEPTH: usize = 5;
/// max number of files or directories in a given Path.
/// 1000 should be more than enough to handle templates and real agents
pub(super) const MAX_AGENT_DIR_ENTRIES: usize = 1000;

mod config_location;
mod discover;
mod merge;
pub use config_location::ConfigLocation;

use crate::source::*;

pub struct AgentResult {
    pub kiro_agent: Agent,
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
    resolved: discover::ResolvedAgents,
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

    #[tracing::instrument(level = "info")]
    pub fn diff(&self) -> Result<()> {
        let agents: Vec<Manifest> = self.merge()?.into_iter().filter(|a| !a.template).collect();
        let all_agents = !self.resolved.has_local;
        for a in agents {
            if all_agents || self.is_local(&a.name) {
                let destination = self
                    .destination_dir(&a.name)
                    .join(format!("{}.json", a.name));
                let kiro_agent = Agent::try_from(&a)?;
                if self.fs.exists(&destination) {
                    println!("-----{}-----", destination.display());
                    let existing = self.fs.read_to_string_sync(&destination)?;
                    match facet_json::from_str::<Agent>(&existing) {
                        Err(e) => eprintln!("warning failed to deserialize {} {e}", a.name),
                        Ok(agent) => {
                            let diff = kiro_agent.diff(&agent);
                            println!("{}", facet_diff::format_diff_default(&diff));
                        }
                    };
                    println!("--------------");
                } else {
                    println!("agent {} is new", a.name);
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(dry_run), level = "info")]
    pub async fn write_all(&self, dry_run: bool) -> Result<Vec<AgentResult>> {
        let agents = self.merge()?;
        let mut results = Vec::with_capacity(agents.len());
        // If no local agents defined, write all (global) agents
        // If local agents exist, only write those
        let write_all_agents = !self.resolved.has_local;
        for agent in agents {
            let span = tracing::debug_span!("agent", name = ?agent.name, local = self.is_local(&agent.name));
            let _enter = span.enter();
            if write_all_agents || self.is_local(&agent.name) {
                results.push(self.write(agent, dry_run).await?);
            }
        }
        Ok(results)
    }

    #[tracing::instrument(skip(dry_run), level = "info")]
    pub(crate) async fn write(&self, agent: Manifest, dry_run: bool) -> Result<AgentResult> {
        let destination = self.destination_dir(&agent.name);
        let result = AgentResult {
            kiro_agent: Agent::try_from(&agent)?,
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

            self.fs
                .write(&out, facet_json::to_string_pretty(&result.kiro_agent)?)
                .await
                .wrap_err_with(|| format!("failed to write file {}", out.display()))?;
        }
        Ok(result)
    }
}
