use crate::app::{App, FocusedPanel};
use crate::backend::PreviewContent;
use crate::config::Config;
use crate::ui::text_utils::truncate_path;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME: Theme = {
        // Try to load custom Tokyo Night Moon theme
        const TOKYONIGHT_THEME: &str = include_str!("../../../themes/tokyonight_moon.tmTheme");
        let mut cursor = std::io::Cursor::new(TOKYONIGHT_THEME.as_bytes());
        ThemeSet::load_from_reader(&mut cursor)
            .unwrap_or_else(|_| {
                // Fallback to built-in theme if custom theme fails to load
                ThemeSet::load_defaults().themes["base16-ocean.dark"].clone()
            })
    };
}

/// Highlight matches within an already-styled line
fn highlight_line_matches(line: Line<'static>, query: &str, highlight_color: Color) -> Line<'static> {
    if query.is_empty() {
        return line;
    }

    let query_lower = query.to_lowercase();

    // Reconstruct the full text to find match positions
    let full_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    let full_text_lower = full_text.to_lowercase();

    // Find all match positions
    let mut match_ranges: Vec<(usize, usize)> = Vec::new();
    for (idx, _) in full_text_lower.match_indices(&query_lower) {
        match_ranges.push((idx, idx + query.len()));
    }

    if match_ranges.is_empty() {
        return line;
    }

    // Process each span and split it if it contains matches
    let mut new_spans = Vec::new();
    let mut char_offset = 0;

    for span in line.spans {
        let span_text = span.content.to_string();
        let span_len = span_text.len();
        let span_end = char_offset + span_len;

        // Find matches that overlap with this span
        let mut last_split = 0;
        let mut span_had_match = false;

        for (match_start, match_end) in &match_ranges {
            // Check if match overlaps with this span
            if *match_end > char_offset && *match_start < span_end {
                span_had_match = true;

                // Calculate positions within the span
                let local_start = if *match_start > char_offset {
                    match_start - char_offset
                } else {
                    0
                };

                let local_end = if *match_end < span_end {
                    match_end - char_offset
                } else {
                    span_len
                };

                // Add text before match
                if local_start > last_split {
                    new_spans.push(Span::styled(
                        span_text[last_split..local_start].to_string(),
                        span.style,
                    ));
                }

                // Add highlighted match
                new_spans.push(Span::styled(
                    span_text[local_start..local_end].to_string(),
                    span.style.fg(highlight_color),
                ));

                last_split = local_end;
            }
        }

        if span_had_match {
            // Add remaining text in span
            if last_split < span_len {
                new_spans.push(Span::styled(
                    span_text[last_split..].to_string(),
                    span.style,
                ));
            }
        } else {
            // No match in this span, keep it as is
            new_spans.push(span);
        }

        char_offset = span_end;
    }

    Line::from(new_spans)
}

pub fn render(frame: &mut Frame, area: Rect, app: &App, config: &Config, is_focused: bool) {
    // Determine border color based on focus
    let border_color = if is_focused {
        config.colors.accent_normal.to_ratatui_color()
    } else {
        config.colors.border.to_ratatui_color()
    };

    // Build path string for the preview title: use the previewed file path when
    // available (file selected), otherwise fall back to the current directory prefix.
    let current_path = {
        let max_width = (area.width as usize).saturating_sub(30); // leave room for indicators
        let raw = app.current_preview_path()
            .unwrap_or_else(|| {
                let prefix = app.current_prefix();
                if prefix.is_empty() { "/" } else { prefix }
            });
        truncate_path(raw, max_width)
    };
    let wrap_indicator = if app.is_wrap_enabled() { " [wrap]" } else { "" };

    if let Some(preview) = app.get_preview() {
        match preview {
            PreviewContent::Text(content) => {
                // Calculate scroll info for title
                let total_lines = content.lines().count();
                let cursor_line = app.preview_cursor_line();
                let visual_mode = app.is_preview_visual_mode();

                let visual_indicator = if visual_mode { " VISUAL" } else { "" };

                let scroll_info = if total_lines > 0 {
                    let percentage = ((cursor_line + 1) * 100) / total_lines;
                    format!(" [{}/{} {}%]", cursor_line + 1, total_lines, percentage)
                } else {
                    String::new()
                };
                let title = format!(" {}{}{}{} ", current_path, wrap_indicator, visual_indicator, scroll_info);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title);

                render_text_preview(frame, area, content, app, block, config);
            }
            PreviewContent::Binary { size, mime_type } => {
                let title = format!(" {}{} ", current_path, wrap_indicator);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title);

                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Binary file",
                        Style::default().fg(config.colors.accent_search.to_ratatui_color()),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Size: {}", format_size(*size)),
                        Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
                    )),
                    Line::from(Span::styled(
                        format!(
                            "Type: {}",
                            mime_type.as_deref().unwrap_or("unknown")
                        ),
                        Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
                    )),
                ];
                let mut paragraph = Paragraph::new(text).block(block);
                if app.is_wrap_enabled() {
                    paragraph = paragraph.wrap(Wrap { trim: false });
                }
                frame.render_widget(paragraph, area);
            }
            PreviewContent::TooLarge { size } => {
                let title = format!(" {}{} ", current_path, wrap_indicator);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title);

                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "File too large for preview",
                        Style::default().fg(config.colors.accent_search.to_ratatui_color()),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Size: {}", format_size(*size)),
                        Style::default().fg(config.colors.text_secondary.to_ratatui_color()),
                    )),
                ];
                let mut paragraph = Paragraph::new(text).block(block);
                if app.is_wrap_enabled() {
                    paragraph = paragraph.wrap(Wrap { trim: false });
                }
                frame.render_widget(paragraph, area);
            }
            PreviewContent::Error(err) => {
                let title = format!(" {}{} ", current_path, wrap_indicator);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title);

                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled("Error", Style::default().fg(config.colors.text_error.to_ratatui_color()))),
                    Line::from(""),
                    Line::from(Span::styled(err, Style::default().fg(config.colors.text_secondary.to_ratatui_color()))),
                ];
                let mut paragraph = Paragraph::new(text).block(block);
                if app.is_wrap_enabled() {
                    paragraph = paragraph.wrap(Wrap { trim: false });
                }
                frame.render_widget(paragraph, area);
            }
        }
    } else {
        let title = format!(" {}{} ", current_path, wrap_indicator);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title);

        // No preview available
        let entry = app.selected_entry();
        let text = if let Some(e) = entry {
            if e.is_dir {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Directory",
                        Style::default().fg(config.colors.file_icon_dir.to_ratatui_color()),
                    )),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Loading preview...",
                        Style::default().fg(config.colors.text_secondary.to_ratatui_color()).add_modifier(Modifier::ITALIC),
                    )),
                ]
            }
        } else {
            vec![Line::from("")]
        };

        let mut paragraph = Paragraph::new(text).block(block);
        if app.is_wrap_enabled() {
            paragraph = paragraph.wrap(Wrap { trim: false });
        }
        frame.render_widget(paragraph, area);
    }
}

