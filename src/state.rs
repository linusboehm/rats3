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
