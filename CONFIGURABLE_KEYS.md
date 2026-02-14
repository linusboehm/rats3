# Configurable Key Bindings Implementation

## Overview

rats3 now supports fully configurable key bindings through the TOML configuration file. Users can customize every keyboard shortcut to match their preferred workflow (vim, emacs, arrow-only, WASD, or completely custom).

## Implementation Details

### Files Modified

1. **src/config.rs**
   - Added `KeyBindings` struct with all action fields
   - Added default functions for each key binding
   - Implemented `matches_key()` function to parse key strings
   - Implemented `parse_key_code()` to support all special keys
   - Added methods: `is_quit()`, `is_move_up()`, etc. to check bindings

2. **src/events.rs**
   - Modified `handle_key()` to accept `&KeyBindings` parameter
   - Changed from hardcoded key checks to config-based checks
   - Maintained backward compatibility with search input

3. **src/main.rs**
   - Load `Config` at startup with `Config::load()`
   - Pass config to `run_app()`
   - Pass `config.key_bindings` to `handle_key()`
   - Graceful fallback to defaults if config fails to load

4. **config.example.toml**
   - Added complete `[key_bindings]` section
   - Documented all available actions
   - Provided examples of alternative configurations

### Key String Parsing

The implementation supports flexible key string formats:

**Format**: `[Modifier-]Key`

**Examples**:
- Simple: `"k"`, `"j"`, `"q"`
- Special: `"Enter"`, `"Escape"`, `"Up"`
- With modifier: `"Ctrl-c"`, `"Alt-x"`, `"Shift-Tab"`

**Parsing Logic**:
```rust
"Ctrl-c" → KeyEvent { code: Char('c'), modifiers: CONTROL }
"k"      → KeyEvent { code: Char('k'), modifiers: empty }
"Enter"  → KeyEvent { code: Enter, modifiers: empty }
```

### Supported Keys

**Modifiers**:
- `Ctrl-` (CONTROL)
- `Alt-` (ALT)
- `Shift-` (SHIFT)

**Special Keys**:
- Navigation: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`
- Editing: `Enter`, `Tab`, `Backspace`, `Escape`, `Space`, `Delete`, `Insert`
- Any single character: case-sensitive

### Configuration Structure

```toml
[key_bindings]
# Each action can have multiple key bindings (array)
action_name = ["Key1", "Key2", "Key3"]

# Examples:
quit = ["Ctrl-c", "Ctrl-q"]           # Either key works
move_up = ["Up", "k", "w"]            # Three ways to move up
navigate_into = ["Enter", "Right", "l"]  # Vim + arrows
```

### Actions Configurable

All 12 actions are configurable:

1. `quit` - Exit application
2. `move_up` - Move selection up
3. `move_down` - Move selection down
4. `jump_up` - Jump up 10 items
5. `jump_down` - Jump down 10 items
6. `jump_to_bottom` - Jump to end
7. `navigate_into` - Enter directory
8. `navigate_up` - Go to parent
9. `clear_search` - Clear search query
10. `download_mode` - Enter download mode
11. `history_mode` - Enter history mode
12. `copy_path` - Copy to clipboard

### Default Bindings

Vim-style with arrow key support:

| Action | Keys |
|--------|------|
| Quit | Ctrl-c, Ctrl-q |
| Move Up | Up, k |
| Move Down | Down, j |
| Jump Up | Ctrl-u, K |
| Jump Down | Ctrl-d, J |
| Jump to Bottom | G, End |
| Navigate Into | Enter, Right, l |
| Navigate Up | Left, h |
| Clear Search | Escape |
| Download Mode | s, S |
| History Mode | r, R |
| Copy Path | y, Y |

## Usage Examples

### Default (Vim + Arrows)
Ships with rats3, works out of the box.

### Emacs-Style
```toml
[key_bindings]
move_up = ["Ctrl-p", "Up"]
move_down = ["Ctrl-n", "Down"]
navigate_into = ["Ctrl-f"]
navigate_up = ["Ctrl-b"]
quit = ["Ctrl-x Ctrl-c"]
```

### Arrow Keys Only
```toml
[key_bindings]
move_up = ["Up"]
move_down = ["Down"]
navigate_into = ["Right", "Enter"]
navigate_up = ["Left"]
jump_up = ["PageUp"]
jump_down = ["PageDown"]
```

### WASD Gaming
```toml
[key_bindings]
move_up = ["w"]
move_down = ["s"]
navigate_into = ["d", "Enter"]
navigate_up = ["a"]
jump_up = ["Shift-w"]
jump_down = ["Shift-s"]
copy_path = ["c"]
```

## Testing

1. **Default bindings**: Run without config
   ```bash
   rm ~/.config/rats3/config.toml
   rats3 --local /tmp
   # Uses defaults, creates new config
   ```

2. **Custom bindings**: Edit config and test
   ```bash
   vim ~/.config/rats3/config.toml
   # Edit [key_bindings] section
   rats3 --local /tmp
   # Try your new keys
   ```

3. **Test file**: Use provided test config
   ```bash
   cp test_keybindings.toml ~/.config/rats3/config.toml
   rats3 --local /tmp
   # Try: w/s to move, q to quit, c to copy
   ```

## Error Handling

- **Invalid TOML**: Falls back to defaults, shows warning
- **Invalid key string**: Ignored, action not triggered
- **Empty bindings**: Action disabled
- **Conflicts**: First matching action wins

## Benefits

1. **Flexibility**: Users choose their preferred keys
2. **Accessibility**: Customize for different keyboard layouts or needs
3. **Multiple options**: Can keep both vim and arrow keys
4. **Familiar patterns**: Use emacs, vim, or gaming conventions
5. **No recompilation**: Just edit TOML and restart

## Future Enhancements

Potential improvements:

- [ ] Reload config without restart (SIGHUP)
- [ ] Multiple modifier support (Ctrl-Alt-k)
- [ ] Key sequences (gg for jump to top)
- [ ] Context-dependent bindings (different keys per mode)
- [ ] Visual config editor in TUI
- [ ] Import/export key binding presets

## Documentation

- **KEY_BINDINGS.md** - Complete user guide with examples
- **config.example.toml** - Annotated example configuration
- **test_keybindings.toml** - Alternative layout for testing
- **README.md** - Updated with key binding section
- **QUICKSTART.md** - Quick start includes customization

## Code Quality

- Type-safe: KeyEvent comparison, not string matching
- Efficient: O(1) HashMap lookups (via match arms)
- Testable: Pure functions for key parsing
- Maintainable: Clear separation between config and logic
- Documented: Inline comments and external docs

## Backward Compatibility

Default bindings match the original hardcoded bindings, so existing users experience no change unless they customize.

## Summary

The configurable key bindings feature provides a robust, flexible system for keyboard customization while maintaining simplicity and performance. Users can now adapt rats3 to their preferred workflow without touching code.
