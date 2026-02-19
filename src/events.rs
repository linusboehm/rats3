use crate::config::KeyBindings;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

/// Read next event with timeout
pub fn read_event(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Handle key event and return action
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    JumpUp(usize),
    JumpDown(usize),
    JumpToBottom,
    JumpToTop,
    NavigateInto,
    NavigateUp,
    EnterSearchMode,
    ExitSearchMode,
    AppendChar(char),
    Backspace,
    ToggleSelection,
    EnterVisualMode,
    ExitVisualMode,
    EnterDownloadMode,
    ExitDownloadMode,
    ConfirmDownload,
    EnterHistoryMode,
    EnterHistoryModeWithSearch,
    ExitHistoryMode,
    CopyPath,
    ToggleWrap,
    FocusPreview,
    FocusExplorer,
    ToggleFocus,
    EnterPreviewVisualMode,
    ExitPreviewVisualMode,
    YankSelection,
    IncreasePreviewWidth,
    DecreasePreviewWidth,
    ToggleHelp,
    EnterPreviewSearch,
    ExitPreviewSearch,
    PreviewSearchNext,
    PreviewSearchPrev,
    ConfirmPreviewSearch,
    CancelDownloads,
    PendingKey(char),
    None,
}

pub fn handle_key(key: KeyEvent, bindings: &KeyBindings, in_search_mode: bool, in_history_mode: bool, in_visual_mode: bool, in_download_mode: bool, preview_focused: bool, preview_visual_mode: bool, preview_search_mode: bool, pending_key: Option<char>) -> Action {
    // Only handle key press events, not release/repeat
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }

    // Always allow quit
    if bindings.is_quit(&key) {
        return Action::Quit;
    }

    // Handle multi-key sequences in normal mode
    if !in_search_mode && !in_history_mode {
        if let Some(pending) = pending_key {
            // We have a pending key, check for completion of sequence
            // Parse the jump_to_top sequence (e.g., "gg")
            let sequence_chars: Vec<char> = bindings.jump_to_top.chars().collect();
            if sequence_chars.len() == 2 {
                let first_char = sequence_chars[0];
                let second_char = sequence_chars[1];

                if pending == first_char && matches!(key.code, KeyCode::Char(c) if c == second_char) {
                    // Sequence completed -> jump to top
                    return Action::JumpToTop;
                }
            }
            // Any other key after pending key - the sequence is broken
            // We'll continue processing this key normally below
        }
    }

    // In search mode, different key handling
    if in_search_mode {
        // Handle multi-key sequence for exit search mode (e.g., "jj")
        if let Some(pending) = pending_key {
            let sequence_chars: Vec<char> = bindings.exit_search_mode.chars().collect();
            if sequence_chars.len() == 2 {
                let first_char = sequence_chars[0];
                let second_char = sequence_chars[1];

                if pending == first_char && matches!(key.code, KeyCode::Char(c) if c == second_char) {
                    // Sequence completed -> exit search mode
                    return Action::ExitSearchMode;
                }
            }
            // Sequence broken, continue processing normally
        }

        // Escape exits search mode
        if matches!(key.code, KeyCode::Esc) {
            return Action::ExitSearchMode;
        }

        // Ctrl+j moves down in search results
        if matches!(key.code, KeyCode::Char('j')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::MoveDown;
        }

        // Ctrl+k moves up in search results
        if matches!(key.code, KeyCode::Char('k')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::MoveUp;
        }

        // Arrow keys also work for navigation in search mode
        if matches!(key.code, KeyCode::Down) {
            return Action::MoveDown;
        }
        if matches!(key.code, KeyCode::Up) {
            return Action::MoveUp;
        }

        // Only Enter (and Right arrow) navigate in search mode, not l/L
        if matches!(key.code, KeyCode::Enter | KeyCode::Right) {
            return Action::NavigateInto;
        }

        // Backspace removes character
        if matches!(key.code, KeyCode::Backspace) {
            return Action::Backspace;
        }

        // Check for start of multi-key sequence for exit search mode
        let sequence_chars: Vec<char> = bindings.exit_search_mode.chars().collect();
        if sequence_chars.len() == 2 && pending_key.is_none() {
            let first_char = sequence_chars[0];
            if matches!(key.code, KeyCode::Char(c) if c == first_char) {
                return Action::PendingKey(first_char);
            }
        }

        // Any printable character adds to search
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::AppendChar(c)
            }
            _ => Action::None,
        }
    } else if in_history_mode {
        // In history mode, limited key handling

        // Escape exits history mode
        if matches!(key.code, KeyCode::Esc) {
            return Action::ExitHistoryMode;
        }

        // Forward slash enters search mode
        if matches!(key.code, KeyCode::Char('/')) {
            return Action::EnterSearchMode;
        }

        // Up/Down navigation
        if bindings.is_move_up(&key) {
            return Action::MoveUp;
        }
        if bindings.is_move_down(&key) {
            return Action::MoveDown;
        }

        // Enter selects history entry and navigates
        if bindings.is_navigate_into(&key) {
            return Action::NavigateInto;
        }

        Action::None
    } else if in_download_mode {
        // Download mode - selecting download destination

        // Escape exits download mode
        if matches!(key.code, KeyCode::Esc) {
            return Action::ExitDownloadMode;
        }

        // Up/Down navigation
        if bindings.is_move_up(&key) {
            return Action::MoveUp;
        }
        if bindings.is_move_down(&key) {
            return Action::MoveDown;
        }

        // Enter confirms download
        if bindings.is_navigate_into(&key) {
            return Action::ConfirmDownload;
        }

        Action::None
    } else if in_visual_mode && !preview_focused {
        // Visual selection mode for file explorer

        // Escape or 'v' exits visual mode
        if matches!(key.code, KeyCode::Esc) {
            return Action::ExitVisualMode;
        }
        if matches!(key.code, KeyCode::Char('v')) && !key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::ExitVisualMode;
        }

        // 's' or download mode key enters download mode with selected files
        if bindings.is_download_mode(&key) {
            return Action::EnterDownloadMode;
        }

        // Space toggles individual file selection
        if matches!(key.code, KeyCode::Char(' ')) {
            return Action::ToggleSelection;
        }

        // Movement keys update selection
        if bindings.is_move_up(&key) {
            return Action::MoveUp;
        }
        if bindings.is_move_down(&key) {
            return Action::MoveDown;
        }
        if bindings.is_jump_up(&key) {
            return Action::JumpUp(10);
        }
        if bindings.is_jump_down(&key) {
            return Action::JumpDown(10);
        }
        if bindings.is_jump_to_bottom(&key) {
            return Action::JumpToBottom;
        }
        // Check for start of multi-key sequences (like gg)
        let sequence_chars: Vec<char> = bindings.jump_to_top.chars().collect();
        if sequence_chars.len() == 2 && pending_key.is_none() {
            let first_char = sequence_chars[0];
            if matches!(key.code, KeyCode::Char(c) if c == first_char) {
                return Action::PendingKey(first_char);
            }
        }

        Action::None
    } else {
        // Normal mode - check all navigation bindings

        // Question mark toggles help
        if matches!(key.code, KeyCode::Char('?')) {
            return Action::ToggleHelp;
        }

        // Forward slash enters search mode (only if preview not focused)
        if !preview_focused && matches!(key.code, KeyCode::Char('/')) {
            return Action::EnterSearchMode;
        }

        // Space toggles selection (only in explorer, not preview)
        if !preview_focused && matches!(key.code, KeyCode::Char(' ')) {
            return Action::ToggleSelection;
        }

        // 'v' enters visual selection mode (only in explorer, not preview)
        if !preview_focused && matches!(key.code, KeyCode::Char('v')) && !key.modifiers.contains(KeyModifiers::CONTROL) {
            return Action::EnterVisualMode;
        }

        // Preview search mode handling
        if preview_search_mode && preview_focused {
            // Escape exits preview search
            if matches!(key.code, KeyCode::Esc) {
                return Action::ExitPreviewSearch;
            }

            // Ctrl+j or Down moves to next result
            if (matches!(key.code, KeyCode::Char('j')) && key.modifiers.contains(KeyModifiers::CONTROL))
                || matches!(key.code, KeyCode::Down)
            {
                return Action::PreviewSearchNext;
            }

            // Ctrl+k or Up moves to previous result
            if (matches!(key.code, KeyCode::Char('k')) && key.modifiers.contains(KeyModifiers::CONTROL))
                || matches!(key.code, KeyCode::Up)
            {
                return Action::PreviewSearchPrev;
            }

            // Enter confirms and jumps to result
            if matches!(key.code, KeyCode::Enter) {
                return Action::ConfirmPreviewSearch;
            }

            // Backspace removes character
            if matches!(key.code, KeyCode::Backspace) {
                return Action::Backspace;
            }

            // Any printable character adds to search
            return match key.code {
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Action::AppendChar(c)
                }
                _ => Action::None,
            };
        } else if preview_focused {
            // Forward slash enters preview search
            if matches!(key.code, KeyCode::Char('/')) && !preview_visual_mode {
                return Action::EnterPreviewSearch;
            }
        }

        // When preview is focused, check for visual mode specific keys
        if preview_focused && !preview_search_mode {
            // In visual mode, Escape exits visual mode
            if preview_visual_mode && matches!(key.code, KeyCode::Esc) {
                return Action::ExitPreviewVisualMode;
            }

            // In visual mode, 'y' yanks the selection
            if preview_visual_mode && bindings.is_yank_selection(&key) {
                return Action::YankSelection;
            }

            // 'v' enters visual mode
            if !preview_visual_mode && bindings.is_preview_visual_mode(&key) {
                return Action::EnterPreviewVisualMode;
            }

            // H/L resize preview width (override navigation when preview focused)
            // H increases (moves divider left, making preview bigger)
            // L decreases (moves divider right, making preview smaller)
            if matches!(key.code, KeyCode::Char('H')) && !key.modifiers.contains(KeyModifiers::CONTROL) {
                return Action::IncreasePreviewWidth;
            }
            if matches!(key.code, KeyCode::Char('L')) && !key.modifiers.contains(KeyModifiers::CONTROL) {
                return Action::DecreasePreviewWidth;
            }

            // Movement keys work in both normal and visual mode
            if bindings.is_move_up(&key) {
                return Action::MoveUp;
            }
            if bindings.is_move_down(&key) {
                return Action::MoveDown;
            }
            if bindings.is_jump_up(&key) {
                return Action::JumpUp(10);
            }
            if bindings.is_jump_down(&key) {
                return Action::JumpDown(10);
            }
        } else {
            // Explorer focused - normal navigation
            if bindings.is_move_up(&key) {
                return Action::MoveUp;
            }
            if bindings.is_move_down(&key) {
                return Action::MoveDown;
            }
            if bindings.is_jump_up(&key) {
                return Action::JumpUp(10);
            }
            if bindings.is_jump_down(&key) {
                return Action::JumpDown(10);
            }
        }
        if bindings.is_jump_to_bottom(&key) {
            return Action::JumpToBottom;
        }
        if bindings.is_navigate_into(&key) {
            return Action::NavigateInto;
        }
        if bindings.is_navigate_up(&key) {
            // If preview is focused, 'h' should just switch to explorer
            // If explorer is already focused, 'h' should navigate up
            if preview_focused {
                return Action::FocusExplorer;
            } else {
                return Action::NavigateUp;
            }
        }
        if bindings.is_download_mode(&key) {
            return Action::EnterDownloadMode;
        }
        if bindings.is_history_mode(&key) {
            return Action::EnterHistoryMode;
        }
        if bindings.is_history_mode_with_search(&key) && !preview_focused {
            return Action::EnterHistoryModeWithSearch;
        }
        if bindings.is_copy_path(&key) {
            return Action::CopyPath;
        }
        if bindings.is_wrap_text(&key) {
            return Action::ToggleWrap;
        }
        if bindings.is_focus_preview(&key) {
            return Action::FocusPreview;
        }
        if bindings.is_focus_explorer(&key) {
            return Action::FocusExplorer;
        }
        if bindings.is_toggle_focus(&key) {
            return Action::ToggleFocus;
        }

        // Check for start of multi-key sequences
        let sequence_chars: Vec<char> = bindings.jump_to_top.chars().collect();
        if sequence_chars.len() == 2 && pending_key.is_none() {
            let first_char = sequence_chars[0];
            if matches!(key.code, KeyCode::Char(c) if c == first_char) {
                return Action::PendingKey(first_char);
            }
        }

        Action::None
    }
}
