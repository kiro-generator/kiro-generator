use {
    crate::{Generator, Result},
    std::collections::{BTreeMap, BTreeSet},
};

pub fn dependencies(generator: &Generator) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut inverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for name in generator.agents.keys() {
        let resolved_chain = generator.inheritance_chain_safe(name);
        for parent in &resolved_chain {
            inverse
                .entry(parent.clone())
                .or_default()
                .insert(name.clone());
        }
    }

    Ok(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn child_appears_as_dependent_of_parent() -> Result<()> {
        let generator = super::super::fixture_generator()?;
        let result = dependencies(&generator)?;
        assert!(result.get("parent").unwrap().contains("child"));
        assert!(!result.contains_key("child"), "child has no dependents");
        Ok(())
    }
}
