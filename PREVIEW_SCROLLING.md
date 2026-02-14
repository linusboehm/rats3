# Preview Window Scrolling

## Overview
The preview window now supports scrolling when focused, allowing you to navigate through file contents without switching back to the explorer.

## Usage

### Focusing the Preview Window
- Press `Tab` to toggle focus between explorer and preview windows
- Press `Ctrl+L` to focus the preview window directly (border turns bright cyan)
- Press `Ctrl+H` to focus back to the explorer window directly

### Scrolling Controls (when preview is focused)
- `j` or `Down`: Scroll down one line
- `k` or `Up`: Scroll up one line
- `Ctrl-d`: Scroll down by half a page (~10 lines)
- `Ctrl-u`: Scroll up by half a page (~10 lines)
- `gg`: Jump to the top of the file
- `G` or `End`: Jump to the bottom of the file

### Visual Mode (vim-style line selection)
- `v`: Enter visual mode (starts selection at current line)
- `j`/`k` or `Up`/`Down`: Extend selection up or down
- `y`: Yank (copy) the selected lines to clipboard
- `Escape`: Exit visual mode without copying

### Resizing the Preview Window
- `h`: Increase preview width by 5% (max 80%) - moves divider left
- `l`: Decrease preview width by 5% (min 20%) - moves divider right
- Default width is configurable (default: 50% equal split)

### Visual Indicators
- **Focused border**: The preview window border turns bright cyan when focused
- **Current line highlight**: When the preview is focused, the current cursor line is highlighted with a full-width background color
- **Visual mode**: When in visual mode, the title shows `VISUAL` and all selected lines are highlighted
- **Scroll position**: The title shows current position, e.g., `Preview [15/100 15%]` or `Preview VISUAL [15/100 15%]`
  - First number: current cursor line (highlighted)
  - Second number: total lines in file
  - Percentage: how far through the file you are

## Configuration
All keybindings and the default split ratio are configurable in `~/.config/rats3/config.toml`:

```toml
# Default preview window width percentage (20-80)
preview_width_percent = 50

[key_bindings]
# Toggle focus between explorer and preview
toggle_focus = ["Tab"]

# Focus preview window (direct)
focus_preview = ["Ctrl-l"]

# Focus explorer window (direct)
focus_explorer = ["Ctrl-h"]

# Movement keys (work in both explorer and preview)
move_up = ["Up", "k"]
move_down = ["Down", "j"]
jump_up = ["Ctrl-u"]
jump_down = ["Ctrl-d"]

# Visual mode in preview
preview_visual_mode = ["v"]
yank_selection = ["y"]
```

## Behavior Notes
- Scroll position and cursor line reset when you select a different file
- When the preview is focused, movement keys move the cursor and scroll the preview
- The cursor line is only highlighted when the preview panel is focused
- When the explorer is focused, movement keys navigate the file list as normal
- When the explorer is focused, `h` goes to parent directory and `l` enters a directory
- When the preview is focused, `h` and `l` resize the preview window width
- Search mode and history mode are unaffected by the focus state
- The viewport automatically follows the cursor to keep it visible
- Visual mode selection is preserved as you move up and down
- Visual mode automatically exits after copying with `y`
- You can also exit visual mode without copying by pressing `Escape`
- Preview width persists for the session but resets to the configured default on restart
- You can set the default preview width in the config file with `preview_width_percent` (20-80)
- History mode displays as a centered overlay (80% width, ~30 lines) on top of the explorer/preview
- Fuzzy search works in history mode - press `/` to filter history entries by typing
