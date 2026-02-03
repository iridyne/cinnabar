use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::{Arc, Mutex};

/// 热键管理器
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    callback: Arc<Mutex<Option<Box<dyn FnMut() + Send + 'static>>>>,
}

impl HotkeyManager {
    /// 创建新的热键管理器
    ///
    /// # Arguments
    /// * `key_code` - 按键代码（如 F3）
    pub fn new(key_code: Code) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to create hotkey manager: {}", e))?;

        let hotkey = HotKey::new(None::<Modifiers>, key_code);

        manager
            .register(hotkey)
            .map_err(|e| anyhow::anyhow!("Failed to register hotkey: {}", e))?;

        Ok(Self {
            manager,
            hotkey,
            callback: Arc::new(Mutex::new(None)),
        })
    }

    /// 设置热键回调函数
    pub fn set_callback<F>(&self, callback: F)
    where
        F: FnMut() + Send + 'static,
    {
        *self.callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// 处理热键事件
    pub fn handle_events(&self) {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey.id() {
                if let Some(callback) = self.callback.lock().unwrap().as_mut() {
                    callback();
                }
            }
        }
    }

    /// 注销热键
    pub fn unregister(&self) -> Result<()> {
        self.manager
            .unregister(self.hotkey)
            .map_err(|e| anyhow::anyhow!("Failed to unregister hotkey: {}", e))
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.unregister();
    }
}
