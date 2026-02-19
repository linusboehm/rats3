use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum file size for preview (in bytes)
    #[serde(default = "default_preview_max_size")]
    pub preview_max_size: usize,

    /// Default preview window width percentage (20-80)
    #[serde(default = "default_preview_width_percent")]
    pub preview_width_percent: u16,

    /// Status message timeout in seconds
    #[serde(default = "default_status_message_timeout_secs")]
    pub status_message_timeout_secs: u64,

    /// Download destinations
    #[serde(default)]
    pub download_destinations: Vec<DownloadDestination>,

    /// Key bindings
    #[serde(default)]
    pub key_bindings: KeyBindings,

    /// Color scheme
    #[serde(default)]
    pub colors: ColorScheme,
}

/// Key binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_quit_keys")]
    pub quit: Vec<String>,

    #[serde(default = "default_move_up_keys")]
    pub move_up: Vec<String>,

    #[serde(default = "default_move_down_keys")]
    pub move_down: Vec<String>,

    #[serde(default = "default_jump_up_keys")]
    pub jump_up: Vec<String>,

    #[serde(default = "default_jump_down_keys")]
    pub jump_down: Vec<String>,

    #[serde(default = "default_jump_to_bottom_keys")]
    pub jump_to_bottom: Vec<String>,

    #[serde(default = "default_jump_to_top_sequence")]
    pub jump_to_top: String,

    #[serde(default = "default_exit_search_mode_sequence")]
    pub exit_search_mode: String,

    #[serde(default = "default_navigate_into_keys")]
    pub navigate_into: Vec<String>,

    #[serde(default = "default_navigate_up_keys")]
    pub navigate_up: Vec<String>,

    #[serde(default = "default_download_mode_keys")]
    pub download_mode: Vec<String>,

    #[serde(default = "default_history_mode_keys")]
    pub history_mode: Vec<String>,

    #[serde(default = "default_history_mode_with_search_keys")]
    pub history_mode_with_search: Vec<String>,

    #[serde(default = "default_copy_path_keys")]
    pub copy_path: Vec<String>,

    #[serde(default = "default_wrap_text_keys")]
    pub wrap_text: Vec<String>,

    #[serde(default = "default_focus_preview_keys")]
    pub focus_preview: Vec<String>,

    #[serde(default = "default_focus_explorer_keys")]
    pub focus_explorer: Vec<String>,

    #[serde(default = "default_toggle_focus_keys")]
    pub toggle_focus: Vec<String>,

    #[serde(default = "default_preview_visual_mode_keys")]
    pub preview_visual_mode: Vec<String>,

    #[serde(default = "default_yank_selection_keys")]
    pub yank_selection: Vec<String>,
}

/// RGB color representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_ratatui_color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

/// Color scheme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Background color
    #[serde(default = "default_background_color")]
    pub background: RgbColor,

    /// Border color for widgets
    #[serde(default = "default_border_color")]
    pub border: RgbColor,

    /// Accent color for normal mode (prompt, highlights)
    #[serde(default = "default_accent_normal_color")]
    pub accent_normal: RgbColor,

    /// Accent color for search mode
    #[serde(default = "default_accent_search_color")]
    pub accent_search: RgbColor,

    /// Primary text color
    #[serde(default = "default_text_primary_color")]
    pub text_primary: RgbColor,

    /// Secondary/dimmed text color
    #[serde(default = "default_text_secondary_color")]
    pub text_secondary: RgbColor,

    /// Error text color
    #[serde(default = "default_text_error_color")]
    pub text_error: RgbColor,

    /// Selection background color
    #[serde(default = "default_selection_bg_color")]
    pub selection_bg: RgbColor,

    /// Directory icon color
    #[serde(default = "default_file_icon_dir_color")]
    pub file_icon_dir: RgbColor,

    /// Rust file icon color
    #[serde(default = "default_file_icon_rust_color")]
    pub file_icon_rust: RgbColor,

    /// Config file icon color
    #[serde(default = "default_file_icon_config_color")]
    pub file_icon_config: RgbColor,

    /// Document file icon color
    #[serde(default = "default_file_icon_doc_color")]
    pub file_icon_doc: RgbColor,

    /// Script file icon color
    #[serde(default = "default_file_icon_script_color")]
    pub file_icon_script: RgbColor,

    /// Default file icon color
    #[serde(default = "default_file_icon_default_color")]
    pub file_icon_default: RgbColor,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: default_background_color(),
            border: default_border_color(),
            accent_normal: default_accent_normal_color(),
            accent_search: default_accent_search_color(),
            text_primary: default_text_primary_color(),
            text_secondary: default_text_secondary_color(),
            text_error: default_text_error_color(),
            selection_bg: default_selection_bg_color(),
            file_icon_dir: default_file_icon_dir_color(),
            file_icon_rust: default_file_icon_rust_color(),
            file_icon_config: default_file_icon_config_color(),
            file_icon_doc: default_file_icon_doc_color(),
            file_icon_script: default_file_icon_script_color(),
            file_icon_default: default_file_icon_default_color(),
        }
    }
}

