use {
    crate::{
        Manifest,
        SourceSlot,
        generator::Generator,
        kg_config::{SearchQuery, Searchable},
        tree::SummaryEntry,
    },
    facet::Facet,
    std::collections::{BTreeMap, BTreeSet},
};

#[derive(Facet)]
pub struct SearchHit {
    fields: BTreeSet<String>,
    summary: SummaryEntry,
}

#[derive(Facet)]
pub struct SearchReport {
    pub pattern: String,
    pub field: Option<String>,
    pub case_sensitive: bool,
    pub results: BTreeMap<String, SearchHit>,
}

pub fn search(
    generator: &Generator,
    pattern: &str,
    field: Option<&str>,
    case_sensitive: bool,
) -> SearchReport {
    let query: SearchQuery<'_> = pattern.into();
    let query = if case_sensitive {
        query.case_sensitive()
    } else {
        query
    };
    let mut results: BTreeMap<String, SearchHit> = BTreeMap::new();

    for (agent, agent_source_slots) in generator.agents.iter() {
        let fields: BTreeSet<String> = agent_source_slots
            .source_slots()
            .iter()
            .flat_map(|slot| search_slot(slot, field, &query))
            .collect();
        if !fields.is_empty() {
            results.insert(agent.clone(), SearchHit {
                fields,
                summary: SummaryEntry::from(agent_source_slots),
            });
        }
    }

    SearchReport {
        pattern: pattern.to_string(),
        field: field.map(str::to_string),
        case_sensitive,
        results,
    }
}

fn search_slot(
    slot: &SourceSlot,
    field_filter: Option<&str>,
    query: &SearchQuery<'_>,
) -> Vec<String> {
    matched_fields(&slot.manifest, query)
        .into_iter()
        .filter(|path| field_filter.is_none_or(|filter| matches_field_filter(path, filter)))
        .collect()
}

fn matched_fields(manifest: &Manifest, query: &SearchQuery<'_>) -> Vec<String> {
    let mut matches = Vec::new();

    matches.extend(named_matches("resources", manifest.resources.iter(), query));
    matches.extend(named_matches("skills", manifest.skills.iter(), query));
    matches.extend(named_matches(
        "mcpServers",
        manifest.mcp_servers.iter(),
        query,
    ));

    if manifest.native_tools.shell.search(query) {
        matches.push(String::from("nativeTools.shell"));
    }
    if manifest.native_tools.aws.search(query) {
        matches.push(String::from("nativeTools.aws"));
    }
    if manifest.native_tools.read.search(query) {
        matches.push(String::from("nativeTools.read"));
    }
    if manifest.native_tools.write.search(query) {
        matches.push(String::from("nativeTools.write"));
    }
    if manifest.native_tools.glob.search(query) {
        matches.push(String::from("nativeTools.glob"));
    }
    if manifest.native_tools.grep.search(query) {
        matches.push(String::from("nativeTools.grep"));
    }
    if manifest.native_tools.web_fetch.search(query) {
        matches.push(String::from("nativeTools.web_fetch"));
    }

    matches
}

fn named_matches<'a, T>(
    prefix: &str,
    items: impl Iterator<Item = (&'a String, &'a T)>,
    query: &SearchQuery<'_>,
) -> Vec<String>
where
    T: Searchable + 'a,
{
    items
        .filter_map(|(name, value)| {
            if query.matches(name) || value.search(query) {
                Some(format!("{prefix}.{name}"))
            } else {
                None
            }
        })
        .collect()
}

fn matches_field_filter(path: &str, filter: &str) -> bool {
    path == filter
        || path
            .strip_prefix(filter)
            .is_some_and(|remainder| remainder.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{KgAgentSource, KgCustomToolConfig, KgFileResource, KgSkillResource},
        std::{collections::BTreeSet, path::PathBuf},
    };

    #[tokio::test]
    #[test_log::test]
    async fn search_returns_direct_definition_sources_only() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = search(&generator, "job-taker-skill", None, false);

        assert_eq!(result.results.len(), 1);
        assert!(result.results.contains_key("parent"));
        assert_eq!(
            result.results.get("parent").unwrap().summary.locations,
            vec![String::from("local-manifest://test")]
        );

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn search_honors_field_filter() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = search(&generator, "git push", Some("nativeTools.shell"), false);

        assert_eq!(result.results.len(), 2);
        assert!(
            result
                .results
                .values()
                .all(|entry| entry.summary.locations == vec![String::from("local-manifest://test")])
        );

        let filtered = search(&generator, "git push", Some("resources"), false);
        assert!(filtered.results.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn search_is_case_insensitive_by_default() -> crate::Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = search(&generator, "GIT PUSH", Some("nativeTools.shell"), false);
        assert_eq!(result.results.len(), 2);

        let case_sensitive = search(&generator, "GIT PUSH", Some("nativeTools.shell"), true);
        assert!(case_sensitive.results.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn search_dedupes_fields_across_source_slots() -> crate::Result<()> {
        let mut generator = super::super::fixture_generator()?;
        let parent = generator
            .agents
            .get_mut("parent")
            .expect("parent should exist");
        parent.global_manifest = SourceSlot {
            path: Some(KgAgentSource::GlobalManifest(PathBuf::from(
                "/tmp/parent.toml",
            ))),
            manifest: parent.local_manifest.manifest.clone(),
        };

        let result = search(&generator, "job-taker-skill", None, false);
        let parent_hit = result.results.get("parent").expect("parent should match");

        assert_eq!(
            parent_hit.fields,
            BTreeSet::from([String::from("skills.taker")])
        );

        Ok(())
    }

    #[test]
    fn matched_fields_returns_multiple_hits() {
        let mut manifest = Manifest::default();
        manifest
            .skills
            .insert(String::from("default"), KgSkillResource {
                locations: BTreeSet::from([String::from("default-skill.md")]),
                ..Default::default()
            });
        manifest
            .resources
            .insert(String::from("docs"), KgFileResource {
                locations: BTreeSet::from([String::from("default.md")]),
                ..Default::default()
            });
        manifest
            .mcp_servers
            .insert(String::from("default"), KgCustomToolConfig {
                command: String::from("default-mcp"),
                ..Default::default()
            });

        let matches = matched_fields(&manifest, &"default".into());
        assert_eq!(matches, vec![
            String::from("resources.docs"),
            String::from("skills.default"),
            String::from("mcpServers.default"),
        ]);
    }
}
