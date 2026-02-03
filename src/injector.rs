use anyhow::{Context, Result};
use arboard::Clipboard;
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, EventType, InputEvent, Key};
use std::thread;
use std::time::Duration;

/// Text injector using clipboard paste strategy
///
/// This module handles text injection by:
/// 1. Setting text to system clipboard
/// 2. Simulating Ctrl+V (or Ctrl+Shift+V for terminals)
///
/// # Permissions
/// Requires access to `/dev/uinput`. Add user to `input` group:
/// ```bash
/// sudo usermod -aG input $USER
/// ```
/// Or create udev rule:
/// ```bash
/// echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-cinnabar.rules
/// sudo udevadm control --reload-rules && sudo udevadm trigger
/// ```
pub struct TextInjector {
    clipboard: Clipboard,
    device: evdev::uinput::VirtualDevice,
}

impl TextInjector {
    /// Create a new TextInjector instance
    ///
    /// # Errors
    /// Returns error if:
    /// - Clipboard initialization fails
    /// - Virtual device creation fails (usually permission issue)
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new().context("Failed to initialize clipboard")?;

        // Register required keys for paste operations
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::KEY_LEFTCTRL);
        keys.insert(Key::KEY_LEFTSHIFT);
        keys.insert(Key::KEY_V);
        keys.insert(Key::KEY_BACKSPACE);
        keys.insert(Key::KEY_ENTER);

        let device = VirtualDeviceBuilder::new()
            .context("Failed to create VirtualDeviceBuilder")?
            .name("Cinnabar Virtual Keyboard")
            .with_keys(&keys)
            .context("Failed to register keys")?
            .build()
            .context(
                "Failed to build virtual device. \
                 Ensure you have permission to access /dev/uinput. \
                 Run: sudo usermod -aG input $USER",
            )?;

        Ok(Self { clipboard, device })
    }

    /// Paste text to GUI applications using Ctrl+V
    ///
    /// # Arguments
    /// * `text` - The text to paste
    ///
    /// # Process
    /// 1. Set text to clipboard
    /// 2. Wait for clipboard sync (50ms)
    /// 3. Simulate Ctrl+V key combination
    pub fn paste_text(&mut self, text: &str) -> Result<()> {
        // Step 1: Set clipboard content
        self.clipboard
            .set_text(text)
            .context("Failed to set clipboard text")?;

        // Step 2: Wait for Wayland compositor to sync clipboard
        thread::sleep(Duration::from_millis(50));

        // Step 3: Simulate Ctrl+V
        self.send_key_combo(&[Key::KEY_LEFTCTRL, Key::KEY_V])
            .context("Failed to send Ctrl+V")?;

        Ok(())
    }

    /// Paste text to terminal applications using Ctrl+Shift+V
    ///
    /// # Arguments
    /// * `text` - The text to paste
    ///
    /// # Process
    /// 1. Set text to clipboard
    /// 2. Wait for clipboard sync (50ms)
    /// 3. Simulate Ctrl+Shift+V key combination
    #[allow(dead_code)]
    pub fn paste_text_terminal(&mut self, text: &str) -> Result<()> {
        // Step 1: Set clipboard content
        self.clipboard
            .set_text(text)
            .context("Failed to set clipboard text")?;

        // Step 2: Wait for Wayland compositor to sync clipboard
        thread::sleep(Duration::from_millis(50));

        // Step 3: Simulate Ctrl+Shift+V
        self.send_key_combo(&[Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_V])
            .context("Failed to send Ctrl+Shift+V")?;

        Ok(())
    }

    /// Send a key combination (press all keys, then release in reverse order)
    ///
    /// # Arguments
    /// * `keys` - Slice of keys to press simultaneously
    fn send_key_combo(&mut self, keys: &[Key]) -> Result<()> {
        // Press all keys
        for &key in keys {
            self.send_key_event(key, 1)?;
        }

        // Small delay between press and release
        thread::sleep(Duration::from_millis(10));

        // Release all keys in reverse order
        for &key in keys.iter().rev() {
            self.send_key_event(key, 0)?;
        }

        // Sync events
        self.device
            .emit(&[InputEvent::new(EventType::SYNCHRONIZATION, 0, 0)])
            .context("Failed to sync events")?;

        Ok(())
    }

    /// Send a single key event (press or release)
    ///
    /// # Arguments
    /// * `key` - The key to press/release
    /// * `value` - 1 for press, 0 for release
    fn send_key_event(&mut self, key: Key, value: i32) -> Result<()> {
        let event = InputEvent::new(EventType::KEY, key.code(), value);
        self.device
            .emit(&[event])
            .context(format!("Failed to send key event: {:?}", key))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires /dev/uinput access
    fn test_injector_creation() {
        let result = TextInjector::new();
        assert!(
            result.is_ok(),
            "Failed to create TextInjector: {:?}",
            result.err()
        );
    }
}
