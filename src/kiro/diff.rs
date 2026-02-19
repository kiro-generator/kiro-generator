use {
    super::{CustomToolConfig, KiroAgent, Knowledge, tools::*},
    facet::Facet,
    std::collections::HashSet,
};

#[derive(Facet, Debug, Clone, Default)]
pub struct NormalizedMcpServer {
    pub name: String,
    #[facet(default, skip_serializing_if = String::is_empty)]
    pub command: String,
    #[facet(default, skip_serializing_if = String::is_empty)]
    pub url: String,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub args: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub env: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub headers: Vec<String>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub timeout: Option<u64>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub disabled: Option<bool>,
}

impl NormalizedMcpServer {
    fn from_entry(name: String, config: CustomToolConfig) -> Self {
        let mut env: Vec<_> = config
            .env
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        env.sort();
        let mut headers: Vec<_> = config
            .headers
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        headers.sort();
        Self {
            name,
            command: config.command,
            url: config.url,
            args: config.args,
            env,
            headers,
            timeout: config.timeout,
            disabled: config.disabled,
        }
    }
}

#[derive(Facet, Debug, Clone, Default)]
pub struct NormalizedToolAlias {
    pub original: String,
    pub alias: String,
}

#[derive(Facet, Debug, Clone, Default)]
#[facet(default, skip_all_unless_truthy)]
pub struct NormalizedHook {
    pub trigger: String,
    pub command: String,
    pub matcher: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_output_size: Option<u32>,
    pub cache_ttl_seconds: Option<u64>,
}

/// A normalized representation of an Agent optimized for stable, deterministic
/// diffing.
///
/// This struct addresses the instability issues in `Agent` by:
/// - Using concrete types instead of `facet_value::Value` for native tools
/// - Sorting all collections (resources, knowledge) for consistent ordering
/// - Separating known native tools from custom MCP tools
///
/// # Stability Guarantees
/// - `resources` and `knowledge` are sorted, ensuring identical agents produce
///   identical diffs
/// - Native tools (shell, aws, read, write) have concrete types for precise
///   field-level diffs
/// - `other_tools` only tracks presence/absence of custom MCP tools (not their
///   detailed settings)
///
/// # Trade-offs
/// - Custom MCP tool settings changes are not detected (only
///   additions/removals)
/// - This is acceptable as custom tools are rare (~1% use case)
#[derive(Facet, Debug, Clone, Default)]
#[facet(default, skip_all_unless_truthy)]
pub struct NormalizedAgent {
    pub name: String,
    pub description: Option<String>,
    pub prompt: Option<String>,
    pub tools: Vec<String>,
    pub allowed_tools: Vec<String>,
    pub resources: Vec<String>,
    pub knowledge: Vec<Knowledge>,
    pub shell: Option<NormalizedExecuteShellTool>,
    pub aws: Option<NormalizedAwsTool>,
    pub read: Option<NormalizedReadTool>,
    pub write: Option<NormalizedWriteTool>,
    pub subagent: Option<NormalizedSubagentTool>,
    pub other_tools: Vec<String>,
    pub model: Option<String>,
    pub mcp_servers: Vec<NormalizedMcpServer>,
    pub tool_aliases: Vec<NormalizedToolAlias>,
    pub hooks: Vec<NormalizedHook>,
    pub include_mcp_json: bool,
}