// Tokyo Night color palette defaults
fn default_background_color() -> RgbColor {
    RgbColor::new(26, 27, 38) // #1a1b26 - dark background
}

fn default_border_color() -> RgbColor {
    RgbColor::new(86, 95, 137) // #565f89 - muted blue-gray
}

fn default_accent_normal_color() -> RgbColor {
    RgbColor::new(125, 207, 255) // #7dcfff - bright cyan
}

fn default_accent_search_color() -> RgbColor {
    RgbColor::new(224, 175, 104) // #e0af68 - warm yellow
}

fn default_text_primary_color() -> RgbColor {
    RgbColor::new(192, 202, 245) // #c0caf5 - light blue-gray
}

fn default_text_secondary_color() -> RgbColor {
    RgbColor::new(86, 95, 137) // #565f89 - muted blue-gray (same as border)
}

fn default_text_error_color() -> RgbColor {
    RgbColor::new(247, 118, 142) // #f7768e - pink-red
}

fn default_selection_bg_color() -> RgbColor {
    RgbColor::new(41, 46, 66) // #292e42 - slightly lighter than background
}

fn default_file_icon_dir_color() -> RgbColor {
    RgbColor::new(125, 207, 255) // #7dcfff - bright cyan
}

fn default_file_icon_rust_color() -> RgbColor {
    RgbColor::new(224, 175, 104) // #e0af68 - warm yellow
}

fn default_file_icon_config_color() -> RgbColor {
    RgbColor::new(255, 158, 100) // #ff9e64 - orange
}

fn default_file_icon_doc_color() -> RgbColor {
    RgbColor::new(192, 202, 245) // #c0caf5 - light blue-gray
}

fn default_file_icon_script_color() -> RgbColor {
    RgbColor::new(158, 206, 106) // #9ece6a - green
}

fn default_file_icon_default_color() -> RgbColor {
    RgbColor::new(192, 202, 245) // #c0caf5 - light blue-gray
}

fn default_preview_max_size() -> usize {
    102400 // 100KB
}

fn default_status_message_timeout_secs() -> u64 {
    5 // 5 seconds
}

fn default_preview_width_percent() -> u16 {
    50 // 50% split
}

// Key binding defaults
fn default_quit_keys() -> Vec<String> {
    vec!["Ctrl-c".to_string(), "Ctrl-q".to_string()]
}

fn default_move_up_keys() -> Vec<String> {
    vec!["Up".to_string(), "k".to_string()]
}

fn default_move_down_keys() -> Vec<String> {
    vec!["Down".to_string(), "j".to_string()]
}

fn default_jump_up_keys() -> Vec<String> {
    vec!["Ctrl-u".to_string(), "K".to_string()]
}

fn default_jump_down_keys() -> Vec<String> {
    vec!["Ctrl-d".to_string(), "J".to_string()]
}

fn default_jump_to_bottom_keys() -> Vec<String> {
    vec!["G".to_string(), "End".to_string()]
}

fn default_jump_to_top_sequence() -> String {
    "gg".to_string()
}

fn default_exit_search_mode_sequence() -> String {
    "jj".to_string()
}

fn default_navigate_into_keys() -> Vec<String> {
    vec!["Enter".to_string(), "Right".to_string(), "l".to_string()]
}

fn default_navigate_up_keys() -> Vec<String> {
    vec!["Left".to_string(), "h".to_string()]
}

fn default_download_mode_keys() -> Vec<String> {
    vec!["s".to_string(), "S".to_string()]
}

fn default_history_mode_keys() -> Vec<String> {
    vec!["r".to_string(), "R".to_string()]
}

fn default_history_mode_with_search_keys() -> Vec<String> {
    vec!["Ctrl-r".to_string()]
}

