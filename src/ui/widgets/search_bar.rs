use crate::app::App;
use crate::config::Config;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config) {
    // Check if preview search is active - it takes priority
    let preview_search_active = app.is_preview_search_active();
    let query = if preview_search_active {
        app.preview_search_query()
    } else {
        app.search_query()
    };
    let in_search_mode = app.is_search_mode() || preview_search_active;

    let text = if in_search_mode {
        // In search mode - show search prompt
        if query.is_empty() {
            Line::from(vec![
                Span::styled(" / ", Style::default().fg(config.colors.accent_search.to_ratatui_color()).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(config.colors.accent_search.to_ratatui_color())), // Cursor
            ])
        } else {
            Line::from(vec![
                Span::styled(" / ", Style::default().fg(config.colors.accent_search.to_ratatui_color()).add_modifier(Modifier::BOLD)),
                Span::styled(query, Style::default().fg(config.colors.text_primary.to_ratatui_color())),
                Span::styled("█", Style::default().fg(config.colors.accent_search.to_ratatui_color())), // Cursor
            ])
        }
    } else {
        // Normal mode - show hint or filtered results
        if query.is_empty() {
            Line::from(vec![
                Span::styled(" ❯ ", Style::default().fg(config.colors.accent_normal.to_ratatui_color()).add_modifier(Modifier::BOLD)),
                Span::styled(
                    "Press / to search",
                    Style::default().fg(config.colors.text_secondary.to_ratatui_color()).add_modifier(Modifier::ITALIC),
                ),
            ])
        } else {
            // Filtered but not in search mode - show results
            Line::from(vec![
                Span::styled(" ❯ ", Style::default().fg(config.colors.accent_normal.to_ratatui_color()).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("Filtered: {}", query),
                    Style::default().fg(config.colors.accent_search.to_ratatui_color()),
                ),
                Span::styled(
                    " (Press / to edit)",
                    Style::default().fg(config.colors.text_secondary.to_ratatui_color()).add_modifier(Modifier::ITALIC),
                ),
            ])
        }
    };

    let title = if preview_search_active {
        " Preview Search Mode "
    } else if in_search_mode {
        " Search Mode "
    } else {
        " Normal Mode "
    };

    let border_color = if in_search_mode {
        config.colors.accent_search.to_ratatui_color()
    } else {
        config.colors.accent_normal.to_ratatui_color()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
