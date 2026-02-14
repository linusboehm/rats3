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
