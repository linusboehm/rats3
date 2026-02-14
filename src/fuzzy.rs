use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

/// Fuzzy matcher for filtering entries
pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Match entries against a query
    /// Returns indices of matching entries, sorted by score (best first)
    pub fn match_entries(&mut self, entries: &[String], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..entries.len()).collect();
        }

        let pattern = Pattern::parse(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
        );

        let mut results: Vec<(usize, u32)> = Vec::new();
        let mut buf = Vec::new();

        for (idx, entry) in entries.iter().enumerate() {
            let haystack = Utf32Str::new(entry, &mut buf);
            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                results.push((idx, score));
            }
            buf.clear();
        }

        // Sort by score (higher is better)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(idx, _)| idx).collect()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let results = matcher.match_entries(&entries, "");
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn test_exact_match() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec!["foo".to_string(), "bar".to_string(), "foobar".to_string()];
        let results = matcher.match_entries(&entries, "foo");
        assert!(results.contains(&0));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_fuzzy_match() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec![
            "main.rs".to_string(),
            "main_test.rs".to_string(),
            "mod.rs".to_string(),
        ];
        let results = matcher.match_entries(&entries, "mnrs");
        assert!(results.contains(&0));
    }
}
