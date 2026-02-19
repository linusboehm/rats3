use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent state for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub last_location: Option<String>,
    #[serde(default)]
    pub history: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_location: None,
            history: Vec::new(),
        }
    }
}

impl AppState {
    /// Get the state file path
    pub fn state_file() -> Result<PathBuf> {
        let state_dir = dirs::state_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
            .context("Could not determine state directory")?;

        let app_state_dir = state_dir.join("rats3");
        fs::create_dir_all(&app_state_dir)
            .context("Failed to create state directory")?;

        Ok(app_state_dir.join("last_location"))
    }

    /// Load state from disk
    pub fn load() -> Result<Self> {
        let path = Self::state_file()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read state file")?;

        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::state_file()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize state")?;

        fs::write(&path, content)
            .context("Failed to write state file")?;

        Ok(())
    }

    /// Update the last location
    pub fn set_last_location(&mut self, location: String) {
        self.last_location = Some(location);
    }

    /// Update the history
    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert_eq!(state.last_location, None);
        assert!(state.history.is_empty());
    }

    #[test]
    fn test_set_last_location() {
        let mut state = AppState::default();
        state.set_last_location("/test/path".to_string());
        assert_eq!(state.last_location, Some("/test/path".to_string()));
    }

    #[test]
    fn test_set_history() {
        let mut state = AppState::default();
        let history = vec!["/path1".to_string(), "/path2".to_string()];
        state.set_history(history.clone());
        assert_eq!(state.history, history);
    }

    #[test]
    fn test_multiple_history_updates() {
        let mut state = AppState::default();
        state.set_history(vec!["/path1".to_string()]);
        assert_eq!(state.history.len(), 1);

        state.set_history(vec!["/path1".to_string(), "/path2".to_string()]);
        assert_eq!(state.history.len(), 2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut state = AppState::default();
        state.set_last_location("/test/location".to_string());
        state.set_history(vec!["/hist1".to_string(), "/hist2".to_string()]);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: AppState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.last_location, state.last_location);
        assert_eq!(deserialized.history, state.history);
    }

    #[test]
    fn test_deserialize_empty_json() {
        let json = "{}";
        let state: AppState = serde_json::from_str(json).unwrap();
        assert_eq!(state.last_location, None);
        assert!(state.history.is_empty());
    }

    #[test]
    fn test_deserialize_with_history_missing() {
        let json = r#"{"last_location":"/test"}"#;
        let state: AppState = serde_json::from_str(json).unwrap();
        assert_eq!(state.last_location, Some("/test".to_string()));
        assert!(state.history.is_empty()); // Should default to empty
    }

    #[test]
    fn test_state_file_path_exists() {
        // Just verify it can generate a path without panic
        let result = AppState::state_file();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("rats3"));
    }
}
