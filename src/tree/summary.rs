use {
    crate::{AgentSourceSlots, generator::Generator},
    facet::Facet,
    std::{
        collections::{BTreeMap, BTreeSet},
        fmt::{Display, Write},
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{Result, tree::fixture_generator},
    };

    #[tokio::test]
    #[test_log::test]
    async fn summary_display_debug() -> Result<()> {
        let generator = fixture_generator()?;
        let agents = summarize_concrete(&generator);
        assert_eq!(1, agents.len());
        let agent = agents.get("child");
        assert!(agent.is_some());
        let agent = agent.unwrap();

        let dis = format!("{agent}");
        assert!(dis.contains("description="));
        assert_eq!(1, agent.inherits.len());
        assert_eq!("parent", agent.inherits_join()?);

        Ok(())
    }
}
