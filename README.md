# rats3 - Rust S3 Navigator

A terminal-based file navigator for S3 and local filesystems, built with Rust and ratatui.

## Features Implemented

### Core Infrastructure (Phase 1) ✓
- Backend trait abstraction for different storage systems
- Local filesystem backend for testing
- **S3 backend fully implemented** (requires Rust 1.91+ for full AWS SDK support)
- State persistence for last location
- Proper async/sync architecture with tokio

### TUI & Navigation (Phase 2) ✓
- Interactive TUI with ratatui (television-style layout)
- Prominent search bar with border and visual cursor
- File list with Nerd Font icons (color-coded by type)
- Proper scrolling with visible selection
- Compact status bar with key binding hints
- Match count display when searching
- Key bindings:
  - `↑/↓` or `j/k`: Navigate up/down
  - `Ctrl-D/Ctrl-U` or `J/K`: Jump by 10 items
  - `G`: Jump to bottom
  - `gg`: Jump to top (vim-style)
  - `Enter/l`: Navigate into directory
  - `Left/h`: Navigate to parent directory
  - `/`: Enter search mode
  - `Escape`: Exit search mode or history mode
  - `Ctrl-C/Ctrl-Q`: Quit
  - `Y`: Copy current path to clipboard
  - `R`: Browse navigation history
  - Type to search/filter in real-time
  - `Backspace`: Remove search character

### Fuzzy Search (Phase 4) ✓
- High-performance fuzzy matching using nucleo-matcher
- Smart case matching (case-insensitive for lowercase queries)
- Results sorted by relevance score
- Real-time filtering as you type
- Visual feedback with match count (e.g., "3/10 matches")
- "No matches found" message when filter returns empty

### Preview & Syntax Highlighting (Phase 5) ✓
- **Split-pane layout**: File list (left) + Preview (right)
- **Syntax highlighting** for 50+ languages (Rust, Python, JS, Shell, TOML, JSON, Markdown, etc.)
- **Tokyo Night Moon theme**: Matching syntax highlighting theme with 100+ scope rules
- **Smart file handling**: Text files with syntax, binary detection, size limits
- **Preview caching**: Fast loading for revisited files
- **Auto-loading**: Preview updates when selection changes
- **Line numbers**: bat-style line numbering with syntax-aware colors
- Uses syntect (same engine as bat and VS Code)

### Additional Features (Phase 6 - Partial) ✓
- **Fully configurable key bindings** via TOML config
- **History mode**: Browse and jump to previously visited directories (R key)
- **Multi-method clipboard** support with fallback (tmux → OSC 52 → system)
- **Configurable color scheme** (Tokyo Night default, custom RGB values)
- **Nerd Font icons** for files and folders with color-coding
- Configuration file support (`~/.config/rats3/config.toml`)
- Preview size limits configurable
- Download destinations configurable
- Multiple key bindings per action
- Support for Ctrl, Alt, Shift modifiers
- Multi-key sequences (e.g., `gg` to jump to top)
- Example configuration provided

## Building

```bash
# Build (without S3 support)
cargo build --release

# Build with S3 support (requires Rust 1.91+)
cargo build --release --features s3
```

## Usage

```bash
# Browse local directory
rats3 --local /path/to/directory

# Browse S3 bucket (requires s3 feature)
rats3 s3://bucket-name/prefix

# Resume last location
rats3
```

## Configuration

On first run, rats3 creates a config file at `~/.config/rats3/config.toml`.

### Key Bindings

All key bindings are fully configurable! Edit your config file:

```toml
[key_bindings]
quit = ["Ctrl-c", "Ctrl-q"]
move_up = ["Up", "k"]
move_down = ["Down", "j"]
# ... see config.example.toml for all options
```

**Key binding features:**
- Multiple keys per action (e.g., both `k` and `Up` move up)
- Modifier support: `Ctrl-`, `Alt-`, `Shift-`
- Special keys: `Enter`, `Escape`, `Tab`, arrow keys, etc.
- Alternative layouts: vim, emacs, arrow-only, WASD, etc.

See `KEY_BINDINGS.md` for complete documentation and examples.

## Documentation

