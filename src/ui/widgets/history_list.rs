use crate::app::App;
use crate::config::Config;
use crate::ui::text_utils;
use crate::ui::text_utils::truncate_path;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};


pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config, is_focused: bool) {
    // Clear the area first to hide underlying content
    frame.render_widget(Clear, area);

    let history = app.history();
    let filtered_indices = app.filtered_history();
    let selected_index = app.history_selected_index();
    let search_query = app.search_query();

    // Determine border color based on focus
    let border_color = if is_focused {
        config.colors.accent_normal.to_ratatui_color()
    } else {
        config.colors.border.to_ratatui_color()
    };

    // Show message if no history or no filtered results
    if filtered_indices.is_empty() {
        let message = if history.is_empty() {
            "No history available"
        } else {
            "No matches found"
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(config.colors.background.to_ratatui_color()))
            .title(" History ");

        let paragraph = ratatui::widgets::Paragraph::new(message)
            .style(Style::default().fg(config.colors.text_secondary.to_ratatui_color()))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
        return;
    }

    // Calculate available width for paths (accounting for borders, icon, and spacing)
    let max_path_width = (area.width as usize).saturating_sub(8); // 2 borders + icon + spacing

    // Create list items from filtered history
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .map(|&idx| {
            let path = &history[idx];
            // Use folder icon for all history entries
            let icon = "\u{f07b}"; //
            let color = config.colors.file_icon_dir.to_ratatui_color();

            // Truncate path if needed (history entries are already full display URIs)
            let display_path = truncate_path(path, max_path_width);
            let name = format!(" {} {}", icon, display_path);

            // Highlight matching text
            let highlight_color = config.colors.accent_search.to_ratatui_color();
            let base_style = Style::default().fg(color);
            let name_spans = text_utils::highlight_matches(
                &name,
                search_query,
                base_style,
                highlight_color,
            );

            let line = Line::from(name_spans);

            ListItem::new(line)
        })
        .collect();

    let title = if app.search_query().is_empty() {
        format!(" History ({} entries) ", history.len())
    } else {
        format!(" History ({}/{} matches) ", filtered_indices.len(), history.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(config.colors.background.to_ratatui_color()))
                .title(title),
        )
        .style(Style::default().bg(config.colors.background.to_ratatui_color()))
        .highlight_style(
            Style::default()
                .bg(config.colors.selection_bg.to_ratatui_color())
                .fg(config.colors.text_primary.to_ratatui_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("❯ ");

    // Create state for scrolling
    let mut list_state = ListState::default();
    list_state.select(Some(selected_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}
