use {facet::Facet, std::collections::HashSet};

#[derive(Facet, Clone, Debug, Default, PartialEq, Eq)]
#[facet(default, deny_unknown_fields)]
pub struct SubagentConfig {
    #[facet(default)]
    pub allow: HashSet<String>,
    #[facet(default)]
    pub deny: HashSet<String>,
}

impl SubagentConfig {
    pub fn merge(mut self, other: Self) -> Self {
        if !other.allow.is_empty() {
            tracing::trace!(count = other.allow.len(), "merging subagent allow");
            self.allow.extend(other.allow);
        }
        if !other.deny.is_empty() {
            tracing::trace!(count = other.deny.len(), "merging subagent deny");
            self.deny.extend(other.deny);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let a = SubagentConfig::default();
        let b = SubagentConfig::default();
        let merged = a.merge(b);
        assert!(merged.allow.is_empty());
        assert!(merged.deny.is_empty());
    }

    #[test]
    fn test_merge_allow() {
        let mut a = SubagentConfig::default();
        a.allow.insert("frontend".to_string());
        let mut b = SubagentConfig::default();
        b.allow.insert("backend".to_string());
        let merged = a.merge(b);
        assert_eq!(merged.allow.len(), 2);
        assert!(merged.allow.contains("frontend"));
        assert!(merged.allow.contains("backend"));
    }

    #[test]
    fn test_merge_deny() {
        let mut a = SubagentConfig::default();
        a.deny.insert("admin".to_string());
        let mut b = SubagentConfig::default();
        b.deny.insert("root".to_string());
        let merged = a.merge(b);
        assert_eq!(merged.deny.len(), 2);
        assert!(merged.deny.contains("admin"));
        assert!(merged.deny.contains("root"));
    }

    #[test]
    fn test_merge_both() {
        let mut a = SubagentConfig::default();
        a.allow.insert("frontend".to_string());
        a.deny.insert("admin".to_string());
        let mut b = SubagentConfig::default();
        b.allow.insert("backend".to_string());
        b.deny.insert("root".to_string());
        let merged = a.merge(b);
        assert_eq!(merged.allow.len(), 2);
        assert_eq!(merged.deny.len(), 2);
        assert!(merged.allow.contains("frontend"));
        assert!(merged.allow.contains("backend"));
        assert!(merged.deny.contains("admin"));
        assert!(merged.deny.contains("root"));
    }
}
