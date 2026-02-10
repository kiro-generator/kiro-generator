use {
    crate::{
        kiro::{KiroAgent, diff::NormalizedAgent},
        os::Fs,
        util::prompt_confirm,
    },
    color_eyre::eyre::WrapErr,
    facet::Facet,
    std::{
        collections::{BTreeMap, BTreeSet},
        path::{Path, PathBuf},
        process::Command,
    },
};

const SKILL_MD: &str = include_str!("../../resources/SKILL.md");
const SKILL_NAME: &str = "kg-helper";

/// Scan a directory for .json files and return (name, path) tuples
async fn find_agent_jsons(fs: &Fs, dir: &Path) -> crate::Result<Vec<(String, PathBuf)>> {
    let mut results = Vec::new();
    if !fs.exists(dir) {
        return Ok(results);
    }
    let mut entries = fs.read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| crate::format_err!("Invalid filename: {}", path.display()))?
                .to_string();
            results.push((name, path));
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

fn parse_agent(content: &str, path: &Path, source: &str) -> crate::Result<AnalysisAgent> {
    let kiro: KiroAgent = facet_json::from_str(content)?;
    Ok(AnalysisAgent {
        source: source.to_string(),
        path: path.display().to_string(),
        agent: kiro.normalize(),
    })
}

type OverlapMap = BTreeMap<String, BTreeSet<String>>;

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct AnalysisAgent {
    source: String,
    path: String,
    #[facet(flatten)]
    agent: NormalizedAgent,
}

fn overlap_allowed_tools(agents: &BTreeMap<String, AnalysisAgent>) -> OverlapMap {
    let mut map: OverlapMap = BTreeMap::new();
    for (name, src) in agents {
        for tool in &src.agent.allowed_tools {
            map.entry(tool.clone()).or_default().insert(name.clone());
        }
    }
    map.retain(|_, v| v.len() > 1);
    map
}

fn overlap_mcp(agents: &BTreeMap<String, AnalysisAgent>) -> OverlapMap {
    let mut map: OverlapMap = BTreeMap::new();
    for (name, src) in agents {
        for tool in &src.agent.other_tools {
            map.entry(tool.clone()).or_default().insert(name.clone());
        }
    }
    map.retain(|_, v| v.len() > 1);
    map
}

fn overlap_shell(agents: &BTreeMap<String, AnalysisAgent>) -> OverlapMap {
    let mut map: OverlapMap = BTreeMap::new();
    for (name, src) in agents {
        if let Some(shell) = &src.agent.shell {
            for cmd in &shell.allowed_commands {
                map.entry(cmd.clone()).or_default().insert(name.clone());
            }
        }
    }
    map.retain(|_, v| v.len() > 1);
    map
}

fn overlap_resources(agents: &BTreeMap<String, AnalysisAgent>) -> OverlapMap {
    let mut map: OverlapMap = BTreeMap::new();
    for (name, src) in agents {
        for r in &src.agent.resources {
            map.entry(r.clone()).or_default().insert(name.clone());
        }
    }
    map.retain(|_, v| v.len() > 1);
    map
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct Overlap {
    allowed_tools: OverlapMap,
    mcp_servers: OverlapMap,
    shell_commands: OverlapMap,
    resources: OverlapMap,
}

#[derive(Facet)]
struct Summary {
    total_agents: usize,
    total_mcp_servers: usize,
    agents_with_shell_settings: usize,
    agents_with_knowledge: usize,
}

#[derive(Facet)]
struct Analysis {
    agents: BTreeMap<String, AnalysisAgent>,
    overlap: Overlap,
    summary: Summary,
}

fn build_analysis(agents: BTreeMap<String, AnalysisAgent>) -> Analysis {
    let mcp_servers: BTreeSet<String> = agents
        .values()
        .flat_map(|src| src.agent.other_tools.iter().cloned())
        .collect();

    let agents_with_shell = agents.values().filter(|s| s.agent.shell.is_some()).count();
    let agents_with_knowledge = agents
        .values()
        .filter(|s| !s.agent.knowledge.is_empty())
        .count();

    let overlap = Overlap {
        allowed_tools: overlap_allowed_tools(&agents),
        mcp_servers: overlap_mcp(&agents),
        shell_commands: overlap_shell(&agents),
        resources: overlap_resources(&agents),
    };

    let summary = Summary {
        total_agents: agents.len(),
        total_mcp_servers: mcp_servers.len(),
        agents_with_shell_settings: agents_with_shell,
        agents_with_knowledge,
    };

    let analysis_agents = agents;

    Analysis {
        agents: analysis_agents,
        overlap,
        summary,
    }
}

fn open_in_editor(path: &Path) -> crate::Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "less".to_string());

    let status = Command::new(&editor)
        .arg(path)
        .status()
        .wrap_err_with(|| format!("Failed to open {} with {}", path.display(), editor))?;

    if !status.success() {
        return Err(crate::format_err!("Editor exited with non-zero status"));
    }

    Ok(())
}

