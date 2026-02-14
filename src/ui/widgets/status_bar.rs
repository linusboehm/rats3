use crate::app::{App, DownloadState};
use crate::config::Config;
use crate::status::StatusSeverity;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config) {
    // Create block with borders all around
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.colors.border.to_ratatui_color()))
        .title(" Status ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // If help is shown, render help instead of normal status
    if app.is_help_shown() {
        render_help(frame, inner, app, config);
        return;
    }

    // Split inner area into left (status message) and right (progress)
    // Progress area should be flexible based on content, but we'll allocate space for it
    let has_progress = !app.downloads().is_empty();

    let chunks = if has_progress {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),        // Status message (left)
                Constraint::Length(60),    // Progress message (right, fixed width)
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100), // Status message takes full width
            ])
            .split(inner)
    };

    // Render status message on the left
    let status_text = if let Some(status_msg) = app.status_message() {
        // Determine color based on severity
        let color = match status_msg.severity {
            StatusSeverity::Info => config.colors.accent_normal.to_ratatui_color(),
            StatusSeverity::Success => config.colors.accent_search.to_ratatui_color(),
            StatusSeverity::Warning => config.colors.accent_search.to_ratatui_color(),
            StatusSeverity::Error => config.colors.text_error.to_ratatui_color(),
        };

        // Word-wrap message to fit in multiple lines
        let max_width = (chunks[0].width as usize).saturating_sub(2); // Account for padding
        let max_lines = chunks[0].height as usize; // Available lines

        // Split message into words and wrap to fit width
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in status_msg.content.split_whitespace() {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            if test_line.len() <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = word.to_string();
                } else {
                    // Word is longer than max_width, truncate it
                    lines.push(format!("{}...", &word[..max_width.saturating_sub(3)]));
                    current_line = String::new();
                }
            }

            if lines.len() >= max_lines {
                break;
            }
        }

        if !current_line.is_empty() && lines.len() < max_lines {
            lines.push(current_line);
        }

        // Convert to Lines with styling based on severity
        lines.into_iter()
            .map(|line| Line::from(vec![Span::styled(
                format!(" {}", line),
                Style::default().fg(color),
            )]))
            .collect::<Vec<_>>()
    } else {
        // Show minimal status by default (just file count)
        let count = app.filtered_indices().len();
        let total = app.entries().len();

        vec![Line::from(vec![
            Span::styled(
                format!(" {}/{} files", count, total),
                Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
            ),
            Span::styled(
                "  Press ? for help",
                Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
            ),
        ])]
    };

    let status_paragraph = Paragraph::new(status_text);
    frame.render_widget(status_paragraph, chunks[0]);

    // Render progress message on the right (if downloads are active)
    if has_progress {
        let progress_text = format_download_progress(app);
        let progress_lines = vec![Line::from(Span::styled(
            progress_text,
            Style::default().fg(config.colors.accent_normal.to_ratatui_color()),
        ))];
        let progress_paragraph = Paragraph::new(progress_lines);
        frame.render_widget(progress_paragraph, chunks[1]);
    }
}

/// Format download progress message
/// Format: "downloading n/m files x/y b (x1/y1 b total) z%"
fn format_download_progress(app: &App) -> String {
    let downloads = app.downloads();
    if downloads.is_empty() {
        return String::new();
    }

    let total_files = downloads.len();
    let completed_files = downloads
        .values()
        .filter(|info| info.status == DownloadState::Complete)
        .count();

    // Get all in-progress downloads
    let in_progress_downloads: Vec<_> = downloads
        .values()
        .filter(|info| info.status == DownloadState::InProgress)
        .collect();

    let in_progress_count = in_progress_downloads.len();

    let total_size: u64 = downloads
        .values()
        .filter_map(|info| info.total)
        .sum();
    let downloaded_size: u64 = downloads.values().map(|info| info.downloaded).sum();

    let overall_progress = if total_size > 0 {
        (downloaded_size as f64 / total_size as f64 * 100.0) as u16
    } else {
        0
    };

    if !in_progress_downloads.is_empty() {
        // Calculate combined stats for all in-progress downloads
        let in_progress_downloaded: u64 = in_progress_downloads.iter().map(|info| info.downloaded).sum();
        let in_progress_total: u64 = in_progress_downloads
            .iter()
            .filter_map(|info| info.total)
            .sum();

        let current_downloaded = format_size(in_progress_downloaded);
        let current_total = if in_progress_total > 0 {
            format_size(in_progress_total)
        } else {
            "?".to_string()
        };

        format!(
            "downloading {}/{} files {} / {} ({} / {} total) {}%",
            in_progress_count,
            total_files,
            current_downloaded,
            current_total,
            format_size(downloaded_size),
            format_size(total_size),
            overall_progress
        )
    } else {
        // All done or all failed
        format!(
            "{}/{} files ({} / {}) {}%",
            completed_files,
            total_files,
            format_size(downloaded_size),
            format_size(total_size),
            overall_progress
        )
    }
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
        format!("{}{}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

/// Render context-sensitive help
fn render_help(frame: &mut Frame, area: Rect, app: &App, config: &Config) {
    use crate::app::{AppMode, FocusedPanel};

    let mode = app.mode();
    let focused_panel = app.focused_panel();
    let preview_visual = app.is_preview_visual_mode();

    let help_lines = match mode {
        AppMode::Search if app.is_searching_history() => {
            vec![
                "Searching History:",
                "Type=filter  Ctrl-j/k=navigate  Enter=select  Esc=exit search",
            ]
        }
        AppMode::Search => {
            vec![
                "Search Mode:",
                "Type=filter  Ctrl-j/k/↑/↓=navigate  Enter=open  Esc=exit search",
            ]
        }
        AppMode::Visual => {
            vec![
                "Visual Mode:",
                "j/k=move & select  Space=toggle  s/S=download  v/Esc=exit",
            ]
        }
        AppMode::History => {
            vec![
                "History Mode:",
                "j/k=move  /=search  Enter=navigate  Esc=exit",
            ]
        }
        AppMode::Download => {
            vec![
                "Download Mode:",
                "j/k=select destination  Enter=confirm  Esc=cancel",
            ]
        }
        AppMode::Normal => {
            if focused_panel == &FocusedPanel::Preview {
                if app.is_preview_search_active() {
                    vec![
                        "Preview Search Mode:",
                        "Type=filter  Ctrl-j/k/↑/↓=next/prev result  Enter=jump  Esc=exit",
                    ]
                } else if preview_visual {
                    vec![
                        "Preview Visual Mode:",
                        "j/k=move  Ctrl-u/d=page  gg/G=top/bottom  y=yank  v/Esc=exit",
                    ]
                } else {
                    vec![
                        "Preview Mode:",
                        "j/k=scroll  Ctrl-u/d=page  gg/G=top/bottom  /=search  v=visual",
                        "w=wrap  Tab=switch to explorer  H/L=resize  ?=help",
                    ]
                }
            } else {
                vec![
                    "Explorer Mode:",
                    "j/k=move  Enter/l=open  h=back  /=search  Space=select  v=visual",
                    "s/S=download  Ctrl-r=history  Y=copy path  q=quit  ?=help",
                ]
            }
        }
    };

    let key_color = config.colors.accent_normal.to_ratatui_color();
    let text_color = config.colors.text_secondary.to_ratatui_color();

    let mut lines = Vec::new();
    for (i, line) in help_lines.iter().enumerate() {
        if i == 0 {
            // Header line
            lines.push(Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(key_color).add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!(" {}", line),
                Style::default().fg(text_color),
            )));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
