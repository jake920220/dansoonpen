//! Bounded, best-effort local metadata. Never send annotation or exception payloads here.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender},
        Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

const FILE_BYTES: u64 = 1024 * 1024;
const FILE_COUNT: usize = 4;
const QUEUE_SIZE: usize = 128;
const EVENT_BYTES: usize = 16 * 1024;

enum Work {
    Entry(Vec<u8>),
    Flush(SyncSender<()>),
}

pub struct Diagnostics {
    sender: SyncSender<Work>,
    started: Instant,
    session: String,
    sequence: AtomicU64,
    dropped: AtomicU64,
    frontend_budget: Mutex<(Instant, u32)>,
}

impl Diagnostics {
    pub fn start(directory: PathBuf) -> io::Result<Self> {
        let mut files = RotatingFile::open(directory, FILE_BYTES, FILE_COUNT)?;
        let (sender, receiver) = mpsc::sync_channel(QUEUE_SIZE);
        std::thread::Builder::new()
            .name("brush-diagnostics".into())
            .spawn(move || {
                while let Ok(work) = receiver.recv() {
                    match work {
                        Work::Entry(bytes) => {
                            if files.write(&bytes).is_err() {
                                eprintln!(
                                    "My Brush: local diagnostics write failed; drawing continues."
                                );
                                break;
                            }
                        }
                        Work::Flush(done) => {
                            // Entries use unbuffered File::write_all. No per-event fsync on the UI thread.
                            let _ = files.file.as_ref().map(File::sync_data);
                            let _ = done.send(());
                        }
                    }
                }
            })?;
        Ok(Self {
            sender,
            started: Instant::now(),
            session: format!("{}-{}", unix_ms(), std::process::id()),
            sequence: AtomicU64::new(1),
            dropped: AtomicU64::new(0),
            frontend_budget: Mutex::new((Instant::now(), 0)),
        })
    }

    pub fn record(&self, event: &'static str, data: Value) -> u64 {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let dropped = self.dropped.swap(0, Ordering::Relaxed);
        let mut value = json!({
            "schema": 1, "session": self.session, "pid": std::process::id(),
            "sequence": sequence, "unixMs": unix_ms(),
            "elapsedMs": self.started.elapsed().as_millis(),
            "event": event, "dropped": dropped, "data": data,
        });
        let mut bytes = serde_json::to_vec(&value).unwrap_or_default();
        if bytes.len() > EVENT_BYTES {
            value["data"] = json!({ "omitted": "event_size_limit" });
            bytes = serde_json::to_vec(&value).unwrap_or_default();
        }
        bytes.push(b'\n');
        if self.sender.try_send(Work::Entry(bytes)).is_err() {
            self.dropped.fetch_add(dropped + 1, Ordering::Relaxed);
        }
        sequence
    }

    fn accept_frontend(&self) -> bool {
        let Ok(mut budget) = self.frontend_budget.try_lock() else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        };
        if budget.0.elapsed() >= Duration::from_secs(5) {
            *budget = (Instant::now(), 0);
        }
        if budget.1 >= 60 {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        budget.1 += 1;
        true
    }

    pub fn flush(&self) {
        let (tx, rx) = mpsc::sync_channel(1);
        let deadline = Instant::now() + Duration::from_millis(300);
        let mut work = Work::Flush(tx);
        loop {
            match self.sender.try_send(work) {
                Ok(()) => {
                    let _ = rx.recv_timeout(deadline.saturating_duration_since(Instant::now()));
                    break;
                }
                Err(mpsc::TrySendError::Full(pending)) if Instant::now() < deadline => {
                    work = pending;
                    // Exit/panic path only. Ordinary drawing events never retry or wait.
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(_) => break,
            }
        }
    }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

struct RotatingFile {
    directory: PathBuf,
    lock: File,
    file: Option<File>,
    size: u64,
    limit: u64,
    count: usize,
}
impl RotatingFile {
    fn open(directory: PathBuf, limit: u64, count: usize) -> io::Result<Self> {
        fs::create_dir_all(&directory)?;
        let lock = private_append(&directory.join("diagnostics.lock"))?;
        let mut result = Self {
            directory,
            lock,
            file: None,
            size: 0,
            limit,
            count,
        };
        result.reopen()?;
        Ok(result)
    }
    fn path(&self, index: usize) -> PathBuf {
        self.directory.join(if index == 0 {
            "diagnostics.jsonl".into()
        } else {
            format!("diagnostics.{index}.jsonl")
        })
    }
    fn reopen(&mut self) -> io::Result<()> {
        let file = private_append(&self.path(0))?;
        self.size = file.metadata()?.len();
        self.file = Some(file);
        Ok(())
    }
    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        // Separate lock survives rotations. Only this background worker waits on disk/other writers.
        self.lock.lock()?;
        let result = self.write_locked(bytes);
        let unlocked = self.lock.unlock();
        result.and(unlocked)
    }
    fn write_locked(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() as u64 > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "diagnostic entry too large",
            ));
        }
        // Another process may have rotated since our last write; never reuse a stale archive handle.
        self.file.take();
        self.reopen()?;
        if self.size + bytes.len() as u64 > self.limit {
            self.file.take();
            remove_if_present(&self.path(self.count - 1))?;
            for index in (1..self.count).rev() {
                let source = self.path(index - 1);
                if source.exists() {
                    fs::rename(source, self.path(index))?;
                }
            }
            self.reopen()?;
        }
        if let Some(file) = &mut self.file {
            file.write_all(bytes)?;
            self.size += bytes.len() as u64;
        }
        Ok(())
    }
}
fn private_append(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}
fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

