use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rats3::{
    app::{App, AppMode, NavigateDirection},
    backend::{local::LocalBackend, Backend, PreviewContent},
    clipboard,
    config::Config,
    events::{handle_key, read_event, Action},
    state::AppState,
    ui,
};
#[cfg(feature = "s3")]
use rats3::backend::s3::S3Backend;
use std::{io, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::mpsc;

/// Progress update messages from download tasks
#[derive(Debug, Clone)]
enum ProgressMessage {
    Update {
        path: String,
        downloaded: u64,
        total: Option<u64>,
    },
    Complete {
        path: String,
    },
    Canceled {
        path: String,
    },
    Error {
        path: String,
        error: String,
    },
}

#[derive(Parser, Debug)]
#[command(name = "rats3")]
#[command(about = "Rust S3 Navigator - Interactive TUI for browsing S3 and local filesystems")]
struct Args {
    /// S3 URI (s3://bucket/prefix) or path to use
    #[arg(value_name = "URI")]
    uri: Option<String>,

    /// Use local filesystem backend (for testing)
    #[arg(long, value_name = "PATH")]
    local: Option<PathBuf>,
}

/// Expand tilde (~) in path to home directory
fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Determine backend and initial prefix
    let (backend, initial_prefix): (Arc<dyn Backend>, String) = if let Some(local_path) = args.local {
        let backend = LocalBackend::new(local_path)?;
        (Arc::new(backend), String::new())
    } else if let Some(uri) = args.uri {
        if uri.starts_with("s3://") {
            #[cfg(feature = "s3")]
            {
                let (bucket, prefix) = S3Backend::from_uri(&uri)?;
                let backend = S3Backend::new(bucket).await?;
                (Arc::new(backend), prefix)
            }
            #[cfg(not(feature = "s3"))]
            {
                println!("S3 support not enabled. Build with --features s3 to enable.");
                std::process::exit(1);
            }
        } else {
            // Treat as local path
            let path = PathBuf::from(&uri);
            let backend = LocalBackend::new(path)?;
            (Arc::new(backend), String::new())
        }
    } else {
        // Load last location from state
        let state = AppState::load()?;
        if let Some(last_location) = state.last_location {
            if last_location.starts_with("s3://") {
                #[cfg(feature = "s3")]
                {
                    let (bucket, prefix) = S3Backend::from_uri(&last_location)?;
                    let backend = S3Backend::new(bucket).await?;
                    (Arc::new(backend), prefix)
                }
                #[cfg(not(feature = "s3"))]
                {
                    println!("Last location was S3 but S3 support not enabled.");
                    println!("Build with --features s3 or provide a local path.");
                    std::process::exit(1);
                }
            } else {
                println!("No URI provided and no valid last location found.");
                println!("Usage: rats3 [s3://bucket/prefix] or rats3 --local /path/to/dir");
                std::process::exit(1);
            }
        } else {
            println!("No URI provided. Please specify an S3 URI or local path.");
            println!("Usage: rats3 [s3://bucket/prefix] or rats3 --local /path/to/dir");
            std::process::exit(1);
        }
    };

    // Load config
    let (config, config_error) = match Config::load() {
        Ok(config) => (config, None),
        Err(e) => {
            eprintln!("Warning: Failed to load config, using defaults: {:#}", e);
            (Config::default(), Some(format!("{:#}", e)))
        }
    };

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend_term = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend_term)?;

    // Run app
    let app_result = run_app(&mut terminal, backend.clone(), initial_prefix, config, config_error).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Save state before exiting (even if there was an error)
    if let Ok((app, final_backend)) = &app_result {
        let mut state = AppState::load().unwrap_or_default();
        state.set_last_location(final_backend.get_display_path(app.current_prefix()));
        state.set_history(app.history().to_vec());
        let _ = state.save();
    }

    app_result.map(|_| ())
}

