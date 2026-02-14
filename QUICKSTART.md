# rats3 Quick Start Guide

## Installation

```bash
cd /local/home/lboehm/test/s3cd/rats3

# Build release version (without S3)
cargo build --release

# Binary will be at: target/release/rats3
```

## Basic Usage

### Browse a local directory

```bash
./target/release/rats3 --local /tmp
```

### Common Operations

1. **Navigate Files**
   - Use ↑/↓ or j/k to move up/down
   - Press Enter or l to enter a directory
   - Press Left or h to go back to parent directory

2. **Search/Filter**
   - Just start typing to filter files
   - The search is fuzzy: "mnrs" will match "main.rs"
   - Press Escape to clear the search

3. **Jump Around**
   - Ctrl-D or J: Jump down 10 items
   - Ctrl-U or K: Jump up 10 items
   - G: Jump to bottom

4. **Copy Path**
   - Press Y to copy current path to clipboard

5. **Quit**
   - Ctrl-C or Ctrl-Q

## Example Session

```bash
# Create some test data
mkdir -p ~/test_project/src/components
mkdir -p ~/test_project/docs
touch ~/test_project/src/main.rs
touch ~/test_project/src/lib.rs
touch ~/test_project/docs/README.md
echo "fn main() {}" > ~/test_project/src/main.rs

# Launch rats3
./target/release/rats3 --local ~/test_project

# Now in rats3:
# 1. See the directory listing
# 2. Type "src" to filter to src directory
# 3. Press Enter to enter src/
# 4. Type "main" to filter to main.rs
# 5. Press Y to copy the path
# 6. Press h to go back
# 7. Press Ctrl-C to quit
```

## Configuration

On first run, rats3 creates a config file at:
```
~/.config/rats3/config.toml
```

Edit it to customize:
- **Key bindings** - Change any keyboard shortcut!
- Preview file size limits
- Download destinations

See `config.example.toml` for all options.

### Customizing Key Bindings

Want different keys? Edit `~/.config/rats3/config.toml`:

```toml
[key_bindings]
# Use whatever keys you prefer!
move_up = ["w", "Up"]        # WASD style
move_down = ["s", "Down"]
quit = ["q", "Ctrl-c"]       # Single 'q' to quit
copy_path = ["c", "Y"]       # 'c' to copy
```

See `KEY_BINDINGS.md` for complete documentation and examples (emacs, arrow-only, WASD, etc.).

## Keyboard Reference

| Key | Action |
|-----|--------|
| ↑/k | Move up |
| ↓/j | Move down |
| Ctrl-U/K | Jump up 10 items |
| Ctrl-D/J | Jump down 10 items |
| G | Jump to bottom |
| Enter/l | Open directory |
| Left/h | Go to parent |
| Type | Filter/search |
| Backspace | Remove search char |
| Escape | Clear search |
| Y | Copy path to clipboard |
| Ctrl-C/Q | Quit |

## Troubleshooting

### "Backend not initialized" error
Make sure you provide a valid directory:
```bash
./target/release/rats3 --local /valid/path
```

### Clipboard not working
Clipboard requires a display server (X11/Wayland on Linux). On headless systems, the copy operation will fail gracefully with an error message.

### Build errors
Make sure you have Rust 1.87 or later:
```bash
rustc --version
```

## Next Steps

- Add more test directories with varied content
- Try fuzzy search with complex queries
- Customize your config file
- Check out README.md for full documentation
