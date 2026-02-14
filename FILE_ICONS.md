# File Icons

rats3 uses Nerd Font icons to display file types, similar to nvim-web-devicons. This provides visual distinction between different file types in the file browser.

## Requirements

To see the icons properly, you need to use a terminal with a **Nerd Font** installed. Popular Nerd Fonts include:

- [FiraCode Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)
- [JetBrains Mono Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)
- [Hack Nerd Font](https://github.com/ryanoasis/nerd-fonts/releases)

## Supported File Types

### Programming Languages

| Icon | Language | Extensions |
|------|----------|------------|
|  | Rust | `.rs` |
|  | JavaScript | `.js`, `.jsx`, `.mjs`, `.cjs` |
|  | TypeScript | `.ts`, `.tsx` |
|  | Python | `.py`, `.pyc`, `.pyd`, `.pyo` |
|  | Go | `.go` |
|  | C | `.c`, `.h` |
|  | C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx` |
|  | Java | `.java` |
|  | Kotlin | `.kt`, `.kts` |
|  | Ruby | `.rb` |
|  | PHP | `.php` |
|  | Lua | `.lua` |

### Web Development

| Icon | Type | Extensions |
|------|------|------------|
|  | HTML | `.html`, `.htm` |
|  | CSS | `.css`, `.scss`, `.sass`, `.less` |

### Configuration Files

| Icon | Type | Extensions |
|------|------|------------|
|  | JSON | `.json` |
|  | YAML | `.yaml`, `.yml` |
|  | TOML | `.toml` |
|  | XML | `.xml` |
|  | INI | `.ini`, `.cfg`, `.conf` |

### Shell Scripts

| Icon | Type | Extensions |
|------|------|------------|
|  | Shell | `.sh`, `.bash`, `.zsh`, `.fish` |

### Documents

| Icon | Type | Extensions |
|------|------|------------|
|  | Markdown | `.md`, `.markdown` |
|  | Text | `.txt` |
|  | PDF | `.pdf` |
|  | Word | `.doc`, `.docx` |

### Data Files

| Icon | Type | Extensions |
|------|------|------------|
|  | CSV | `.csv` |
|  | SQL | `.sql` |
|  | Database | `.db`, `.sqlite`, `.sqlite3` |

### Images

| Icon | Type | Extensions |
|------|------|------------|
|  | Raster | `.png`, `.jpg`, `.jpeg`, `.gif`, `.bmp`, `.ico` |
|  | Vector | `.svg` |

### Archives

| Icon | Type | Extensions |
|------|------|------------|
|  | Archive | `.zip`, `.tar`, `.gz`, `.bz2`, `.xz`, `.7z`, `.rar` |

### Special Files

| Icon | File | Description |
|------|------|-------------|
|  | Directories | All directories |
|  | `.gitignore` | Git ignore file |
|  | `.gitmodules` | Git submodules |
|  | `Dockerfile` | Docker container file |
|  | `docker-compose.yml` | Docker compose file |
|  | `Makefile` | Build automation |
|  | `LICENSE` | License file |
|  | `README.md` | Project readme |
|  | `Cargo.toml` | Rust package manifest |
|  | `Cargo.lock` | Rust dependencies lock |
|  | `package.json` | Node.js package manifest |
|  | `tsconfig.json` | TypeScript config |
|  | `webpack.config.js` | Webpack config |
|  | `.vim` | Vim config |
|  | `.log` | Log files |
|  | `.lock` | Lock files |
|  | Default | Unknown file types |

## Color Coding

Icons are color-coded based on the file type, using the configured color scheme:

- **Directories**: Cyan (by default)
- **Rust files**: Yellow/Orange
- **Config files**: Orange
- **Scripts**: Green
- **Documents**: Light blue
- **Error/Git**: Pink-red
- **Archives**: Rust color
- **Default**: Light blue-gray

All colors can be customized in `~/.config/rats3/config.toml` under the `[colors]` section.

## Adding Custom Icons

To add support for additional file types, you can submit a pull request to update the `get_file_icon()` function in `src/ui/widgets/file_list.rs`.

## Installation

### macOS

```bash
brew tap homebrew/cask-fonts
brew install --cask font-jetbrains-mono-nerd-font
```

### Linux

```bash
# Download and install a Nerd Font manually
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
wget https://github.com/ryanoasis/nerd-fonts/releases/latest/download/JetBrainsMono.zip
unzip JetBrainsMono.zip
fc-cache -fv
```

### Windows

Download a Nerd Font from the [releases page](https://github.com/ryanoasis/nerd-fonts/releases) and install it through the Windows Font settings.

## Terminal Configuration

Make sure your terminal is configured to use the Nerd Font you installed:

- **Alacritty**: Set `font.normal.family` in `~/.config/alacritty/alacritty.yml`
- **iTerm2**: Preferences → Profiles → Text → Font
- **Windows Terminal**: Settings → Profiles → Appearance → Font face
- **Kitty**: Set `font_family` in `~/.config/kitty/kitty.conf`
