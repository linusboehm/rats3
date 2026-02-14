use crate::app::App;
use crate::config::{Config, DownloadDestination};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config, destinations: &[DownloadDestination]) {
    // Clear the area first to hide underlying content
    frame.render_widget(Clear, area);

    let selected_index = app.download_destination_index();

    // Determine border color
    let border_color = config.colors.accent_normal.to_ratatui_color();

    // Show message if no destinations configured
    if destinations.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(config.colors.background.to_ratatui_color()))
            .title(" Select Download Destination ");

        let paragraph = ratatui::widgets::Paragraph::new("No download destinations configured\n\nEdit ~/.config/rats3/config.toml to add destinations")
            .style(Style::default().fg(config.colors.text_secondary.to_ratatui_color()))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
        return;
    }

    // Create list items from destinations
    let items: Vec<ListItem> = destinations
        .iter()
        .map(|dest| {
            // Use download icon
            let icon = "\u{f019}"; //
            let color = config.colors.accent_normal.to_ratatui_color();

            let name = format!(" {} {}", icon, dest.name);
            let path = format!("    {}", dest.path);

            let lines = vec![
                Line::from(vec![
                    Span::styled(name, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(path, Style::default().fg(config.colors.text_secondary.to_ratatui_color())),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let selected_count = app.selected_count();
    let title = format!(" Select Download Destination ({} files selected) ", selected_count);

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
