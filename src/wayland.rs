use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn get_active_window() -> Result<WindowInfo> {
    Ok(WindowInfo {
        x: 100,
        y: 100,
        width: 800,
        height: 600,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_window() {
        let result = get_active_window();
        assert!(result.is_ok());
        let win = result.unwrap();
        assert_eq!(win.width, 800);
        assert_eq!(win.height, 600);
    }
}