pub fn record(app: &tauri::AppHandle, event: &'static str, data: Value) -> u64 {
    app.try_state::<Diagnostics>()
        .map_or(0, |log| log.record(event, data))
}

pub fn windows(app: &tauri::AppHandle, reason: &'static str) {
    let windows = app.webview_windows();
    let entries: Vec<_> = windows.iter().take(16).map(|(name, window)| {
        json!({ "window": name, "native": crate::platform::window_diagnostics(window).ok() })
    }).collect();
    record(
        app,
        "windows.snapshot",
        json!({ "reason": reason, "count": windows.len(), "windows": entries }),
    );
}

pub fn state(app: &tauri::AppHandle, data: &crate::RuntimeState, reason: &'static str) {
    let displays: Vec<_> = data.app.displays.iter().take(16).map(|d| json!({
        "window": crate::label(&d.id), "x": d.x, "y": d.y, "width": d.width,
        "height": d.height, "scale": d.scale_factor, "connected": d.connected, "primary": d.is_primary,
    })).collect();
    record(
        app,
        "state.snapshot",
        json!({
            "reason": reason, "mode": data.app.mode, "revision": data.app.revision,
            "activeWindow": data.app.active_display_id.as_deref().map(crate::label),
            "annotationsVisible": data.app.annotations_visible, "cursorEnabled": data.app.cursor_enabled,
            "captureShortcut": data.capturing_shortcut, "pressedIds": data.pressed,
            "registeredIds": data.registered.iter().map(|k| k.id()).collect::<Vec<_>>(),
            "hasError": data.app.error.is_some(), "displayCount": data.app.displays.len(), "displays": displays,
        }),
    );
}

