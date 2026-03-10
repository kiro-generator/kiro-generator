pub struct SearchQuery<'a> {
    needle: &'a str,
    needle_lower: String,
    case_sensitive: bool,
}

impl<'a> SearchQuery<'a> {
    pub fn new(needle: &'a str, case_sensitive: bool) -> Self {
        Self {
            needle,
            needle_lower: needle.to_lowercase(),
            case_sensitive,
        }
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    pub fn matches(&self, haystack: &str) -> bool {
        if self.case_sensitive {
            haystack.contains(self.needle)
        } else {
            haystack.to_lowercase().contains(&self.needle_lower)
        }
    }
}

impl<'a> From<&'a str> for SearchQuery<'a> {
    fn from(needle: &'a str) -> Self {
        Self::new(needle, false)
    }
}

pub trait Searchable {
    fn search(&self, query: &SearchQuery<'_>) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_matches_case_insensitively_by_default() {
        let query: SearchQuery<'_> = "skill".into();
        assert!(query.matches("SKILL.md"));
    }

    #[test]
    fn query_honors_case_sensitive_mode() {
        let query: SearchQuery<'_> = "skill".into();
        let query = query.case_sensitive();
        assert!(!query.matches("SKILL.md"));
        assert!(query.matches("skill.md"));
    }
}