async fn install_skill(fs: &Fs, skill_path: &Path, home_dir: &Path) -> crate::Result<()> {
    if !fs.exists(skill_path) {
        return Err(crate::format_err!(
            "SKILL.md not found at {}",
            skill_path.display()
        ));
    }

    println!("Opening {} for review...", skill_path.display());
    open_in_editor(skill_path)?;

    println!();
    if !prompt_confirm("Install this skill to ~/.kiro/skills/kg-helper/SKILL.md?") {
        println!("Installation cancelled.");
        return Ok(());
    }

    let skill_dir = home_dir.join(".kiro").join("skills").join(SKILL_NAME);
    let skill_md_path = skill_dir.join("SKILL.md");

    fs.create_dir_all(&skill_dir)
        .await
        .wrap_err_with(|| format!("Failed to create {}", skill_dir.display()))?;

    let content = fs.read_to_string(skill_path).await?;
    fs.write(&skill_md_path, &content)
        .await
        .wrap_err_with(|| format!("Failed to write {}", skill_md_path.display()))?;

    println!("✓ Installed {}", skill_md_path.display());
    println!();
    println!("Done! Start kiro-cli and ask:");
    println!("  \"Help me set up kg for my project\"");

    Ok(())
}

pub async fn execute(fs: &Fs, home_dir: &Path, install_path: Option<PathBuf>) -> crate::Result<()> {
    if let Some(path) = install_path {
        return install_skill(fs, &path, home_dir).await;
    }
    let global_agents_dir = home_dir.join(".kiro").join("agents");
    let global_files = find_agent_jsons(fs, &global_agents_dir).await?;
    let total = global_files.len();

    if total > 0 {
        println!("Found {total} agent JSON file(s):");
        for (name, path) in &global_files {
            println!("  [global] {} ({})", name, path.display());
        }
    } else {
        println!("No existing ~/.kiro/agents/*.json files found.");
        println!("That's fine -- the skill will help you start from scratch.");
    }

    let mut agents = BTreeMap::new();
    for (name, path) in &global_files {
        let content = fs
            .read_to_string(path)
            .await
            .wrap_err_with(|| format!("Failed to read {}", path.display()))?;
        let agent = parse_agent(&content, path, "global")?;
        agents.insert(name.clone(), agent);
    }

    let skill_dir = home_dir.join(".kiro").join("skills").join(SKILL_NAME);
    let refs_dir = skill_dir.join("references");
    let analysis_path = refs_dir.join("analysis.json");
    let temp_skill_path = std::env::temp_dir().join("kg-helper-SKILL.md");

    println!();
    println!("This will create:");
    if !agents.is_empty() {
        println!("  {}", analysis_path.display());
    }
    println!("  {} (for review)", temp_skill_path.display());

    if !prompt_confirm("\nProceed?") {
        println!("Aborted.");
        return Ok(());
    }

    if !agents.is_empty() {
        fs.create_dir_all(&refs_dir)
            .await
            .wrap_err_with(|| format!("Failed to create {}", refs_dir.display()))?;

        let analysis = build_analysis(agents);
        let json = facet_json::to_string(&analysis).wrap_err("Failed to serialize analysis")?;
        fs.write(&analysis_path, &json)
            .await
            .wrap_err_with(|| format!("Failed to write {}", analysis_path.display()))?;
        println!("✓ Created {}", analysis_path.display());
    }

    fs.write(&temp_skill_path, SKILL_MD)
        .await
        .wrap_err_with(|| format!("Failed to write {}", temp_skill_path.display()))?;
    println!("✓ Created {}", temp_skill_path.display());

    println!();
    println!("Next steps:");
    println!("  1. Review the SKILL.md file (ask another agent to check it if you want)");
    println!(
        "  2. Run: kg bootstrap --install {}",
        temp_skill_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_agent_jsons_empty() -> crate::Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from("/home/testuser");
        let files = find_agent_jsons(&fs, &home.join(".kiro").join("agents")).await?;
        assert!(files.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent() -> crate::Result<()> {
        let json = r#"{
            "name": "rust-dev",
            "description": "Rust development agent",
            "tools": ["*"],
            "allowedTools": ["read", "grep", "@cargo"],
            "mcpServers": {
                "cargo": { "command": "cargo-mcp", "args": [] }
            },
            "toolsSettings": {
                "shell": {
                    "allowedCommands": ["cargo .*", "git status"],
                    "deniedCommands": ["cargo publish .*"]
                }
            },
            "resources": ["file://README.md"]
        }"#;

        let agent = parse_agent(json, Path::new("/test/rust-dev.json"), "global")?;
        assert_eq!(agent.agent.name, "rust-dev");
        assert_eq!(agent.source, "global");
        assert_eq!(agent.agent.allowed_tools.len(), 3);
        assert!(agent.agent.shell.is_some());
        let shell = agent.agent.shell.unwrap();
        assert_eq!(shell.allowed_commands.len(), 2);
        assert_eq!(shell.denied_commands.len(), 1);
        assert_eq!(agent.agent.resources, vec!["file://README.md"]);
        Ok(())
    }

    #[tokio::test]
    async fn test_overlap_detection() -> crate::Result<()> {
        let a = parse_agent(
            r#"{"name":"agent-a","allowedTools":["read","grep","glob"],"mcpServers":{"context7":{"command":"c7"}},"toolsSettings":{"shell":{"allowedCommands":["git status"]}},"resources":["file://README.md"]}"#,
            Path::new("/a.json"),
            "global",
        )?;
        let b = parse_agent(
            r#"{"name":"agent-b","allowedTools":["read","grep","write"],"mcpServers":{"context7":{"command":"c7"},"github":{"command":"gh"}},"toolsSettings":{"shell":{"allowedCommands":["git status","cargo .*"]}},"resources":["file://README.md","file://AGENTS.md"]}"#,
            Path::new("/b.json"),
            "global",
        )?;

        let mut agents = BTreeMap::new();
        agents.insert("agent-a".into(), a);
        agents.insert("agent-b".into(), b);

        let tools = overlap_allowed_tools(&agents);
        assert!(tools.contains_key("grep"));
        assert!(tools.contains_key("read"));
        assert!(!tools.contains_key("glob"));

        let shell = overlap_shell(&agents);
        assert!(shell.contains_key("git status"));
        assert!(!shell.contains_key("cargo .*"));

        let resources = overlap_resources(&agents);
        assert!(resources.contains_key("file://README.md"));

        Ok(())
    }

    #[tokio::test]
    async fn test_analysis_summary() -> crate::Result<()> {
        let a = parse_agent(
            r#"{"name":"a","mcpServers":{"git":{"command":"git-mcp"}},"toolsSettings":{"shell":{"allowedCommands":["git status"]}},"resources":["file://docs"]}"#,
            Path::new("/a.json"),
            "global",
        )?;
        let b = parse_agent(
            r#"{"name":"b","mcpServers":{"git":{"command":"git-mcp"},"cargo":{"command":"cargo-mcp"}}}"#,
            Path::new("/b.json"),
            "global",
        )?;

        let mut agents = BTreeMap::new();
        agents.insert("a".into(), a);
        agents.insert("b".into(), b);

        let analysis = build_analysis(agents);
        assert_eq!(analysis.summary.total_agents, 2);
        assert_eq!(analysis.summary.agents_with_shell_settings, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_skill_md_embedded() -> crate::Result<()> {
        assert!(SKILL_MD.contains("kg-helper"));
        assert!(SKILL_MD.contains("Bootstrap"));
        Ok(())
    }
}