impl KiroAgent {
    pub fn normalize(self) -> NormalizedAgent {
        let mut shell = None;
        let mut aws = None;
        let mut read = None;
        let mut write = None;
        let mut subagent = None;
        let mut other_tools = Vec::new();

        for (tool_name, value) in self.tools_settings {
            let json = facet_json::to_string(&value).unwrap_or_default();
            match tool_name.as_str() {
                "shell" => {
                    if let Ok(tool) = facet_json::from_str::<ExecuteShellTool>(&json) {
                        shell = Some(tool.into());
                    }
                }
                "aws" => {
                    if let Ok(tool) = facet_json::from_str::<AwsTool>(&json) {
                        aws = Some(tool.into());
                    }
                }
                "read" => {
                    if let Ok(tool) = facet_json::from_str::<ReadTool>(&json) {
                        read = Some(tool.into());
                    }
                }
                "write" => {
                    if let Ok(tool) = facet_json::from_str::<WriteTool>(&json) {
                        write = Some(tool.into());
                    }
                }
                "subagent" => {
                    if let Ok(tool) = facet_json::from_str::<SubagentTool>(&json) {
                        subagent = Some(tool.into());
                    }
                }
                _ => {
                    other_tools.push(tool_name);
                }
            }
        }

        other_tools.sort();
        let mut resources = HashSet::new();
        let mut knowledge: Vec<Knowledge> = Vec::new();

        for resource in self.resources {
            if let Some(s) = resource.as_string() {
                resources.insert(s.to_string());
            } else {
                // Try to parse as Knowledge object
                let json = facet_json::to_string(&resource).unwrap_or_default();
                match facet_json::from_str::<Knowledge>(&json) {
                    Ok(k) => {
                        knowledge.push(k);
                    }
                    Err(e) => tracing::warn!("unable to decode knowledge '{json}'\n{e}"),
                };
            }
        }

        let mut resources: Vec<_> = resources.into_iter().collect();
        resources.sort();

        let mut tools: Vec<_> = self.tools.into_iter().collect();
        tools.sort();

        let mut allowed: Vec<_> = self.allowed_tools.into_iter().collect();
        allowed.sort();

        knowledge.sort_by(|a, b| a.name.cmp(&b.name));

        let mut mcp_servers: Vec<_> = self
            .mcp_servers
            .into_iter()
            .map(|(name, config)| NormalizedMcpServer::from_entry(name, config))
            .collect();
        mcp_servers.sort_by(|a, b| a.name.cmp(&b.name));

        let mut tool_aliases: Vec<_> = self
            .tool_aliases
            .into_iter()
            .map(|(original, alias)| NormalizedToolAlias { original, alias })
            .collect();
        tool_aliases.sort_by(|a, b| a.original.cmp(&b.original));

        let mut hooks: Vec<_> = self
            .hooks
            .into_iter()
            .flat_map(|(trigger, entries)| {
                entries.into_iter().map(move |h| NormalizedHook {
                    trigger: trigger.clone(),
                    command: h.command,
                    matcher: h.matcher,
                    timeout_ms: h.timeout_ms,
                    max_output_size: h.max_output_size,
                    cache_ttl_seconds: h.cache_ttl_seconds,
                })
            })
            .collect();
        hooks.sort_by(|a, b| a.trigger.cmp(&b.trigger).then(a.command.cmp(&b.command)));

        NormalizedAgent {
            name: self.name,
            description: self.description,
            prompt: self.prompt,
            tools,
            allowed_tools: allowed,
            resources,
            knowledge,
            shell,
            aws,
            read,
            write,
            subagent,
            other_tools,
            model: self.model,
            mcp_servers,
            tool_aliases,
            hooks,
            include_mcp_json: self.include_mcp_json,
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, rediff::FacetDiff};

    #[test]
    fn test_default_agent() -> crate::Result<()> {
        let agent = KiroAgent {
            name: "test".to_string(),
            ..Default::default()
        };
        assert_eq!("test", format!("{agent}"));

        let kg_agent = crate::Manifest::default();
        let agent = KiroAgent::try_from(&kg_agent)?;
        assert_eq!(agent.tools, KiroAgent::default().tools);

        Ok(())
    }

    #[test]
    fn test_normalized_agent_diff_identical() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            description: Some("Test agent".to_string()),
            prompt: Some("You are a test agent".to_string()),
            ..Default::default()
        };
        let agent2 = agent1.clone();

        let diff = agent1.diff(&agent2);
        assert!(diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_prompt_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            prompt: Some("Original prompt".to_string()),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            prompt: Some("Changed prompt".to_string()),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_stability() {
        // Create agents with resources in different order
        let mut agent1 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://b.md".to_string(), "file://a.md".to_string()],
            ..Default::default()
        };
        let mut agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string(), "file://b.md".to_string()],
            ..Default::default()
        };

        // Sort both to normalize
        agent1.resources.sort();
        agent2.resources.sort();

        // After sorting, should be equal
        let diff = agent1.diff(&agent2);
        assert!(diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_resources_added() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string()],
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string(), "file://b.md".to_string()],
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_knowledge_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            knowledge: vec![Knowledge {
                name: "kb1".to_string(),
                knowledge_type: "best".to_string(),
                source: Some("file://docs".to_string()),
                description: Some("Original".to_string()),
                index_type: None,
                auto_update: None,
            }],
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            knowledge: vec![Knowledge {
                name: "kb1".to_string(),
                knowledge_type: "best".to_string(),
                source: Some("file://docs".to_string()),
                description: Some("Changed".to_string()),
                index_type: None,
                auto_update: None,
            }],
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_shell_tool_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            shell: Some(NormalizedExecuteShellTool {
                allowed_commands: vec!["git status".to_string()],
                denied_commands: vec![],
                deny_by_default: false,
                auto_allow_readonly: Some(true),
            }),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            shell: Some(NormalizedExecuteShellTool {
                allowed_commands: vec!["git fetch".to_string(), "git status".to_string()],
                denied_commands: vec![],
                deny_by_default: false,
                auto_allow_readonly: Some(true),
            }),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_allowed_tools_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            allowed_tools: vec!["read".to_string()],
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            allowed_tools: vec!["read".to_string(), "write".to_string()],
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_empty_to_populated() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string()],
            shell: Some(NormalizedExecuteShellTool {
                allowed_commands: vec![],
                denied_commands: vec![],
                deny_by_default: false,
                auto_allow_readonly: Some(true),
            }),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalize_malformed_knowledge() {
        use facet_value::Value;

        let agent = KiroAgent {
            name: "test".to_string(),
            resources: vec![
                Value::from("file://valid.md"),
                Value::from(42), // Not a string or valid Knowledge
            ],
            ..Default::default()
        };

        let normalized = agent.normalize();
        assert_eq!(normalized.resources.len(), 1);
        assert_eq!(normalized.resources[0], "file://valid.md");
        assert!(normalized.knowledge.is_empty());
    }
}
