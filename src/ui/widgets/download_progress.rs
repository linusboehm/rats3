use crate::app::{DownloadInfo, DownloadState};
use crate::config::Config;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};
use std::collections::HashMap;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    downloads: &HashMap<String, DownloadInfo>,
    config: &Config,
) {
    if downloads.is_empty() {
        return;
    }

    // Sort downloads by path for consistent ordering
    let mut sorted_downloads: Vec<_> = downloads.iter().collect();
    sorted_downloads.sort_by(|(path_a, _), (path_b, _)| path_a.cmp(path_b));

    // Calculate totals
    let total_files = sorted_downloads.len();
    let completed_files = sorted_downloads.iter().filter(|(_, info)| info.status == DownloadState::Complete).count();
    let failed_files = sorted_downloads.iter().filter(|(_, info)| matches!(info.status, DownloadState::Error(_))).count();

    let total_size: u64 = sorted_downloads.iter()
        .filter_map(|(_, info)| info.total)
        .sum();
    let downloaded_size: u64 = sorted_downloads.iter()
        .map(|(_, info)| info.downloaded)
        .sum();

    let overall_progress = if total_size > 0 {
        (downloaded_size as f64 / total_size as f64 * 100.0) as u16
    } else {
        0
    };

    // Create main block with title
    let title = if failed_files > 0 {
        format!(
            " Downloads: {}/{} files | {} / {} | {}% ({} failed) ",
            completed_files,
            total_files,
            format_size(downloaded_size),
            format_size(total_size),
            overall_progress,
            failed_files
        )
    } else {
        format!(
            " Downloads: {}/{} files | {} / {} | {}% ",
            completed_files,
            total_files,
            format_size(downloaded_size),
            format_size(total_size),
            overall_progress
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(config.colors.border.to_ratatui_color()))
        .title(title)
        .style(Style::default().bg(Color::Reset));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Create layout for each download
    let download_constraints: Vec<Constraint> = sorted_downloads
        .iter()
        .map(|_| Constraint::Length(2))
        .collect();

    let download_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(download_constraints)
        .split(inner_area);

    // Render each download
    for (idx, (path, info)) in sorted_downloads.iter().enumerate() {
        if idx >= download_chunks.len() {
            break;
        }

        render_download_item(frame, download_chunks[idx], path, info, config);
    }
}

fn render_download_item(
    frame: &mut Frame,
    area: Rect,
    path: &str,
    info: &DownloadInfo,
    config: &Config,
) {
    // Extract filename from path
    let filename = path.split('/').last().unwrap_or(path);

    let (label, ratio, style) = match &info.status {
        DownloadState::InProgress => {
            let progress = if let Some(total) = info.total {
                if total > 0 {
                    (info.downloaded as f64 / total as f64).min(1.0)
                } else {
                    0.0
                }
            } else {
                // Unknown total size, show indeterminate progress
                0.0
            };

            let size_str = if let Some(total) = info.total {
                format!(
                    "{} / {}",
                    format_size(info.downloaded),
                    format_size(total)
                )
            } else {
                format_size(info.downloaded)
            };

            (
                format!("⬇ {} - {}", filename, size_str),
                progress,
                Style::default().fg(config.colors.accent_normal.to_ratatui_color()),
            )
        }
        DownloadState::Complete => (
            format!("✓ {} - Complete", filename),
            1.0,
            Style::default().fg(Color::Green),
        ),
        DownloadState::Canceled => (
            format!("⊘ {} - Canceled", filename),
            0.0,
            Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
        ),
        DownloadState::Error(err) => (
            format!("✗ {} - Error: {}", filename, err),
            0.0,
            Style::default().fg(config.colors.text_error.to_ratatui_color()),
        ),
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).border_style(
            Style::default().fg(config.colors.border.to_ratatui_color()),
        ))
        .gauge_style(style)
        .ratio(ratio)
        .label(label);

    frame.render_widget(gauge, area);
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
