use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplicationActivationOptions, NSRunningApplication, NSWindow, NSWindowCollectionBehavior,
    NSWorkspace,
};
use tauri::WebviewWindow;
#[derive(Clone, Default)]
pub struct PreviousFocus(Option<Retained<NSRunningApplication>>);
impl PreviousFocus {
    pub fn capture(&mut self) {
        if let Some(app) = NSWorkspace::sharedWorkspace().frontmostApplication() {
            let pid = app.processIdentifier();
            if pid != std::process::id() as i32 {
                self.0 = Some(app);
            }
        }
    }
    #[allow(deprecated)]
    pub fn restore(&mut self) -> Result<(), String> {
        if let Some(app) = &self.0 {
            if !app.isTerminated()
                && !app
                    .activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps)
            {
                return Err("이전 앱을 활성화하지 못했습니다. 해당 앱을 한 번 클릭하세요.".into());
            }
        }
        Ok(())
    }
}
pub fn configure_overlay(window: &WebviewWindow) -> Result<(), String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    // Tauri owns this NSWindow. Called only on its main thread and never retained here.
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    ns.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    ns.setLevel(25); // Above full-screen presentation windows, below system menus/security UI.
    ns.setHidesOnDeactivate(false);
    Ok(())
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
struct Rect {
    origin: Point,
    size: Size,
}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> Rect;
    fn CGDisplayVendorNumber(display: u32) -> u32;
    fn CGDisplayModelNumber(display: u32) -> u32;
    fn CGDisplaySerialNumber(display: u32) -> u32;
}
pub fn display_identity(monitor: &tauri::Monitor) -> String {
    let mut displays = [0u32; 64];
    let mut count = 0;
    unsafe {
        if CGGetActiveDisplayList(64, displays.as_mut_ptr(), &mut count) == 0 {
            for &id in &displays[..count.min(64) as usize] {
                let bounds = CGDisplayBounds(id);
                let scale = monitor.scale_factor();
                if (bounds.origin.x * scale - monitor.position().x as f64).abs() < 2.
                    && (bounds.origin.y * scale - monitor.position().y as f64).abs() < 2.
                {
                    let serial = CGDisplaySerialNumber(id);
                    // Serial-less panels use the session's CoreGraphics ID, never their location.
                    return format!(
                        "mac-{}-{}-{}-{}",
                        CGDisplayVendorNumber(id),
                        CGDisplayModelNumber(id),
                        serial,
                        if serial == 0 { id } else { 0 }
                    );
                }
            }
        }
    }
    format!(
        "mac-fallback-{}-{}-{}",
        monitor.name().map(String::as_str).unwrap_or("screen"),
        monitor.position().x,
        monitor.position().y
    )
}
