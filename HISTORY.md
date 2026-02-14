# History Mode

rats3 tracks your navigation history and allows you to quickly jump back to previously visited directories.

## Features

- **Automatic tracking**: Every directory you navigate into is automatically added to history
- **Deduplicated**: Visiting the same directory again moves it to the top (most recent)
- **Limited size**: History is kept to the last 100 entries
- **Quick access**: Press `R` (or your configured `history_mode` key) to enter history mode

## Using History Mode

### Entering History Mode

Press `R` to open the history browser. The file list pane will switch to show your navigation history with the most recent directories at the top.

### Navigation in History Mode

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up in history list |
| `↓` / `j` | Move down in history list |
| `Enter` / `l` | Navigate to selected directory |
| `Esc` | Exit history mode and return to normal browsing |

### What Gets Tracked

History tracks directories you've navigated into using:
- `Enter` / `l` - Navigating into a directory
- Selecting a history entry in history mode

History does NOT track:
- Navigating up to parent directories with `h` / `←`
- Movements within the same directory

## Configuration

You can customize the key binding for entering history mode in `~/.config/rats3/config.toml`:

```toml
[key_bindings]
history_mode = "R"  # Default: shift+r
```

## Display

In history mode:
- The title shows " History (N entries) " where N is the total number of history entries
- Each entry shows a folder icon  followed by the directory path
- Long paths are truncated intelligently from the left, keeping the most relevant (rightmost) parts
- The selected entry is highlighted with a `❯` indicator

## Technical Details

- History is stored in-memory during the session
- Maximum 100 entries (oldest entries are removed when limit is reached)
- Entries are ordered with most recent first
- Duplicate entries are removed automatically (moved to top when revisited)
- History is cleared when the application exits

## Tips

1. **Quick navigation**: Use history mode to quickly jump back to directories you were working in earlier
2. **Recent work**: The top entries show your most recent navigation, useful for switching between active work areas
3. **Long paths**: For deeply nested S3 prefixes or local directories, history mode shows the full paths you've visited
4. **Combine with search**: After returning from history mode, you can immediately start typing `/` to search within that directory

## Example Workflow

```
1. Browse to s3://my-bucket/data/2024/january/reports/
   → Added to history

2. Navigate to s3://my-bucket/logs/application/
   → Added to history

3. Navigate to s3://my-bucket/backups/daily/
   → Added to history

4. Press 'R' to open history mode
   → See:
     backups/daily/          (most recent)
     logs/application/
     data/2024/january/reports/

5. Select "logs/application/" and press Enter
   → Navigate back to logs directory
```

## Status Messages

- **"No history available"**: Displayed when pressing `R` with no history entries yet
- History mode is indicated in the window title showing the count of entries

## Keyboard Shortcuts Summary

| Mode | Key | Action |
|------|-----|--------|
| Normal | `R` | Enter history mode |
| History | `↑` / `k` | Move up |
| History | `↓` / `j` | Move down |
| History | `Enter` / `l` | Select and navigate |
| History | `Esc` | Exit history mode |
