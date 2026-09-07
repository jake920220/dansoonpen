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
pub fn display_identity(monitor: &tauri::Monitor) -> String {
    format!(
        "win-{}",
        monitor.name().map(String::as_str).unwrap_or("screen")
    )
}
