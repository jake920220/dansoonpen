use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub display_id: String,
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    pub clicks: u32,
}
impl Reading {
    pub fn local(display_id: String, x: f64, y: f64, width: f64, height: f64, clicks: u32) -> Self {
        let visible =
            x.is_finite() && y.is_finite() && x >= 0. && y >= 0. && x < width && y < height;
        Self {
            display_id,
            x: if visible { x } else { 0. },
            y: if visible { y } else { 0. },
            visible,
            clicks,
        }
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    #[serde(flatten)]
    pub reading: Reading,
    pub sequence: u64,
}
#[derive(Default)]
pub struct Stream {
    latest: HashMap<String, Frame>,
    sequence: u64,
}
impl Stream {
    pub fn clear(&mut self) {
        self.latest.clear();
    }
    pub fn for_window(&self, window: &str) -> Option<&Frame> {
        self.latest
            .values()
            .find(|frame| crate::label(&frame.reading.display_id) == window)
    }

    fn update(&mut self, reading: Reading) -> Option<Frame> {
        if self
            .latest
            .get(&reading.display_id)
            .is_some_and(|f| f.reading == reading)
        {
            return None;
        }
        self.sequence += 1;
        let frame = Frame {
            reading,
            sequence: self.sequence,
        };
        self.latest
            .insert(frame.reading.display_id.clone(), frame.clone());
        Some(frame)
    }
}
pub struct Monitor(Arc<(Mutex<bool>, Condvar)>);
impl Monitor {
    pub fn start(app: tauri::AppHandle) -> Self {
        let flag = Arc::new((Mutex::new(false), Condvar::new()));
        let worker = flag.clone();
        std::thread::spawn(move || loop {
            let (mutex, wake) = &*worker;
            let Ok(enabled) = mutex.lock() else {
                break;
            };
            let Ok(enabled) = wake.wait_while(enabled, |enabled| !*enabled) else {
                break;
            };
            drop(enabled);
            let inner = app.clone();
            let (tx, rx) = std::sync::mpsc::sync_channel(1);
            if crate::dispatch_native(&app, move || {
                let state = crate::lock(&inner);
                if let Ok(mut data) = state.0.lock() {
                    if let Err(error) = poll(&inner, &mut data) {
                        data.app.cursor_enabled = false;
                        inner.state::<Monitor>().set_enabled(false);
                        crate::report(&inner, &mut data, error);
                    }
                }
                let _ = tx.send(());
            })
            .is_err()
                || rx.recv().is_err()
            {
                break;
            }
            // At most one native callback in flight; no catch-up queue after a busy frame.
            std::thread::sleep(Duration::from_millis(33));
        });
        Self(flag)
    }
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut value) = self.0 .0.lock() {
            *value = enabled;
            self.0 .1.notify_one();
        }
    }
}
pub(crate) fn poll(app: &tauri::AppHandle, data: &mut crate::RuntimeState) -> Result<(), String> {
    if !data.app.cursor_enabled {
        return Ok(());
    }
    for display in data.app.displays.iter().filter(|d| d.connected) {
        let Some(window) = app.get_webview_window(&crate::label(&display.id)) else {
            continue;
        };
        let reading = crate::platform::cursor_reading(&window, display.id.clone())?;
        if let Some(frame) = data.cursor.update(reading) {
            window
                .emit("brush-cursor", frame)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_between_displays_hides_previous_halo_without_idle_traffic() {
        let mut stream = Stream::default();
        let a = Reading::local("external".into(), 10., 20., 2560., 1440., 1);
        let b = Reading::local("retina".into(), -2550., 20., 1470., 956., 1);
        assert!(stream.update(a.clone()).unwrap().reading.visible);
        assert!(!stream.update(b.clone()).unwrap().reading.visible);
        assert!(stream.update(a).is_none());
        assert!(stream.update(b).is_none());
        let hidden = stream
            .update(Reading::local(
                "external".into(),
                2600.,
                20.,
                2560.,
                1440.,
                1,
            ))
            .unwrap();
        let visible = stream
            .update(Reading::local("retina".into(), 40., 20., 1470., 956., 1))
            .unwrap();
        assert!(!hidden.reading.visible);
        assert!(visible.reading.visible);
        assert!(visible.sequence > hidden.sequence);
        assert_eq!(
            stream
                .for_window(&crate::label("external"))
                .unwrap()
                .reading,
            hidden.reading
        );
        assert_eq!(
            stream.for_window(&crate::label("retina")).unwrap().reading,
            visible.reading
        );
        stream.clear();
        assert!(stream.for_window(&crate::label("retina")).is_none());
        assert!(stream.update(visible.reading).unwrap().sequence > visible.sequence);
    }
    #[test]
    fn outside_coordinates_never_leak_and_idle_frames_are_suppressed() {
        let mut stream = Stream::default();
        let a = Reading::local("retina".into(), 100., 200., 1470., 956., 3);
        assert!(stream.update(a.clone()).is_some());
        assert!(stream.update(a).is_none());
        let outside = Reading::local("retina".into(), -300., 50., 1470., 956., 3);
        assert!(!outside.visible);
        assert_eq!(outside.x, 0.);
        assert!(stream.update(outside).is_some());
        assert!(stream
            .update(Reading::local("retina".into(), -600., 90., 1470., 956., 3))
            .is_none());
        let click = stream
            .update(Reading::local("retina".into(), 100., 200., 1470., 956., 4))
            .unwrap();
        assert_eq!(click.sequence, 3);
        assert!(!Reading::local("bad".into(), f64::NAN, 0., 10., 10., 0).visible);
    }
}