/// Independent worker can still record timeouts when AppKit or the session mutex stops responding.
pub fn start_watchdog(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(10));
        let sent = Instant::now();
        let probe = record(&app, "native.probe_sent", json!({}));
        let (tx, rx) = mpsc::sync_channel(1);
        let inner = app.clone();
        if crate::dispatch_native(&app, move || {
            if let Some(session) = inner.try_state::<crate::Session>() {
                match session.0.try_lock() {
                    Ok(data) => state(&inner, &data, "probe"),
                    Err(_) => {
                        record(&inner, "native.session_busy", json!({ "probe": probe }));
                    }
                }
            }
            windows(&inner, "probe");
            record(
                &inner,
                "native.probe_ack",
                json!({ "probe": probe, "elapsedMs": sent.elapsed().as_millis() }),
            );
            let _ = tx.send(());
        })
        .is_err()
        {
            record(&app, "native.dispatch_failed", json!({ "probe": probe }));
            break;
        }
        // Never queue another callback while one probe is outstanding.
        loop {
            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(()) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    record(
                        &app,
                        "native.probe_pending",
                        json!({ "probe": probe, "elapsedMs": sent.elapsed().as_millis() }),
                    );
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    });
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontendKind {
    Ready,
    Heartbeat,
    State,
    Scene,
    CanvasContextLost,
    CanvasContextRestored,
    Error,
    UnhandledRejection,
    Pagehide,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrontendEvent {
    kind: FrontendKind,
    mode: Option<crate::model::Mode>,
    revision: Option<u64>,
    scene_revision: Option<u64>,
    annotation_count: Option<u32>,
    canvas_width: Option<u32>,
    canvas_height: Option<u32>,
    viewport_width: Option<u32>,
    viewport_height: Option<u32>,
    dpr: Option<f64>,
    focused: Option<bool>,
    visible: Option<bool>,
    opaque_background: Option<bool>,
}
impl FrontendEvent {
    fn valid(&self) -> bool {
        [
            self.canvas_width,
            self.canvas_height,
            self.viewport_width,
            self.viewport_height,
        ]
        .into_iter()
        .flatten()
        .all(|n| n <= 65536)
            && self.annotation_count.is_none_or(|n| n <= 1_000_000)
            && self.dpr.is_none_or(|n| n.is_finite() && n > 0. && n <= 16.)
            && [self.revision, self.scene_revision]
                .into_iter()
                .flatten()
                .all(|n| n <= 9_007_199_254_740_991)
    }
}

#[tauri::command]
pub fn record_diagnostic(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    event: FrontendEvent,
) {
    // No Session lock. IPC delivery can still stop when the native event loop is stuck;
    // the independent watchdog is what records that case.
    if crate::authorized(&window).is_err() || !event.valid() {
        return;
    }
    if let Some(log) = app.try_state::<Diagnostics>() {
        if log.accept_frontend() {
            log.record(
                "frontend",
                json!({ "window": window.label(), "event": event }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "my-brush-diagnostics-{name}-{}-{}",
            std::process::id(),
            unix_ms()
        ))
    }
    #[test]
    fn rotation_is_bounded_and_keeps_latest_entries_across_reopen() {
        let dir = temporary("rotation");
        let mut log = RotatingFile::open(dir.clone(), 80, 3).unwrap();
        for i in 0..80 {
            log.write(format!("{{\"sequence\":{i}}}\n").as_bytes())
                .unwrap();
        }
        drop(log);
        let mut reopened = RotatingFile::open(dir.clone(), 80, 3).unwrap();
        reopened.write(b"{\"sequence\":80}\n").unwrap();
        let paths: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .collect();
        assert_eq!(paths.len(), 3);
        assert!(paths.iter().all(|p| p.metadata().unwrap().len() <= 80));
        assert!(fs::read_to_string(reopened.path(0)).unwrap().contains("80"));
        for path in paths {
            for line in fs::read_to_string(path).unwrap().lines() {
                serde_json::from_str::<Value>(line).unwrap();
            }
        }
        drop(reopened);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn independent_writers_share_rotation_budget_without_stale_handles() {
        let dir = temporary("two-writers");
        let mut first = RotatingFile::open(dir.clone(), 80, 3).unwrap();
        let mut second = RotatingFile::open(dir.clone(), 80, 3).unwrap();
        for i in 0..100 {
            let line = format!("{{\"sequence\":{i}}}\n");
            if i % 2 == 0 {
                first.write(line.as_bytes()).unwrap();
            } else {
                second.write(line.as_bytes()).unwrap();
            }
        }
        let mut sequences = vec![];
        for index in 0..3 {
            let content = fs::read_to_string(first.path(index)).unwrap();
            assert!(content.len() <= 80);
            for line in content.lines() {
                let value: Value = serde_json::from_str(line).unwrap();
                sequences.push(value["sequence"].as_u64().unwrap());
            }
        }
        sequences.sort();
        let expected: Vec<_> = (*sequences.first().unwrap()..100).collect();
        assert_eq!(sequences, expected);
        drop(first);
        drop(second);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn full_queue_does_not_block_and_reports_loss() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let log = Diagnostics {
            sender,
            started: Instant::now(),
            session: "test".into(),
            sequence: AtomicU64::new(1),
            dropped: AtomicU64::new(0),
            frontend_budget: Mutex::new((Instant::now(), 0)),
        };
        log.record("first", json!({}));
        log.record("dropped", json!({}));
        receiver.recv().unwrap();
        log.record("last", json!({}));
        let Work::Entry(bytes) = receiver.recv().unwrap() else {
            panic!()
        };
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["dropped"], 1);
        assert_eq!(value["sequence"], 3);
        assert_eq!(value["session"], "test");
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }
    #[test]
    fn frontend_rejects_content_and_invalid_measurements() {
        assert!(serde_json::from_value::<FrontendEvent>(
            json!({"kind":"error","message":"private lecture"})
        )
        .is_err());
        assert!(serde_json::from_value::<FrontendEvent>(json!({"kind":"typed_key"})).is_err());
        let good: FrontendEvent =
            serde_json::from_value(json!({"kind":"heartbeat","canvasWidth":2560,"dpr":2})).unwrap();
        assert!(good.valid());
        let bad: FrontendEvent =
            serde_json::from_value(json!({"kind":"heartbeat","canvasWidth":999999})).unwrap();
        assert!(!bad.valid());
    }
    #[test]
    fn exit_flush_waits_for_a_full_queue_to_drain_within_its_budget() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let log = Diagnostics {
            sender,
            started: Instant::now(),
            session: "test".into(),
            sequence: AtomicU64::new(1),
            dropped: AtomicU64::new(0),
            frontend_budget: Mutex::new((Instant::now(), 0)),
        };
        log.record("before_exit", json!({}));
        let worker = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            assert!(matches!(receiver.recv().unwrap(), Work::Entry(_)));
            let Work::Flush(done) = receiver.recv_timeout(Duration::from_millis(500)).unwrap()
            else {
                panic!()
            };
            done.send(()).unwrap();
        });
        log.flush();
        worker.join().unwrap();
    }
    #[test]
    fn writer_persists_session_events_without_waiting_for_app_exit() {
        let dir = temporary("writer");
        let log = Diagnostics::start(dir.clone()).unwrap();
        log.record("session.start", json!({"version":"test"}));
        log.flush();
        let line = fs::read_to_string(dir.join("diagnostics.jsonl")).unwrap();
        let value: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["event"], "session.start");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(dir.join("diagnostics.jsonl"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        drop(log);
        // The background writer may still be releasing its handle on Windows.
        for _ in 0..20 {
            if fs::remove_dir_all(&dir).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        fs::remove_dir_all(dir).unwrap();
    }
}
