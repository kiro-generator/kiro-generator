mod dependents;
mod details;
mod search;
mod summary;
pub use {dependents::*, details::*, search::*, summary::*};

#[cfg(test)]
use crate::{Fs, Generator, GeneratorConfig, KgAgentSource, toml_parse};
#[cfg(test)]
pub fn fixture_generator() -> crate::Result<Generator> {
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
                    path: Some(KgAgentSource::LocalManifest(std::path::PathBuf::from(
                        "test",
                    ))),
                    manifest: v.clone(),
                },
                global_agent_file: Default::default(),
                local_agent_file: Default::default(),
            })
        })
        .collect();

    Ok(generator)
}
