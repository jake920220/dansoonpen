use crate::{
    model::{DisplayInfo, Mode, Point},
    RuntimeState,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, WebviewWindow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub display_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub collapsed: bool,
    pub detailed: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            display_id: None,
            x: 0.5,
            y: 0.,
            collapsed: false,
            detailed: false,
        }
    }
}
impl Preferences {
    pub fn valid(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && (0. ..=1.).contains(&self.x)
            && (0. ..=1.).contains(&self.y)
            && self
                .display_id
                .as_ref()
                .is_none_or(|id| !id.is_empty() && id.len() <= 256)
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub display_id: Option<String>,
    pub position: Preferences,
    pub revision: u64,
}
struct Drag {
    token: String,
    owner: String,
    offset_x: f64,
    offset_y: f64,
    width: f64,
    height: f64,
}
pub struct Toolbar {
    pub state: State,
    drag: Option<Drag>,
}
impl Toolbar {
    pub fn new(position: Preferences) -> Self {
        Self {
            state: State {
                display_id: None,
                position,
                revision: 0,
            },
            drag: None,
        }
    }
    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }
    fn owns_drag(&self, owner: &str, token: &str) -> bool {
        self.drag
            .as_ref()
            .is_some_and(|d| d.owner == owner && d.token == token)
    }
}
fn selected(displays: &[DisplayInfo], preferred: Option<&str>) -> Option<String> {
    displays
        .iter()
        .find(|d| d.connected && Some(d.id.as_str()) == preferred)
        .or_else(|| displays.iter().find(|d| d.connected && d.is_primary))
        .or_else(|| displays.iter().find(|d| d.connected))
        .map(|d| d.id.clone())
}
fn publish(app: &tauri::AppHandle, data: &mut RuntimeState) {
    data.toolbar.state.revision += 1;
    let _ = app.emit("brush-toolbar", &data.toolbar.state);
}
pub fn reconcile(app: &tauri::AppHandle, data: &mut RuntimeState) {
    let id = selected(
        &data.app.displays,
        data.toolbar.state.position.display_id.as_deref(),
    );
    if id != data.toolbar.state.display_id {
        data.toolbar.cancel_drag();
        data.toolbar.state.display_id = id;
        publish(app, data);
    }
}
fn normalized(coordinate: f64, viewport: f64, size: f64) -> f64 {
    let min = 10_f64.min(((viewport - size) / 2.).max(0.));
    let travel = (viewport - size - 2. * min).max(0.);
    if travel > 0. {
        ((coordinate - min) / travel).clamp(0., 1.)
    } else {
        0.5
    }
}
fn point_on_display(
    source: &DisplayInfo,
    target: &DisplayInfo,
    point: &Point,
    logical_desktop: bool,
) -> Point {
    if logical_desktop {
        Point {
            x: f64::from(source.x) / source.scale_factor + point.x
                - f64::from(target.x) / target.scale_factor,
            y: f64::from(source.y) / source.scale_factor + point.y
                - f64::from(target.y) / target.scale_factor,
        }
    } else {
        Point {
            x: (f64::from(source.x) + point.x * source.scale_factor - f64::from(target.x))
                / target.scale_factor,
            y: (f64::from(source.y) + point.y * source.scale_factor - f64::from(target.y))
                / target.scale_factor,
        }
    }
}
fn move_to_point(
    app: &tauri::AppHandle,
    data: &mut RuntimeState,
    point: &Point,
) -> Result<(), String> {
    let Some(drag) = &data.toolbar.drag else {
        return Ok(());
    };
    let Some(source) = data
        .app
        .displays
        .iter()
        .find(|d| d.connected && crate::label(&d.id) == drag.owner)
    else {
        return Ok(());
    };
    for display in data.app.displays.iter().filter(|d| d.connected) {
        // The captured pointer event remains relative to its source WebView even
        // outside that screen. Never re-read the mouse after pointer-up/IPC delay.
        let local = point_on_display(source, display, point, cfg!(target_os = "macos"));
        let cursor = crate::cursor::Reading::local(
            display.id.clone(),
            local.x,
            local.y,
            f64::from(display.width) / display.scale_factor,
            f64::from(display.height) / display.scale_factor,
            0,
        );
        if !cursor.visible {
            continue;
        }
        let width = f64::from(display.width) / display.scale_factor;
        let height = f64::from(display.height) / display.scale_factor;
        let mut next = data.toolbar.state.position.clone();
        next.display_id = Some(display.id.clone());
        next.x = normalized(
            cursor.x - drag.offset_x,
            width,
            drag.width.min((width - 20.).max(0.)),
        );
        next.y = normalized(cursor.y - drag.offset_y, height, drag.height);
        if next != data.toolbar.state.position || data.toolbar.state.display_id != next.display_id {
            data.toolbar.state.display_id = next.display_id.clone();
            data.toolbar.state.position = next;
            publish(app, data);
        }
        break;
    }
    Ok(())
}
fn save(app: &tauri::AppHandle, data: &mut RuntimeState) -> Result<(), String> {
    let mut settings = data.app.settings.clone();
    settings.toolbar = data.toolbar.state.position.clone();
    if let Err(error) = crate::settings::save(&data.settings_path, &settings) {
        data.toolbar.state.position = data.app.settings.toolbar.clone();
        data.toolbar.state.display_id = selected(
            &data.app.displays,
            data.toolbar.state.position.display_id.as_deref(),
        );
        publish(app, data);
        return Err(error);
    }
    data.app.settings = settings;
    crate::emit_state(app, data);
    Ok(())
}
#[tauri::command]
pub async fn get_toolbar(window: WebviewWindow, app: tauri::AppHandle) -> Result<State, String> {
    crate::native_command(app.clone(), "get_toolbar", move || {
        crate::authorized(&window)?;
        Ok(crate::lock(&app)
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .toolbar
            .state
            .clone())
    })
    .await
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DragStart {
    token: String,
    offset_x: f64,
    offset_y: f64,
    width: f64,
    height: f64,
}
#[tauri::command]
pub async fn begin_toolbar_drag(
    window: WebviewWindow,
    app: tauri::AppHandle,
    drag: DragStart,
) -> Result<(), String> {
    crate::native_command(app.clone(), "begin_toolbar_drag", move || {
        let state = crate::lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        if data.app.mode != Mode::Draw
            || data
                .toolbar
                .state
                .display_id
                .as_deref()
                .map(crate::label)
                .as_deref()
                != Some(window.label())
        {
            return Err("표시된 도구막대의 손잡이를 사용해 주세요.".into());
        }
        if drag.token.is_empty()
            || drag.token.len() > 64
            || ![drag.offset_x, drag.offset_y, drag.width, drag.height]
                .iter()
                .all(|n| n.is_finite() && (0. ..=10000.).contains(n))
            || drag.width == 0.
            || drag.height == 0.
        {
            return Err("도구막대 이동 좌표가 올바르지 않습니다.".into());
        }
        data.toolbar.drag = Some(Drag {
            token: drag.token,
            owner: window.label().into(),
            offset_x: drag.offset_x,
            offset_y: drag.offset_y,
            width: drag.width,
            height: drag.height,
        });
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn move_toolbar(
    window: WebviewWindow,
    app: tauri::AppHandle,
    token: String,
    point: Point,
    finish: bool,
) -> Result<(), String> {
    crate::native_command(app.clone(), "move_toolbar", move || {
        let state = crate::lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        if data.app.mode != Mode::Draw || !data.toolbar.owns_drag(window.label(), &token) {
            return Ok(());
        }
        if ![point.x, point.y]
            .iter()
            .all(|n| n.is_finite() && n.abs() <= 100_000.)
        {
            return Err("도구막대 이동 좌표가 올바르지 않습니다.".into());
        }
        let result = move_to_point(&app, &mut data, &point);
        if finish {
            data.toolbar.cancel_drag();
            result?;
            save(&app, &mut data)?;
        } else {
            result?;
        }
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn update_toolbar(
    window: WebviewWindow,
    app: tauri::AppHandle,
    position: Preferences,
) -> Result<(), String> {
    crate::native_command(app.clone(), "update_toolbar", move || {
        let state = crate::lock(&app);
        let mut data = state.0.lock().map_err(|e| e.to_string())?;
        if !position.valid()
            || data.toolbar.drag.is_some()
            || data.toolbar.state.display_id != position.display_id
            || position.display_id.as_deref().map(crate::label).as_deref() != Some(window.label())
        {
            return Err("현재 도구막대에서 위치를 바꿔 주세요.".into());
        }
        data.toolbar.state.position = position;
        publish(&app, &mut data);
        save(&app, &mut data)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn display(id: &str, connected: bool, is_primary: bool) -> DisplayInfo {
        DisplayInfo {
            id: id.into(),
            name: id.into(),
            x: 0,
            y: 0,
            width: 2560,
            height: 1440,
            scale_factor: 1.,
            connected,
            is_primary,
        }
    }
    #[test]
    fn preferred_panel_screen_returns_after_temporary_disconnect() {
        let mut screens = vec![display("main", true, true), display("retina", true, false)];
        assert_eq!(selected(&screens, None).as_deref(), Some("main"));
        assert_eq!(
            selected(&screens, Some("retina")).as_deref(),
            Some("retina")
        );
        screens[1].connected = false;
        assert_eq!(selected(&screens, Some("retina")).as_deref(), Some("main"));
        screens[1].connected = true;
        assert_eq!(
            selected(&screens, Some("retina")).as_deref(),
            Some("retina")
        );
    }
    #[test]
    fn mixed_dpi_drop_keeps_grip_offset_and_panel_inside_destination() {
        assert_eq!(normalized(110., 1470., 600.), 100. / 850.);
        assert_eq!(normalized(-30., 2560., 600.), 0.);
        assert_eq!(normalized(2500., 2560., 600.), 1.);
        assert_eq!(normalized(100., 300., 340.), 0.5);
    }
    #[test]
    fn captured_drag_point_crosses_mixed_dpi_displays_in_both_directions() {
        let main = display("main", true, true);
        let mut retina = display("retina", true, false);
        retina.x = 5120;
        retina.y = 968;
        retina.width = 2940;
        retina.height = 1912;
        retina.scale_factor = 2.;
        let target = point_on_display(&main, &retina, &Point { x: 2850., y: 700. }, true);
        assert_eq!((target.x, target.y), (290., 216.));
        let back = point_on_display(&retina, &main, &Point { x: -500., y: 200. }, true);
        assert_eq!((back.x, back.y), (2060., 684.));
        let same = point_on_display(&main, &main, &Point { x: 400., y: 200. }, true);
        assert_eq!((same.x, same.y), (400., 200.));
        retina.x = 2560;
        retina.y = 0;
        let windows = point_on_display(&main, &retina, &Point { x: 2850., y: 700. }, false);
        assert_eq!((windows.x, windows.y), (145., 350.));
    }
    #[test]
    fn stale_or_foreign_drag_cannot_move_the_panel() {
        let mut toolbar = Toolbar::new(Preferences::default());
        toolbar.drag = Some(Drag {
            token: "new".into(),
            owner: "a".into(),
            offset_x: 10.,
            offset_y: 10.,
            width: 600.,
            height: 44.,
        });
        assert!(toolbar.owns_drag("a", "new"));
        assert!(!toolbar.owns_drag("a", "old"));
        assert!(!toolbar.owns_drag("b", "new"));
        toolbar.cancel_drag();
        assert!(!toolbar.owns_drag("a", "new"));
        assert!(!Preferences {
            x: f64::NAN,
            ..Preferences::default()
        }
        .valid());
    }
}
