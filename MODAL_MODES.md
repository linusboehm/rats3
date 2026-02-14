# Modal Editing: Normal Mode vs Search Mode

## Overview

rats3 now uses **modal editing** similar to vim - you're either in Normal mode (for navigation) or Search mode (for filtering files).

## Modes

### Normal Mode (Default)
The default mode for navigating and browsing files.

**Visual Indicators:**
- Border color: **Cyan**
- Title: `" Normal Mode "`
- Prompt: `❯`
- Status: Shows all navigation keys

**Available Actions:**
- **/** - Enter search mode
- **j/k** or **↑↓** - Move up/down
- **J/K** or **Ctrl-D/U** - Jump 10 items
- **G** - Jump to bottom
- **gg** - Jump to top (configurable)
- **Enter/L** - Navigate into directory
- **H/Left** - Navigate to parent
- **S** - Download mode (not yet implemented)
- **R** - History mode (not yet implemented)
- **Y** - Copy path to clipboard
- **Ctrl-C/Q** - Quit

**Key Difference:**
In Normal mode, single letter keys like `s`, `j`, `k`, etc. work for navigation - they DON'T go into the search bar!

### Search Mode
Activated by pressing `/` - used for filtering files.

**Visual Indicators:**
- Border color: **Yellow** (indicates search mode active)
- Title: `" Search Mode "`
- Prompt: `/` (instead of `❯`)
- Cursor: Yellow `█`
- Status: "Type to search | Esc exit search"

**Available Actions:**
- **Type any character** - Add to search query
- **Backspace** - Remove last character
- **Enter** - Navigate into directory and exit search mode
- **Esc** - Exit search mode and clear search
- **Ctrl-C/Q** - Quit (still works)

**Key Difference:**
In Search mode, almost all keys become search input - navigation keys are disabled.

## Usage Examples

### Example 1: Navigate and Search

```
1. Start in Normal mode
   [ /tmp/test_data ❯ Press / to search ]

2. Press 'j' to move down (navigates, doesn't search)
   Selection moves down

3. Press '/' to enter Search mode
   [ /tmp/test_data / █ ]  (yellow border)

4. Type "rust"
   [ /tmp/test_data / rust█ ]  (filters files)

5. Press Enter to select and exit Search mode
   (or press Esc to just exit)
   Back to Normal mode
   [ /tmp/test_data ❯ Press / to search ]
```

### Example 2: Quick Search

```
1. Press '/' to enter Search mode
2. Type "test"
3. See results filtered in real-time
4. Press Esc when done
```

### Example 3: Edit Search

```
1. You have a filter active: "config"
   [ /tmp/test_data ❯ Filtered: config (Press / to edit) ]

2. Press '/' to enter Search mode
   [ /tmp/test_data / config█ ]

3. Type more or backspace to edit
4. Press Esc when done
```

## Benefits

### 1. Cleaner Key Bindings
- **Normal mode**: All letters available for navigation
- No accidental searches when trying to navigate
- Lowercase `s`, `r`, `h`, `l`, `j`, `k` work without conflicts

### 2. Intentional Searching
- Press `/` to explicitly enter search
- Clear visual feedback (yellow border)
- Know when you're searching vs navigating

### 3. Vim-like Workflow
- Familiar for vim users
- Mode-based editing is proven UX
- `/` for search is standard in many tools

### 4. Better Status Display
- Context-appropriate help in status bar
- Normal mode: Shows all navigation options
- Search mode: Shows search-specific help

## Visual Comparison

### Normal Mode:
```
┌─ Normal Mode ─────────────────────────────────────────┐  (Cyan)
│  ❯ Press / to search                                   │
└────────────────────────────────────────────────────────┘
┌─ /tmp/test_data ──────┬───────────────────────────────┐
│  docs                  │      Preview                  │
│❯  src                  │                               │
│  config.toml           │      Preview...               │
└────────────────────────┴───────────────────────────────┘
─────────────────────────────────────────────────────────
 / search  jk move  Enter open  h back  Y copy │ 3/3
```

### Search Mode:
```
┌─ Search Mode ─────────────────────────────────────────┐  (Yellow)
│  / test█                                                │
└────────────────────────────────────────────────────────┘
┌─ /tmp/test_data ──────┬───────────────────────────────┐
│❯  tests                │      Preview                  │
│  test.txt              │                               │
└────────────────────────┴───────────────────────────────┘
─────────────────────────────────────────────────────────
 Type to search  Esc exit search │ 2/3
```

## Configuration

The `/` key to enter search mode is **hardcoded** for consistency.
`Esc` to exit search mode is also **hardcoded**.

All other keys are configurable via `~/.config/rats3/config.toml`, including:
- The `gg` sequence for jumping to top (configurable as `jump_to_top = "gg"`)
- All single-key navigation commands

## Migration from Previous Version

**Before:** Typing any character would immediately filter
**Now:** Press `/` first, then type

If you preferred the old behavior where any key searches, the modal approach is still better because:
1. More predictable (know when you're searching)
2. Cleaner navigation (use lowercase letters freely)
3. Industry standard (`/` for search in vim, less, man pages, etc.)

## Comparison to Other Tools

| Tool | Search Activation |
|------|------------------|
| vim | `/` |
| less | `/` |
| man | `/` |
| fzf | Immediate typing |
| telescope.nvim | Immediate typing |
| **rats3** | **`/` (modal)** |

We chose `/` (like vim/less) over immediate typing (like fzf) because:
- Cleaner navigation with single-letter keys
- Clear visual mode indication
- More powerful key binding options in normal mode
- Familiar to vim users

## Tips

1. **Want to search quickly?** Just press `/` and start typing
2. **Forgot which mode?** Look at the border color (Cyan=Normal, Yellow=Search)
3. **Stuck in search mode?** Press `Esc` or `Enter` (if navigating)
4. **Want to edit your search?** Press `/` to re-enter search mode
5. **Want to clear search?** Press `Esc` in search mode
6. **Navigate after searching?** Press `Enter` - it exits search mode automatically

## Future Enhancements

Possible additions:
- [ ] `n/N` to jump between search results (like vim)
- [ ] `?` for backwards search
- [ ] Search history (recall previous searches)
- [ ] Regex search mode
- [ ] Case-sensitive toggle in search mode