fn default_copy_path_keys() -> Vec<String> {
    vec!["y".to_string(), "Y".to_string()]
}

fn default_wrap_text_keys() -> Vec<String> {
    vec!["w".to_string()]
}

fn default_focus_preview_keys() -> Vec<String> {
    vec!["Ctrl-l".to_string()]
}

fn default_focus_explorer_keys() -> Vec<String> {
    vec!["Ctrl-h".to_string()]
}

fn default_toggle_focus_keys() -> Vec<String> {
    vec!["Tab".to_string()]
}

fn default_preview_visual_mode_keys() -> Vec<String> {
    vec!["v".to_string()]
}

fn default_yank_selection_keys() -> Vec<String> {
    vec!["y".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadDestination {
    pub name: String,
    pub path: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: default_quit_keys(),
            move_up: default_move_up_keys(),
            move_down: default_move_down_keys(),
            jump_up: default_jump_up_keys(),
            jump_down: default_jump_down_keys(),
            jump_to_bottom: default_jump_to_bottom_keys(),
            jump_to_top: default_jump_to_top_sequence(),
            exit_search_mode: default_exit_search_mode_sequence(),
            navigate_into: default_navigate_into_keys(),
            navigate_up: default_navigate_up_keys(),
            download_mode: default_download_mode_keys(),
            history_mode: default_history_mode_keys(),
            history_mode_with_search: default_history_mode_with_search_keys(),
            copy_path: default_copy_path_keys(),
            wrap_text: default_wrap_text_keys(),
            focus_preview: default_focus_preview_keys(),
            focus_explorer: default_focus_explorer_keys(),
            toggle_focus: default_toggle_focus_keys(),
            preview_visual_mode: default_preview_visual_mode_keys(),
            yank_selection: default_yank_selection_keys(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preview_max_size: default_preview_max_size(),
            preview_width_percent: default_preview_width_percent(),
            status_message_timeout_secs: default_status_message_timeout_secs(),
            download_destinations: vec![
                DownloadDestination {
                    name: "Downloads".to_string(),
                    path: "~/Downloads".to_string(),
                },
                DownloadDestination {
                    name: "Temp".to_string(),
                    path: "/tmp".to_string(),
                },
            ],
            key_bindings: KeyBindings::default(),
            colors: ColorScheme::default(),
        }
    }
}

impl KeyBindings {
    /// Check if a key event matches any of the given key strings
    fn matches_any(&self, key: &KeyEvent, key_strings: &[String]) -> bool {
        key_strings.iter().any(|s| matches_key(key, s))
    }

    pub fn is_quit(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.quit)
    }

    pub fn is_move_up(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.move_up)
    }

    pub fn is_move_down(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.move_down)
    }

    pub fn is_jump_up(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.jump_up)
    }

    pub fn is_jump_down(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.jump_down)
    }

    pub fn is_jump_to_bottom(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.jump_to_bottom)
    }

    pub fn is_navigate_into(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.navigate_into)
    }

    pub fn is_navigate_up(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.navigate_up)
    }

    pub fn is_download_mode(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.download_mode)
    }

    pub fn is_history_mode(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.history_mode)
    }

    pub fn is_history_mode_with_search(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.history_mode_with_search)
    }

    pub fn is_copy_path(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.copy_path)
    }

    pub fn is_wrap_text(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.wrap_text)
    }

    pub fn is_focus_preview(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.focus_preview)
    }

    pub fn is_focus_explorer(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.focus_explorer)
    }

    pub fn is_toggle_focus(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.toggle_focus)
    }

    pub fn is_preview_visual_mode(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.preview_visual_mode)
    }

    pub fn is_yank_selection(&self, key: &KeyEvent) -> bool {
        self.matches_any(key, &self.yank_selection)
    }
}

