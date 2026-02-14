use crate::backend::{Backend, Entry, ListResult, PreviewContent};
use crate::fuzzy::FuzzyMatcher;
use crate::status::StatusMessage;
use anyhow::Result;
use crossterm::event::KeyEvent;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// Events that can occur in the application
#[derive(Debug)]
pub enum AppEvent {
    /// Key press from terminal
    KeyPress(KeyEvent),
    /// Directory listing result
    ListResult(Result<ListResult>),
    /// Preview content ready
    PreviewReady(String, PreviewContent),
    /// Quit signal
    Quit,
}

/// Download status information for a single file
#[derive(Debug)]
pub struct DownloadInfo {
    pub path: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub status: DownloadState,
    pub completed_at: Option<std::time::Instant>,
    pub cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadState {
    InProgress,
    Complete,
    Canceled,
    Error(String),
}

/// Application mode
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Normal browsing mode (navigation)
    Normal,
    /// Search/filter mode (typing search query)
    Search,
    /// Visual selection mode (selecting file spans)
    Visual,
    /// History browsing
    History,
    /// Download destination selection
    Download,
}

/// Focused panel
#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPanel {
    /// Explorer window (file list)
    Explorer,
    /// Preview window
    Preview,
}

/// Main application state
pub struct App {
    /// Backend for storage operations
    backend: Arc<dyn Backend>,
    /// Current prefix/path
    current_prefix: String,
    /// Current directory entries
    entries: Vec<Entry>,
    /// Filtered entries (after fuzzy search)
    filtered_entries: Vec<usize>,
    /// Currently selected index in filtered list
    selected_index: usize,
    /// Search query
    search_query: String,
    /// Current mode
    mode: AppMode,
    /// Should quit
    should_quit: bool,
    /// Status message
    status_message: Option<StatusMessage>,
    /// Fuzzy matcher
    fuzzy_matcher: FuzzyMatcher,
    /// Preview cache (path -> content)
    preview_cache: HashMap<String, PreviewContent>,
    /// Currently displayed preview path
    current_preview_path: Option<String>,
    /// Pending key for multi-key sequences (e.g., waiting for second 'g' in 'gg')
    pending_key: Option<char>,
    /// History of visited paths (most recent first)
    history: Vec<String>,
    /// Filtered history indices (after fuzzy search)
    filtered_history: Vec<usize>,
    /// Selected index in history mode
    history_selected_index: usize,
    /// Whether we're searching within history (vs searching files)
    searching_history: bool,
    /// Whether to wrap text in preview
    wrap_text: bool,
    /// Currently focused panel
    focused_panel: FocusedPanel,
    /// Preview scroll offset (number of lines scrolled)
    preview_scroll_offset: usize,
    /// Preview cursor line (highlighted line in preview)
    preview_cursor_line: usize,
    /// Whether preview is in visual mode
    preview_visual_mode: bool,
    /// Visual mode selection start line
    preview_visual_start: usize,
    /// Preview window width percentage (0-100)
    preview_width_percent: u16,
    /// Selected file indices (for multi-file selection)
    selected_files: HashSet<usize>,
    /// Visual selection mode start index
    visual_start_index: Option<usize>,
    /// Selected download destination index
    download_destination_index: usize,
    /// Active and recent downloads (file path -> download info)
    downloads: HashMap<String, DownloadInfo>,
    /// Whether to show help/keyboard shortcuts
    show_help: bool,
    /// Whether preview search mode is active
    preview_search_active: bool,
    /// Preview search query
    preview_search_query: String,
    /// Preview search results (line numbers)
    preview_search_results: Vec<usize>,
    /// Currently selected search result index
    preview_search_selected: usize,
}

impl App {
    pub fn new(backend: Arc<dyn Backend>, initial_prefix: String, preview_width_percent: u16) -> Self {
        // Clamp preview width to valid range
        let preview_width = preview_width_percent.clamp(20, 80);

        Self {
            backend,
            current_prefix: initial_prefix,
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            mode: AppMode::Normal,
            should_quit: false,
            status_message: None,
            fuzzy_matcher: FuzzyMatcher::new(),
            preview_cache: HashMap::new(),
            current_preview_path: None,
            pending_key: None,
            history: Vec::new(),
            filtered_history: Vec::new(),
            history_selected_index: 0,
            searching_history: false,
            wrap_text: false,
            focused_panel: FocusedPanel::Explorer,
            preview_scroll_offset: 0,
            preview_cursor_line: 0,
            preview_visual_mode: false,
            preview_visual_start: 0,
            preview_width_percent: preview_width,
            selected_files: HashSet::new(),
            visual_start_index: None,
            download_destination_index: 0,
            downloads: HashMap::new(),
            show_help: false,
            preview_search_active: false,
            preview_search_query: String::new(),
            preview_search_results: Vec::new(),
            preview_search_selected: 0,
        }
    }

