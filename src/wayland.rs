use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn get_active_window() -> Result<WindowInfo> {
    // 简化实现：返回屏幕中心位置
    Ok(WindowInfo {
        x: 100,
        y: 100,
        width: 800,
        height: 600,
    })
}
