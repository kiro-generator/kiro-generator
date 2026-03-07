use {
    crate::{AgentSourceSlots, Result, generator::Generator},
    facet::Facet,
    std::{
        collections::{BTreeMap, BTreeSet},
        fmt::{Display, Write},
        path::PathBuf,
    },
};

#[derive(Clone, Debug, Facet, Default, PartialEq, Eq)]
pub struct SummaryEntry {
    #[facet(skip)]
    pub name: String,
    pub description: String,
    pub inherits: BTreeSet<String>,
    pub locations: Vec<String>,
}

impl SummaryEntry {
    pub fn inherits_join(&self) -> crate::Result<String> {
        if self.inherits.is_empty() {
            return Ok(String::new());
        }
        let mut join = String::with_capacity(self.inherits.len() * 6);
        for i in &self.inherits {
            write!(&mut join, "{i},")?;
        }

        join.remove(join.len() - 1);
        Ok(join)
    }
}

impl Display for SummaryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "description={} inherits={}",
            self.description,
            self.inherits.len()
        )
    }
}

impl From<&AgentSourceSlots> for SummaryEntry {
    fn from(slots: &AgentSourceSlots) -> Self {
        Self {
            name: slots.name.clone(),
            description: slots.merged.description.clone().unwrap_or_default(),
            inherits: slots.merged.inherits.iter().cloned().collect(),
            locations: slots.locations().iter().map(|p| p.to_string()).collect(),
        }
    }
}

#[derive(Clone, Debug, Facet, Default, PartialEq, Eq)]
pub struct SummaryReport {
    pub agents: BTreeMap<String, SummaryEntry>,
    pub templates: BTreeMap<String, SummaryEntry>,
}

impl Display for SummaryReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "agents={} templates={}",
            self.agents.len(),
            self.templates.len()
        )
    }
}

pub fn summarize_concrete(generator: &Generator) -> BTreeMap<String, SummaryEntry> {
    generator
        .concrete_agents()
        .into_iter()
        .map(|a| (a.name.clone(), a.into()))
        .collect()
}

pub fn summarize_templates(generator: &Generator) -> BTreeMap<String, SummaryEntry> {
    generator
        .template_agents()
        .into_iter()
        .map(|a| (a.name.clone(), a.into()))
        .collect()
}

pub fn summarize(generator: &Generator) -> SummaryReport {
    SummaryReport {
        agents: summarize_concrete(generator),
        templates: summarize_templates(generator),
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{GeneratorConfig, os::Fs, source::KgAgentSource, toml_parse},
        std::path::PathBuf,
    };

    fn fixture_generator() -> Result<Generator> {
        let raw = include_str!("../../fixtures/manifest-test/test-merge-agent.toml");
        let fs = Fs::new();
        let mut generator = Generator::new(
            fs,
            crate::ConfigLocation::Local,
            crate::output::OutputFormat::Json,
        )?;
        let agents: GeneratorConfig = toml_parse(raw)?;
        let agents = agents.populate_names();
        generator.agents = agents
            .agents
            .iter()
            .map(|(k, v)| {
                (k.clone(), crate::AgentSourceSlots {
                    name: k.clone(),
                    merged: v.clone(),
                    global_manifest: Default::default(),
                    local_manifest: crate::SourceSlot {
                        path: Some(KgAgentSource::LocalManifest(PathBuf::from("test"))),
                        manifest: v.clone(),
                    },
                    global_agent_file: Default::default(),
                    local_agent_file: Default::default(),
                })
            })
            .collect();

        Ok(generator)
    }

    #[tokio::test]
    #[test_log::test]
    async fn summary_groups_agents_and_templates_deterministically() -> Result<()> {
        let generator = fixture_generator()?;
        let report = summarize(&generator);

        assert!(!report.agents.is_empty());
        assert!(!report.templates.is_empty());

        let concrete_names: Vec<_> = report.agents.keys().cloned().collect();
        let mut concrete_sorted = concrete_names.clone();
        concrete_sorted.sort();
        assert_eq!(concrete_names, concrete_sorted);

        let template_names: Vec<_> = report.templates.keys().cloned().collect();
        let mut template_sorted = template_names.clone();
        template_sorted.sort();
        assert_eq!(template_names, template_sorted);

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    async fn split_summaries_match_combined_summary() -> Result<()> {
        let generator = fixture_generator()?;
        let concrete = summarize_concrete(&generator);
        let templates = summarize_templates(&generator);
        let report = summarize(&generator);

        assert_eq!(report.agents, concrete);
        assert_eq!(report.templates, templates);

        Ok(())
    }
}