- **[KEY_BINDINGS.md](KEY_BINDINGS.md)** - Complete key binding configuration guide
- **[COLOR_SCHEME.md](COLOR_SCHEME.md)** - Color scheme customization and themes
- **[FILE_ICONS.md](FILE_ICONS.md)** - Nerd Font icon reference
- **[MODAL_MODES.md](MODAL_MODES.md)** - Modal editing system (Normal/Search modes)
- **[CLIPBOARD.md](CLIPBOARD.md)** - Clipboard integration (tmux, OSC 52, system)
- **[HISTORY.md](HISTORY.md)** - History mode usage and features

## Architecture

### Project Structure
```
rats3/
├── src/
│   ├── main.rs           # CLI entry point & TUI initialization
│   ├── lib.rs            # Library exports
│   ├── app.rs            # Main app state & logic
│   ├── backend/
│   │   ├── mod.rs        # Backend trait
│   │   ├── local.rs      # Local filesystem implementation
│   │   └── s3.rs         # S3 implementation (stub)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs     # Main layout
│   │   └── widgets/      # File list, search bar, status bar
│   ├── fuzzy.rs          # Fuzzy matching
│   ├── events.rs         # Key binding handling
│   └── state.rs          # State persistence
└── Cargo.toml
```

### Key Design Decisions

**Async/Sync Bridge**: The app uses tokio for async backend operations but maintains a synchronous rendering loop. Backend operations (listing, preview) are awaited during event handling.

**Backend Abstraction**: The `Backend` trait allows easy switching between S3 and local filesystem, making the app testable without S3 access.

**Fuzzy Matching**: Uses nucleo-matcher for fast, accurate fuzzy search with smart case matching and relevance scoring.

**Optional S3**: S3 support is gated behind a feature flag since the AWS SDK requires Rust 1.91+, allowing the app to compile with older Rust versions for testing.

## Future Work

### Phase 3: S3 Backend Implementation ✓
- [x] Full S3 listing with pagination
- [x] Object preview with size limits
- [x] File and folder downloads
- [x] Proper error handling for S3 operations
- **READ-ONLY implementation**: All S3 operations use only read methods (ListObjectsV2, GetObject, HeadObject)

### Phase 5: Preview & Syntax Highlighting
- [ ] File preview pane
- [ ] Syntax highlighting with syntect
- [ ] Binary file detection
- [ ] Loading indicators for async preview

### Phase 6: Additional Features
- [x] Configuration file (`~/.config/rats3/config.toml`)
- [x] Clipboard support for copying paths (Y key)
- [x] History tracking and browsing (R key)
- [x] Color scheme configuration
- [x] Nerd Font icons
- [x] Multi-key sequences support
- [ ] Download destination selection mode (S key)
- [ ] Full implementation of download workflow

### Phase 7: Polish & Testing
- [ ] Comprehensive error handling
- [ ] Integration tests
- [ ] Example configuration file
- [ ] Better loading states
- [ ] Download progress indicators

## Dependencies

- **ratatui** (0.27) - TUI framework
- **crossterm** (0.27) - Terminal handling
- **tokio** (1.49) - Async runtime
- **nucleo-matcher** (0.3) - Fuzzy matching
- **syntect** (5.3) - Syntax highlighting (ready for Phase 5)
- **serde/serde_json** - State serialization
- **clap** (4.4) - CLI parsing
- **chrono** (0.4.38) - Timestamps
- **anyhow** - Error handling

**Optional**:
- **aws-sdk-s3** (1.30+) - S3 client (requires Rust 1.91+)
- **aws-config** (1.1+) - AWS configuration

## Testing

The local backend can be used to test all TUI features without S3:

```bash
# Create test directory
mkdir -p /tmp/test_data/src/components
touch /tmp/test_data/src/main.rs
echo "test content" > /tmp/test_data/README.md

# Run rats3
cargo run -- --local /tmp/test_data
```

## Notes

- State is saved to `~/.local/state/rats3/last_location`
- Rust 1.87+ required (tested with 1.87.0)
- Rust 1.91+ required for S3 support due to AWS SDK requirements
- Uses careful dependency version pinning to work with Rust 1.87

## License

This project was created as a Rust implementation of the Python `s3cd` tool.
