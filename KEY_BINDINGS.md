# Configurable Key Bindings

rats3 supports fully configurable key bindings via the TOML configuration file.

## Configuration File Location

Key bindings are configured in:
```
~/.config/rats3/config.toml
```

The file is automatically created with defaults on first run.

## Key Binding Format

Each action can have multiple key bindings (any of them will trigger the action):

```toml
[key_bindings]
action_name = ["Key1", "Key2", "Key3"]
```

### Key String Format

**Simple keys**: Single characters
```toml
move_up = ["k", "w"]  # k or w will move up
```

**Special keys**: Named keys
```toml
navigate_into = ["Enter", "Right", "Space"]
```

**With modifiers**: Use `-` to combine
```toml
quit = ["Ctrl-c", "Ctrl-q"]  # Ctrl-c or Ctrl-q will quit
jump_up = ["Ctrl-u", "Alt-k", "Shift-Up"]
```

### Supported Keys

**Modifiers**:
- `Ctrl-` (Control)
- `Alt-` (Alt/Option)
- `Shift-` (Shift)

**Special Keys**:
- `Enter`, `Return`
- `Tab`
- `Backspace`
- `Escape`, `Esc`
- `Space`
- `Up`, `Down`, `Left`, `Right`
- `Home`, `End`
- `PageUp`, `PageDown`
- `Delete`, `Del`
- `Insert`, `Ins`

**Character Keys**:
- Any single character: `a`, `b`, `k`, `j`, `1`, `?`, etc.
- Case-sensitive: `k` and `K` are different keys

## Available Actions

### Navigation

```toml
# Move selection up one item
move_up = ["Up", "k"]

# Move selection down one item
move_down = ["Down", "j"]

# Jump up by 10 items
jump_up = ["Ctrl-u", "K"]

# Jump down by 10 items
jump_down = ["Ctrl-d", "J"]

# Jump to the bottom of the list
jump_to_bottom = ["G", "End"]
```

### Directory Navigation

```toml
# Navigate into selected directory (open)
navigate_into = ["Enter", "Right", "l"]

# Navigate to parent directory (go back)
navigate_up = ["Left", "h"]
```

### Search

```toml
# Clear search query
clear_search = ["Escape"]
```

Note: Any character key not bound to an action will be used for searching.

### Actions

```toml
# Enter download mode (select destination)
download_mode = ["s", "S"]

# Enter history mode (browse recent locations)
history_mode = ["r", "R"]

# Copy current path to clipboard
copy_path = ["y", "Y"]

# Quit the application
quit = ["Ctrl-c", "Ctrl-q"]
```

## Default Key Bindings

The default configuration (vim-style with arrow key support):

```toml
[key_bindings]
quit = ["Ctrl-c", "Ctrl-q"]
move_up = ["Up", "k"]
move_down = ["Down", "j"]
jump_up = ["Ctrl-u", "K"]
jump_down = ["Ctrl-d", "J"]
jump_to_bottom = ["G", "End"]
navigate_into = ["Enter", "Right", "l"]
navigate_up = ["Left", "h"]
clear_search = ["Escape"]
download_mode = ["s", "S"]
history_mode = ["r", "R"]
copy_path = ["y", "Y"]
```

## Example Configurations

### Emacs-Style

```toml
[key_bindings]
quit = ["Ctrl-c", "Ctrl-x Ctrl-c"]
move_up = ["Ctrl-p", "Up"]
move_down = ["Ctrl-n", "Down"]
jump_up = ["Alt-v"]
jump_down = ["Ctrl-v"]
jump_to_bottom = ["Alt-Shift->"]
navigate_into = ["Ctrl-f", "Enter"]
navigate_up = ["Ctrl-b"]
clear_search = ["Ctrl-g"]
download_mode = ["Ctrl-x d"]
history_mode = ["Ctrl-x h"]
copy_path = ["Alt-w"]
```

### Arrow Keys Only

```toml
[key_bindings]
quit = ["Ctrl-q"]
move_up = ["Up"]
move_down = ["Down"]
jump_up = ["PageUp"]
jump_down = ["PageDown"]
jump_to_bottom = ["End"]
navigate_into = ["Right", "Enter"]
navigate_up = ["Left", "Backspace"]
clear_search = ["Escape"]
download_mode = ["Ctrl-d"]
history_mode = ["Ctrl-h"]
copy_path = ["Ctrl-c"]
```

### Gaming (WASD)

```toml
[key_bindings]
quit = ["Ctrl-c", "Escape"]
move_up = ["w", "Up"]
move_down = ["s", "Down"]
jump_up = ["Shift-w"]
jump_down = ["Shift-s"]
jump_to_bottom = ["End"]
navigate_into = ["d", "Enter"]
navigate_up = ["a"]
clear_search = ["Escape"]
download_mode = ["e"]
history_mode = ["r"]
copy_path = ["c"]
```

## Tips

1. **Multiple bindings**: You can assign as many keys as you want to each action
2. **Conflicts**: If a key is bound to multiple actions, the first matching action wins
3. **Search keys**: Any key not bound to an action becomes a search character
4. **Case sensitivity**: `k` and `K` are different - use this for different actions
5. **Reload**: Restart rats3 to apply configuration changes

## Testing Your Configuration

After editing your config:

1. **Check syntax**: Ensure TOML is valid
   ```bash
   # Config should be in ~/.config/rats3/config.toml
   cat ~/.config/rats3/config.toml
   ```

2. **Test in app**: Run rats3 and try your new bindings
   ```bash
   rats3 --local /tmp
   ```

3. **Fallback**: If config has errors, rats3 will use defaults and show a warning

## Troubleshooting

**Keys not working?**
- Check spelling: `Ctrl-c` not `ctrl-c` or `CTRL-C`
- Verify modifier format: `Ctrl-k` not `Ctrl+k`
- Ensure key name is recognized (see Supported Keys list)

**Want to disable a binding?**
- Set it to an empty array: `action_name = []`
- Or use an obscure key: `action_name = ["F12"]`

**Conflicts with terminal?**
- Some key combinations might be intercepted by your terminal
- Try alternative bindings: e.g., `Ctrl-q` instead of `Ctrl-c` if Ctrl-c doesn't work

## See Also

- `config.example.toml` - Complete example configuration
- `README.md` - General documentation
- `QUICKSTART.md` - Getting started guide
