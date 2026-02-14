use crate::app::{App, DownloadState};
use crate::config::Config;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the progress pane showing active downloads and other background tasks
pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config) {
    let downloads = app.downloads();

    if downloads.is_empty() {
        // Show empty state
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(config.colors.border.to_ratatui_color()))
            .title(" Progress ");

        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No active tasks",
                Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
            )),
        ];

        let paragraph = Paragraph::new(empty_text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    // Calculate totals
    let mut sorted_downloads: Vec<_> = downloads.iter().collect();
    sorted_downloads.sort_by(|(path_a, _), (path_b, _)| path_a.cmp(path_b));

    let total_files = sorted_downloads.len();
    let completed_files = sorted_downloads
        .iter()
        .filter(|(_, info)| info.status == DownloadState::Complete)
        .count();
    let in_progress_files = sorted_downloads
        .iter()
        .filter(|(_, info)| info.status == DownloadState::InProgress)
        .count();
    let failed_files = sorted_downloads
        .iter()
        .filter(|(_, info)| matches!(info.status, DownloadState::Error(_)))
        .count();

    let total_size: u64 = sorted_downloads
        .iter()
        .filter_map(|(_, info)| info.total)
        .sum();
    let downloaded_size: u64 = sorted_downloads.iter().map(|(_, info)| info.downloaded).sum();

    let overall_progress = if total_size > 0 {
        (downloaded_size as f64 / total_size as f64 * 100.0) as u16
    } else {
        0
    };

    // Get spinner character
    let spinner = get_spinner_char();

    // Create block with title
    let title = if in_progress_files > 0 {
        format!(" {} Downloads ", spinner)
    } else {
        " Downloads ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.colors.border.to_ratatui_color()))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build content
    let mut lines = Vec::new();

    // Summary line
    let summary_style = Style::default().fg(config.colors.text_primary.to_ratatui_color());
    lines.push(Line::from(vec![
        Span::styled("Files: ", Style::default().fg(config.colors.text_secondary.to_ratatui_color())),
        Span::styled(
            format!("{}/{}", completed_files, total_files),
            summary_style,
        ),
    ]));

    // Size line
    lines.push(Line::from(vec![
        Span::styled("Size: ", Style::default().fg(config.colors.text_secondary.to_ratatui_color())),
        Span::styled(
            format!("{} / {}", format_size(downloaded_size), format_size(total_size)),
            summary_style,
        ),
    ]));

    // Progress line
    let progress_color = if in_progress_files > 0 {
        config.colors.accent_normal.to_ratatui_color()
    } else if failed_files > 0 {
        config.colors.text_error.to_ratatui_color()
    } else {
        config.colors.accent_search.to_ratatui_color()
    };

    lines.push(Line::from(vec![
        Span::styled("Progress: ", Style::default().fg(config.colors.text_secondary.to_ratatui_color())),
        Span::styled(
            format!("{}%", overall_progress),
            Style::default().fg(progress_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Status line
    if failed_files > 0 {
        lines.push(Line::from(vec![
            Span::styled("Failed: ", Style::default().fg(config.colors.text_secondary.to_ratatui_color())),
            Span::styled(
                format!("{}", failed_files),
                Style::default().fg(config.colors.text_error.to_ratatui_color()),
            ),
        ]));
    }

    // Empty line separator
    lines.push(Line::from(""));

    // Progress bar visualization
    let bar_width = (inner.width as usize).saturating_sub(2);
    let filled_width = (bar_width as f64 * overall_progress as f64 / 100.0) as usize;
    let empty_width = bar_width.saturating_sub(filled_width);

    let bar = format!("{}{}",
        "█".repeat(filled_width),
        "░".repeat(empty_width)
    );

    lines.push(Line::from(Span::styled(
        bar,
        Style::default().fg(progress_color),
    )));

    // Empty line
    lines.push(Line::from(""));

    // Individual file statuses (show up to 5)
    for (path, info) in sorted_downloads.iter().take(5) {
        let filename = path.split('/').last().unwrap_or(path);

        // Truncate filename if too long
        let max_name_len = (inner.width as usize).saturating_sub(5);
        let display_name = if filename.len() > max_name_len {
            format!("{}...", &filename[..max_name_len.saturating_sub(3)])
        } else {
            filename.to_string()
        };

        let (icon, status_color) = match &info.status {
            DownloadState::InProgress => ("⬇", config.colors.accent_normal.to_ratatui_color()),
            DownloadState::Complete => ("✓", config.colors.accent_search.to_ratatui_color()),
            DownloadState::Canceled => ("⊘", config.colors.text_secondary.to_ratatui_color()),
            DownloadState::Error(_) => ("✗", config.colors.text_error.to_ratatui_color()),
        };

        let file_progress = if let Some(total) = info.total {
            if total > 0 {
                (info.downloaded as f64 / total as f64 * 100.0) as u16
            } else {
                0
            }
        } else {
            0
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(status_color)),
            Span::styled(
                display_name,
                Style::default().fg(config.colors.text_primary.to_ratatui_color()),
            ),
            Span::styled(
                format!(" {}%", file_progress),
                Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
            ),
        ]));
    }

    // Show "and N more" if there are more files
    if sorted_downloads.len() > 5 {
        lines.push(Line::from(Span::styled(
            format!("  ...and {} more", sorted_downloads.len() - 5),
            Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Get current spinner character based on time
fn get_spinner_char() -> &'static str {
    const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let millis = now.as_millis();
    let frame_idx = (millis / 80) as usize % SPINNER_FRAMES.len();
    SPINNER_FRAMES[frame_idx]
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}
