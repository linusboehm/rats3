use crate::app::{App, AppMode, FocusedPanel};
use crate::config::Config;
use crate::ui::widgets::{download_selector, file_list, history_list, preview, search_bar, status_bar};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App, config: &Config) {
    let area = frame.size();

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar with border
            Constraint::Min(0),     // Main content area
            Constraint::Length(5),  // Status pane with borders all around (fixed 5 lines)
        ])
        .split(area);

    // Split main content area horizontally: file list | preview
    let preview_width = app.preview_width_percent();
    let explorer_width = 100 - preview_width;
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(explorer_width), // File list (left)
            Constraint::Percentage(preview_width),   // Preview (right)
        ])
        .split(vertical_chunks[1]);

    // Render search bar
    search_bar::render(frame, vertical_chunks[0], app, config);

    // Check which panel is focused
    let explorer_focused = app.focused_panel() == &FocusedPanel::Explorer;
    let preview_focused = app.focused_panel() == &FocusedPanel::Preview;

    // Always render file list and preview
    file_list::render(frame, content_chunks[0], app, config, explorer_focused);
    preview::render(frame, content_chunks[1], app, config, preview_focused);

    // Render status bar
    status_bar::render(frame, vertical_chunks[2], app, config);

    // Render history overlay if in history mode or searching history
    if app.mode() == &AppMode::History || (app.is_search_mode() && app.is_searching_history()) {
        let history_area = centered_rect(80, 30, vertical_chunks[1]);
        history_list::render(frame, history_area, app, config, true);
    }

    // Render download destination selector if in download mode
    if app.mode() == &AppMode::Download {
        let download_area = centered_rect(70, 20, vertical_chunks[1]);
        download_selector::render(frame, download_area, app, config, &config.download_destinations);
    }
}

/// Create a centered rectangle within the given area
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    // Create vertical layout to center vertically
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min((area.height.saturating_sub(height)) / 2),
        ])
        .split(area);

    // Create horizontal layout to center horizontally
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