    /// Check if the app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Set quit flag
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get current entries
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Get filtered entry indices
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_entries
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get current prefix
    pub fn current_prefix(&self) -> &str {
        &self.current_prefix
    }

    /// Get search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Get current mode
    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    /// Get status message
    pub fn status_message(&self) -> Option<&StatusMessage> {
        self.status_message.as_ref()
    }

    /// Set status message with explicit severity
    pub fn set_status(&mut self, message: StatusMessage) {
        self.status_message = Some(message);
    }

    /// Show an info message (default status)
    pub fn show_info(&mut self, message: impl Into<String>) {
        self.status_message = Some(StatusMessage::info(message));
    }

    /// Show a success message
    pub fn show_success(&mut self, message: impl Into<String>) {
        self.status_message = Some(StatusMessage::success(message));
    }

    /// Show a warning message
    pub fn show_warning(&mut self, message: impl Into<String>) {
        self.status_message = Some(StatusMessage::warning(message));
    }

    /// Show an error message
    pub fn show_error(&mut self, message: impl Into<String>) {
        self.status_message = Some(StatusMessage::error(message));
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Clear status message if it's been shown for more than the given duration
    pub fn clear_status_if_expired(&mut self, timeout_secs: u64) {
        if let Some(msg) = &self.status_message {
            if msg.is_expired(std::time::Duration::from_secs(timeout_secs)) {
                self.clear_status();
            }
        }
    }

    /// Update entries from listing result
    pub fn update_entries(&mut self, result: ListResult) {
        self.entries = result.entries;
        self.current_prefix = result.prefix;
        self.apply_filter();
        // Clear selections when navigating to a new directory
        self.clear_selection();
    }

    /// Update entries and select a specific entry by name
    pub fn update_entries_and_select(&mut self, result: ListResult, select_name: &str) {
        self.entries = result.entries;
        self.current_prefix = result.prefix;
        self.apply_filter();

        // Find the entry with the given name and select it
        for (filtered_idx, &entry_idx) in self.filtered_entries.iter().enumerate() {
            if let Some(entry) = self.entries.get(entry_idx) {
                if entry.name == select_name {
                    self.selected_index = filtered_idx;
                    break;
                }
            }
        }
    }

    /// Update search query and re-filter
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;

        // Apply filter to either history or entries depending on what we're searching
        if self.searching_history || self.mode == AppMode::History {
            self.apply_history_filter();
        } else {
            self.apply_filter();
        }
    }

    /// Apply fuzzy filter to entries
    fn apply_filter(&mut self) {
        let entry_names: Vec<String> = self.entries.iter().map(|e| e.name.clone()).collect();
        self.filtered_entries = self.fuzzy_matcher.match_entries(&entry_names, &self.search_query);

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_entries.len() {
            self.selected_index = 0;
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if !self.filtered_entries.is_empty() && self.selected_index < self.filtered_entries.len() - 1 {
            self.selected_index += 1;
        }
    }

    /// Jump up by multiple items
    pub fn jump_up(&mut self, count: usize) {
        self.selected_index = self.selected_index.saturating_sub(count);
    }

    /// Jump down by multiple items
    pub fn jump_down(&mut self, count: usize) {
        let max = self.filtered_entries.len().saturating_sub(1);
        self.selected_index = (self.selected_index + count).min(max);
    }

    /// Jump to bottom
    pub fn jump_to_bottom(&mut self) {
        if !self.filtered_entries.is_empty() {
            self.selected_index = self.filtered_entries.len() - 1;
        }
    }

    /// Jump to top
    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
    }

    /// Set pending key for multi-key sequences
    pub fn set_pending_key(&mut self, key: char) {
        self.pending_key = Some(key);
    }

    /// Get pending key
    pub fn pending_key(&self) -> Option<char> {
        self.pending_key
    }

    /// Clear pending key
    pub fn clear_pending_key(&mut self) {
        self.pending_key = None;
    }

