use std::sync::{Arc, Mutex};

/// GUI 应用状态
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// 待机状态
    Idle,
    /// 监听中
    Listening,
    /// 识别中
    Recognizing,
    /// 注入文本
    Injecting,
}

/// GUI 状态管理器
pub struct StateManager {
    state: Arc<Mutex<AppState>>,
    recognized_text: Arc<Mutex<String>>,
}

impl StateManager {
    /// 创建新的状态管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AppState::Idle)),
            recognized_text: Arc::new(Mutex::new(String::new())),
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    /// 设置状态
    pub fn set_state(&self, state: AppState) {
        *self.state.lock().unwrap() = state;
    }

    /// 获取识别文本
    pub fn get_text(&self) -> String {
        self.recognized_text.lock().unwrap().clone()
    }

    /// 设置识别文本
    pub fn set_text(&self, text: String) {
        *self.recognized_text.lock().unwrap() = text;
    }

    /// 清空识别文本
    pub fn clear_text(&self) {
        self.recognized_text.lock().unwrap().clear();
    }

    /// 获取状态描述
    pub fn get_state_description(&self) -> &'static str {
        match self.get_state() {
            AppState::Idle => "待机",
            AppState::Listening => "监听中...",
            AppState::Recognizing => "识别中",
            AppState::Injecting => "注入文本",
        }
    }

    /// 获取状态图标
    pub fn get_state_icon(&self) -> &'static str {
        match self.get_state() {
            AppState::Idle => "🎤",
            AppState::Listening => "🔴",
            AppState::Recognizing => "🟢",
            AppState::Injecting => "✅",
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
