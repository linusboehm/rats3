/// Status message severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum StatusSeverity {
    /// Informational message (default)
    Info,
    /// Success message (e.g., operation completed successfully)
    Success,
    /// Warning message (e.g., non-critical issue)
    Warning,
    /// Error message (e.g., operation failed)
    Error,
}

/// Status message with content and severity
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub content: String,
    pub severity: StatusSeverity,
    pub timestamp: std::time::Instant,
}

impl StatusMessage {
    /// Create a new info message
    pub fn info(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            severity: StatusSeverity::Info,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new success message
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            severity: StatusSeverity::Success,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new warning message
    pub fn warning(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            severity: StatusSeverity::Warning,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new error message
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            severity: StatusSeverity::Error,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if the message has expired (shown for more than the given duration)
    pub fn is_expired(&self, duration: std::time::Duration) -> bool {
        self.timestamp.elapsed() > duration
    }
}
