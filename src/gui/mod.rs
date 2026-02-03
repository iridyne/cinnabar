mod hotkey;
mod state;
mod window;

pub use state::AppState;
pub use window::CinnabarWindow;

use anyhow::{Context, Result};
use eframe::egui;
use global_hotkey::hotkey::Code;
use hotkey::HotkeyManager;
use std::sync::{Arc, Mutex};

/// 运行 GUI 模式
pub fn run_gui_mode() -> Result<()> {
    // 创建热键管理器
    let hotkey_manager = Arc::new(Mutex::new(
        HotkeyManager::new(Code::F3).context("Failed to create hotkey manager")?,
    ));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([250.0, 150.0])
            .with_min_inner_size([200.0, 120.0])
            .with_always_on_top()
            .with_decorations(true)
            .with_resizable(false)
            .with_transparent(false),
        ..Default::default()
    };

    let hotkey_manager_clone = Arc::clone(&hotkey_manager);

    eframe::run_native(
        "Cinnabar",
        options,
        Box::new(move |cc| {
            let mut window = CinnabarWindow::new(cc);

            // 设置热键回调
            let state_manager_ref = window.state_manager();

            hotkey_manager_clone.lock().unwrap().set_callback(move || {
                let state_manager = state_manager_ref.lock().unwrap();

                // 切换状态
                match state_manager.get_state() {
                    AppState::Idle => {
                        state_manager.set_state(AppState::Listening);
                    }
                    AppState::Listening | AppState::Recognizing => {
                        state_manager.set_state(AppState::Idle);
                        state_manager.clear_text();
                    }
                    AppState::Injecting => {
                        // 注入状态下不响应热键
                    }
                }
            });

            // 设置热键管理器引用到窗口
            window.set_hotkey_manager(Arc::clone(&hotkey_manager_clone));

            Ok(Box::new(window))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}
