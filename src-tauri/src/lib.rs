mod cursor;
mod model;
mod platform;
mod settings;
mod shortcut_registry;

use model::*;
use std::{collections::HashSet, path::PathBuf, str::FromStr, sync::Mutex, time::Duration};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

struct RuntimeState {
    app: AppState,
    scenes: SceneStore,
    cursor: cursor::Stream,
    previous_focus: platform::PreviousFocus,
    settings_path: PathBuf,
    registered: Vec<Shortcut>,
    pressed: HashSet<u32>,
    capturing_shortcut: bool,
}
struct Session(Mutex<RuntimeState>);
struct TrayShortcuts {
    draw: MenuItem<tauri::Wry>,
    clear: MenuItem<tauri::Wry>,
    visibility: MenuItem<tauri::Wry>,
    cursor: MenuItem<tauri::Wry>,
}
impl TrayShortcuts {
    fn update(&self, settings: &AppSettings) -> tauri::Result<()> {
        // Native accelerators supply the right-aligned, platform-specific labels.
        self.draw.set_accelerator(Some(&settings.toggle_shortcut))?;
        self.clear.set_accelerator(Some(&settings.clear_shortcut))?;
        self.visibility.set_accelerator(
            (!settings.visibility_shortcut.is_empty()).then_some(&settings.visibility_shortcut),
        )
    }
}
// Windows WebView2 creation must run outside synchronous webview/menu callbacks.
// AppKit operations, in contrast, must run on the macOS main thread.
fn dispatch_native<F: FnOnce() + Send + 'static>(
    app: &tauri::AppHandle,
    work: F,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        app.run_on_main_thread(work).map_err(|e| e.to_string())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = app;
        std::thread::spawn(work);
        Ok(())
    }
}
async fn native_command<T: Send + 'static, F: FnOnce() -> Result<T, String> + Send + 'static>(
    app: tauri::AppHandle,
    work: F,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        dispatch_native(&app, move || {
            let _ = tx.send(work());
        })?;
        rx.recv().map_err(|e| e.to_string())?
    })
    .await
    .map_err(|e| e.to_string())?
}
fn lock(app: &tauri::AppHandle) -> tauri::State<'_, Session> {
    app.state::<Session>()
}
fn authorized(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "control" || window.label().starts_with("overlay-") {
        Ok(())
    } else {
        Err("허용되지 않은 창입니다.".into())
    }
}
fn control_only(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == "control" {
        Ok(())
    } else {
        Err("설정 창에서만 사용할 수 있는 명령입니다.".into())
    }
}
fn label(id: &str) -> String {
    let hash = id.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("overlay-{hash:016x}")
}
fn emit_state(app: &tauri::AppHandle, data: &mut RuntimeState) {
    if let Some(tray) = app.try_state::<TrayShortcuts>() {
        let _ = tray.visibility.set_text(if data.app.annotations_visible {
            "필기 잠시 숨기기"
        } else {
            "숨긴 필기 다시 표시"
        });
    }
    if let Some(tray) = app.try_state::<TrayShortcuts>() {
        let _ = tray.cursor.set_text(if data.app.cursor_enabled {
            "커서 강조 끄기"
        } else {
            "커서 강조 켜기"
        });
    }
    data.app.revision += 1;
    let _ = app.emit("brush-state", &data.app);
}
fn emit_scenes(app: &tauri::AppHandle, updates: Vec<SceneUpdate>) {
    for update in updates {
        let _ = app.emit("brush-scene", update);
    }
}
fn fade(data: &RuntimeState) -> u32 {
    if data.app.settings.reduce_motion {
        0
    } else {
        350
    }
}
fn connected(data: &RuntimeState, id: &str) -> bool {
    data.app.displays.iter().any(|d| d.id == id && d.connected)
}

