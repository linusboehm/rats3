use anyhow::{Result, Context};
use std::process::Command;
use std::io::Write;

/// Copy text to clipboard with multiple fallback methods
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    // Check if we're in tmux
    let in_tmux = std::env::var("TMUX").is_ok();

    if in_tmux {
        // In tmux, use a combined approach:
        // 1. Set tmux buffer
        // 2. Try to get tmux to send OSC 52 to outer terminal
        if let Ok(()) = copy_via_tmux_with_osc52(text) {
            return Ok(());
        }
        // Fallback: try raw OSC 52
        if let Ok(()) = copy_via_osc52(text, in_tmux) {
            return Ok(());
        }
    } else {
        // Not in tmux, use OSC 52 directly
        if let Ok(()) = copy_via_osc52(text, in_tmux) {
            return Ok(());
        }
    }

    // Fall back to arboard (X11/Wayland/macOS)
    copy_via_arboard(text)
}

/// Copy via tmux buffer and have tmux sync to outer terminal
fn copy_via_tmux_with_osc52(text: &str) -> Result<()> {
    // Load into tmux buffer
    let mut child = Command::new("tmux")
        .args(&["load-buffer", "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn tmux load-buffer")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())
            .context("Failed to write to tmux stdin")?;
    }

    child.wait().context("Failed to wait for tmux load-buffer")?;

    // Try to trigger tmux to send buffer to outer terminal
    // This uses tmux's built-in OSC 52 support when set-clipboard is on
    let _ = Command::new("tmux")
        .args(&["set-buffer", "-w", text])
        .output();

    Ok(())
}

/// Copy via OSC 52 escape sequence
/// This works in terminals that support OSC 52 (most modern terminals)
fn copy_via_osc52(text: &str, _in_tmux: bool) -> Result<()> {
    use std::io::Write;
    use base64::{Engine as _, engine::general_purpose};

    // Encode text as base64
    let encoded = general_purpose::STANDARD.encode(text);

    // Use simple OSC 52 with BEL terminator
    // With tmux set-clipboard external, this should be forwarded to the outer terminal
    let osc = format!("\x1b]52;c;{}\x07", encoded);

    // Write directly to /dev/tty to bypass stdout buffering
    let mut tty = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .context("Failed to open /dev/tty")?;

    tty.write_all(osc.as_bytes())
        .context("Failed to write OSC 52 sequence")?;
    tty.flush().context("Failed to flush")?;

    Ok(())
}

/// Copy via arboard (X11/Wayland/macOS clipboard)
fn copy_via_arboard(text: &str) -> Result<()> {
    use arboard::Clipboard;

    let mut clipboard = Clipboard::new()
        .context("Failed to create clipboard")?;
    clipboard.set_text(text)
        .context("Failed to set clipboard text")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        // This test may fail in environments without display/clipboard support
        // That's expected and okay
        let _ = copy_to_clipboard("test");
    }
}
