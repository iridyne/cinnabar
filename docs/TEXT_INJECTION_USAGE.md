# Text Injection Module Usage Guide

## Overview

The `TextInjector` module provides text injection functionality for Cinnabar using the clipboard paste strategy. This approach is ideal for Wayland environments where direct character injection is difficult, especially for Chinese characters.

## Architecture

```
┌─────────────────┐
│ Speech Recognition │
└────────┬──────────┘
         │ recognized text
         ▼
┌─────────────────┐
│  TextInjector   │
├─────────────────┤
│ 1. Set Clipboard│
│ 2. Wait 50ms    │
│ 3. Simulate     │
│    Ctrl+V       │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│ Target App      │
└─────────────────┘
```

## Prerequisites

### System Permissions

The TextInjector requires access to `/dev/uinput` to create virtual keyboard devices.

**Option 1: Add user to input group (requires logout)**
```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-100
sudo usermod -aG input $USER
# Logout and login again
```

**Option 2: Create udev rule (no logout required)**
```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-100
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-cinnabar.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Dependencies

Already included in `Cargo.toml`:
- `arboard = "3.4"` - Cross-platform clipboard management
- `evdev = "0.12"` - Virtual input device creation

## Basic Usage

### Initialization

```cinnabar/src/main.rs#L1-50
use injector::TextInjector;

fn main() -> Result<()> {
    // Initialize the text injector
    let mut injector = TextInjector::new()?;
    
    // ... rest of your code
}
```

### Paste to GUI Applications

Use `paste_text()` for GUI applications (browsers, editors, etc.):

```cinnabar/src/main.rs#L1-50
// After speech recognition produces text
let recognized_text = "我觉得 Rust 很强大。";

// Inject text using Ctrl+V
injector.paste_text(recognized_text)?;
```

### Paste to Terminal Applications

Use `paste_text_terminal()` for terminal applications:

```cinnabar/src/main.rs#L1-50
// For terminal applications (uses Ctrl+Shift+V)
let recognized_text = "cargo run --release";

injector.paste_text_terminal(recognized_text)?;
```

## Integration Example

### Complete Integration in main.rs

```cinnabar/src/main.rs#L1-300
mod ffi;
mod injector;
mod resampler;

use injector::TextInjector;
use anyhow::Result;

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize text injector
    let mut injector = TextInjector::new()
        .context("Failed to initialize text injector. Check permissions.")?;
    
    // ... existing audio setup code ...
    
    let mut last_result = String::new();
    
    while running.load(Ordering::Relaxed) {
        if let Ok(samples) = rx.recv_timeout(Duration::from_millis(100)) {
            // ... existing audio processing ...
            
            let result = recognizer.get_result(&stream);
            let trimmed = result.trim();
            
            if !trimmed.is_empty() {
                let has_sentence_end = trimmed.ends_with('。')
                    || trimmed.ends_with('？')
                    || trimmed.ends_with('！')
                    || trimmed.ends_with('.')
                    || trimmed.ends_with('?')
                    || trimmed.ends_with('!');
                
                if has_sentence_end && trimmed != last_result {
                    // Sentence complete - inject text
                    if let Err(e) = injector.paste_text(trimmed) {
                        eprintln!("Failed to inject text: {}", e);
                    }
                    
                    // Display confirmation
                    println!("\n✅ Injected: {}", trimmed);
                    last_result.clear();
                } else {
                    // Partial result - just display
                    print!("\r\x1b[K{}", trimmed);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                    last_result = trimmed.to_string();
                }
            }
        }
    }
    
    Ok(())
}
```

## CLI Arguments

Add command-line options to control injection behavior:

```cinnabar/src/main.rs#L14-30
#[derive(Parser, Debug)]
#[command(name = "cinnabar")]
struct Args {
    // ... existing args ...
    
    /// Enable automatic text injection
    #[arg(long)]
    inject: bool,
    
    /// Use terminal paste mode (Ctrl+Shift+V)
    #[arg(long)]
    terminal_mode: bool,
}
```

Usage:
```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-200
# Enable text injection
cargo run --release -- --inject

# Enable text injection for terminal
cargo run --release -- --inject --terminal-mode

# Without injection (display only)
cargo run --release
```

## Error Handling

### Permission Errors

```cinnabar/src/main.rs#L1-50
match TextInjector::new() {
    Ok(injector) => {
        println!("✅ Text injection enabled");
        Some(injector)
    }
    Err(e) => {
        eprintln!("⚠️  Text injection disabled: {}", e);
        eprintln!("   Run: sudo usermod -aG input $USER");
        None
    }
}
```

### Clipboard Errors

```cinnabar/src/main.rs#L1-50
if let Err(e) = injector.paste_text(text) {
    eprintln!("Failed to inject text: {}", e);
    // Fallback: just print to stdout
    println!("{}", text);
}
```

## Timing Considerations

### Clipboard Sync Delay

The module includes a 50ms delay after setting clipboard content to ensure Wayland compositor synchronization:

```cinnabar/src/injector.rs#L70-80
// Wait for Wayland compositor to sync clipboard
thread::sleep(Duration::from_millis(50));
```

If you experience paste failures, you can increase this delay by modifying the `paste_text` method.

### Key Event Timing

A 10ms delay is included between key press and release:

```cinnabar/src/injector.rs#L115-125
// Small delay between press and release
thread::sleep(Duration::from_millis(10));
```

## Testing

### Manual Test

```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-300
# 1. Open a text editor (gedit, kate, etc.)
# 2. Run Cinnabar with injection enabled
cargo run --release -- --inject

# 3. Speak a sentence
# 4. Verify text appears in the editor
```

### Terminal Test

```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-300
# 1. Open a terminal
# 2. Run Cinnabar in terminal mode
cargo run --release -- --inject --terminal-mode

# 3. Speak a command
# 4. Verify text appears in the terminal
```

## Troubleshooting

### Issue: Permission Denied

**Error**: `Failed to build virtual device. Ensure you have permission to access /dev/uinput`

**Solution**: Add user to input group or create udev rule (see Prerequisites)

### Issue: Text Not Pasting

**Possible Causes**:
1. Clipboard sync delay too short (increase from 50ms to 100ms)
2. Target application doesn't support Ctrl+V
3. Wayland clipboard protocol issue

**Debug Steps**:
```cinnabar/docs/TEXT_INJECTION_USAGE.md#L1-300
# Test clipboard manually
echo "test" | xclip -selection clipboard  # X11
wl-copy "test"  # Wayland

# Verify uinput device
ls -l /dev/uinput
```

### Issue: Chinese Characters Not Working

**Note**: This should work correctly with the clipboard paste strategy. If issues occur:
1. Verify clipboard supports UTF-8
2. Check target application's encoding settings
3. Test with `wl-paste` to verify clipboard content

## Performance Considerations

- **Latency**: ~60ms total (50ms clipboard sync + 10ms key events)
- **CPU**: Minimal overhead
- **Memory**: ~1KB per injection

## Security Considerations

- The virtual keyboard can inject input to any application
- Only enable injection when actively using voice input
- Consider adding a hotkey to toggle injection on/off
- The clipboard content is temporarily visible to other applications

## Future Enhancements

- [ ] Hotkey activation (Ctrl+Space to toggle injection)
- [ ] Application-specific paste modes
- [ ] Configurable delays
- [ ] Injection history/undo
- [ ] Direct character injection fallback for X11

---

**Created**: 2026-02-03  
**Version**: 1.0.0  
**Status**: Production Ready