fn ensure_overlay(
    app: &tauri::AppHandle,
    data: &RuntimeState,
    id: &str,
) -> Result<WebviewWindow, String> {
    let display = data
        .app
        .displays
        .iter()
        .find(|d| d.id == id && d.connected)
        .ok_or("연결된 화면을 선택하세요.")?;
    let name = label(id);
    let window = if let Some(w) = app.get_webview_window(&name) {
        w
    } else {
        let mut url =
            tauri::Url::parse("http://localhost/index.html").map_err(|e| e.to_string())?;
        url.query_pairs_mut()
            .append_pair("view", "overlay")
            .append_pair("display", id);
        let app_url = format!("index.html?{}", url.query().unwrap_or_default());
        let window = WebviewWindowBuilder::new(app, &name, WebviewUrl::App(app_url.into()))
            .title("My Brush 필기")
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .skip_taskbar(true)
            .focused(false)
            .focusable(false)
            .visible(false)
            .resizable(false)
            .build()
            .map_err(|e| e.to_string())?;
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| e.to_string())?;
        platform::configure_overlay(&window)?;
        window
    };
    // Monitor coordinates are physical pixels. Frontend uses CSS/logical pixels.
    window
        .set_size(PhysicalSize::new(display.width, display.height))
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(display.x, display.y))
        .map_err(|e| e.to_string())?;
    Ok(window)
}
fn release_inputs(app: &tauri::AppHandle) -> Result<(), String> {
    let mut errors = vec![];
    for (name, w) in app.webview_windows() {
        if name.starts_with("overlay-") {
            if let Err(e) = w.set_ignore_cursor_events(true) {
                errors.push(e.to_string());
                let _ = w.hide();
            }
            if let Err(e) = w.set_focusable(false) {
                errors.push(e.to_string());
                let _ = w.hide();
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "입력 복구 중 창을 숨겼습니다: {}",
            errors.join(", ")
        ))
    }
}
fn change_mode(app: &tauri::AppHandle, data: &mut RuntimeState, mode: Mode) -> Result<(), String> {
    let result = (|| {
        release_inputs(app)?;
        if mode == Mode::Draw {
            let id = data
                .app
                .active_display_id
                .clone()
                .ok_or("연결된 화면이 없습니다.")?;
            let window = ensure_overlay(app, data, &id)?;
            if data.app.mode != Mode::Draw {
                data.previous_focus.capture();
            }
            window.show().map_err(|e| e.to_string())?;
            window.set_focusable(true).map_err(|e| e.to_string())?;
            window
                .set_ignore_cursor_events(false)
                .map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        } else {
            data.previous_focus.restore()?;
        }
        Ok::<_, String>(())
    })();
    match result {
        Ok(()) => {
            data.app.mode = mode;
            if mode == Mode::Draw {
                data.app.annotations_visible = true;
            }
            data.app.error = None;
            emit_state(app, data);
            Ok(())
        }
        Err(e) => {
            let _ = release_inputs(app);
            let _ = data.previous_focus.restore();
            data.app.mode = Mode::Interact;
            data.app.error = Some(e.clone());
            emit_state(app, data);
            Err(e)
        }
    }
}
fn toggle_cursor_highlight(app: &tauri::AppHandle, data: &mut RuntimeState) -> Result<(), String> {
    if !data.app.cursor_enabled {
        let id = data
            .app
            .active_display_id
            .clone()
            .ok_or("연결된 화면이 없습니다.")?;
        ensure_overlay(app, data, &id)?
            .show()
            .map_err(|e| e.to_string())?;
    }
    data.app.cursor_enabled = !data.app.cursor_enabled;
    data.cursor.latest = None;
    app.state::<cursor::Monitor>()
        .set_enabled(data.app.cursor_enabled);
    emit_state(app, data);
    Ok(())
}
#[tauri::command]
async fn toggle_cursor(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let state = lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        toggle_cursor_highlight(&app, &mut data)
    })
    .await
}
#[tauri::command]
async fn get_cursor(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<Option<cursor::Frame>, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let state = lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        cursor::poll(&app, &mut data)?;
        Ok(data.cursor.latest.clone())
    })
    .await
}
fn toggle_visibility(app: &tauri::AppHandle, data: &mut RuntimeState) -> Result<(), String> {
    data.app.annotations_visible = !data.app.annotations_visible;
    // Hiding exits drawing; restoring never steals focus or changes history.
    if !data.app.annotations_visible && data.app.mode == Mode::Draw {
        change_mode(app, data, Mode::Interact)
    } else {
        emit_state(app, data);
        Ok(())
    }
}
#[tauri::command]
async fn toggle_annotations(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let state = lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        toggle_visibility(&app, &mut data)
    })
    .await
}
fn clear_and_interact(app: &tauri::AppHandle, data: &mut RuntimeState) -> Result<(), String> {
    clear_scope_and_interact(app, data, None)
}
fn clear_scope_and_interact(
    app: &tauri::AppHandle,
    data: &mut RuntimeState,
    display: Option<String>,
) -> Result<(), String> {
    let duration = fade(data);
    // Advance the clear generation before mode-exit handlers can submit a final edit.
    // Keep the overlay visible so its existing fade can finish after input is released.
    let updates = if let Some(id) = display {
        data.scenes.clear_display(&id, duration)?
    } else {
        data.scenes.clear(duration)
    };
    emit_scenes(app, updates);
    if data.app.mode == Mode::Draw {
        change_mode(app, data, Mode::Interact)?;
    }
    Ok(())
}
fn report(app: &tauri::AppHandle, data: &mut RuntimeState, error: String) {
    data.app.error = Some(error);
    emit_state(app, data);
}
fn shortcuts(settings: &AppSettings) -> Result<Vec<Shortcut>, String> {
    settings.validate()?;
    let settings_key = Shortcut::from_str(if cfg!(target_os = "macos") {
        "Command+Comma"
    } else {
        "Control+Comma"
    })
    .map_err(|e| e.to_string())?;
    let mut keys = Vec::new();
    for (name, value, optional) in [
        ("그리기", &settings.toggle_shortcut, false),
        ("전체 삭제", &settings.clear_shortcut, false),
        ("필기 숨김", &settings.visibility_shortcut, true),
    ] {
        if optional && value.is_empty() {
            continue;
        }
        let key = Shortcut::from_str(value).map_err(|e| format!("{name} 단축키 형식 오류: {e}"))?;
        if key.mods.is_empty() {
            return Err("전역 단축키에는 보조 키가 필요합니다.".into());
        }
        if key.id() == settings_key.id() {
            return Err("이 조합은 설정 열기에 사용됩니다.".into());
        }
        if keys.iter().any(|k: &Shortcut| k.id() == key.id()) {
            return Err("단축키는 서로 다른 조합으로 지정해 주세요.".into());
        }
        keys.push(key);
    }
    Ok(keys)
}
fn replace_shortcuts(
    app: &tauri::AppHandle,
    data: &mut RuntimeState,
    wanted: Vec<Shortcut>,
) -> Result<(), String> {
    struct NativeRegistry<'a>(&'a tauri::AppHandle);
    impl shortcut_registry::Registry for NativeRegistry<'_> {
        fn register(&mut self, key: Shortcut) -> Result<(), String> {
            self.0
                .global_shortcut()
                .register(key)
                .map_err(|e| e.to_string())
        }
        fn unregister(&mut self, key: Shortcut) -> Result<(), String> {
            self.0
                .global_shortcut()
                .unregister(key)
                .map_err(|e| e.to_string())
        }
    }
    let result = shortcut_registry::replace(&mut NativeRegistry(app), &mut data.registered, wanted);
    data.pressed.clear();
    result
}
fn refresh_displays(app: &tauri::AppHandle, data: &mut RuntimeState) -> Result<(), String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let primary = app
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .map(|m| platform::display_identity(&m));
    let mut next = data.app.displays.clone();
    for d in &mut next {
        d.connected = false;
        d.is_primary = false;
    }
    for monitor in monitors {
        let id = platform::display_identity(&monitor);
        let d = DisplayInfo {
            id: id.clone(),
            name: monitor
                .name()
                .cloned()
                .unwrap_or_else(|| "디스플레이".into()),
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width,
            height: monitor.size().height,
            scale_factor: monitor.scale_factor(),
            is_primary: primary.as_ref() == Some(&id),
            connected: true,
        };
        if let Some(old) = next.iter_mut().find(|d| d.id == id) {
            *old = d;
        } else {
            next.push(d);
        }
        data.scenes.ensure(&id);
    }
    if data.app.displays == next {
        return Ok(());
    }
    data.app.displays = next;
    let active_connected = data
        .app
        .active_display_id
        .as_ref()
        .is_some_and(|id| connected(data, id));
    if !active_connected {
        let _ = change_mode(app, data, Mode::Interact);
        data.app.active_display_id = data
            .app
            .displays
            .iter()
            .find(|d| d.connected && d.is_primary)
            .or_else(|| data.app.displays.iter().find(|d| d.connected))
            .map(|d| d.id.clone());
    }
    if data.app.cursor_enabled {
        if let Some(id) = &data.app.active_display_id {
            ensure_overlay(app, data, id)?
                .show()
                .map_err(|e| e.to_string())?;
        }
    }
    for d in &data.app.displays {
        if let Some(w) = app.get_webview_window(&label(&d.id)) {
            if d.connected {
                ensure_overlay(app, data, &d.id)?;
                w.show().map_err(|e| e.to_string())?;
            } else {
                let _ = w.set_ignore_cursor_events(true);
                let _ = w.set_focusable(false);
                w.hide().map_err(|e| e.to_string())?;
            }
        }
    }
    emit_state(app, data);
    Ok(())
}

