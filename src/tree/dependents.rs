use {
    crate::{Generator, Result},
    facet::Facet,
    std::collections::{BTreeMap, BTreeSet},
};

#[derive(Facet)]
struct TreeDependent {
    agent: String,
    chain: Vec<String>,
}

#[derive(Facet)]
struct TreeInvert {
    dependents: Vec<TreeDependent>,
}

fn build_reverse_map(generator: &Generator) -> BTreeMap<String, BTreeSet<String>> {
    let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, slots) in &generator.agents {
        for parent in &slots.merged.inherits {
            reverse
                .entry(parent.clone())
                .or_default()
                .insert(name.clone());
        }
    }
    reverse
}

pub fn dependencies(
    generator: &Generator,
    names: &[String],
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut result: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, agent) in &generator.agents {
        result.insert(
            name.clone(),
            BTreeSet::from_iter(generator.inheritance_chain(name)?),
        );
    }
    Ok(result)
}
