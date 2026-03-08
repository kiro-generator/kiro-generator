use {
    crate::{AgentSourceSlots, Manifest, generator::Generator},
    facet::Facet,
    std::collections::{BTreeMap, BTreeSet},
};

#[derive(Facet)]
pub struct TreeSource {
    pub source_type: String,
    pub path: String,
    pub modified_fields: BTreeSet<String>,
}

impl PartialOrd for TreeSource {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TreeSource {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source_type.cmp(&other.source_type)
    }
}
impl PartialEq for TreeSource {
    fn eq(&self, other: &Self) -> bool {
        self.source_type == other.source_type && self.path == other.path
    }
}

impl Eq for TreeSource {}

#[derive(Facet)]
pub struct TreeDetail {
    pub template: bool,
    pub output: String,
    pub description: String,
    pub inherits: BTreeSet<String>,
    pub resolved_chain: BTreeSet<String>,
    pub sources: BTreeSet<TreeSource>,
}

fn collect_sources(slots: &AgentSourceSlots) -> BTreeSet<TreeSource> {
    [
        &slots.local_agent_file,
        &slots.local_manifest,
        &slots.global_manifest,
        &slots.global_agent_file,
    ]
    .into_iter()
    .filter_map(|slot| {
        let src = slot.path.as_ref()?;
        Some(TreeSource {
            source_type: src.source_type().to_string(),
            path: src.path().to_string_lossy().into_owned(),
            modified_fields: manifest_fields(&slot.manifest).into_iter().collect(),
        })
    })
    .collect()
}

pub fn details(generator: &Generator, names: &[String]) -> BTreeMap<String, TreeDetail> {
    let mut out: BTreeMap<String, TreeDetail> = BTreeMap::new();
    for name in names {
        if let Some(agent) = generator.agents.get(name) {
            let manifest = &agent.merged;
            let output = if manifest.template {
                String::new()
            } else {
                generator
                    .destination_dir(name)
                    .join(format!("{name}.json"))
                    .to_string_lossy()
                    .into_owned()
            };
            let resolved_chain = generator.inheritance_chain_safe(name);
            let sources = collect_sources(agent);

            out.insert(name.clone(), TreeDetail {
                template: manifest.template,
                output,
                description: manifest.description.clone().unwrap_or_default(),
                inherits: manifest.inherits.iter().cloned().collect(),
                resolved_chain,
                sources,
            });
        }
    }
    out
}

fn manifest_fields(manifest: &Manifest) -> Vec<String> {
    let mut fields = Vec::new();
    if manifest.template {
        fields.push("template".to_string());
    }
    if manifest.description.is_some() {
        fields.push("description".to_string());
    }
    if !manifest.inherits.is_empty() {
        fields.push("inherits".to_string());
    }
    if manifest.prompt.is_some() {
        fields.push("prompt".to_string());
    }
    if !manifest.resources.is_empty() {
        for k in manifest.resources.keys() {
            fields.push(format!("resources.{k}"));
        }
    }
    if !manifest.skills.is_empty() {
        for k in manifest.skills.keys() {
            fields.push(format!("skills.{k}"));
        }
    }
    if !manifest.knowledge.is_empty() {
        for k in manifest.knowledge.keys() {
            fields.push(format!("knowledge.{k}"));
        }
    }
    if manifest.include_mcp_json.is_some() {
        fields.push("useLegacyMcpJson".to_string());
    }
    if !manifest.tools.is_empty() {
        fields.push("tools".to_string());
    }
    if !manifest.allowed_tools.is_empty() {
        fields.push("allowedTools".to_string());
    }
    if manifest.model.is_some() {
        fields.push("model".to_string());
    }
    if !manifest.hooks.is_empty() {
        for k in manifest.hooks.keys() {
            fields.push(format!("hooks.{k}"));
        }
    }
    if !manifest.mcp_servers.is_empty() {
        for k in manifest.mcp_servers.keys() {
            fields.push(format!("mcpServers.{k}"));
        }
    }
    if !manifest.tool_aliases.is_empty() {
        for k in manifest.tool_aliases.keys() {
            fields.push(format!("toolAliases.{k}"));
        }
    }
    if manifest.native_tools != Default::default() {
        if manifest.native_tools.shell != Default::default() {
            fields.push("nativeTools.shell".to_string());
        }

        if manifest.native_tools.aws != Default::default() {
            fields.push("nativeTools.aws".to_string());
        }

        if manifest.native_tools.read != Default::default() {
            fields.push("nativeTools.read".to_string());
        }

        if manifest.native_tools.write != Default::default() {
            fields.push("nativeTools.write".to_string());
        }

        if manifest.native_tools.web_fetch != Default::default() {
            fields.push("nativeTools.web_fetch".to_string());
        }

        if manifest.native_tools.glob != Default::default() {
            fields.push("nativeTools.glob".to_string());
        }
        if manifest.native_tools.grep != Default::default() {
            fields.push("nativeTools.grep".to_string());
        }
    }
    if !manifest.tool_settings.is_empty() {
        for k in manifest.tool_settings.keys() {
            fields.push(format!("toolSettings.{k}"));
        }
    }
    if manifest.keyboard_shortcut.is_some() {
        fields.push("keyboardShortcut".to_string());
    }
    if manifest.welcome_message.is_some() {
        fields.push("welcomeMessage".to_string());
    }
    if manifest.subagents != Default::default() {
        fields.push("subagents".to_string());
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn details_returns_child_with_resolved_chain() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = details(&generator, &["child".into()]);
        let child = result.get("child").expect("child should exist");
        assert!(!child.template);
        assert!(child.resolved_chain.contains("parent"));
        assert_eq!(child.inherits, BTreeSet::from(["parent".into()]));
        assert!(!child.sources.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn details_skips_unknown_agent() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = details(&generator, &["nonexistent".into()]);
        assert!(result.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn details_template_has_empty_output() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = details(&generator, &["parent".into()]);
        let parent = result.get("parent").expect("parent should exist");
        assert!(parent.template);
        assert!(parent.output.is_empty());
        Ok(())
    }
}