#[tauri::command]
async fn get_state(window: WebviewWindow, app: tauri::AppHandle) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        Ok(lock(&app).0.lock().map_err(|e| e.to_string())?.app.clone())
    })
    .await
}
#[tauri::command]
async fn set_mode(
    window: WebviewWindow,
    app: tauri::AppHandle,
    mode: Mode,
) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        change_mode(&app, &mut d, mode)?;
        Ok(d.app.clone())
    })
    .await
}
#[tauri::command]
async fn select_display(
    window: WebviewWindow,
    app: tauri::AppHandle,
    display_id: String,
) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        if !connected(&d, &display_id) {
            return Err("연결되지 않은 화면입니다.".into());
        }
        let mode = d.app.mode;
        if d.app.cursor_enabled {
            ensure_overlay(&app, &d, &display_id)?
                .show()
                .map_err(|e| e.to_string())?;
        }
        d.app.active_display_id = Some(display_id);
        change_mode(&app, &mut d, mode)?;
        Ok(d.app.clone())
    })
    .await
}
#[tauri::command]
async fn update_settings(
    window: WebviewWindow,
    app: tauri::AppHandle,
    settings: SettingsPatch,
) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        let brush = settings.brush_patch().apply(&d.app.brush);
        let settings = settings.apply(&d.app.settings);
        let wanted = shortcuts(&settings)?;
        if d.capturing_shortcut
            && (settings.toggle_shortcut != d.app.settings.toggle_shortcut
                || settings.clear_shortcut != d.app.settings.clear_shortcut)
        {
            return Err("단축키 입력을 마친 뒤 적용해 주세요.".into());
        }
        let old = d.registered.clone();
        let wanted = if d.capturing_shortcut { vec![] } else { wanted };
        if let Err(e) = replace_shortcuts(&app, &mut d, wanted) {
            let _ = change_mode(&app, &mut d, Mode::Interact);
            report(&app, &mut d, e.clone());
            return Err(e);
        }
        if let Err(e) = settings::save(&d.settings_path, &settings) {
            let rollback = replace_shortcuts(&app, &mut d, old);
            let error = if let Err(r) = rollback {
                format!("{e}; 단축키 복원 실패: {r}. 트레이에서 복구하세요.")
            } else {
                e
            };
            let _ = change_mode(&app, &mut d, Mode::Interact);
            report(&app, &mut d, error.clone());
            return Err(error);
        }
        d.app.settings = settings;
        d.app.brush = brush;
        d.app.error = app
            .state::<TrayShortcuts>()
            .update(&d.app.settings)
            .err()
            .map(|e| format!("설정은 저장했지만 메뉴 단축키 표시를 갱신하지 못했습니다: {e}"));
        emit_state(&app, &mut d);
        Ok(d.app.clone())
    })
    .await
}
#[tauri::command]
async fn update_brush(
    window: WebviewWindow,
    app: tauri::AppHandle,
    brush: BrushPatch,
) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        let brush = brush.apply(&d.app.brush);
        brush.validate()?;
        d.app.brush = brush;
        emit_state(&app, &mut d);
        Ok(d.app.clone())
    })
    .await
}
#[tauri::command]
async fn update_preset(
    window: WebviewWindow,
    app: tauri::AppHandle,
    index: usize,
    name: Option<String>,
    brush: Option<BrushSettings>,
) -> Result<AppState, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        let next = d.app.settings.replace_preset(index, name, brush)?;
        settings::save(&d.settings_path, &next)?;
        d.app.settings = next;
        emit_state(&app, &mut d);
        Ok(d.app.clone())
    })
    .await
}
fn end_shortcut_capture(app: &tauri::AppHandle, d: &mut RuntimeState) -> Result<(), String> {
    if d.capturing_shortcut {
        d.capturing_shortcut = false;
        let wanted = shortcuts(&d.app.settings)?;
        if let Err(e) = replace_shortcuts(app, d, wanted) {
            report(app, d, e.clone());
            return Err(e);
        }
    }
    Ok(())
}
#[tauri::command]
async fn capture_shortcut(
    window: WebviewWindow,
    app: tauri::AppHandle,
    active: bool,
) -> Result<(), String> {
    native_command(app.clone(), move || {
        control_only(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        if active {
            if !window.is_focused().map_err(|e| e.to_string())? {
                return Err("설정 창을 선택한 뒤 단축키를 입력해 주세요.".into());
            }
            replace_shortcuts(&app, &mut d, vec![])?;
            d.capturing_shortcut = true;
        } else {
            end_shortcut_capture(&app, &mut d)?;
        }
        Ok(())
    })
    .await
}
#[tauri::command]
async fn get_scene(
    window: WebviewWindow,
    app: tauri::AppHandle,
    display_id: String,
) -> Result<SceneSnapshot, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        if window.label() != "control" && window.label() != label(&display_id) {
            return Err("다른 화면의 장면을 요청할 수 없습니다.".into());
        }
        lock(&app)
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .scenes
            .snapshot(&display_id)
    })
    .await
}
#[tauri::command]
async fn apply_edit(
    window: WebviewWindow,
    app: tauri::AppHandle,
    edit: SceneEdit,
) -> Result<SceneSnapshot, String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        // A pointer-up or text commit may already be in flight when native mode changes.
        // Physical input gates enforce selection; accept the owner window's final edit.
        if window.label() != label(&edit.display_id) || !connected(&d, &edit.display_id) {
            return Err("해당 화면의 필기 창에서만 편집할 수 있습니다.".into());
        }
        let id = edit.display_id.clone();
        let duration = fade(&d);
        let updates = d.scenes.apply(edit, duration)?;
        emit_scenes(&app, updates);
        d.scenes.snapshot(&id)
    })
    .await
}
#[tauri::command]
async fn clear_all(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        clear_and_interact(&app, &mut d)
    })
    .await
}
#[tauri::command]
async fn clear_current(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let state = lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        let id = data
            .app
            .active_display_id
            .clone()
            .ok_or("연결된 화면이 없습니다.")?;
        clear_scope_and_interact(&app, &mut data, Some(id))
    })
    .await
}
#[tauri::command]
async fn undo(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        emit_scenes(&app, d.scenes.undo());
        Ok(())
    })
    .await
}
#[tauri::command]
async fn redo(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        let s = lock(&app);
        let mut d = s.0.lock().map_err(|e| e.to_string())?;
        let duration = fade(&d);
        emit_scenes(&app, d.scenes.redo(duration));
        Ok(())
    })
    .await
}
fn open_control(app: &tauri::AppHandle) -> Result<(), String> {
    let s = lock(app);
    let mut d = s.0.lock().map_err(|e| e.to_string())?;
    d.previous_focus.capture();
    // Recovery UI must remain reachable even if restoring another app's focus fails.
    let _ = change_mode(app, &mut d, Mode::Interact);
    drop(d);
    let window = app
        .get_webview_window("control")
        .ok_or("설정 창이 없습니다.")?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    window
        .emit("brush-open-settings", ())
        .map_err(|e| e.to_string())
}
#[tauri::command]
async fn show_control(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        authorized(&window)?;
        open_control(&app)
    })
    .await
}
#[tauri::command]
async fn quit_app(window: WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    native_command(app.clone(), move || {
        control_only(&window)?;
        let _ = release_inputs(&app);
        app.exit(0);
        Ok(())
    })
    .await
}
fn tray(app: &tauri::AppHandle, settings: &AppSettings) -> tauri::Result<()> {
    let control = MenuItem::with_id(
        app,
        "control",
        "설정 열기",
        true,
        Some("CommandOrControl+Comma"),
    )?;
    let draw = MenuItem::with_id(app, "draw", "그리기", true, Some(&settings.toggle_shortcut))?;
    let interact = MenuItem::with_id(
        app,
        "interact",
        "앱 조작으로 복귀 (입력 복구)",
        true,
        Some("Escape"),
    )?;
    let clear = MenuItem::with_id(
        app,
        "clear",
        "모든 화면 필기 지우기",
        true,
        Some(&settings.clear_shortcut),
    )?;
    let cursor = MenuItem::with_id(app, "cursor", "커서 강조 켜기", true, None::<&str>)?;
    let clear_current_item = MenuItem::with_id(
        app,
        "clear-current",
        "선택한 화면 필기 지우기",
        true,
        None::<&str>,
    )?;
    let visibility = MenuItem::with_id(
        app,
        "visibility",
        "필기 잠시 숨기기",
        true,
        (!settings.visibility_shortcut.is_empty()).then_some(&settings.visibility_shortcut),
    )?;
    // Cmd+Q is already provided by the standard macOS application menu.
    let quit_shortcut = cfg!(target_os = "macos").then_some("Command+Q");
    let quit = MenuItem::with_id(app, "quit", "My Brush 종료", true, quit_shortcut)?;
    let menu = Menu::with_items(
        app,
        &[
            &control,
            &draw,
            &interact,
            &visibility,
            &cursor,
            &clear_current_item,
            &clear,
            &quit,
        ],
    )?;
    TrayIconBuilder::with_id("brush")
        .tooltip("My Brush — 화면 필기")
        .icon(tauri::image::Image::new_owned(tray_pixels(), 32, 32))
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, event| {
            let handle = app.clone();
            let event_id = event.id.as_ref().to_string();
            let _ = dispatch_native(app, move || {
                let app = &handle;

                if event_id.as_str() == "quit" {
                    let _ = release_inputs(app);
                    app.exit(0);
                    return;
                }
                if event_id.as_str() == "control" {
                    let _ = open_control(app);
                    return;
                }
                let state = lock(app);
                if let Ok(mut d) = state.0.lock() {
                    match event_id.as_str() {
                        "draw" => {
                            let _ = change_mode(app, &mut d, Mode::Draw);
                        }
                        "interact" => {
                            let _ = change_mode(app, &mut d, Mode::Interact);
                        }
                        "cursor" => {
                            if let Err(e) = toggle_cursor_highlight(app, &mut d) {
                                report(app, &mut d, e);
                            }
                        }
                        "visibility" => {
                            let _ = toggle_visibility(app, &mut d);
                        }
                        "clear-current" => {
                            if let Some(id) = d.app.active_display_id.clone() {
                                let _ = clear_scope_and_interact(app, &mut d, Some(id));
                            }
                        }
                        "clear" => {
                            let _ = clear_and_interact(app, &mut d);
                        }
                        _ => {}
                    }
                };
            });
        })
        .build(app)?;
    app.manage(TrayShortcuts {
        draw,
        clear,
        visibility,
        cursor,
    });
    Ok(())
}
fn tray_pixels() -> Vec<u8> {
    let mut pixels = vec![0; 32 * 32 * 4];
    for y in 4usize..28 {
        for x in 4usize..28 {
            if x.abs_diff(31 - y) < 4 {
                let i = (y * 32 + x) * 4;
                pixels[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    pixels
}

pub fn run() {
    tauri::Builder::default()
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "open-settings" {
                let handle = app.clone();
                let _ = dispatch_native(app, move || {
                    let _ = open_control(&handle);
                });
            }
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, key, event| {
                    let handle = app.clone();
                    let id = key.id();
                    let pressed = event.state == ShortcutState::Pressed;
                    let _ = dispatch_native(app, move || {
                        let state = lock(&handle);
                        if let Ok(mut d) = state.0.lock() {
                            if !pressed {
                                d.pressed.remove(&id);
                                return;
                            }
                            if d.capturing_shortcut {
                                return;
                            }
                            if !d.pressed.insert(id) {
                                return;
                            }
                            if Shortcut::from_str(&d.app.settings.toggle_shortcut)
                                .is_ok_and(|s| s.id() == id)
                            {
                                let next = if d.app.mode == Mode::Draw {
                                    Mode::Interact
                                } else {
                                    Mode::Draw
                                };
                                let _ = change_mode(&handle, &mut d, next);
                            } else if Shortcut::from_str(&d.app.settings.clear_shortcut)
                                .is_ok_and(|s| s.id() == id)
                            {
                                let _ = clear_and_interact(&handle, &mut d);
                            } else if Shortcut::from_str(&d.app.settings.visibility_shortcut)
                                .is_ok_and(|s| s.id() == id)
                            {
                                let _ = toggle_visibility(&handle, &mut d);
                            }
                        };
                    });
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_mode,
            select_display,
            update_settings,
            update_brush,
            update_preset,
            capture_shortcut,
            get_scene,
            apply_edit,
            clear_all,
            clear_current,
            toggle_annotations,
            toggle_cursor,
            get_cursor,
            undo,
            redo,
            show_control,
            quit_app
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let path = app.path().app_config_dir()?.join("settings.json");
            let (loaded, error) = match settings::load(&path) {
                Ok(s) => (s, None),
                Err(e) => (AppSettings::default(), Some(e)),
            };
            app.manage(Session(Mutex::new(RuntimeState {
                app: AppState {
                    cursor_enabled: false,
                    annotations_visible: true,
                    mode: Mode::Interact,
                    active_display_id: None,
                    displays: vec![],
                    settings: loaded.clone(),
                    brush: BrushSettings::from_defaults(&loaded),
                    revision: 0,
                    error,
                },
                scenes: SceneStore::default(),
                cursor: cursor::Stream::default(),
                previous_focus: Default::default(),
                settings_path: path,
                registered: vec![],
                pressed: HashSet::new(),
                capturing_shortcut: false,
            })));
            {
                let state = app.state::<Session>();
                state
                    .0
                    .lock()
                    .map_err(|e| e.to_string())?
                    .previous_focus
                    .capture();
            }
            WebviewWindowBuilder::new(
                app,
                "control",
                WebviewUrl::App("index.html?view=control".into()),
            )
            .title("My Brush")
            .inner_size(940., 760.)
            .min_inner_size(680., 540.)
            .build()?;
            #[cfg(target_os = "macos")]
            {
                let menu = Menu::default(app.handle())?;
                if let Some(submenu) = menu
                    .items()?
                    .first()
                    .and_then(|item| item.as_submenu())
                    .cloned()
                {
                    let settings_item = MenuItem::with_id(
                        app,
                        "open-settings",
                        "설정…",
                        true,
                        Some("Command+Comma"),
                    )?;
                    submenu.insert(&settings_item, 1)?;
                }
                app.set_menu(menu)?;
            }
            app.manage(cursor::Monitor::start(app.handle().clone()));
            tray(app.handle(), &loaded)?;
            {
                let state = app.state::<Session>();
                let mut d = state.0.lock().map_err(|e| e.to_string())?;
                let original_error = d.app.error.take();
                if let Err(e) = refresh_displays(app.handle(), &mut d) {
                    report(app.handle(), &mut d, e);
                }
                match shortcuts(&d.app.settings)
                    .and_then(|keys| replace_shortcuts(app.handle(), &mut d, keys))
                {
                    Ok(()) => {}
                    Err(e) => report(app.handle(), &mut d, e),
                }
                if d.app.error.is_none() {
                    d.app.error = original_error;
                }
                emit_state(app.handle(), &mut d);
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(3));
                let inner = handle.clone();
                if dispatch_native(&handle, move || {
                    let state = lock(&inner);
                    if let Ok(mut d) = state.0.lock() {
                        if let Err(e) = refresh_displays(&inner, &mut d) {
                            let _ = change_mode(&inner, &mut d, Mode::Interact);
                            if d.app.error.as_ref() != Some(&e) {
                                report(&inner, &mut d, e);
                            }
                        }
                    };
                })
                .is_err()
                {
                    break;
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "control" {
                if matches!(
                    event,
                    tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Focused(false)
                ) {
                    let app = window.app_handle().clone();
                    let handle = app.clone();
                    let _ = dispatch_native(&app, move || {
                        let s = lock(&handle);
                        if let Ok(mut d) = s.0.lock() {
                            let _ = end_shortcut_capture(&handle, &mut d);
                        };
                    });
                }
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("My Brush 실행에 실패했습니다");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_validation_rejects_equivalent_keys_and_bare_letters() {
        let mut settings = AppSettings {
            toggle_shortcut: "Alt+Shift+D".into(),
            clear_shortcut: "Shift+Alt+D".into(),
            ..AppSettings::default()
        };
        assert!(shortcuts(&settings).is_err());
        settings.clear_shortcut = "X".into();
        assert!(shortcuts(&settings).is_err());
        settings.clear_shortcut = "not a shortcut".into();
        assert!(shortcuts(&settings).is_err());
        assert!(shortcuts(&AppSettings::default()).is_ok());
    }
    #[test]
    fn recorded_key_codes_parse_and_settings_shortcut_is_reserved() {
        for key in [
            "Shift+Super+D",
            "Control+Alt+F6",
            "Alt+BracketLeft",
            "Alt+Backquote",
            "Alt+ArrowLeft",
        ] {
            assert!(Shortcut::from_str(key).is_ok(), "{key}");
        }
        let settings = AppSettings {
            toggle_shortcut: if cfg!(target_os = "macos") {
                "Command+Comma"
            } else {
                "Control+Comma"
            }
            .into(),
            ..AppSettings::default()
        };
        assert!(shortcuts(&settings).is_err());
    }
}