    /// Add path to history (most recent first, avoid duplicates)
    pub fn add_to_history(&mut self, path: String) {
        // Remove existing entry if present
        self.history.retain(|p| p != &path);
        // Add to front
        self.history.insert(0, path);
        // Keep only last 100 entries
        if self.history.len() > 100 {
            self.history.truncate(100);
        }
    }

    /// Get history entries
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Get filtered history indices
    pub fn filtered_history(&self) -> &[usize] {
        &self.filtered_history
    }

    /// Load history from state
    pub fn load_history(&mut self, history: Vec<String>) {
        self.history = history;
        // Keep only last 100 entries
        if self.history.len() > 100 {
            self.history.truncate(100);
        }
        self.apply_history_filter();
    }

    /// Apply fuzzy filter to history entries
    fn apply_history_filter(&mut self) {
        self.filtered_history = self.fuzzy_matcher.match_entries(&self.history, &self.search_query);

        // Reset selection if out of bounds
        if self.history_selected_index >= self.filtered_history.len() {
            self.history_selected_index = 0;
        }
    }

    /// Get selected history index
    pub fn history_selected_index(&self) -> usize {
        self.history_selected_index
    }

    /// Move up in history
    pub fn history_move_up(&mut self) {
        if self.history_selected_index > 0 {
            self.history_selected_index -= 1;
        }
    }

    /// Move down in history
    pub fn history_move_down(&mut self) {
        if !self.filtered_history.is_empty() && self.history_selected_index < self.filtered_history.len() - 1 {
            self.history_selected_index += 1;
        }
    }

    /// Get selected history entry
    pub fn selected_history_entry(&self) -> Option<&String> {
        self.filtered_history
            .get(self.history_selected_index)
            .and_then(|&idx| self.history.get(idx))
    }

    /// Enter history mode
    pub fn enter_history_mode(&mut self) {
        self.mode = AppMode::History;
        self.history_selected_index = 0;
        self.searching_history = false;
        self.search_query.clear();
        self.apply_history_filter();
    }

