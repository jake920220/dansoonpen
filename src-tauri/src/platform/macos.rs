use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationOptions, NSRunningApplication, NSWindow,
    NSWindowCollectionBehavior, NSWorkspace,
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

fn display_frame(display: &crate::model::DisplayInfo, desktop_top: f64) -> [f64; 4] {
    let scale = display.scale_factor;
    let width = f64::from(display.width) / scale;
    let height = f64::from(display.height) / scale;
    [
        f64::from(display.x) / scale,
        desktop_top - f64::from(display.y) / scale - height,
        width,
        height,
    ]
}

pub fn place_overlay(
    window: &WebviewWindow,
    display: &crate::model::DisplayInfo,
    desktop_top: f64,
) -> Result<(), String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    // Main thread only. Convert using the destination display, never the window's old scale.
    // Set the complete AppKit frame synchronously before enabling input or reading the cursor.
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    let [x, y, width, height] = display_frame(display, desktop_top);
    let mut frame = ns.frame();
    frame.origin.x = x;
    frame.origin.y = y;
    frame.size.width = width;
    frame.size.height = height;
    ns.setFrame_display(frame, true);
    Ok(())
}

pub fn show_overlay(window: &WebviewWindow) -> Result<(), String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    // Showing every display must not activate each window or steal keyboard focus.
    unsafe { &*raw.cast::<NSWindow>() }.orderFrontRegardless();
    Ok(())
}

pub fn set_overlay_input(window: &WebviewWindow, enabled: bool) -> Result<(), String> {
    window.set_focusable(enabled).map_err(|e| e.to_string())?;
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    ns.setIgnoresMouseEvents(!enabled);
    if !enabled && ns.isKeyWindow() {
        ns.resignKeyWindow();
    }
    Ok(())
}

#[allow(deprecated)]
pub fn focus_overlay(window: &WebviewWindow) -> Result<(), String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    let main =
        MainThreadMarker::new().ok_or("필기 창 활성화는 메인 스레드에서 실행해야 합니다.")?;
    // NSRunningApplication only sends an asynchronous activation request. Use the
    // application's own activation path, then select the key window and webview.
    NSApplication::sharedApplication(main).activateIgnoringOtherApps(true);
    ns.makeKeyAndOrderFront(None);
    let webview: &tauri::Webview = window.as_ref();
    webview.set_focus().map_err(|e| e.to_string())?;
    // AppKit may finish activation after this callback. An immediate isKeyWindow
    // check must not undo the user's draw request before that event is processed.
    Ok(())
}
pub fn window_diagnostics(window: &WebviewWindow) -> Result<serde_json::Value, String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    // Metadata only; caller must be on the AppKit main thread. Never capture screen contents.
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    let frame = ns.frame();
    Ok(serde_json::json!({
        "framePoints": {"x":frame.origin.x,"y":frame.origin.y,"width":frame.size.width,"height":frame.size.height},
        "coordinateSystem":"appkit_bottom_left_points", "scale":ns.backingScaleFactor(),
        "visible":ns.isVisible(), "key":ns.isKeyWindow(), "canBecomeKey":ns.canBecomeKeyWindow(),
        "ignoresMouse":ns.ignoresMouseEvents(), "opaque":ns.isOpaque(), "alpha":ns.alphaValue(),
        "level":ns.level(), "onActiveSpace":ns.isOnActiveSpace(), "occlusion":ns.occlusionState().bits(),
    }))
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

pub fn cursor_reading(
    window: &WebviewWindow,
    display_id: String,
) -> Result<crate::cursor::Reading, String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    // Main thread only. AppKit returns window-local points, avoiding mixed-DPI desktop conversion.
    let ns = unsafe { &*raw.cast::<NSWindow>() };
    let point = ns.mouseLocationOutsideOfEventStream();
    let size = ns.frame().size;
    let clicks = unsafe { CGEventSourceCounterForEventType(0, 1) }; // Combined session, left mouse down.
    Ok(crate::cursor::Reading::local(
        display_id,
        point.x,
        size.height - point.y,
        size.width,
        size.height,
        clicks,
    ))
}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceCounterForEventType(state: i32, event: u32) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> crate::model::DisplayInfo {
        crate::model::DisplayInfo {
            id: "test".into(),
            name: "test".into(),
            x,
            y,
            width,
            height,
            scale_factor,
            is_primary: false,
            connected: true,
        }
    }

    #[test]
    fn wake_incident_retina_frame_uses_destination_scale() {
        // Actual September 15 layout: 1x external primary, 2x Retina to its right.
        assert_eq!(
            display_frame(&display(5120, 968, 2940, 1912, 2.), 1440.),
            [2560., 0., 1470., 956.]
        );
        assert_eq!(
            display_frame(&display(0, 0, 2560, 1440, 1.), 1440.),
            [0., 0., 2560., 1440.]
        );
    }

    #[test]
    fn layouts_left_above_and_below_primary_keep_their_origins() {
        assert_eq!(
            display_frame(&display(-2940, 0, 2940, 1912, 2.), 1440.),
            [-1470., 484., 1470., 956.]
        );
        assert_eq!(
            display_frame(&display(0, -1912, 2940, 1912, 2.), 1440.),
            [0., 1440., 1470., 956.]
        );
        assert_eq!(
            display_frame(&display(0, 2880, 2940, 1912, 2.), 1440.),
            [0., -956., 1470., 956.]
        );
        assert_eq!(
            display_frame(&display(0, 0, 2940, 1912, 2.), 956.),
            [0., 0., 1470., 956.]
        );
    }
}
