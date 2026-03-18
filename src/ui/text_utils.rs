use ratatui::{
    style::{Color, Style},
    text::Span,
};

/// Split text into spans with specific character positions highlighted.
/// Positions are char indices into `text` (as returned by nucleo-matcher).
pub fn highlight_positions(
    text: &str,
    positions: &[u32],
    normal_style: Style,
    highlight_color: Color,
) -> Vec<Span<'static>> {
    if positions.is_empty() {
        return vec![Span::styled(text.to_string(), normal_style)];
    }

    let highlight_style = normal_style.fg(highlight_color);
    let pos_set: std::collections::HashSet<u32> = positions.iter().copied().collect();

    let mut spans = Vec::new();
    let mut current = String::new();
    let mut current_highlighted = false;

    for (char_idx, ch) in text.chars().enumerate() {
        let is_highlighted = pos_set.contains(&(char_idx as u32));
        if is_highlighted != current_highlighted && !current.is_empty() {
            let style = if current_highlighted { highlight_style } else { normal_style };
            spans.push(Span::styled(std::mem::take(&mut current), style));
        }
        current_highlighted = is_highlighted;
        current.push(ch);
    }

    if !current.is_empty() {
        let style = if current_highlighted { highlight_style } else { normal_style };
        spans.push(Span::styled(current, style));
    }

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
    fn test_no_positions_returns_original() {
        let result = highlight_positions("Hello World", &[], Style::default(), Color::Red);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Hello World");
    }

    #[test]
    fn test_single_position() {
        // Highlight char at index 6 ('W')
        let result = highlight_positions("Hello World", &[6], Style::default(), Color::Red);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "W");
        assert_eq!(result[1].style.fg, Some(Color::Red));
        assert_eq!(result[2].content, "orld");
    }

    #[test]
    fn test_consecutive_positions() {
        // Highlight chars 6,7,8,9,10 → "World"
        let result = highlight_positions("Hello World", &[6, 7, 8, 9, 10], Style::default(), Color::Red);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "World");
        assert_eq!(result[1].style.fg, Some(Color::Red));
    }

    #[test]
    fn test_scattered_positions() {
        // Highlight first and last char
        let result = highlight_positions("Hello", &[0, 4], Style::default(), Color::Red);
        assert_eq!(result[0].content, "H");
        assert_eq!(result[0].style.fg, Some(Color::Red));
        assert_eq!(result[1].content, "ell");
        assert_eq!(result[2].content, "o");
        assert_eq!(result[2].style.fg, Some(Color::Red));
    }

    #[test]
    fn test_unicode_positions() {
        // '世' is char index 6, '界' is char index 7
        let result = highlight_positions("Hello 世界", &[6, 7], Style::default(), Color::Red);
        assert_eq!(result[0].content, "Hello ");
        assert_eq!(result[1].content, "世界");
        assert_eq!(result[1].style.fg, Some(Color::Red));
    }
}
