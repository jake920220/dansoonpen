use tauri::WebviewWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, IsWindow, SetForegroundWindow,
};
#[derive(Clone, Copy, Default)]
pub struct PreviousFocus(Option<(isize, u32)>);
impl PreviousFocus {
    pub fn capture(&mut self) {
        unsafe {
            let window = GetForegroundWindow();
            let mut pid = 0;
            GetWindowThreadProcessId(window, &mut pid);
            if !window.is_null() && pid != std::process::id() {
                self.0 = Some((window as isize, pid));
            }
        }
    }
    pub fn restore(&mut self) -> Result<(), String> {
        if let Some((handle, expected_pid)) = self.0 {
            unsafe {
                let window = handle as _;
                let mut actual_pid = 0;
                GetWindowThreadProcessId(window, &mut actual_pid);
                if actual_pid == expected_pid
                    && IsWindow(window) != 0
                    && SetForegroundWindow(window) == 0
                {
                    return Err(
                        "이전 앱을 활성화하지 못했습니다. 해당 앱을 한 번 클릭하세요.".into(),
                    );
                }
            }
        }
        Ok(())
    }
}
pub fn configure_overlay(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}
pub fn window_diagnostics(window: &WebviewWindow) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "positionPhysical":window.outer_position().ok(), "sizePhysical":window.outer_size().ok(),
        "scale":window.scale_factor().ok(), "visible":window.is_visible().ok(), "focused":window.is_focused().ok(),
    }))
}
pub fn display_identity(monitor: &tauri::Monitor) -> String {
    format!(
        "win-{}",
        monitor.name().map(String::as_str).unwrap_or("screen")
    )
}

pub fn cursor_reading(
    window: &WebviewWindow,
    display_id: String,
) -> Result<crate::cursor::Reading, String> {
    use windows_sys::Win32::{
        Foundation::POINT,
        UI::{
            Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
            WindowsAndMessaging::GetCursorPos,
        },
    };
    let mut point = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut point) } == 0 {
        return Err("커서 위치를 읽지 못했습니다.".into());
    }
    let origin = window.inner_position().map_err(|e| e.to_string())?;
    let size = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let clicks = u32::from(unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } < 0);
    Ok(crate::cursor::Reading::local(
        display_id,
        (point.x as f64 - origin.x as f64) / scale,
        (point.y as f64 - origin.y as f64) / scale,
        size.width as f64 / scale,
        size.height as f64 / scale,
        clicks,
    ))
}