fn render_text_preview(
    frame: &mut Frame,
    area: Rect,
    content: &str,
    app: &App,
    block: Block,
    config: &Config,
) {
    // Try to get file extension for syntax highlighting
    let file_path = app.get_selected_file_path();
    let extension = file_path
        .as_ref()
        .and_then(|p| std::path::Path::new(p).extension())
        .and_then(|e| e.to_str());

    let all_lines = if let Some(ext) = extension {
        // Special handling for CSV files
        if ext == "csv" {
            highlight_csv(content, config)
        } else if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(ext) {
            // Try syntax highlighting
            highlight_text(content, syntax, config)
        } else {
            // No syntax found, plain text
            // Note: TOML files fall back to plain text as syntect doesn't include TOML by default
            plain_text_lines(content, config)
        }
    } else {
        plain_text_lines(content, config)
    };

    // Apply scroll offset and cursor/visual highlighting
    let scroll_offset = app.preview_scroll_offset();
    let cursor_line = app.preview_cursor_line();
    let is_focused = matches!(app.focused_panel(), FocusedPanel::Preview);
    let visual_mode = app.is_preview_visual_mode();
    let (visual_start, visual_end) = if visual_mode {
        app.get_preview_visual_range()
    } else {
        (0, 0)
    };

    // Check if we should filter lines based on search
    let search_active = app.is_preview_search_active();
    let search_query = app.preview_search_query();
    let should_filter = search_active && !search_query.is_empty();
    let search_results = if should_filter {
        app.preview_search_results()
    } else {
        &[]
    };

    // Calculate available width for padding (subtract borders)
    let available_width = area.width.saturating_sub(2) as usize;

    let lines: Vec<Line> = all_lines
        .into_iter()
        .enumerate()
        .filter(|(line_idx, _)| {
            // If search is active with a query, only show matching lines
            if should_filter {
                search_results.contains(line_idx)
            } else {
                true
            }
        })
        .skip(if should_filter { 0 } else { scroll_offset })
        .map(|(line_idx, mut line)| {
            // Determine if this line should be highlighted
            let should_highlight = is_focused && if visual_mode {
                // In visual mode, highlight all lines in the selection
                line_idx >= visual_start && line_idx <= visual_end
            } else {
                // In normal mode, only highlight the cursor line
                line_idx == cursor_line
            };

            if should_highlight {
                // Calculate current line width (approximate)
                let mut line_width = 0;
                for span in &line.spans {
                    // Count visible characters (approximation)
                    line_width += span.content.chars().count();
                }

                // Apply highlight style to all spans in the line
                for span in &mut line.spans {
                    span.style = span.style.bg(config.colors.selection_bg.to_ratatui_color());
                }

                // Add padding to fill the rest of the line
                if line_width < available_width {
                    let padding = " ".repeat(available_width.saturating_sub(line_width));
                    line.spans.push(Span::styled(
                        padding,
                        Style::default().bg(config.colors.selection_bg.to_ratatui_color())
                    ));
                }
            }

            // Highlight search matches if preview search is active
            if search_active && !search_query.is_empty() {
                let highlight_color = config.colors.accent_search.to_ratatui_color();
                line = highlight_line_matches(line, search_query, highlight_color);
            }

            line
        })
        .collect();

    let mut paragraph = Paragraph::new(lines).block(block);

    // Apply wrapping if enabled
    if app.is_wrap_enabled() {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    frame.render_widget(paragraph, area);
}

fn highlight_text(content: &str, syntax: &syntect::parsing::SyntaxReference, config: &Config) -> Vec<Line<'static>> {
    // Use the Tokyo Night Moon theme
    let mut highlighter = HighlightLines::new(syntax, &THEME);

    // Count total lines for width calculation
    let total_lines = content.lines().count();
    let line_num_width = format!("{}", total_lines).len();

    let mut lines = Vec::new();
    let mut line_number = 1;

    for line in LinesWithEndings::from(content) {
        let ranges = highlighter.highlight_line(line, &SYNTAX_SET).unwrap_or_default();

        // Create line number span
        let line_num_str = format!("{:>width$} │ ", line_number, width = line_num_width);
        let line_num_span = Span::styled(
            line_num_str,
            Style::default().fg(config.colors.text_secondary.to_ratatui_color())
        );

        // Create content spans
        let content_spans: Vec<Span> = ranges
            .into_iter()
            .map(|(style, text)| {
                let fg = syntect_to_ratatui_color(style.foreground);
                Span::styled(text.to_string(), Style::default().fg(fg))
            })
            .collect();

        // Combine line number and content
        let mut all_spans = vec![line_num_span];
        all_spans.extend(content_spans);

        lines.push(Line::from(all_spans));
        line_number += 1;
    }

    lines
}