/// Check if a path should be added to history
/// Filters out paths ending in just numbers (e.g., "folder/8323")
fn should_add_to_history(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // Get the last component of the path
    let last_component = path
        .trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .last();

    if let Some(component) = last_component {
        // Check if it's all digits
        !component.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Create a backend from a full display URI (e.g. "s3://bucket/prefix").
/// Returns the backend and the bare prefix to pass to list().
async fn create_backend_from_uri(uri: &str) -> Result<(Arc<dyn Backend>, String)> {
    if uri.starts_with("s3://") {
        #[cfg(feature = "s3")]
        {
            let (bucket, prefix) = S3Backend::from_uri(uri)?;
            let backend = S3Backend::new(bucket).await?;
            return Ok((Arc::new(backend), prefix));
        }
        #[cfg(not(feature = "s3"))]
        anyhow::bail!("S3 support not enabled (build with --features s3)");
    }
    anyhow::bail!("Unsupported URI scheme: {}", uri)
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    backend: Arc<dyn Backend>,
    initial_prefix: String,
    config: Config,
    config_error: Option<String>,
) -> Result<(App, Arc<dyn Backend>)> {
    let mut backend = backend;
    let mut app = App::new(backend.clone(), initial_prefix.clone(), config.preview_width_percent);

    // Load history from state
    if let Ok(state) = AppState::load() {
        app.load_history(state.history);
    }

    // Do initial listing
    match backend.list(&initial_prefix).await {
        Ok(result) => {
            app.update_entries(result);
            // Add initial location to history (if it's not a numeric folder)
            if should_add_to_history(&initial_prefix) {
                app.add_to_history(backend.get_display_path(&initial_prefix));
            }
        }
        Err(e) => {
            app.show_error(format!("Error listing directory: {}", e));
        }
    }

    // Show config error if there was one
    if let Some(error) = config_error {
        app.show_warning(format!("Config file error (using defaults): {}", error));
    }

    // Load initial preview
    if let Some((path, needs_loading)) = app.needs_preview_load() {
        if needs_loading {
            let backend_clone = backend.clone();
            let max_size = config.preview_max_size;
            tokio::spawn(async move {
                // Preview loading happens in background
                let _ = backend_clone.get_preview(&path, max_size).await;
            });
        } else {
            app.update_current_preview_path(path);
        }
    }

    // Create channel for download progress updates
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressMessage>();

    // Main event loop
    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, &app, &config))?;

        // Check if should quit
        if app.should_quit() {
            break;
        }

        // Clear expired status messages
        app.clear_status_if_expired(config.status_message_timeout_secs);

        // Remove expired downloads (completed > 5s ago)
        app.remove_expired_downloads();

        // Process download progress messages
        while let Ok(msg) = progress_rx.try_recv() {
            match msg {
                ProgressMessage::Update { path, downloaded, total } => {
                    app.update_download(path, downloaded, total);
                }
                ProgressMessage::Complete { path } => {
                    app.complete_download(path.clone());

                    // Check if all downloads are complete
                    let downloads = app.downloads();
                    let all_complete = downloads.values().all(|d| {
                        d.status == rats3::app::DownloadState::Complete ||
                        d.status == rats3::app::DownloadState::Canceled ||
                        matches!(d.status, rats3::app::DownloadState::Error(_))
                    });

                    if all_complete {
                        let completed = downloads.values()
                            .filter(|d| d.status == rats3::app::DownloadState::Complete)
                            .count();
                        let failed = downloads.values()
                            .filter(|d| matches!(d.status, rats3::app::DownloadState::Error(_)))
                            .count();
                        let canceled = downloads.values()
                            .filter(|d| d.status == rats3::app::DownloadState::Canceled)
                            .count();

                        if canceled > 0 {
                            app.show_info(format!("Canceled {} download(s)", canceled));
                        } else if failed > 0 {
                            app.show_warning(format!("Downloaded {} file(s), {} failed", completed, failed));
                        } else {
                            app.show_success(format!("Downloaded {} file(s)", completed));
                        }
                    }
                }
                ProgressMessage::Canceled { path } => {
                    if let Some(info) = app.downloads().get(&path) {
                        if info.status == rats3::app::DownloadState::InProgress {
                            // Only mark as canceled if still in progress (not already complete/error)
                            app.cancel_download(path);
                        }
                    }
                }
                ProgressMessage::Error { path, error } => {
                    app.fail_download(path.clone(), error.clone());
                }
            }
        }

        // Read events with timeout
        if let Some(event) = read_event(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = event {
                let in_history_mode = app.mode() == &AppMode::History;
                let in_visual_mode = app.mode() == &AppMode::Visual;
                let in_download_mode = app.mode() == &AppMode::Download;
                let preview_focused = matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview);
                let preview_visual_mode = app.is_preview_visual_mode();
                let preview_search_mode = app.is_preview_search_active();

                // Check if Escape is pressed while downloads are active (not in a modal mode)
                let action = if matches!(key.code, crossterm::event::KeyCode::Esc)
                    && !app.is_search_mode()
                    && !in_history_mode
                    && !in_download_mode
                    && !in_visual_mode
                    && !preview_visual_mode
                    && !preview_search_mode
                    && app.has_active_downloads() {
                    Action::CancelDownloads
                } else {
                    handle_key(key, &config.key_bindings, app.is_search_mode(), in_history_mode, in_visual_mode, in_download_mode, preview_focused, preview_visual_mode, preview_search_mode, app.pending_key())
                };
                match action {
                    Action::Quit => {
                        app.quit();
                    }
                    Action::MoveUp => {
                        app.clear_pending_key();
                        if app.mode() == &AppMode::Download {
                            app.download_move_up();
                        } else if app.mode() == &AppMode::History || app.is_searching_history() {
                            app.history_move_up();
                        } else if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            // Calculate visible height (terminal height / 2 for preview, minus borders)
                            let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                            app.preview_scroll_up(visible_height);
                        } else {
                            app.move_up();
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::MoveDown => {
                        app.clear_pending_key();
                        if app.mode() == &AppMode::Download {
                            app.download_move_down(config.download_destinations.len());
                        } else if app.mode() == &AppMode::History || app.is_searching_history() {
                            app.history_move_down();
                        } else if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            // Calculate max lines from preview content and visible height
                            if let Some(preview) = app.get_preview() {
                                let max_lines = match preview {
                                    rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                    _ => 0,
                                };
                                let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                                app.preview_scroll_down(max_lines, visible_height);
                            }
                        } else {
                            app.move_down();
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::JumpUp(count) => {
                        app.clear_pending_key();
                        if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            app.preview_scroll_page_up(count);
                        } else {
                            app.jump_up(count);
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::JumpDown(count) => {
                        app.clear_pending_key();
                        if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            // Calculate max lines from preview content and visible height
                            if let Some(preview) = app.get_preview() {
                                let max_lines = match preview {
                                    rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                    _ => 0,
                                };
                                let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                                app.preview_scroll_page_down(count, max_lines, visible_height);
                            }
                        } else {
                            app.jump_down(count);
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::JumpToBottom => {
                        app.clear_pending_key();
                        if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            // Calculate max lines from preview content and visible height
                            if let Some(preview) = app.get_preview() {
                                let max_lines = match preview {
                                    rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                    _ => 0,
                                };
                                let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                                app.preview_jump_to_bottom(max_lines, visible_height);
                            }
                        } else {
                            app.jump_to_bottom();
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::JumpToTop => {
                        app.clear_pending_key();
                        if matches!(app.focused_panel(), rats3::app::FocusedPanel::Preview) {
                            app.preview_jump_to_top();
                        } else {
                            app.jump_to_top();
                            // Update visual selection if in visual mode
                            if app.mode() == &AppMode::Visual {
                                app.update_visual_selection();
                            }
                            // Load preview for new selection
                            load_preview_if_needed(&mut app, &backend, &config).await;
                        }
                    }
                    Action::NavigateInto => {
                        app.clear_pending_key();

                        // Handle history mode - select entry and navigate
                        // Check both History mode and Search mode with searching_history flag
                        if app.mode() == &AppMode::History || (app.is_search_mode() && app.is_searching_history()) {
                            if let Some(selected_uri) = app.selected_history_entry().map(|s| s.clone()) {
                                let nav_prefix = if let Some(prefix) = backend.uri_to_prefix(&selected_uri) {
                                    // Same backend
                                    Some(prefix)
                                } else {
                                    // Different backend — try to switch
                                    match create_backend_from_uri(&selected_uri).await {
                                        Ok((new_backend, prefix)) => {
                                            backend = new_backend;
                                            app.set_backend(backend.clone());
                                            Some(prefix)
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Cannot switch backend: {}", e));
                                            None
                                        }
                                    }
                                };

                                if let Some(nav_prefix) = nav_prefix {
                                    app.exit_history_mode();
                                    match backend.list(&nav_prefix).await {
                                        Ok(result) => {
                                            app.update_entries(result);
                                            app.clear_status();
                                            // Re-add to history to bump it to the top
                                            if should_add_to_history(&nav_prefix) {
                                                app.add_to_history(backend.get_display_path(&nav_prefix));
                                            }
                                            // Load preview for first item
                                            load_preview_if_needed(&mut app, &backend, &config).await;
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Error: {}", e));
                                        }
                                    }
                                }
                            }
                        } else {
                            // Check if selected item is a file or directory
                            let is_file = app.selected_entry().map(|e| !e.is_dir).unwrap_or(false);

                            if is_file {
                                // If it's a file, focus the preview window
                                app.focus_preview();

                                // Exit search mode if we were in it
                                if app.is_search_mode() {
                                    app.exit_search_mode();
                                }
                            } else {
                                // Get the navigation target BEFORE exiting search mode
                                // (otherwise the selection index will be wrong)
                                let nav_result = app.navigate(NavigateDirection::Into);

                                // Exit search mode when navigating
                                let was_in_search = app.is_search_mode();
                                if was_in_search {
                                    app.exit_search_mode();
                                }

                                if let Some((new_prefix, _)) = nav_result {
                                    match backend.list(&new_prefix).await {
                                        Ok(result) => {
                                            app.update_entries(result);
                                            app.clear_status();
                                            // Add to history (skip folders ending in just numbers)
                                            if should_add_to_history(&new_prefix) {
                                                app.add_to_history(backend.get_display_path(&new_prefix));
                                            }
                                            // Load preview for first item
                                            load_preview_if_needed(&mut app, &backend, &config).await;
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Error: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Action::NavigateUp => {
                        app.clear_pending_key();
                        if let Some((new_prefix, select_name)) = app.navigate(NavigateDirection::Up) {
                            match backend.list(&new_prefix).await {
                                Ok(result) => {
                                    if let Some(name) = select_name {
                                        app.update_entries_and_select(result, &name);
                                    } else {
                                        app.update_entries(result);
                                    }
                                    app.clear_status();
                                    // Load preview for selected item
                                    load_preview_if_needed(&mut app, &backend, &config).await;
                                }
                                Err(e) => {
                                    app.show_error(format!("Error: {}", e));
                                }
                            }
                        }
                    }
                    Action::EnterSearchMode => {
                        app.clear_pending_key();
                        app.enter_search_mode();
                        app.clear_status();
                    }
                    Action::ExitSearchMode => {
                        app.clear_pending_key();
                        app.exit_search_mode();
                        app.clear_status();
                    }
                    Action::AppendChar(c) => {
                        app.clear_pending_key();
                        if app.is_preview_search_active() {
                            app.append_preview_search_char(c);
                        } else {
                            app.append_search_char(c);
                        }
                        app.clear_status();
                    }
                    Action::Backspace => {
                        app.clear_pending_key();
                        if app.is_preview_search_active() {
                            app.backspace_preview_search();
                        } else {
                            app.backspace_search();
                        }
                        app.clear_status();
                    }
                    Action::ToggleSelection => {
                        app.clear_pending_key();

                        // If in visual mode, exit it first so individual toggles are preserved
                        if app.mode() == &AppMode::Visual {
                            app.exit_visual_mode();
                        }

                        app.toggle_selection();
                        let count = app.selected_count();
                        if count > 0 {
                            app.show_info(format!("{} file(s) selected", count));
                        } else {
                            app.clear_status();
                        }
                    }
                    Action::EnterVisualMode => {
                        app.clear_pending_key();
                        app.enter_visual_mode();
                    }
                    Action::ExitVisualMode => {
                        app.clear_pending_key();
                        app.exit_visual_mode();
                        let count = app.selected_count();
                        if count > 0 {
                            app.show_info(format!("{} file(s) selected", count));
                        } else {
                            app.clear_status();
                        }
                    }
                    Action::EnterDownloadMode => {
                        app.clear_pending_key();

                        // Exit visual mode if we're in it
                        if app.mode() == &AppMode::Visual {
                            app.exit_visual_mode();
                        }

                        // If no files selected, auto-select the current file
                        if app.selected_count() == 0 {
                            // Check if current selection is a file (not directory)
                            let is_file = app.selected_entry().map(|e| !e.is_dir).unwrap_or(false);

                            if is_file {
                                // Auto-select the current file
                                app.toggle_selection();
                            } else {
                                app.show_warning("Cannot download directories. Select files with Space or 'v' first.");
                            }
                        }

                        // Now check if we have files to download
                        if app.selected_count() == 0 {
                            // Still no files (was a directory or empty)
                            if app.selected_entry().is_none() {
                                app.show_info("No files selected. Select files with Space or 'v' first.");
                            }
                        } else if config.download_destinations.is_empty() {
                            app.show_warning("No download destinations configured. Edit ~/.config/rats3/config.toml");
                        } else {
                            app.enter_download_mode();
                        }
                    }
                    Action::ExitDownloadMode => {
                        app.clear_pending_key();
                        app.exit_download_mode();
                        app.clear_status();
                    }
                    Action::ConfirmDownload => {
                        app.clear_pending_key();
                        let dest_idx = app.download_destination_index();
                        if let Some(destination) = config.download_destinations.get(dest_idx) {
                            let selected_paths = app.get_selected_file_paths();

                            // Expand tilde in destination path
                            let dest_path = expand_tilde(&destination.path);

                            // Check if destination exists, create if needed
                            if let Err(e) = std::fs::create_dir_all(&dest_path) {
                                app.show_error(format!("Failed to create directory {}: {}", dest_path.display(), e));
                                continue;
                            }

                            // Exit download mode
                            app.exit_download_mode();

                            // Download files in background with progress tracking
                            // (progress will be shown in download progress overlay)
                            let backend_clone = backend.clone();
                            let dest_path_clone = dest_path.clone();

                            for file_path in selected_paths.clone() {
                                // Create cancellation channel
                                let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();

                                // Initialize download tracking with cancellation support
                                app.start_download(file_path.clone(), cancel_tx);

                                let backend_clone2 = backend_clone.clone();
                                let dest_path_clone2 = dest_path_clone.clone();
                                let file_path_clone = file_path.clone();
                                let progress_tx_clone = progress_tx.clone();

                                tokio::spawn(async move {
                                    let file_name = file_path_clone.split('/').last().unwrap_or(&file_path_clone);
                                    let target_path = dest_path_clone2.join(file_name);

                                    // Create progress callback
                                    let path_for_callback = file_path_clone.clone();
                                    let tx_for_callback = progress_tx_clone.clone();
                                    let progress_callback = Box::new(move |downloaded: u64, total: Option<u64>| {
                                        let _ = tx_for_callback.send(ProgressMessage::Update {
                                            path: path_for_callback.clone(),
                                            downloaded,
                                            total,
                                        });
                                    });

                                    // Download file with cancellation support
                                    let download_future = backend_clone2.download_file(
                                        &file_path_clone,
                                        &target_path,
                                        Some(progress_callback)
                                    );

                                    tokio::select! {
                                        result = download_future => {
                                            // Download completed (success or error)
                                            if let Err(e) = result {
                                                let _ = progress_tx_clone.send(ProgressMessage::Error {
                                                    path: file_path_clone.clone(),
                                                    error: e.to_string(),
                                                });
                                            } else {
                                                let _ = progress_tx_clone.send(ProgressMessage::Complete {
                                                    path: file_path_clone.clone(),
                                                });
                                            }
                                        }
                                        _ = &mut cancel_rx => {
                                            // Download was canceled
                                            // Try to delete the partial file
                                            let _ = std::fs::remove_file(&target_path);

                                            let _ = progress_tx_clone.send(ProgressMessage::Canceled {
                                                path: file_path_clone.clone(),
                                            });
                                        }
                                    }
                                });
                            }

                            // Clear selection after initiating download
                            app.clear_selection();
                        }
                    }
                    Action::EnterHistoryMode => {
                        app.clear_pending_key();
                        if !app.history().is_empty() {
                            app.enter_history_mode();
                        }
                    }
                    Action::EnterHistoryModeWithSearch => {
                        app.clear_pending_key();
                        if !app.history().is_empty() {
                            app.enter_history_mode();
                            app.enter_search_mode();
                        }
                    }
                    Action::ExitHistoryMode => {
                        app.clear_pending_key();
                        app.exit_history_mode();
                        app.clear_status();
                    }
                    Action::CopyPath => {
                        app.clear_pending_key();
                        let path = backend.get_display_path(app.current_prefix());
                        match clipboard::copy_to_clipboard(&path) {
                            Ok(_) => {
                                app.show_success(format!("Copied to clipboard: {}", path));
                            }
                            Err(e) => {
                                app.show_error(format!("Failed to copy: {}", e));
                            }
                        }
                    }
                    Action::ToggleWrap => {
                        app.clear_pending_key();
                        app.toggle_wrap();
                        let status = if app.is_wrap_enabled() {
                            "Text wrapping enabled"
                        } else {
                            "Text wrapping disabled"
                        };
                        app.show_info(status);
                    }
                    Action::FocusPreview => {
                        app.clear_pending_key();
                        app.focus_preview();
                    }
                    Action::FocusExplorer => {
                        app.clear_pending_key();
                        app.focus_explorer();
                    }
                    Action::ToggleFocus => {
                        app.clear_pending_key();
                        app.toggle_focus();
                    }
                    Action::EnterPreviewVisualMode => {
                        app.clear_pending_key();
                        app.enter_preview_visual_mode();
                    }
                    Action::ExitPreviewVisualMode => {
                        app.clear_pending_key();
                        app.exit_preview_visual_mode();
                    }
                    Action::YankSelection => {
                        app.clear_pending_key();
                        // Get selected lines from preview
                        if let Some(preview) = app.get_preview() {
                            match preview {
                                rats3::backend::PreviewContent::Text(content) => {
                                    let (start, end) = app.get_preview_visual_range();
                                    let lines: Vec<&str> = content.lines().collect();
                                    let selected_lines: Vec<&str> = lines.iter()
                                        .enumerate()
                                        .filter(|(i, _)| *i >= start && *i <= end)
                                        .map(|(_, line)| *line)
                                        .collect();
                                    let selected_text = selected_lines.join("\n");

                                    match clipboard::copy_to_clipboard(&selected_text) {
                                        Ok(_) => {
                                            let line_count = selected_lines.len();
                                            app.show_success(format!("Copied {} line{} to clipboard",
                                                line_count,
                                                if line_count == 1 { "" } else { "s" }));
                                        }
                                        Err(e) => {
                                            app.show_error(format!("Failed to copy: {}", e));
                                        }
                                    }
                                    app.exit_preview_visual_mode();
                                }
                                _ => {}
                            }
                        }
                    }
                    Action::IncreasePreviewWidth => {
                        app.clear_pending_key();
                        app.increase_preview_width();
                    }
                    Action::DecreasePreviewWidth => {
                        app.clear_pending_key();
                        app.decrease_preview_width();
                    }
                    Action::ToggleHelp => {
                        app.clear_pending_key();
                        app.toggle_help();
                    }
                    Action::EnterPreviewSearch => {
                        app.clear_pending_key();
                        app.set_preview_search_query(String::new());
                    }
                    Action::ExitPreviewSearch => {
                        app.clear_pending_key();
                        app.clear_preview_search();
                    }
                    Action::PreviewSearchNext => {
                        app.clear_pending_key();
                        // Calculate max lines and visible height for scroll limit
                        if let Some(preview) = app.get_preview() {
                            let max_lines = match preview {
                                rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                _ => 0,
                            };
                            let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                            app.preview_search_next(max_lines, visible_height);
                        }
                    }
                    Action::PreviewSearchPrev => {
                        app.clear_pending_key();
                        // Calculate max lines and visible height for scroll limit
                        if let Some(preview) = app.get_preview() {
                            let max_lines = match preview {
                                rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                _ => 0,
                            };
                            let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                            app.preview_search_prev(max_lines, visible_height);
                        }
                    }
                    Action::ConfirmPreviewSearch => {
                        app.clear_pending_key();
                        // Calculate max lines and visible height for scroll limit
                        if let Some(preview) = app.get_preview() {
                            let max_lines = match preview {
                                rats3::backend::PreviewContent::Text(content) => content.lines().count(),
                                _ => 0,
                            };
                            let visible_height = terminal.size().unwrap().height.saturating_sub(10) as usize;
                            app.confirm_preview_search(max_lines, visible_height);
                        }
                    }
                    Action::CancelDownloads => {
                        app.clear_pending_key();
                        let canceled = app.cancel_all_downloads();
                        if canceled > 0 {
                            app.show_info(format!("Canceled {} download(s)", canceled));
                        }
                    }
                    Action::PendingKey(c) => {
                        app.set_pending_key(c);
                    }
                    Action::None => {
                        app.clear_pending_key();
                    }
                }
            }
        }
    }

    Ok((app, backend))
}

/// Load preview if needed for current selection
async fn load_preview_if_needed(app: &mut App, backend: &Arc<dyn Backend>, config: &Config) {
    if let Some((path, needs_loading)) = app.needs_preview_load() {
        if needs_loading {
            // Need to fetch preview
            match backend.get_preview(&path, config.preview_max_size).await {
                Ok(content) => {
                    app.set_preview(path, content);
                }
                Err(e) => {
                    app.set_preview(path, PreviewContent::Error(e.to_string()));
                }
            }
        } else {
            // Already cached, just update current preview path
            app.update_current_preview_path(path);
        }
    } else {
        // No file selected (e.g., directory selected) - clear preview
        app.clear_preview();
    }
}
