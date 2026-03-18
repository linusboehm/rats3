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

    /// Match entries against a query.
    /// Returns (entry_index, matched_char_positions) pairs sorted by score (best first).
    /// Positions are char indices into the original entry string.
    /// When query is empty, positions are empty (no highlighting needed).
    pub fn match_entries(&mut self, entries: &[String], query: &str) -> Vec<(usize, Vec<u32>)> {
        if query.is_empty() {
            return (0..entries.len()).map(|i| (i, vec![])).collect();
        }

        let pattern = Pattern::parse(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
        );

        let mut results: Vec<(usize, u32, Vec<u32>)> = Vec::new();
        let mut buf = Vec::new();
        let mut indices = Vec::new();

        for (idx, entry) in entries.iter().enumerate() {
            let haystack = Utf32Str::new(entry, &mut buf);
            indices.clear();
            if let Some(score) = pattern.indices(haystack, &mut self.matcher, &mut indices) {
                results.push((idx, score, indices.clone()));
            }
            buf.clear();
        }

        // Sort by score (higher is better)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(idx, _, positions)| (idx, positions)).collect()
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

    fn indices(results: &[(usize, Vec<u32>)]) -> Vec<usize> {
        results.iter().map(|(i, _)| *i).collect()
    }

    #[test]
    fn test_empty_query() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let results = matcher.match_entries(&entries, "");
        assert_eq!(indices(&results), vec![0, 1, 2]);
    }

    #[test]
    fn test_exact_match() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec!["foo".to_string(), "bar".to_string(), "foobar".to_string()];
        let results = matcher.match_entries(&entries, "foo");
        let idxs = indices(&results);
        assert!(idxs.contains(&0));
        assert!(idxs.contains(&2));
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
        assert!(indices(&results).contains(&0));
    }

    #[test]
    fn test_positions_returned() {
        let mut matcher = FuzzyMatcher::new();
        let entries = vec!["main.rs".to_string()];
        let results = matcher.match_entries(&entries, "mn");
        assert_eq!(results.len(), 1);
        // Positions should be non-empty for a match
        assert!(!results[0].1.is_empty());
    }
}
