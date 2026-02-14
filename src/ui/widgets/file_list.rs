use crate::app::App;
use crate::config::Config;
use crate::ui::text_utils;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Get Nerd Font icon and color for a file or directory
/// Based on nvim-web-devicons
fn get_file_icon(name: &str, is_dir: bool, config: &Config) -> (&'static str, Color) {
    if is_dir {
        return ("\u{f07b}", config.colors.file_icon_dir.to_ratatui_color()); //
    }

    // Check exact filename matches first
    let icon_color = match name {
        ".gitignore" => ("\u{f1d3}", config.colors.text_error.to_ratatui_color()), //
        ".gitmodules" => ("\u{f1d3}", config.colors.text_error.to_ratatui_color()), //
        ".gitattributes" => ("\u{f1d3}", config.colors.text_error.to_ratatui_color()), //
        "Dockerfile" => ("\u{f308}", config.colors.accent_normal.to_ratatui_color()), //
        "docker-compose.yml" => ("\u{f308}", config.colors.accent_normal.to_ratatui_color()), //
        "Makefile" => ("\u{e615}", config.colors.file_icon_config.to_ratatui_color()), //
        "LICENSE" => ("\u{f718}", config.colors.accent_search.to_ratatui_color()), //
        "README.md" | "README" => ("\u{f48a}", config.colors.accent_normal.to_ratatui_color()), //
        "Cargo.toml" => ("\u{e7a8}", config.colors.file_icon_rust.to_ratatui_color()), //
        "Cargo.lock" => ("\u{e7a8}", config.colors.file_icon_rust.to_ratatui_color()), //
        "package.json" => ("\u{e718}", config.colors.file_icon_script.to_ratatui_color()), //
        "package-lock.json" => ("\u{e718}", config.colors.file_icon_script.to_ratatui_color()), //
        "tsconfig.json" => ("\u{e628}", config.colors.accent_normal.to_ratatui_color()), //
        "webpack.config.js" => ("\u{fc29}", config.colors.accent_normal.to_ratatui_color()), //
        _ => {
            // Fall back to extension-based matching
            let ext = std::path::Path::new(name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            match ext {
                // Rust
                "rs" => ("\u{e7a8}", config.colors.file_icon_rust.to_ratatui_color()), //

                // JavaScript/TypeScript
                "js" | "jsx" | "mjs" | "cjs" => ("\u{e74e}", config.colors.accent_search.to_ratatui_color()), //
                "ts" | "tsx" => ("\u{e628}", config.colors.accent_normal.to_ratatui_color()), //

                // Python
                "py" | "pyc" | "pyd" | "pyo" => ("\u{e73c}", config.colors.accent_normal.to_ratatui_color()), //

                // Go
                "go" => ("\u{e627}", config.colors.accent_normal.to_ratatui_color()), //

                // C/C++
                "c" | "h" => ("\u{e61e}", config.colors.accent_normal.to_ratatui_color()), //
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => ("\u{e61d}", config.colors.accent_normal.to_ratatui_color()), //

                // Java/Kotlin
                "java" => ("\u{e738}", config.colors.file_icon_config.to_ratatui_color()), //
                "kt" | "kts" => ("\u{e634}", config.colors.file_icon_config.to_ratatui_color()), //

                // Web
                "html" | "htm" => ("\u{e736}", config.colors.file_icon_config.to_ratatui_color()), //
                "css" | "scss" | "sass" | "less" => ("\u{e749}", config.colors.accent_normal.to_ratatui_color()), //

                // Config files
                "json" => ("\u{e60b}", config.colors.accent_search.to_ratatui_color()), //
                "yaml" | "yml" => ("\u{f481}", config.colors.file_icon_config.to_ratatui_color()), //
                "toml" => ("\u{e615}", config.colors.file_icon_config.to_ratatui_color()), //
                "xml" => ("\u{e619}", config.colors.file_icon_config.to_ratatui_color()), //
                "ini" | "cfg" | "conf" => ("\u{e615}", config.colors.file_icon_config.to_ratatui_color()), //

                // Shell
                "sh" | "bash" | "zsh" | "fish" => ("\u{f489}", config.colors.file_icon_script.to_ratatui_color()), //

                // Documents
                "md" | "markdown" => ("\u{e73e}", config.colors.accent_normal.to_ratatui_color()), //
                "txt" => ("\u{f15c}", config.colors.file_icon_doc.to_ratatui_color()), //
                "pdf" => ("\u{f1c1}", config.colors.text_error.to_ratatui_color()), //
                "doc" | "docx" => ("\u{f1c2}", config.colors.accent_normal.to_ratatui_color()), //

                // Data
                "csv" => ("\u{f1c3}", config.colors.file_icon_script.to_ratatui_color()), //
                "sql" => ("\u{e706}", config.colors.file_icon_doc.to_ratatui_color()), //
                "db" | "sqlite" | "sqlite3" => ("\u{e706}", config.colors.file_icon_doc.to_ratatui_color()), //

                // Images
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" => ("\u{f1c5}", config.colors.file_icon_config.to_ratatui_color()), //
                "svg" => ("\u{f1c5}", config.colors.accent_search.to_ratatui_color()), //

                // Archives
                "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => ("\u{f410}", config.colors.file_icon_rust.to_ratatui_color()), //

                // Ruby
                "rb" => ("\u{e739}", config.colors.text_error.to_ratatui_color()), //

                // PHP
                "php" => ("\u{e73d}", config.colors.file_icon_config.to_ratatui_color()), //

                // Lua
                "lua" => ("\u{e620}", config.colors.accent_normal.to_ratatui_color()), //

                // Vim
                "vim" => ("\u{e62b}", config.colors.file_icon_script.to_ratatui_color()), //

                // Git
                "git" => ("\u{f1d3}", config.colors.text_error.to_ratatui_color()), //

                // Docker
                "dockerfile" => ("\u{f308}", config.colors.accent_normal.to_ratatui_color()), //

                // Lock files
                "lock" => ("\u{f023}", config.colors.text_secondary.to_ratatui_color()), //

                // Log files
                "log" => ("\u{f18d}", config.colors.text_secondary.to_ratatui_color()), //

                // Default
                _ => ("\u{f15b}", config.colors.file_icon_default.to_ratatui_color()), //
            }
        }
    };

    icon_color
}

/// Truncate path if too long, keeping the most relevant (rightmost) parts
fn truncate_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        return path.to_string();
    }

    // Reserve space for ".../" prefix
    let prefix = ".../";
    if max_width <= prefix.len() {
        return prefix.to_string();
    }

    let available = max_width - prefix.len();

    // Split path into components
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return "/".to_string();
    }

    // Build path from right to left until we run out of space
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

pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config, is_focused: bool) {
    let entries = app.entries();
    let filtered_indices = app.filtered_indices();
    let selected_index = app.selected_index();
    let search_query = app.search_query();

    // Determine border color based on focus
    let border_color = if is_focused {
        config.colors.accent_normal.to_ratatui_color()
    } else {
        config.colors.border.to_ratatui_color()
    };

    // Show message if no entries
    if filtered_indices.is_empty() {
        let raw_path = if app.current_prefix().is_empty() {
            "/".to_string()
        } else {
            app.current_prefix().to_string()
        };

        let max_path_width = (area.width as usize).saturating_sub(4);
        let display_path = truncate_path(&raw_path, max_path_width);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", display_path));

        let message = if entries.is_empty() {
            "No files found"
        } else {
            "No matches found"
        };

        let paragraph = ratatui::widgets::Paragraph::new(message)
            .style(Style::default().fg(config.colors.text_secondary.to_ratatui_color()))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
        return;
    }

    // Calculate line number width (at least 2 digits)
    let max_line_num = filtered_indices.len();
    let line_num_width = max_line_num.to_string().len().max(2);

    // Create list items
    let items: Vec<ListItem> = filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, &entry_idx)| {
            let entry = &entries[entry_idx];

            // Line number (1-indexed for display)
            let line_num = display_idx + 1;
            let line_num_str = format!("{:>width$} ", line_num, width = line_num_width);

            // Check if file is selected
            let is_selected = app.is_file_selected(entry_idx);

            // Format entry name with icon
            let (icon, color) = get_file_icon(&entry.name, entry.is_dir, config);
            let selection_dot = "● "; // Blue dot for selected files

            // Add size info if file
            let size_info = if let Some(size) = entry.size {
                format!("  {}", format_size(size))
            } else {
                String::new()
            };

            // Apply visual mode style: background color for selected files
            let bg = if is_selected {
                config.colors.selection_bg.to_ratatui_color()
            } else {
                Color::Reset
            };

            // Highlight matching text in file name
            let name_with_icon = format!("{} {}", icon, entry.name);
            let highlight_color = config.colors.accent_search.to_ratatui_color();
            let base_style = Style::default().fg(color).bg(bg);

            let name_spans = text_utils::highlight_matches(
                &name_with_icon,
                search_query,
                base_style,
                highlight_color,
            );

            let line = if is_selected {
                // Selected: split the name to color the dot separately
                let mut spans = vec![
                    Span::styled(line_num_str, Style::default()
                        .fg(config.colors.text_secondary.to_ratatui_color())
                        .bg(bg)),
                    Span::styled(selection_dot, Style::default()
                        .fg(Color::Rgb(100, 149, 237)) // Cornflower blue dot (darker blue)
                        .bg(bg)),
                ];
                spans.extend(name_spans);
                spans.push(Span::styled(size_info, Style::default()
                    .fg(config.colors.text_secondary.to_ratatui_color())
                    .bg(bg)));
                Line::from(spans)
            } else {
                // Not selected: normal style
                let mut spans = vec![
                    Span::styled(line_num_str, Style::default()
                        .fg(config.colors.text_secondary.to_ratatui_color())),
                    Span::raw(" "),
                ];
                spans.extend(name_spans);
                spans.push(Span::styled(size_info, Style::default()
                    .fg(config.colors.text_secondary.to_ratatui_color())));
                Line::from(spans)
            };

            ListItem::new(line)
        })
        .collect();

    // Show path as title with filtered count if searching
    let raw_path = if app.current_prefix().is_empty() {
        "/".to_string()
    } else {
        app.current_prefix().to_string()
    };

    // Calculate max width for path (accounting for borders, spaces, and potential match count)
    let max_path_width = if app.search_query().is_empty() {
        // " path " + borders (2) = path gets (width - 4)
        (area.width as usize).saturating_sub(4)
    } else {
        // " path (X/Y matches) " - estimate match count takes ~20 chars
        (area.width as usize).saturating_sub(24)
    };

    let display_path = truncate_path(&raw_path, max_path_width);

    let selected_count = app.selected_count();
    let title = if app.search_query().is_empty() {
        if selected_count > 0 {
            format!(" {} [{} selected] ", display_path, selected_count)
        } else {
            format!(" {} ", display_path)
        }
    } else {
        if selected_count > 0 {
            format!(
                " {} ({}/{} matches) [{} selected] ",
                display_path,
                filtered_indices.len(),
                entries.len(),
                selected_count
            )
        } else {
            format!(
                " {} ({}/{} matches) ",
                display_path,
                filtered_indices.len(),
                entries.len()
            )
        }
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        )
        .highlight_style(
            Style::default()
                .bg(config.colors.selection_bg.to_ratatui_color())
                .fg(config.colors.text_primary.to_ratatui_color())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("❯ ");

    // Create state for scrolling with offset to show context below
    let mut list_state = ListState::default();
    list_state.select(Some(selected_index));

    // Calculate visible height (accounting for borders and title)
    let visible_height = area.height.saturating_sub(3) as usize; // 2 for borders, 1 for title

    // Ensure at least 3 lines are visible below the selected item
    // Calculate offset so selected item is not too close to bottom
    let scroll_offset = 3; // Number of lines to keep visible below selection
    if filtered_indices.len() > visible_height {
        // Calculate the maximum position where we still have 3 lines below
        let max_position_from_top = visible_height.saturating_sub(scroll_offset + 1);

        // If selected index is beyond this position, we need to scroll
        if selected_index > max_position_from_top {
            let offset = selected_index.saturating_sub(max_position_from_top);
            *list_state.offset_mut() = offset;
        }
    }

    frame.render_stateful_widget(list, area, &mut list_state);
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