/// Parse a key string like "Ctrl-c", "Up", "k" into a KeyEvent match
fn matches_key(key: &KeyEvent, key_string: &str) -> bool {
    let parts: Vec<&str> = key_string.split('-').collect();

    let mut expected_modifiers = KeyModifiers::empty();
    let expected_code = if parts.len() > 1 {
        // Has modifier like "Ctrl-c" or "Shift-k"
        for modifier in &parts[..parts.len() - 1] {
            match modifier.to_lowercase().as_str() {
                "ctrl" => expected_modifiers |= KeyModifiers::CONTROL,
                "alt" => expected_modifiers |= KeyModifiers::ALT,
                "shift" => expected_modifiers |= KeyModifiers::SHIFT,
                _ => return false,
            }
        }
        parse_key_code(parts[parts.len() - 1])
    } else {
        // No explicit modifier, just a key
        parse_key_code(parts[0])
    };

    if let Some(code) = expected_code {
        // For uppercase letters without explicit Shift- prefix,
        // the terminal sends them with SHIFT modifier, so we need to match that
        let is_implicit_uppercase = parts.len() == 1 &&
            matches!(code, KeyCode::Char(c) if c.is_uppercase());

        if is_implicit_uppercase {
            // Config: "S" means Shift+S (terminal sends 'S' with SHIFT modifier)
            key.code == code && key.modifiers == KeyModifiers::SHIFT
        } else {
            // Everything else: exact match
            key.code == code && key.modifiers == expected_modifiers
        }
    } else {
        false
    }
}

/// Parse a key code string into KeyCode
fn parse_key_code(s: &str) -> Option<KeyCode> {
    // For single character keys, preserve case (don't lowercase)
    if s.len() == 1 {
        return s.chars().next().map(KeyCode::Char);
    }

    // For special keys, match case-insensitively
    match s.to_lowercase().as_str() {
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" => Some(KeyCode::Backspace),
        "escape" | "esc" => Some(KeyCode::Esc),
        "space" => Some(KeyCode::Char(' ')),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        _ => None,
    }
}

impl Config {
    /// Get config file path
    pub fn config_file() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .context("Could not determine config directory")?;

        let app_config_dir = config_dir.join("rats3");
        fs::create_dir_all(&app_config_dir)
            .context("Failed to create config directory")?;

        Ok(app_config_dir.join("config.toml"))
    }

    /// Load config from disk, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_file()?;

        if !path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read config file")?;

        toml::from_str(&content)
            .context("Failed to parse config file")
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_file()?;
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, content)
            .context("Failed to write config file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.preview_max_size, 102400); // 100KB
        assert_eq!(config.preview_width_percent, 50);
        assert_eq!(config.status_message_timeout_secs, 5);
    }

    #[test]
    fn test_rgb_color_to_ratatui() {
        let color = RgbColor { r: 255, g: 128, b: 0 };
        assert_eq!(color.to_ratatui_color(), Color::Rgb(255, 128, 0));
    }

    #[test]
    fn test_rgb_color_black() {
        let color = RgbColor { r: 0, g: 0, b: 0 };
        assert_eq!(color.to_ratatui_color(), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_rgb_color_white() {
        let color = RgbColor { r: 255, g: 255, b: 255 };
        assert_eq!(color.to_ratatui_color(), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn test_default_color_scheme() {
        let colors = ColorScheme::default();
        assert_eq!(colors.background.r, 26);
        assert_eq!(colors.background.g, 27);
        assert_eq!(colors.background.b, 38);
    }

    #[test]
    fn test_download_destination() {
        let dest = DownloadDestination {
            name: "Test".to_string(),
            path: "/tmp".to_string(),
        };
        assert_eq!(dest.name, "Test");
        assert_eq!(dest.path, "/tmp");
    }

    #[test]
    fn test_key_bindings_default() {
        let bindings = KeyBindings::default();
        assert!(!bindings.quit.is_empty());
        assert!(!bindings.move_up.is_empty());
        assert!(!bindings.move_down.is_empty());
    }

    #[test]
    fn test_key_bindings_is_quit() {
        let bindings = KeyBindings::default();
        let quit_key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(bindings.is_quit(&quit_key));
    }

    #[test]
    fn test_key_bindings_is_move_up() {
        let bindings = KeyBindings::default();
        let up_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert!(bindings.is_move_up(&up_key));
    }

    #[test]
    fn test_key_bindings_is_move_down() {
        let bindings = KeyBindings::default();
        let down_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());
        assert!(bindings.is_move_down(&down_key));
    }

    #[test]
    fn test_serialize_deserialize_config() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.preview_max_size, config.preview_max_size);
    }

    #[test]
    fn test_config_file_path() {
        let result = Config::config_file();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("rats3"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_download_destinations_default() {
        let config = Config::default();
        assert_eq!(config.download_destinations.len(), 2);
        assert_eq!(config.download_destinations[0].name, "Downloads");
        assert_eq!(config.download_destinations[1].name, "Temp");
    }
}

