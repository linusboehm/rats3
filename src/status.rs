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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_status_severity_equality() {
        assert_eq!(StatusSeverity::Info, StatusSeverity::Info);
        assert_eq!(StatusSeverity::Success, StatusSeverity::Success);
        assert_eq!(StatusSeverity::Warning, StatusSeverity::Warning);
        assert_eq!(StatusSeverity::Error, StatusSeverity::Error);
        assert_ne!(StatusSeverity::Info, StatusSeverity::Error);
    }

    #[test]
    fn test_info_message() {
        let msg = StatusMessage::info("Test info");
        assert_eq!(msg.content, "Test info");
        assert_eq!(msg.severity, StatusSeverity::Info);
    }

    #[test]
    fn test_success_message() {
        let msg = StatusMessage::success("Test success");
        assert_eq!(msg.content, "Test success");
        assert_eq!(msg.severity, StatusSeverity::Success);
    }

    #[test]
    fn test_warning_message() {
        let msg = StatusMessage::warning("Test warning");
        assert_eq!(msg.content, "Test warning");
        assert_eq!(msg.severity, StatusSeverity::Warning);
    }

    #[test]
    fn test_error_message() {
        let msg = StatusMessage::error("Test error");
        assert_eq!(msg.content, "Test error");
        assert_eq!(msg.severity, StatusSeverity::Error);
    }

    #[test]
    fn test_message_with_string() {
        let msg = StatusMessage::info(String::from("String message"));
        assert_eq!(msg.content, "String message");
    }

    #[test]
    fn test_message_not_expired_immediately() {
        let msg = StatusMessage::info("Test");
        assert!(!msg.is_expired(Duration::from_secs(1)));
    }

    #[test]
    fn test_message_expired_after_duration() {
        let msg = StatusMessage::info("Test");
        std::thread::sleep(Duration::from_millis(100));
        assert!(msg.is_expired(Duration::from_millis(50)));
    }

    #[test]
    fn test_message_not_expired_with_long_duration() {
        let msg = StatusMessage::info("Test");
        assert!(!msg.is_expired(Duration::from_secs(10)));
    }

    #[test]
    fn test_multiple_messages_different_severities() {
        let info = StatusMessage::info("Info");
        let success = StatusMessage::success("Success");
        let warning = StatusMessage::warning("Warning");
        let error = StatusMessage::error("Error");

        assert_eq!(info.severity, StatusSeverity::Info);
        assert_eq!(success.severity, StatusSeverity::Success);
        assert_eq!(warning.severity, StatusSeverity::Warning);
        assert_eq!(error.severity, StatusSeverity::Error);
    }
}