fn plain_text_lines(content: &str, config: &Config) -> Vec<Line<'static>> {
    let lines_vec: Vec<&str> = content.lines().collect();
    let total_lines = lines_vec.len();
    let line_num_width = format!("{}", total_lines).len();

    lines_vec
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            let line_number = idx + 1;
            let line_num_str = format!("{:>width$} │ ", line_number, width = line_num_width);

            Line::from(vec![
                Span::styled(
                    line_num_str,
                    Style::default().fg(config.colors.text_secondary.to_ratatui_color())
                ),
                Span::styled(
                    line.to_string(),
                    Style::default().fg(config.colors.text_primary.to_ratatui_color())
                ),
            ])
        })
        .collect()
}

fn highlight_csv(content: &str, config: &Config) -> Vec<Line<'static>> {
    let lines_vec: Vec<&str> = content.lines().collect();
    let total_lines = lines_vec.len();
    let line_num_width = format!("{}", total_lines).len();

    // Define colors for different columns (cycle through these)
    let column_colors = [
        config.colors.accent_normal.to_ratatui_color(),     // Cyan
        config.colors.accent_search.to_ratatui_color(),     // Yellow
        config.colors.file_icon_script.to_ratatui_color(),  // Green
        config.colors.file_icon_config.to_ratatui_color(),  // Orange
        config.colors.file_icon_doc.to_ratatui_color(),     // Light blue
    ];

    lines_vec
        .into_iter()
        .enumerate()
        .map(|(idx, line)| {
            let line_number = idx + 1;
            let line_num_str = format!("{:>width$} │ ", line_number, width = line_num_width);

            // Line number span
            let mut spans = vec![
                Span::styled(
                    line_num_str,
                    Style::default().fg(config.colors.text_secondary.to_ratatui_color())
                )
            ];

            // Parse CSV columns (simple comma split for now)
            let columns: Vec<&str> = line.split(',').collect();

            // First line (header) - use bold style
            if idx == 0 {
                for (col_idx, column) in columns.iter().enumerate() {
                    let color = column_colors[col_idx % column_colors.len()];
                    spans.push(Span::styled(
                        column.to_string(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    ));
                    if col_idx < columns.len() - 1 {
                        spans.push(Span::styled(
                            ",",
                            Style::default().fg(config.colors.text_secondary.to_ratatui_color())
                        ));
                    }
                }
            } else {
                // Data rows - normal style
                for (col_idx, column) in columns.iter().enumerate() {
                    let color = column_colors[col_idx % column_colors.len()];
                    spans.push(Span::styled(
                        column.to_string(),
                        Style::default().fg(color)
                    ));
                    if col_idx < columns.len() - 1 {
                        spans.push(Span::styled(
                            ",",
                            Style::default().fg(config.colors.text_secondary.to_ratatui_color())
                        ));
                    }
                }
            }

            Line::from(spans)
        })
        .collect()
}

fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
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
