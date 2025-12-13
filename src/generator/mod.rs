use {
    crate::{
        Result,
        agent::{KgAgent, ToolMerge, ToolTarget},
        merging_format::MergingTomlFormat,
        os::Fs,
    },
    color_eyre::eyre::{Context, eyre},
    config::{Config, FileSourceString},
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fmt::{self, Debug},
        path::PathBuf,
    },
    super_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, *},
};
mod config_location;
mod discover;
mod merge;
mod source;
pub use config_location::ConfigLocation;
use source::*;

pub struct AgentResult {
    pub agent: KgAgent,
    pub writable: bool,
    pub destination: PathBuf,
}

impl From<AgentResult> for Row {
    fn from(agent_result: AgentResult) -> Row {
        let mut row = Row::new();
        let dest = if agent_result.agent.skeleton() {
            Cell::new(format!("{} skeleton", "💀"))
        } else if agent_result.destination.is_absolute() {
            Cell::new("$HOME/.kiro/agents")
        } else {
            Cell::new(".kiro/agents")
        };
        row.add_cell(Cell::new(agent_result.agent.name));
        row.add_cell(dest);
        row
    }
}

/// Container for all agent declarations from kg.toml files
#[derive(Debug, Default, Deserialize)]
struct KgConfig {
    #[serde(default)]
    agents: HashMap<String, serde_json::Value>,
}

impl KgConfig {
    fn get(&self, name: &str) -> Result<String> {
        self.agents.get(name).map_or(Ok(String::new()), |value| {
            toml::to_string(value).wrap_err_with(|| format!("failed to toml serialize {name}"))
        })
    }
}

/// Main generator that orchestrates agent discovery and merging
#[derive(Serialize)]
pub struct Generator {
    global_path: PathBuf,
    agents: HashMap<String, KgAgent>,
    local_agents: HashSet<String>, // Agents defined in local kg.toml
    #[serde(skip)]
    fs: Fs,
}

impl Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "global_path={} exists={} local_agents={}",
            self.global_path.display(),
            self.fs.exists(&self.global_path),
            self.local_agents.len()
        )
    }
}

impl Generator {
    /// Create a new Generator with explicit configuration location
    pub fn new(fs: Fs, location: ConfigLocation) -> Result<Self> {
        let global_path = location.global();
        let (agents, local_agents) = discover::agents(&fs, &location)?;
        Ok(Self {
            global_path,
            agents,
            local_agents,
            fs,
        })
    }

    /// Check if an agent is defined in local kg.toml
    pub fn is_local(&self, agent_name: impl AsRef<str>) -> bool {
        self.local_agents.contains(agent_name.as_ref())
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
        let only_global = self.local_agents.is_empty();
        for agent in agents {
            if only_global || self.is_local(&agent.name) {
                results.push(self.write(agent, dry_run).await?);
            }
        }
        Ok(results)
    }

    #[tracing::instrument(skip(dry_run), level = "info")]
    pub(crate) async fn write(&self, agent: KgAgent, dry_run: bool) -> Result<AgentResult> {
        let destination = self.destination_dir(&agent.name);
        let result = AgentResult {
            writable: !agent.skeleton(),
            destination,
            agent,
        };
        if dry_run {
            return Ok(result);
        }
        if !self.fs.exists(&result.destination) {
            self.fs.create_dir_all(&result.destination).await?;
        }
        if result.writable {
            result.agent.write(&self.fs, &result.destination).await?;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::agent::{
            Agent,
            AwsTool,
            ExecuteShellTool,
            ToolTarget,
            WriteTool,
            hook::HookTrigger,
        },
    };

    #[tokio::test]
    #[test_log::test]
    async fn test_discover_agents() -> Result<()> {
        let fs = Fs::new();
        match _discover_agents(fs.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let dest = PathBuf::from(".kiro").join("agents");
                if fs.exists(&dest) {
                    let _dir = fs.read_dir(&dest).await?;
                    // TODO spit contents via tracing::error
                }
                Err(e)
            }
        }
    }

