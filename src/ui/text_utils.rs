use ratatui::{
    style::{Color, Style},
    text::Span,
};

/// Split text into spans with highlighted matches
/// Performs case-insensitive matching
/// Returns owned Span<'static> to avoid lifetime issues
pub fn highlight_matches(
    text: &str,
    query: &str,
    normal_style: Style,
    highlight_color: Color,
) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), normal_style)];
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    // Find all matches
    for (idx, _) in text_lower.match_indices(&query_lower) {
        // Add text before match
        if idx > last_end {
            spans.push(Span::styled(
                text[last_end..idx].to_string(),
                normal_style,
            ));
        }

        // Add highlighted match
        let match_end = idx + query.len();
        spans.push(Span::styled(
            text[idx..match_end].to_string(),
            normal_style.fg(highlight_color),
        ));

        last_end = match_end;
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::styled(
            text[last_end..].to_string(),
            normal_style,
        ));
    }

    // If no matches found, return the original text
    if spans.is_empty() {
        vec![Span::styled(text.to_string(), normal_style)]
    } else {
        spans
    }
}

/// Truncate a path if too long, keeping the most relevant (rightmost) parts.
/// Shows ".../" prefix when truncated.
pub fn truncate_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        return path.to_string();
    }

    let prefix = ".../";
    if max_width <= prefix.len() {
        return prefix.to_string();
    }

    let available = max_width - prefix.len();
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return "/".to_string();
    }

    let mut result = String::new();
    let mut remaining = available;

    for part in parts.iter().rev() {
        let part_len = part.len() + 1; // +1 for '/'
        if remaining >= part_len {
            if result.is_empty() {
                result = part.to_string();
            } else {
                result = format!("{}/{}", part, result);
            }
            remaining -= part_len;
        } else {
            break;
        }
    }

    format!("{}{}", prefix, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_original() {
        let text = "Hello World";
        let result = highlight_matches(text, "", Style::default(), Color::Red);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello World");
    }

    #[test]
    fn test_no_match_returns_original() {
        let text = "Hello World";
        let result = highlight_matches(text, "xyz", Style::default(), Color::Red);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello World");
    }

    #[test]
    fn test_single_match() {
        let text = "Hello World";
        let result = highlight_matches(text, "World", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "World");
        assert_eq!(result[1].style.fg, Some(Color::Red));
    }

    #[test]
    fn test_case_insensitive_match() {
        let text = "Hello World";
        let result = highlight_matches(text, "world", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "World");
    }

    #[test]
    fn test_multiple_matches() {
        let text = "foo bar foo baz";
        let result = highlight_matches(text, "foo", Style::default(), Color::Yellow);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].content, "foo");
        assert_eq!(result[1].content, " bar ");
        assert_eq!(result[2].content, "foo");
        assert_eq!(result[3].content, " baz");
    }

    #[test]
    fn test_match_at_beginning() {
        let text = "Hello World";
        let result = highlight_matches(text, "Hello", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello");
        assert_eq!(result[1].content, " World");
    }

    #[test]
    fn test_match_at_end() {
        let text = "Hello World";
        let result = highlight_matches(text, "World", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "World");
    }

    #[test]
    fn test_entire_text_matches() {
        let text = "test";
        let result = highlight_matches(text, "test", Style::default(), Color::Green);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "test");
        assert_eq!(result[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn test_overlapping_would_not_occur() {
        // Query can't overlap with itself in this implementation
        let text = "aaaa";
        let result = highlight_matches(text, "aa", Style::default(), Color::Blue);
        // Should match "aa" at positions 0 and 2
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "aa");
        assert_eq!(result[1].content, "aa");
    }

    #[test]
    fn test_preserves_original_case() {
        let text = "HeLLo WoRLd";
        let result = highlight_matches(text, "hello", Style::default(), Color::Red);
        assert_eq!(result[0].content, "HeLLo");
        assert_eq!(result[1].content, " WoRLd");
    }

    #[test]
    fn test_special_characters() {
        let text = "file.txt";
        let result = highlight_matches(text, ".txt", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "file");
        assert_eq!(result[1].content, ".txt");
    }

    #[test]
    fn test_unicode_characters() {
        let text = "Hello 世界";
        let result = highlight_matches(text, "世界", Style::default(), Color::Red);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "世界");
    }
}
