use {
    crate::{
        Result,
        agent::{Agent, ToolTarget},
        config::KdlAgent,
        os::Fs,
    },
    miette::{Context, IntoDiagnostic},
    serde::Serialize,
    std::{
        collections::{HashMap, HashSet},
        fmt::{self, Debug},
        path::PathBuf,
    },
};
mod config_location;
mod discover;
mod merge;
pub use config_location::ConfigLocation;

use crate::source::*;

pub struct AgentResult {
    pub kiro_agent: Agent,
    pub agent: KdlAgent,
    pub writable: bool,
    pub destination: PathBuf,
}

impl AgentResult {
    pub fn is_template(&self) -> bool {
        self.agent.is_template()
    }

    pub fn overrides(&self, target: &ToolTarget) -> Vec<String> {
        match target {
            ToolTarget::Read => self
                .agent
                .native_tool
                .read
                .overrides
                .iter()
                .map(|f| f.to_string())
                .collect(),
            ToolTarget::Write => self
                .agent
                .native_tool
                .write
                .overrides
                .iter()
                .map(|f| f.to_string())
                .collect(),
            ToolTarget::Shell => self
                .agent
                .native_tool
                .shell
                .overrides
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
#[derive(Serialize)]
pub struct Generator {
    global_path: PathBuf,
    resolved: discover::ResolvedAgents,
    #[serde(skip)]
    fs: Fs,
    #[serde(skip)]
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
        let global_path = location.global_kg();
        let resolved = discover::discover(&fs, &location, &format)?;
        Ok(Self {
            global_path,
            resolved,
            fs,
            format,
        })
    }

    /// Check if an agent is defined in local kg.kdl
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

    pub async fn write_all(&self, dry_run: bool) -> Result<Vec<AgentResult>> {
        let agents = self.merge()?;
        let mut results = Vec::with_capacity(agents.len());
        // If no local agents defined, write all (global) agents
        // If local agents exist, only write those
        let write_all_agents = self.resolved.has_local;
        for agent in agents {
            if write_all_agents || self.is_local(&agent.name) {
                results.push(self.write(agent, dry_run).await?);
            }
        }
        Ok(results)
    }

    #[tracing::instrument(skip(dry_run), level = "info")]
    pub(crate) async fn write(&self, agent: KdlAgent, dry_run: bool) -> Result<AgentResult> {
        let destination = self.destination_dir(&agent.name);
        let result = AgentResult {
            kiro_agent: Agent::try_from(&agent)?,
            writable: !agent.is_template(),
            destination,
            agent,
        };
        result.kiro_agent.validate()?;
        if dry_run {
            return Ok(result);
        }
        if !self.fs.exists(&result.destination) {
            self.fs
                .create_dir_all(&result.destination)
                .await
                .into_diagnostic()
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
                .write(
                    &out,
                    serde_json::to_string_pretty(&result.kiro_agent).into_diagnostic()?,
                )
                .await
                .into_diagnostic()
                .wrap_err_with(|| format!("failed to write file {}", out.display()))?;
        }
        Ok(result)
    }
}