    /// Exit history mode
    pub fn exit_history_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.searching_history = false;
        self.search_query.clear();
    }

    /// Get selected entry
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.filtered_entries
            .get(self.selected_index)
            .and_then(|&idx| self.entries.get(idx))
    }

    /// Navigate into selected directory or get parent
    /// Returns (new_prefix, optional_name_to_select)
    pub fn navigate(&mut self, direction: NavigateDirection) -> Option<(String, Option<String>)> {
        match direction {
            NavigateDirection::Into => {
                let entry = self.selected_entry()?;
                if entry.is_dir {
                    let new_prefix = if self.current_prefix.is_empty() {
                        entry.name.clone()
                    } else {
                        format!("{}/{}", self.current_prefix, entry.name)
                    };
                    Some((new_prefix, None))
                } else {
                    None
                }
            }
            NavigateDirection::Up => {
                // Extract the last directory name to select after going up
                let current_dir_name = if self.current_prefix.is_empty() {
                    None
                } else {
                    // Get the last component of the path
                    self.current_prefix
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .last()
                        .map(|s| s.to_string())
                };

                self.backend
                    .get_parent(&self.current_prefix)
                    .map(|parent| (parent, current_dir_name))
            }
        }
    }

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        // If we're in history mode, remember that we're searching history
        self.searching_history = self.mode == AppMode::History;
        self.mode = AppMode::Search;
    }

    /// Exit search mode and clear search
    pub fn exit_search_mode(&mut self) {
        // If we were searching history, go back to history mode
        if self.searching_history {
            self.mode = AppMode::History;
            self.searching_history = false;
        } else {
            self.mode = AppMode::Normal;
        }
        self.set_search_query(String::new());
    }

    /// Check if in search mode
    pub fn is_search_mode(&self) -> bool {
        self.mode == AppMode::Search
    }

    /// Check if searching history
    pub fn is_searching_history(&self) -> bool {
        self.searching_history
    }

    /// Append character to search query (only in search mode)
    pub fn append_search_char(&mut self, c: char) {
        if self.is_search_mode() {
            self.search_query.push(c);

            // Apply filter to either history or entries depending on what we're searching
            if self.searching_history {
                self.apply_history_filter();
            } else {
                self.apply_filter();
            }
        }
    }

    /// Remove last character from search query (only in search mode)
    pub fn backspace_search(&mut self) {
        if self.is_search_mode() {
            self.search_query.pop();

            // Apply filter to either history or entries depending on what we're searching
            if self.searching_history {
                self.apply_history_filter();
            } else {
                self.apply_filter();
            }
        }
    }

    /// Get backend reference
    pub fn backend(&self) -> &Arc<dyn Backend> {
        &self.backend
    }

    /// Get the path for the currently selected file (for preview)
    pub fn get_selected_file_path(&self) -> Option<String> {
        let entry = self.selected_entry()?;
        if entry.is_dir {
            return None; // Don't preview directories
        }
        Some(if self.current_prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", self.current_prefix, entry.name)
        })
    }

    /// Set preview content for a path
    pub fn set_preview(&mut self, path: String, content: PreviewContent) {
        self.preview_cache.insert(path.clone(), content);
        self.current_preview_path = Some(path);
        self.reset_preview_scroll();
    }

    /// Get current preview content
    pub fn get_preview(&self) -> Option<&PreviewContent> {
        self.current_preview_path.as_ref()
            .and_then(|path| self.preview_cache.get(path))
    }

    /// Check if preview needs loading for current selection
    /// Returns (path, needs_loading) - path is always returned if available
    pub fn needs_preview_load(&self) -> Option<(String, bool)> {
        let path = self.get_selected_file_path()?;
        let needs_loading = !self.preview_cache.contains_key(&path);
        Some((path, needs_loading))
    }

    /// Update current preview path (for cached items)
    pub fn update_current_preview_path(&mut self, path: String) {
        if self.preview_cache.contains_key(&path) {
            self.current_preview_path = Some(path);
            self.reset_preview_scroll();
        }
    }

    /// Clear current preview (when directory is selected)
    pub fn clear_preview(&mut self) {
        self.current_preview_path = None;
        self.reset_preview_scroll();
    }

    /// Toggle text wrapping in preview
    pub fn toggle_wrap(&mut self) {
        self.wrap_text = !self.wrap_text;
    }

    /// Check if text wrapping is enabled
    pub fn is_wrap_enabled(&self) -> bool {
        self.wrap_text
    }

    /// Get currently focused panel
    pub fn focused_panel(&self) -> &FocusedPanel {
        &self.focused_panel
    }

    /// Focus the preview panel
    pub fn focus_preview(&mut self) {
        self.focused_panel = FocusedPanel::Preview;
    }

    /// Focus the explorer panel
    pub fn focus_explorer(&mut self) {
        self.focused_panel = FocusedPanel::Explorer;
    }

    /// Toggle focus between explorer and preview
    pub fn toggle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Explorer => FocusedPanel::Preview,
            FocusedPanel::Preview => FocusedPanel::Explorer,
        };
    }

    /// Toggle selection of currently selected file (ignore directories)
    pub fn toggle_selection(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }

        let filtered_idx = self.selected_index;
        if let Some(&entry_idx) = self.filtered_entries.get(filtered_idx) {
            if let Some(entry) = self.entries.get(entry_idx) {
                // Ignore directories
                if entry.is_dir {
                    self.show_warning("Cannot select directories");
                    return;
                }

                // Toggle selection
                if self.selected_files.contains(&entry_idx) {
                    self.selected_files.remove(&entry_idx);
                } else {
                    self.selected_files.insert(entry_idx);
                }
            }
        }
    }

    /// Enter visual selection mode
    pub fn enter_visual_mode(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }

        let filtered_idx = self.selected_index;
        if let Some(&entry_idx) = self.filtered_entries.get(filtered_idx) {
            if let Some(entry) = self.entries.get(entry_idx) {
                // Ignore directories
                if entry.is_dir {
                    self.show_warning("Cannot select directories");
                    return;
                }

                self.mode = AppMode::Visual;
                self.visual_start_index = Some(filtered_idx);
                self.selected_files.insert(entry_idx);
                self.show_info("-- VISUAL --");
            }
        }
    }

    /// Exit visual selection mode
    pub fn exit_visual_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.visual_start_index = None;
        self.clear_status();
    }

    /// Update visual selection when moving cursor
    pub fn update_visual_selection(&mut self) {
        if self.mode != AppMode::Visual {
            return;
        }

        let Some(start_idx) = self.visual_start_index else {
            return;
        };

        // Clear previous visual selection
        self.selected_files.clear();

        // Select all files in range from start to current
        let current_idx = self.selected_index;
        let (from, to) = if start_idx <= current_idx {
            (start_idx, current_idx)
        } else {
            (current_idx, start_idx)
        };

        for filtered_idx in from..=to {
            if let Some(&entry_idx) = self.filtered_entries.get(filtered_idx) {
                if let Some(entry) = self.entries.get(entry_idx) {
                    // Only select files, not directories
                    if !entry.is_dir {
                        self.selected_files.insert(entry_idx);
                    }
                }
            }
        }
    }

    /// Get list of selected file paths
    pub fn get_selected_file_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for &entry_idx in &self.selected_files {
            if let Some(entry) = self.entries.get(entry_idx) {
                if !entry.is_dir {
                    let full_path = if self.current_prefix.is_empty() {
                        entry.name.clone()
                    } else {
                        format!("{}/{}", self.current_prefix, entry.name)
                    };
                    paths.push(full_path);
                }
            }
        }
        paths.sort();
        paths
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_files.clear();
        self.visual_start_index = None;
    }

    /// Check if a file is selected
    pub fn is_file_selected(&self, entry_idx: usize) -> bool {
        self.selected_files.contains(&entry_idx)
    }

    /// Get number of selected files
    pub fn selected_count(&self) -> usize {
        self.selected_files.len()
    }

    /// Enter download mode
    pub fn enter_download_mode(&mut self) {
        // Can only download if files are selected
        if self.selected_files.is_empty() {
            return;
        }
        self.mode = AppMode::Download;
        self.download_destination_index = 0;
    }

    /// Exit download mode
    pub fn exit_download_mode(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Move up in download destination list
    pub fn download_move_up(&mut self) {
        if self.download_destination_index > 0 {
            self.download_destination_index -= 1;
        }
    }

    /// Move down in download destination list
    pub fn download_move_down(&mut self, max: usize) {
        if self.download_destination_index < max.saturating_sub(1) {
            self.download_destination_index += 1;
        }
    }

    /// Get selected download destination index
    pub fn download_destination_index(&self) -> usize {
        self.download_destination_index
    }

    /// Get preview scroll offset
    pub fn preview_scroll_offset(&self) -> usize {
        self.preview_scroll_offset
    }

    /// Get preview cursor line
    pub fn preview_cursor_line(&self) -> usize {
        self.preview_cursor_line
    }

    /// Scroll preview up by one line
    pub fn preview_scroll_up(&mut self, _visible_height: usize) {
        if self.preview_cursor_line > 0 {
            self.preview_cursor_line -= 1;
            // Adjust scroll offset if cursor goes above visible area
            if self.preview_cursor_line < self.preview_scroll_offset {
                self.preview_scroll_offset = self.preview_cursor_line;
            }
        }
    }

    /// Scroll preview down by one line
    pub fn preview_scroll_down(&mut self, max_lines: usize, visible_height: usize) {
        if max_lines > 0 && self.preview_cursor_line < max_lines - 1 {
            self.preview_cursor_line += 1;
            // Adjust scroll offset if cursor goes below visible area
            let max_visible_line = self.preview_scroll_offset + visible_height - 1;
            if self.preview_cursor_line > max_visible_line {
                self.preview_scroll_offset = self.preview_cursor_line.saturating_sub(visible_height - 1);
            }
        }
    }

    /// Scroll preview up by page (half screen)
    pub fn preview_scroll_page_up(&mut self, page_size: usize) {
        self.preview_cursor_line = self.preview_cursor_line.saturating_sub(page_size);
        self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(page_size);
    }

    /// Scroll preview down by page (half screen)
    pub fn preview_scroll_page_down(&mut self, page_size: usize, max_lines: usize, visible_height: usize) {
        if max_lines > 0 {
            self.preview_cursor_line = (self.preview_cursor_line + page_size).min(max_lines - 1);
            self.preview_scroll_offset = (self.preview_scroll_offset + page_size).min(max_lines.saturating_sub(visible_height));
        }
    }

    /// Jump to top of preview
    pub fn preview_jump_to_top(&mut self) {
        self.preview_cursor_line = 0;
        self.preview_scroll_offset = 0;
    }

    /// Jump to bottom of preview
    pub fn preview_jump_to_bottom(&mut self, max_lines: usize, visible_height: usize) {
        if max_lines > 0 {
            self.preview_cursor_line = max_lines - 1;
            // Limit to max 4 empty lines at bottom (if file is long enough)
            let max_empty_lines = 4;
            if visible_height > max_empty_lines && max_lines >= visible_height - max_empty_lines {
                self.preview_scroll_offset = max_lines - (visible_height - max_empty_lines);
            } else {
                self.preview_scroll_offset = 0;
            }
        }
    }

    /// Check if preview is in visual mode
    pub fn is_preview_visual_mode(&self) -> bool {
        self.preview_visual_mode
    }

    /// Enter preview visual mode
    pub fn enter_preview_visual_mode(&mut self) {
        self.preview_visual_mode = true;
        self.preview_visual_start = self.preview_cursor_line;
    }

    /// Exit preview visual mode
    pub fn exit_preview_visual_mode(&mut self) {
        self.preview_visual_mode = false;
    }

    /// Get visual selection range (start_line, end_line) - inclusive, sorted
    pub fn get_preview_visual_range(&self) -> (usize, usize) {
        let start = self.preview_visual_start;
        let end = self.preview_cursor_line;
        if start <= end {
            (start, end)
        } else {
            (end, start)
        }
    }

    /// Get preview window width percentage
    pub fn preview_width_percent(&self) -> u16 {
        self.preview_width_percent
    }

    /// Increase preview width
    pub fn increase_preview_width(&mut self) {
        self.preview_width_percent = (self.preview_width_percent + 5).min(80);
    }

    /// Decrease preview width
    pub fn decrease_preview_width(&mut self) {
        self.preview_width_percent = (self.preview_width_percent.saturating_sub(5)).max(20);
    }

    /// Reset preview scroll offset (called when preview content changes)
    pub fn reset_preview_scroll(&mut self) {
        self.preview_scroll_offset = 0;
        self.preview_cursor_line = 0;
        self.preview_visual_mode = false;
        self.preview_visual_start = 0;
    }

    /// Start tracking a download with cancellation support
    pub fn start_download(&mut self, path: String, cancel_tx: tokio::sync::oneshot::Sender<()>) {
        self.downloads.insert(path.clone(), DownloadInfo {
            path,
            downloaded: 0,
            total: None,
            status: DownloadState::InProgress,
            completed_at: None,
            cancel_tx: Some(cancel_tx),
        });
    }

    /// Update download progress
    pub fn update_download(&mut self, path: String, downloaded: u64, total: Option<u64>) {
        if let Some(info) = self.downloads.get_mut(&path) {
            info.downloaded = downloaded;
            info.total = total;
            info.status = DownloadState::InProgress;
        } else {
            // Fallback if start_download wasn't called
            self.downloads.insert(path.clone(), DownloadInfo {
                path,
                downloaded,
                total,
                status: DownloadState::InProgress,
                completed_at: None,
                cancel_tx: None,
            });
        }
    }

    /// Mark download as complete
    pub fn complete_download(&mut self, path: String) {
        if let Some(info) = self.downloads.get_mut(&path) {
            info.status = DownloadState::Complete;
            info.completed_at = Some(std::time::Instant::now());
        }
    }

    /// Mark download as failed
    pub fn fail_download(&mut self, path: String, error: String) {
        if let Some(info) = self.downloads.get_mut(&path) {
            info.status = DownloadState::Error(error);
            info.completed_at = Some(std::time::Instant::now());
        }
    }

    /// Mark download as canceled
    pub fn cancel_download(&mut self, path: String) {
        if let Some(info) = self.downloads.get_mut(&path) {
            info.status = DownloadState::Canceled;
            info.completed_at = Some(std::time::Instant::now());
        }
    }

    /// Cancel all in-progress downloads
    pub fn cancel_all_downloads(&mut self) -> usize {
        let mut canceled_count = 0;

        for info in self.downloads.values_mut() {
            if info.status == DownloadState::InProgress {
                // Send cancel signal if we have the sender
                if let Some(cancel_tx) = info.cancel_tx.take() {
                    let _ = cancel_tx.send(());
                }
                info.status = DownloadState::Canceled;
                info.completed_at = Some(std::time::Instant::now());
                canceled_count += 1;
            }
        }

        canceled_count
    }

    /// Check if any downloads are in progress
    pub fn has_active_downloads(&self) -> bool {
        self.downloads.values().any(|info| info.status == DownloadState::InProgress)
    }

    /// Get all downloads
    pub fn downloads(&self) -> &HashMap<String, DownloadInfo> {
        &self.downloads
    }

    /// Remove expired downloads (completed > 5 seconds ago)
    pub fn remove_expired_downloads(&mut self) {
        let now = std::time::Instant::now();
        self.downloads.retain(|_, info| {
            if let Some(completed_at) = info.completed_at {
                now.duration_since(completed_at).as_secs() < 5
            } else {
                true // Keep in-progress downloads
            }
        });
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Check if help is shown
    pub fn is_help_shown(&self) -> bool {
        self.show_help
    }

    /// Get preview search query
    pub fn preview_search_query(&self) -> &str {
        &self.preview_search_query
    }

    /// Check if preview search is active
    pub fn is_preview_search_active(&self) -> bool {
        self.preview_search_active
    }

    /// Set preview search query and update results
    pub fn set_preview_search_query(&mut self, query: String) {
        self.preview_search_active = true;
        self.preview_search_query = query;
        self.update_preview_search_results();
    }

    /// Append character to preview search query
    pub fn append_preview_search_char(&mut self, c: char) {
        self.preview_search_query.push(c);
        self.update_preview_search_results();
    }

    /// Remove last character from preview search query
    pub fn backspace_preview_search(&mut self) {
        self.preview_search_query.pop();
        self.update_preview_search_results();
    }

    /// Clear preview search
    pub fn clear_preview_search(&mut self) {
        self.preview_search_active = false;
        self.preview_search_query.clear();
        self.preview_search_results.clear();
        self.preview_search_selected = 0;
    }

    /// Update preview search results based on current query
    fn update_preview_search_results(&mut self) {
        self.preview_search_results.clear();
        self.preview_search_selected = 0;

        if self.preview_search_query.is_empty() {
            return;
        }

        // Get preview content and clone it to avoid borrow checker issues
        let content_opt = self.get_preview().and_then(|preview| {
            if let PreviewContent::Text(content) = preview {
                Some(content.clone())
            } else {
                None
            }
        });

        // Search in the cloned content
        if let Some(content) = content_opt {
            let query_lower = self.preview_search_query.to_lowercase();
            for (line_num, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    self.preview_search_results.push(line_num);
                }
            }
        }
    }

    /// Get preview search results
    pub fn preview_search_results(&self) -> &[usize] {
        &self.preview_search_results
    }

    /// Get current preview search result index
    pub fn preview_search_selected(&self) -> usize {
        self.preview_search_selected
    }

    /// Move to next preview search result
    pub fn preview_search_next(&mut self, max_lines: usize, visible_height: usize) {
        if !self.preview_search_results.is_empty() {
            self.preview_search_selected = (self.preview_search_selected + 1) % self.preview_search_results.len();
            self.jump_to_preview_search_result(max_lines, visible_height);
        }
    }

    /// Move to previous preview search result
    pub fn preview_search_prev(&mut self, max_lines: usize, visible_height: usize) {
        if !self.preview_search_results.is_empty() {
            if self.preview_search_selected == 0 {
                self.preview_search_selected = self.preview_search_results.len() - 1;
            } else {
                self.preview_search_selected -= 1;
            }
            self.jump_to_preview_search_result(max_lines, visible_height);
        }
    }

    /// Jump to currently selected search result
    fn jump_to_preview_search_result(&mut self, max_lines: usize, visible_height: usize) {
        if let Some(&line_num) = self.preview_search_results.get(self.preview_search_selected) {
            self.preview_cursor_line = line_num;
            // Center the result in the view if possible, but limit max empty lines at bottom
            let center_offset = line_num.saturating_sub(5);

            // Check if centering would create more than 4 empty lines at bottom
            let max_empty_lines = 4;
            let max_offset = if visible_height > max_empty_lines && max_lines >= visible_height - max_empty_lines {
                max_lines - (visible_height - max_empty_lines)
            } else {
                0
            };

            self.preview_scroll_offset = center_offset.min(max_offset);
        }
    }

    /// Confirm preview search result (jump to it and exit search)
    pub fn confirm_preview_search(&mut self, max_lines: usize, visible_height: usize) {
        self.jump_to_preview_search_result(max_lines, visible_height);
        self.clear_preview_search();
    }
}

/// Navigation direction
#[derive(Debug, Clone, Copy)]
pub enum NavigateDirection {
    Into,
    Up,
}
