use {
    crate::{
        kiro::{KiroAgent, diff::NormalizedAgent},
        os::Fs,
    },
    color_eyre::eyre::WrapErr,
    facet::Facet,
    std::{
        collections::{BTreeMap, BTreeSet},
        path::{Path, PathBuf},
        process::Command,
    },
};

const SKILL_MD: &str = include_str!("../../resources/kg-helper/SKILL.md");
const BOOTSTRAP_MD: &str = include_str!("../../resources/kg-helper/references/bootstrap.md");
const SCHEMAS_MD: &str = include_str!("../../resources/kg-helper/references/schemas.md");
const TEMPLATES_MD: &str = include_str!("../../resources/kg-helper/references/templates.md");
const KIRO_AGENT_SCHEMA: &str = include_str!("../../schemas/kiro-agent.json");
const KG_MANIFEST_SCHEMA: &str = include_str!("../../schemas/manifest.json");
const KG_AGENT_SCHEMA: &str = include_str!("../../schemas/agent.json");
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

#[allow(dead_code)]
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

async fn install_skill(fs: &Fs, home_dir: &Path) -> crate::Result<()> {
    let skill_dir = home_dir.join(".kiro").join("skills").join(SKILL_NAME);
    let ref_dir = home_dir
        .join(".kiro")
        .join("skills")
        .join(SKILL_NAME)
        .join("references");
    let assets_dir = home_dir
        .join(".kiro")
        .join("skills")
        .join(SKILL_NAME)
        .join("assets");

    for p in [&skill_dir, &ref_dir, &assets_dir] {
        fs.create_dir_all(p)
            .await
            .wrap_err_with(|| format!("Failed to create {}", p.display()))?;
    }

    let skill_path = skill_dir.join("SKILL.md");
    fs.write(&skill_path, SKILL_MD)
        .await
        .wrap_err_with(|| format!("Failed to write {}", skill_path.display()))?;
    println!("✓ Installed {}", skill_path.display());

    // Install reference files
    let bootstrap_path = ref_dir.join("bootstrap.md");
    fs.write(&bootstrap_path, BOOTSTRAP_MD)
        .await
        .wrap_err_with(|| format!("Failed to write {}", bootstrap_path.display()))?;

    let schemas_path = ref_dir.join("schemas.md");
    fs.write(&schemas_path, SCHEMAS_MD)
        .await
        .wrap_err_with(|| format!("Failed to write {}", schemas_path.display()))?;

    let templates_path = ref_dir.join("templates.md");
    fs.write(&templates_path, TEMPLATES_MD)
        .await
        .wrap_err_with(|| format!("Failed to write {}", templates_path.display()))?;

    // Install schema files
    let kiro_agent_schema_path = assets_dir.join("kiro-agent.json");
    fs.write(&kiro_agent_schema_path, KIRO_AGENT_SCHEMA)
        .await
        .wrap_err_with(|| format!("Failed to write {}", kiro_agent_schema_path.display()))?;

    let kg_manifest_schema_path = assets_dir.join("kg-manifest.json");
    fs.write(&kg_manifest_schema_path, KG_MANIFEST_SCHEMA)
        .await
        .wrap_err_with(|| format!("Failed to write {}", kg_manifest_schema_path.display()))?;

    let kg_agent_schema_path = assets_dir.join("kg-agent.json");
    fs.write(&kg_agent_schema_path, KG_AGENT_SCHEMA)
        .await
        .wrap_err_with(|| format!("Failed to write {}", kg_agent_schema_path.display()))?;

    println!();
    println!("Done! Start kiro-cli and ask:");
    println!("  \"Help me set up kg for my project\"");

    Ok(())
}

pub async fn execute(fs: &Fs, home_dir: &Path) -> crate::Result<()> {
    let skill_dir = home_dir.join(".kiro").join("skills").join(SKILL_NAME);
    let refs_dir = skill_dir.join("references");

    // Nuke and recreate
    if fs.exists(&skill_dir) {
        fs.remove_dir_all(&skill_dir)
            .await
            .wrap_err_with(|| format!("Failed to remove {}", skill_dir.display()))?;
    }
    install_skill(fs, home_dir).await?;

    // Scan agents
    let global_agents_dir = home_dir.join(".kiro").join("agents");
    let global_files = find_agent_jsons(fs, &global_agents_dir).await?;

    if !global_files.is_empty() {
        println!("Found {} agent JSON file(s)", global_files.len());
        let mut agents = BTreeMap::new();
        for (name, path) in &global_files {
            let content = fs.read_to_string(path).await?;
            let agent = parse_agent(&content, path, "global")?;
            agents.insert(name.clone(), agent);
        }

        let analysis = build_analysis(agents);
        let json = facet_json::to_string(&analysis)?;
        let analysis_path = refs_dir.join("analysis.json");
        fs.write(&analysis_path, &json).await?;
        println!("✓ {}", analysis_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_agent_jsons() -> crate::Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from(crate::os::ACTIVE_USER_HOME);
        let files = find_agent_jsons(&fs, &home.join(".kiro").join("agents")).await?;
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].0, "default");
        assert_eq!(files[1].0, "rust");
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
        assert!(SKILL_MD.contains("bootstrap"));
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_bootstrap_force_e2e() -> crate::Result<()> {
        let fs = Fs::new();
        let home = PathBuf::from(crate::os::ACTIVE_USER_HOME);

        execute(&fs, &home).await?;

        let skill_dir = home.join(".kiro").join("skills").join(SKILL_NAME);
        let ref_dir = skill_dir.join("references");

        // Verify SKILL.md
        let skill_path = skill_dir.join("SKILL.md");
        assert!(fs.exists(&skill_path), "SKILL.md not created");

        // Verify reference files
        let bootstrap_path = ref_dir.join("bootstrap.md");
        assert!(fs.exists(&bootstrap_path), "bootstrap.md not created");

        let schemas_path = ref_dir.join("schemas.md");
        assert!(fs.exists(&schemas_path), "schemas.md not created");

        let templates_path = ref_dir.join("templates.md");
        assert!(fs.exists(&templates_path), "templates.md not created");

        // Verify analysis.json
        let analysis_path = ref_dir.join("analysis.json");
        assert!(fs.exists(&analysis_path), "analysis.json not created");

        let content = fs.read_to_string(&analysis_path).await?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        assert_eq!(value["summary"]["total_agents"], 2);
        assert!(value["agents"]["default"].is_object());
        assert!(value["agents"]["rust"].is_object());

        // Verify schema files
        let assets_dir = skill_dir.join("assets");
        let kiro_agent_schema = assets_dir.join("kiro-agent.json");
        assert!(fs.exists(&kiro_agent_schema), "kiro-agent.json not created");

        let kg_manifest_schema = assets_dir.join("kg-manifest.json");
        assert!(
            fs.exists(&kg_manifest_schema),
            "kg-manifest.json not created"
        );

        let kg_agent_schema = assets_dir.join("kg-agent.json");
        assert!(fs.exists(&kg_agent_schema), "kg-agent.json not created");

        Ok(())
    }
}