    async fn _discover_agents(fs: Fs) -> Result<()> {
        let location = ConfigLocation::Local;
        let generator = Generator::new(fs.clone(), location)?;
        assert!(!generator.agents.is_empty());
        assert_eq!(4, generator.agents.len());
        assert_eq!(4, generator.local_agents.len());
        // Check that base agent exists and is a skeleton
        assert!(generator.agents.contains_key("base"));
        if let Some(base) = generator.agents.get("base") {
            assert!(base.skeleton());
        }
        tracing::debug!(
            "Loaded Agent Generator Config:\n{}",
            serde_json::to_string_pretty(&generator)?
        );
        let results = generator.write_all(false).await?;

        for r in &results {
            let agent = &r.agent;
            let destination = PathBuf::from(".kiro")
                .join("agents")
                .join(format!("{}.json", agent.name));
            tracing::info!("checking output at {}", destination.as_os_str().display());
            //            tracing::info!("{}",
            // serde_json::to_string_pretty(agent).unwrap());
            if agent.skeleton() {
                assert!(!fs.exists(&destination));
            } else {
                assert!(fs.exists(&destination));
            }
        }
        let content = fs
            .read_to_string(PathBuf::from(".kiro").join("agents").join("aws-test.json"))
            .await?;
        let kiro_agent: Agent = serde_json::from_str(&content)?;
        assert_eq!("aws-test", kiro_agent.name);
        assert_eq!(
            "all the AWS tools you want",
            kiro_agent.description.clone().unwrap_or_default()
        );
        assert!(kiro_agent.model.is_none());
        assert_eq!(
            "you are an AWS expert",
            kiro_agent.prompt.clone().unwrap_or_default()
        );
        assert_eq!(1, kiro_agent.tools.len());
        assert!(kiro_agent.tools.contains("*"));
        tracing::info!("{:?}", kiro_agent.allowed_tools);
        assert_eq!(4, kiro_agent.allowed_tools.len());
        let allowed_tools = ["read", "knowledge", "@fetch", "@awsdocs"];
        for tool in allowed_tools {
            assert!(kiro_agent.allowed_tools.contains(tool));
        }
        tracing::info!("{:?}", kiro_agent.mcp_servers.mcp_servers.keys());
        assert_eq!(4, kiro_agent.mcp_servers.mcp_servers.len());
        for mcp in ["awsbilling", "awsdocs", "cargo", "rustdocs"] {
            assert!(kiro_agent.mcp_servers.mcp_servers.contains_key(mcp));
        }

        tracing::info!("{:?}", kiro_agent.resources);
        assert_eq!(3, kiro_agent.resources.len());
        for r in [
            "file://.amazonq/rules/**/*.md",
            "file://AGENTS.md",
            "file://README.md",
        ] {
            assert!(kiro_agent.resources.contains(r));
        }

        tracing::info!("{:?}", kiro_agent.tools_settings.keys());
        assert_eq!(4, kiro_agent.tools_settings.len());
        let aws_tool: AwsTool = kiro_agent.get_tool(ToolTarget::Aws);
        tracing::info!("{:?}", aws_tool);
        assert!(aws_tool.auto_allow_readonly);
        assert_eq!(2, aws_tool.allowed_services.len());
        assert_eq!(1, aws_tool.denied_services.len());
        assert!(aws_tool.allowed_services.contains("ec2"));
        assert!(aws_tool.allowed_services.contains("s3"));
        assert!(aws_tool.denied_services.contains("iam"));

        assert!(kiro_agent.tool_aliases.is_empty());

        let content = fs
            .read_to_string(
                PathBuf::from(".kiro")
                    .join("agents")
                    .join("dependabot.json"),
            )
            .await?;
        let kiro_agent: Agent = serde_json::from_str(&content)?;
        assert_eq!("dependabot", kiro_agent.name);
        let exec_tool: ExecuteShellTool = kiro_agent.get_tool(ToolTarget::Shell);
        tracing::info!("{:?}", exec_tool);
        assert!(exec_tool.allowed_commands.contains("git commit .*"));
        assert!(exec_tool.allowed_commands.contains("git push .*"));
        assert!(!exec_tool.denied_commands.contains("git commit .*"));
        assert!(!exec_tool.denied_commands.contains("git push .*"));

        let fs_tool: WriteTool = kiro_agent.get_tool(ToolTarget::Write);
        tracing::info!("{:?}", fs_tool);
        assert!(fs_tool.allowed_paths.contains(".*Cargo.toml.*"));
        assert!(!fs_tool.denied_paths.contains(".*Cargo.toml.*"));

        tracing::info!("{:?}", kiro_agent.hooks);
        assert_eq!(2, kiro_agent.hooks.len());
        assert!(kiro_agent.hooks.contains_key(&HookTrigger::AgentSpawn));
        assert_eq!(
            2,
            kiro_agent
                .hooks
                .get(&HookTrigger::AgentSpawn)
                .unwrap()
                .len()
        );
        Ok(())
    }
}
