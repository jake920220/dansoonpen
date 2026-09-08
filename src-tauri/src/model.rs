use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub const MAX_HISTORY: usize = 128;
pub const MAX_HISTORY_BYTES: usize = 128 * 1024 * 1024;
const MAX_SCENE_BYTES: usize = 64 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Draw,
    Interact,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Annotation {
    Stroke {
        id: String,
        points: Vec<Point>,
        color: String,
        width: f64,
        #[serde(default = "opaque")]
        opacity: f64,
    },
    Arrow {
        id: String,
        start: Point,
        end: Point,
        color: String,
        width: f64,
    },
    Text {
        id: String,
        x: f64,
        y: f64,
        text: String,
        color: String,
        #[serde(rename = "fontSize")]
        font_size: f64,
    },
}
impl Annotation {
    pub fn id(&self) -> &str {
        match self {
            Self::Stroke { id, .. } | Self::Text { id, .. } | Self::Arrow { id, .. } => id,
        }
    }
    pub fn bytes(&self) -> usize {
        128 + self.id().len()
            + match self {
                Self::Stroke { points, .. } => points.len() * 16,
                Self::Text { text, .. } => text.len(),
                Self::Arrow { .. } => 32,
            }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id().is_empty() || self.id().len() > 128 {
            return Err("주석 ID가 올바르지 않습니다.".into());
        }
        let valid = match self {
            Self::Stroke {
                points,
                color,
                width,
                opacity,
                ..
            } => {
                !points.is_empty()
                    && points.len() <= 100_000
                    && points.iter().all(|p| coordinate(p.x) && coordinate(p.y))
                    && valid_color(color)
                    && finite_range(*width, 1., 64.)
                    && finite_range(*opacity, 0.1, 1.)
            }
            Self::Arrow {
                start,
                end,
                color,
                width,
                ..
            } => {
                coordinate(start.x)
                    && coordinate(start.y)
                    && coordinate(end.x)
                    && coordinate(end.y)
                    && (end.x - start.x).hypot(end.y - start.y) >= 2.
                    && valid_color(color)
                    && finite_range(*width, 1., 32.)
            }
            Self::Text {
                x,
                y,
                text,
                color,
                font_size,
                ..
            } => {
                coordinate(*x)
                    && coordinate(*y)
                    && !text.trim().is_empty()
                    && text.len() <= 40_000
                    && !text.contains('\0')
                    && valid_color(color)
                    && finite_range(*font_size, 8., 144.)
            }
        };
        if valid {
            Ok(())
        } else {
            Err("주석 좌표·색상·크기 또는 길이가 허용 범위를 벗어났습니다.".into())
        }
    }
}
fn opaque() -> f64 {
    1.
}
fn marker_width() -> f64 {
    24.
}
fn marker_opacity() -> f64 {
    0.32
}
fn coordinate(n: f64) -> bool {
    finite_range(n, -100_000., 100_000.)
}
fn finite_range(n: f64, min: f64, max: f64) -> bool {
    n.is_finite() && n >= min && n <= max
}
pub fn valid_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Pen,
    Eraser,
    Text,
    Arrow,
    Highlighter,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrushSettings {
    #[serde(default = "eraser_size")]
    pub eraser_size: f64,
    #[serde(default = "marker_width")]
    pub highlighter_width: f64,
    #[serde(default = "marker_opacity")]
    pub highlighter_opacity: f64,
    pub tool: Tool,
    pub color: String,
    pub width: f64,
    pub text_size: f64,
}
impl BrushSettings {
    pub fn from_defaults(settings: &AppSettings) -> Self {
        Self {
            eraser_size: settings.eraser_size,
            tool: Tool::Pen,
            highlighter_width: marker_width(),
            highlighter_opacity: marker_opacity(),
            color: settings.color.clone(),
            width: settings.width,
            text_size: settings.text_size,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if finite_range(self.eraser_size, 16., 128.)
            && valid_color(&self.color)
            && finite_range(self.width, 1., 32.)
            && finite_range(self.text_size, 8., 144.)
            && finite_range(self.highlighter_width, 8., 64.)
            && finite_range(self.highlighter_opacity, 0.1, 0.8)
        {
            Ok(())
        } else {
            Err("도구 색상·굵기·글자 크기가 허용 범위를 벗어났습니다.".into())
        }
    }
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrushPatch {
    pub eraser_size: Option<f64>,
    pub highlighter_width: Option<f64>,
    pub highlighter_opacity: Option<f64>,
    pub tool: Option<Tool>,
    pub color: Option<String>,
    pub width: Option<f64>,
    pub text_size: Option<f64>,
}
impl BrushPatch {
    pub fn apply(self, current: &BrushSettings) -> BrushSettings {
        BrushSettings {
            eraser_size: self.eraser_size.unwrap_or(current.eraser_size),
            highlighter_width: self.highlighter_width.unwrap_or(current.highlighter_width),
            highlighter_opacity: self
                .highlighter_opacity
                .unwrap_or(current.highlighter_opacity),
            tool: self.tool.unwrap_or(current.tool),
            color: self.color.unwrap_or_else(|| current.color.clone()),
            width: self.width.unwrap_or(current.width),
            text_size: self.text_size.unwrap_or(current.text_size),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrushPreset {
    pub name: String,
    pub brush: BrushSettings,
}
fn default_presets() -> Vec<BrushPreset> {
    [
        ("기본 강조", Tool::Pen, "#ffcf56", 12., 40.),
        ("빨간 밑줄", Tool::Pen, "#ff6b6b", 4., 40.),
        ("민트 메모", Tool::Text, "#57d9c6", 6., 44.),
    ]
    .into_iter()
    .map(|(name, tool, color, width, text_size)| BrushPreset {
        name: name.into(),
        brush: BrushSettings {
            eraser_size: eraser_size(),
            highlighter_width: marker_width(),
            highlighter_opacity: marker_opacity(),
            tool,
            color: color.into(),
            width,
            text_size,
        },
    })
    .collect()
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CursorSettings {
    pub color: String,
    pub size: f64,
    pub show_clicks: bool,
}
impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            color: "#ffcf56".into(),
            size: 48.,
            show_clicks: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    #[serde(default = "eraser_size")]
    pub eraser_size: f64,
    #[serde(default)]
    pub cursor: CursorSettings,
    #[serde(default = "default_presets")]
    pub presets: Vec<BrushPreset>,
    pub version: u32,
    pub color: String,
    pub width: f64,
    pub text_size: f64,
    pub quick_colors: Vec<String>,
    pub toggle_shortcut: String,
    pub clear_shortcut: String,
    #[serde(default = "visibility_shortcut")]
    pub visibility_shortcut: String,
    pub reduce_motion: bool,
}
fn eraser_size() -> f64 {
    48.
}
fn visibility_shortcut() -> String {
    if cfg!(target_os = "macos") {
        "Alt+V"
    } else {
        "Alt+Shift+V"
    }
    .into()
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 5,
            eraser_size: eraser_size(),
            cursor: CursorSettings::default(),
            presets: default_presets(),
            color: "#ffcf56".into(),
            width: 12.,
            text_size: 40.,
            quick_colors: [
                "#ffcf56", "#ff6b6b", "#57d9c6", "#78a9ff", "#c4a0ff", "#ffffff",
            ]
            .map(String::from)
            .to_vec(),
            toggle_shortcut: if cfg!(target_os = "macos") {
                "Alt+Z"
            } else {
                "Alt+Shift+Z"
            }
            .into(),
            clear_shortcut: if cfg!(target_os = "macos") {
                "Alt+X"
            } else {
                "Alt+Shift+X"
            }
            .into(),
            visibility_shortcut: visibility_shortcut(),
            reduce_motion: false,
        }
    }
}
impl AppSettings {
    pub fn replace_preset(
        &self,
        index: usize,
        name: Option<String>,
        brush: Option<BrushSettings>,
    ) -> Result<Self, String> {
        let mut next = self.clone();
        let preset = next
            .presets
            .get_mut(index)
            .ok_or("프리셋 번호가 올바르지 않습니다.")?;
        if let Some(name) = name {
            preset.name = name.trim().into();
        }
        if let Some(brush) = brush {
            preset.brush = brush;
        }
        next.validate()?;
        Ok(next)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !valid_color(&self.cursor.color)
            || !finite_range(self.cursor.size, 24., 96.)
            || self.version != 5
            || !finite_range(self.eraser_size, 16., 128.)
            || self.presets.len() != 3
            || self.presets.iter().any(|p| {
                p.name.trim().is_empty()
                    || p.name.chars().count() > 24
                    || p.name.chars().any(char::is_control)
                    || p.brush.validate().is_err()
            })
            || !valid_color(&self.color)
            || !finite_range(self.width, 1., 32.)
            || !finite_range(self.text_size, 8., 144.)
            || self.quick_colors.len() != 6
            || !self.quick_colors.iter().all(|s| valid_color(s))
            || self.toggle_shortcut.len() > 80
            || self.clear_shortcut.len() > 80
            || self.visibility_shortcut.len() > 80
        {
            return Err("설정 형식 또는 허용 범위가 올바르지 않습니다.".into());
        }
        Ok(())
    }
}
// Apply only fields edited by a window, so delayed palette writes cannot overwrite
// shortcut or color changes made in another window.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    pub eraser_size: Option<f64>,
    pub cursor: Option<CursorSettings>,
    pub presets: Option<Vec<BrushPreset>>,
    pub version: Option<u32>,
    pub color: Option<String>,
    pub width: Option<f64>,
    pub text_size: Option<f64>,
    pub quick_colors: Option<Vec<String>>,
    pub toggle_shortcut: Option<String>,
    pub clear_shortcut: Option<String>,
    pub visibility_shortcut: Option<String>,
    pub reduce_motion: Option<bool>,
}
impl SettingsPatch {
    pub fn brush_patch(&self) -> BrushPatch {
        BrushPatch {
            eraser_size: self.eraser_size,
            color: self.color.clone(),
            width: self.width,
            text_size: self.text_size,
            ..Default::default()
        }
    }
    pub fn apply(self, current: &AppSettings) -> AppSettings {
        AppSettings {
            eraser_size: self.eraser_size.unwrap_or(current.eraser_size),
            cursor: self.cursor.unwrap_or_else(|| current.cursor.clone()),
            presets: self.presets.unwrap_or_else(|| current.presets.clone()),
            version: self.version.unwrap_or(current.version),
            color: self.color.unwrap_or_else(|| current.color.clone()),
            width: self.width.unwrap_or(current.width),
            text_size: self.text_size.unwrap_or(current.text_size),
            quick_colors: self
                .quick_colors
                .unwrap_or_else(|| current.quick_colors.clone()),
            toggle_shortcut: self
                .toggle_shortcut
                .unwrap_or_else(|| current.toggle_shortcut.clone()),
            clear_shortcut: self
                .clear_shortcut
                .unwrap_or_else(|| current.clear_shortcut.clone()),
            visibility_shortcut: self
                .visibility_shortcut
                .unwrap_or_else(|| current.visibility_shortcut.clone()),
            reduce_motion: self.reduce_motion.unwrap_or(current.reduce_motion),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub is_primary: bool,
    pub connected: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: u64,
    pub message: String,
    pub created_at_ms: u64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub feedback: Option<Feedback>,
    pub clear_undo_token: Option<u64>,
    pub brush_generation: u64,
    pub cursor_enabled: bool,
    pub annotations_visible: bool,
    pub mode: Mode,
    pub active_display_id: Option<String>,
    pub displays: Vec<DisplayInfo>,
    pub settings: AppSettings,
    pub brush: BrushSettings,
    pub revision: u64,
    pub error: Option<String>,
}
impl AppState {
    // Called only after native input/focus transition succeeds. Re-selecting a
    // display while already drawing must not reset the current tool.
    pub fn complete_mode(&mut self, mode: Mode) {
        if mode == Mode::Draw && self.mode != Mode::Draw {
            self.brush_generation += 1;
            self.brush = BrushSettings::from_defaults(&self.settings);
        }
        self.mode = mode;
        if mode == Mode::Draw {
            self.annotations_visible = true;
        }
    }
    pub fn apply_brush(&mut self, patch: BrushPatch, generation: u64) -> Result<(), String> {
        // Ignore delayed palette/tool writes belonging to an earlier drawing entry.
        if generation != self.brush_generation {
            return Ok(());
        }
        let brush = patch.apply(&self.brush);
        brush.validate()?;
        self.brush = brush;
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSnapshot {
    pub display_id: String,
    pub clear_generation: u64,
    pub revision: u64,
    pub annotations: Vec<Arc<Annotation>>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneUpdate {
    pub scene: SceneSnapshot,
    pub fade_out: Vec<Arc<Annotation>>,
    pub fade_duration_ms: u32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneEdit {
    pub display_id: String,
    pub clear_generation: u64,
    pub added: Vec<Annotation>,
    pub removed_ids: Vec<String>,
}
#[derive(Default)]
struct Scene {
    clear_generation: u64,
    revision: u64,
    items: Vec<Arc<Annotation>>,
}
struct Change {
    display: String,
    before: Vec<Arc<Annotation>>,
    after: Vec<Arc<Annotation>>,
}
struct Action {
    changes: Vec<Change>,
    bytes: usize,
}
#[derive(Default)]
pub struct SceneStore {
    history_revision: u64,
    scenes: HashMap<String, Scene>,
    undo: VecDeque<Action>,
    redo: Vec<Action>,
}
impl SceneStore {
    pub fn history_revision(&self) -> u64 {
        self.history_revision
    }
    pub fn undo_token(&self) -> Option<u64> {
        (!self.undo.is_empty()).then_some(self.history_revision)
    }
    pub fn undo_if_unchanged(&mut self, token: u64) -> Vec<SceneUpdate> {
        if token != self.history_revision {
            return vec![];
        }
        self.undo()
    }

    pub fn ensure(&mut self, id: &str) {
        self.scenes.entry(id.into()).or_default();
    }
    pub fn snapshot(&self, id: &str) -> Result<SceneSnapshot, String> {
        let s = self.scenes.get(id).ok_or("알 수 없는 화면입니다.")?;
        Ok(SceneSnapshot {
            display_id: id.into(),
            clear_generation: s.clear_generation,
            revision: s.revision,
            annotations: s.items.clone(),
        })
    }
    fn commit(&mut self, changes: Vec<Change>, fade: u32) -> Vec<SceneUpdate> {
        if changes.is_empty() {
            return vec![];
        }
        self.history_revision += 1;
        let updates = self.install(&changes, false, fade);
        // Conservative accounting includes all retained references and annotation payloads.
        // Arc shares actual payloads between the live scene and every history snapshot.
        let bytes = changes
            .iter()
            .map(|c| {
                c.before
                    .iter()
                    .chain(&c.after)
                    .map(|a| a.bytes())
                    .sum::<usize>()
            })
            .sum();
        self.undo.push_back(Action { changes, bytes });
        self.redo.clear();
        while self.undo.len() > MAX_HISTORY
            || self.undo.iter().map(|a| a.bytes).sum::<usize>() > MAX_HISTORY_BYTES
        {
            self.undo.pop_front();
        }
        updates
    }
    fn install(&mut self, changes: &[Change], backwards: bool, fade: u32) -> Vec<SceneUpdate> {
        changes
            .iter()
            .map(|c| {
                let next = if backwards { &c.before } else { &c.after };
                let scene = self.scenes.entry(c.display.clone()).or_default();
                let ids: HashSet<_> = next.iter().map(|a| a.id()).collect();
                let removed = scene
                    .items
                    .iter()
                    .filter(|a| !ids.contains(a.id()))
                    .cloned()
                    .collect();
                scene.items = next.clone();
                scene.revision += 1;
                SceneUpdate {
                    scene: SceneSnapshot {
                        display_id: c.display.clone(),
                        clear_generation: scene.clear_generation,
                        revision: scene.revision,
                        annotations: scene.items.clone(),
                    },
                    fade_out: removed,
                    fade_duration_ms: fade,
                }
            })
            .collect()
    }
    pub fn apply(&mut self, edit: SceneEdit, fade: u32) -> Result<Vec<SceneUpdate>, String> {
        let scene = self
            .scenes
            .get(&edit.display_id)
            .ok_or("알 수 없는 화면입니다.")?;
        // Clear barriers are display-local: pending edits on other screens survive.
        if edit.clear_generation < scene.clear_generation {
            return Ok(vec![]);
        }
        if edit.clear_generation > scene.clear_generation {
            return Err("필기 세대가 현재 화면보다 앞섭니다. 화면을 다시 불러오세요.".into());
        }
        if edit.added.len() > 1000 || edit.removed_ids.len() > 100_000 {
            return Err("한 번에 편집할 수 있는 주석 수를 초과했습니다.".into());
        }
        let ids: HashSet<&str> = scene.items.iter().map(|a| a.id()).collect();
        let removed: HashSet<&str> = edit.removed_ids.iter().map(String::as_str).collect();
        if removed.len() != edit.removed_ids.len() || !removed.iter().all(|id| ids.contains(*id)) {
            return Err("삭제 대상이 오래되었거나 중복되었습니다. 화면을 다시 불러오세요.".into());
        }
        // Only explicit text replacements can retain an ID. Preserve its layer
        // and one history action; ordinary additions still require fresh IDs.
        let mut added_ids = HashSet::new();
        for a in &edit.added {
            a.validate()?;
            let replacing_text = removed.contains(a.id())
                && matches!(a, Annotation::Text { .. })
                && scene.items.iter().any(|old| {
                    old.id() == a.id() && matches!(old.as_ref(), Annotation::Text { .. })
                });
            if !added_ids.insert(a.id()) || (ids.contains(a.id()) && !replacing_text) {
                return Err("중복된 주석 ID입니다.".into());
            }
        }
        let added: Vec<_> = edit.added.into_iter().map(Arc::new).collect();
        let replacements: HashMap<_, _> = added.iter().map(|a| (a.id(), a.clone())).collect();
        let mut after: Vec<_> = scene
            .items
            .iter()
            .filter_map(|a| {
                replacements
                    .get(a.id())
                    .cloned()
                    .or_else(|| (!removed.contains(a.id())).then(|| a.clone()))
            })
            .collect();
        after.extend(added.iter().filter(|a| !ids.contains(a.id())).cloned());
        let total = self
            .scenes
            .iter()
            .filter(|(id, _)| **id != edit.display_id)
            .map(|(_, s)| s.items.iter().map(|a| a.bytes()).sum::<usize>())
            .sum::<usize>()
            + after.iter().map(|a| a.bytes()).sum::<usize>();
        if total > MAX_SCENE_BYTES {
            return Err(
                "필기 메모리 한도(64 MiB)에 도달했습니다. 기존 필기를 지운 뒤 다시 시도하세요."
                    .into(),
            );
        }
        if removed.is_empty() && after.len() == scene.items.len() {
            return Ok(vec![]);
        }
        let before = scene.items.clone();
        Ok(self.commit(
            vec![Change {
                display: edit.display_id,
                before,
                after,
            }],
            fade,
        ))
    }
    pub fn clear(&mut self, fade: u32) -> Vec<SceneUpdate> {
        self.clear_displays(self.scenes.keys().cloned().collect(), fade)
    }
    pub fn clear_display(&mut self, id: &str, fade: u32) -> Result<Vec<SceneUpdate>, String> {
        if !self.scenes.contains_key(id) {
            return Err("알 수 없는 화면입니다.".into());
        }
        Ok(self.clear_displays(vec![id.into()], fade))
    }
    fn clear_displays(&mut self, ids: Vec<String>, fade: u32) -> Vec<SceneUpdate> {
        let mut changes = Vec::new();
        let mut empty_updates = Vec::new();
        for id in ids {
            let scene = self.scenes.get_mut(&id).expect("known display");
            scene.clear_generation += 1;
            if scene.items.is_empty() {
                // An empty native scene may have an in-flight stroke in its webview.
                scene.revision += 1;
                empty_updates.push(SceneUpdate {
                    scene: SceneSnapshot {
                        display_id: id,
                        clear_generation: scene.clear_generation,
                        revision: scene.revision,
                        annotations: vec![],
                    },
                    fade_out: vec![],
                    fade_duration_ms: fade,
                });
            } else {
                changes.push(Change {
                    display: id,
                    before: scene.items.clone(),
                    after: vec![],
                });
            }
        }
        let mut updates = self.commit(changes, fade);
        updates.extend(empty_updates);
        updates
    }
    pub fn undo(&mut self) -> Vec<SceneUpdate> {
        if let Some(a) = self.undo.pop_back() {
            self.history_revision += 1;
            let updates = self.install(&a.changes, true, 0);
            self.redo.push(a);
            updates
        } else {
            vec![]
        }
    }
    pub fn redo(&mut self, fade: u32) -> Vec<SceneUpdate> {
        if let Some(a) = self.redo.pop() {
            self.history_revision += 1;
            let updates = self.install(&a.changes, false, fade);
            self.undo.push_back(a);
            updates
        } else {
            vec![]
        }
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn reentry_resets_to_saved_pen_and_rejects_late_tool_writes() {
        let settings = AppSettings {
            width: 14.,
            color: "#57d9c6".into(),
            ..Default::default()
        };
        let mut state = AppState {
            feedback: None,
            clear_undo_token: None,
            brush_generation: 0,
            cursor_enabled: false,
            annotations_visible: true,
            mode: Mode::Interact,
            active_display_id: None,
            displays: vec![],
            brush: BrushSettings::from_defaults(&settings),
            settings,
            revision: 0,
            error: None,
        };
        state.complete_mode(Mode::Draw);
        let first = state.brush_generation;
        state
            .apply_brush(
                BrushPatch {
                    tool: Some(Tool::Text),
                    color: Some("#ff6b6b".into()),
                    ..Default::default()
                },
                first,
            )
            .unwrap();
        state.complete_mode(Mode::Draw); // Focus/display re-selection is not a fresh entry.
        assert_eq!(state.brush.tool, Tool::Text);
        assert_eq!(state.brush_generation, first);
        state.complete_mode(Mode::Interact);
        assert_eq!(state.brush.tool, Tool::Text);
        state.complete_mode(Mode::Draw);
        assert_eq!(state.brush, BrushSettings::from_defaults(&state.settings));
        assert_eq!(state.brush.width, 14.);
        state
            .apply_brush(
                BrushPatch {
                    tool: Some(Tool::Eraser),
                    width: Some(32.),
                    ..Default::default()
                },
                first,
            )
            .unwrap();
        assert_eq!(state.brush, BrushSettings::from_defaults(&state.settings));
        state
            .apply_brush(
                BrushPatch {
                    tool: Some(Tool::Highlighter),
                    ..Default::default()
                },
                state.brush_generation,
            )
            .unwrap();
        assert_eq!(state.brush.tool, Tool::Highlighter);
    }
    use super::*;
    fn text_note(id: &str, text: &str) -> Annotation {
        Annotation::Text {
            id: id.into(),
            x: 30.,
            y: 60.,
            text: text.into(),
            color: "#ffcf56".into(),
            font_size: 40.,
        }
    }
    #[test]
    fn clear_receipt_restores_all_displays_once_and_cannot_undo_new_work() {
        let mut store = SceneStore::default();
        store.ensure("a");
        store.ensure("b");
        add(&mut store, "a", "old-a");
        add(&mut store, "b", "old-b");
        store.clear(350);
        let receipt = store.undo_token().unwrap();
        let restored = store.undo_if_unchanged(receipt);
        assert_eq!(restored.len(), 2);
        assert_eq!(store.snapshot("a").unwrap().annotations[0].id(), "old-a");
        assert_eq!(store.snapshot("b").unwrap().annotations[0].id(), "old-b");
        assert!(store.undo_if_unchanged(receipt).is_empty());
        store.clear(350);
        let receipt = store.undo_token().unwrap();
        add(&mut store, "a", "fresh");
        assert!(store.undo_if_unchanged(receipt).is_empty());
        assert_eq!(store.snapshot("a").unwrap().annotations[0].id(), "fresh");
        store.undo();
        store.redo(350);
        assert!(store.undo_if_unchanged(receipt).is_empty());
        assert_eq!(store.snapshot("a").unwrap().annotations[0].id(), "fresh");
    }
    #[test]
    fn text_position_and_style_are_one_undoable_replacement() {
        let mut store = SceneStore::default();
        store.ensure("a");
        let old = text_note("text", "내용 유지");
        store
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 0,
                    added: vec![old.clone()],
                    removed_ids: vec![],
                },
                350,
            )
            .unwrap();
        let mut changed = old.clone();
        if let Annotation::Text {
            x,
            y,
            color,
            font_size,
            ..
        } = &mut changed
        {
            *x = 200.;
            *y = 300.;
            *color = "#57d9c6".into();
            *font_size = 64.;
        }
        let updates = store
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 0,
                    added: vec![changed.clone()],
                    removed_ids: vec!["text".into()],
                },
                350,
            )
            .unwrap();
        assert!(updates[0].fade_out.is_empty());
        assert_eq!(store.snapshot("a").unwrap().annotations.len(), 1);
        store.undo();
        assert_eq!(
            serde_json::to_value(&store.snapshot("a").unwrap().annotations[0]).unwrap(),
            serde_json::to_value(&old).unwrap()
        );
        store.redo(350);
        assert_eq!(
            serde_json::to_value(&store.snapshot("a").unwrap().annotations[0]).unwrap(),
            serde_json::to_value(&changed).unwrap()
        );
        store.clear(350);
        assert!(store
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 0,
                    added: vec![changed],
                    removed_ids: vec!["text".into()]
                },
                350
            )
            .unwrap()
            .is_empty());
        assert!(store.snapshot("a").unwrap().annotations.is_empty());
    }
    #[test]
    fn text_replacement_keeps_its_layer_and_undo_redo_without_ghost_fade() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.apply(
            SceneEdit {
                display_id: "a".into(),
                clear_generation: 0,
                added: vec![text_note("text", "원문"), stroke("above")],
                removed_ids: vec![],
            },
            350,
        )
        .unwrap();
        let updates = s
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 0,
                    added: vec![text_note("text", "수정한 글\n두 번째 줄")],
                    removed_ids: vec!["text".into()],
                },
                350,
            )
            .unwrap();
        assert!(updates[0].fade_out.is_empty());
        assert_eq!(
            s.snapshot("a")
                .unwrap()
                .annotations
                .iter()
                .map(|a| a.id())
                .collect::<Vec<_>>(),
            vec!["text", "above"]
        );
        s.undo();
        assert!(
            matches!(s.snapshot("a").unwrap().annotations[0].as_ref(), Annotation::Text { text, .. } if text == "원문")
        );
        assert!(s.redo(350)[0].fade_out.is_empty());
        assert!(
            matches!(s.snapshot("a").unwrap().annotations[0].as_ref(), Annotation::Text { text, .. } if text == "수정한 글\n두 번째 줄")
        );
        let generation = s.snapshot("a").unwrap().clear_generation;
        s.clear(350);
        assert!(s
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: generation,
                    added: vec![text_note("text", "늦은 수정")],
                    removed_ids: vec!["text".into()]
                },
                350
            )
            .unwrap()
            .is_empty());
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
    }
    #[test]
    fn repeated_scoped_clears_reject_old_text_but_keep_other_display_edits() {
        let mut store = SceneStore::default();
        store.ensure("a");
        store.ensure("b");
        for round in 0..100 {
            store.clear(350);
            let generation_a = store.snapshot("a").unwrap().clear_generation;
            let generation_b = store.snapshot("b").unwrap().clear_generation;
            store
                .apply(
                    SceneEdit {
                        display_id: "a".into(),
                        clear_generation: generation_a,
                        added: vec![text_note("text", "원문")],
                        removed_ids: vec![],
                    },
                    350,
                )
                .unwrap();
            store.clear_display("a", 350).unwrap();
            assert!(store
                .apply(
                    SceneEdit {
                        display_id: "a".into(),
                        clear_generation: generation_a,
                        added: vec![text_note("text", "늦은 IME 확정")],
                        removed_ids: vec!["text".into()],
                    },
                    350
                )
                .unwrap()
                .is_empty());
            store
                .apply(
                    SceneEdit {
                        display_id: "b".into(),
                        clear_generation: generation_b,
                        added: vec![text_note("other", &format!("다른 화면 {round}"))],
                        removed_ids: vec![],
                    },
                    350,
                )
                .unwrap();
            add(&mut store, "a", "fresh");
            assert_eq!(store.snapshot("a").unwrap().annotations[0].id(), "fresh");
            assert_eq!(store.snapshot("b").unwrap().annotations.len(), 1);
            store.undo(); // Only the new A stroke is undone.
            assert!(store.snapshot("a").unwrap().annotations.is_empty());
            assert_eq!(store.snapshot("b").unwrap().annotations.len(), 1);
            store.redo(350);
            assert_eq!(store.snapshot("a").unwrap().annotations[0].id(), "fresh");
            assert!(store.undo.len() <= MAX_HISTORY);
        }
    }
    #[test]
    fn text_replacement_rejects_implicit_or_duplicate_ids_and_wrong_types_atomically() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.apply(
            SceneEdit {
                display_id: "a".into(),
                clear_generation: 0,
                added: vec![text_note("text", "원문"), stroke("stroke")],
                removed_ids: vec![],
            },
            350,
        )
        .unwrap();
        for (added, removed_ids) in [
            (vec![text_note("text", "수정")], vec![]),
            (
                vec![text_note("text", "수정"), text_note("text", "중복")],
                vec!["text".into()],
            ),
            (vec![stroke("text")], vec!["text".into()]),
            (
                vec![text_note("stroke", "타입 변경")],
                vec!["stroke".into()],
            ),
        ] {
            let revision = s.snapshot("a").unwrap().revision;
            assert!(s
                .apply(
                    SceneEdit {
                        display_id: "a".into(),
                        clear_generation: 0,
                        added,
                        removed_ids
                    },
                    350
                )
                .is_err());
            assert_eq!(s.snapshot("a").unwrap().revision, revision);
        }
    }
    #[test]
    fn arrow_and_highlighter_validation_and_legacy_defaults() {
        let legacy = r##"{"kind":"stroke","id":"old","points":[{"x":1,"y":2}],"color":"#ffcf56","width":12}"##;
        let annotation: Annotation = serde_json::from_str(legacy).unwrap();
        annotation.validate().unwrap();
        assert!(matches!(annotation, Annotation::Stroke { opacity: 1., .. }));
        let marker: Annotation =
            serde_json::from_str(&legacy.replace("12}", "64,\"opacity\":0.32}")).unwrap();
        marker.validate().unwrap();
        let mut bad = marker.clone();
        if let Annotation::Stroke { opacity, .. } = &mut bad {
            *opacity = f64::NAN;
        }
        assert!(bad.validate().is_err());
        let arrow = Annotation::Arrow {
            id: "arrow".into(),
            start: Point { x: 10., y: 20. },
            end: Point { x: 110., y: 20. },
            color: "#ffcf56".into(),
            width: 4.,
        };
        arrow.validate().unwrap();
        let mut bad = arrow.clone();
        if let Annotation::Arrow { end, .. } = &mut bad {
            end.x = 10.;
        }
        assert!(bad.validate().is_err());
        let mut scene = SceneStore::default();
        scene.ensure("a");
        scene
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 0,
                    added: vec![marker, arrow],
                    removed_ids: vec![],
                },
                0,
            )
            .unwrap();
        assert_eq!(scene.snapshot("a").unwrap().annotations.len(), 2);
        scene.clear(350);
        scene.undo();
        assert_eq!(scene.snapshot("a").unwrap().annotations.len(), 2);
    }
    #[test]
    fn old_presets_gain_marker_settings_and_reject_invisible_markers() {
        let old = r##"{"tool":"pen","color":"#ffcf56","width":12,"textSize":28}"##;
        let mut brush: BrushSettings = serde_json::from_str(old).unwrap();
        assert_eq!(brush.highlighter_width, 24.);
        assert_eq!(brush.highlighter_opacity, 0.32);
        brush.tool = Tool::Highlighter;
        brush.validate().unwrap();
        brush.highlighter_opacity = 0.;
        assert!(brush.validate().is_err());
    }
    fn stroke(id: &str) -> Annotation {
        Annotation::Stroke {
            opacity: 1.,
            id: id.into(),
            points: vec![Point { x: 1., y: 2. }],
            color: "#abcdef".into(),
            width: 5.,
        }
    }
    fn add(store: &mut SceneStore, display: &str, id: &str) {
        let generation = store.snapshot(display).unwrap().clear_generation;
        store
            .apply(
                SceneEdit {
                    clear_generation: generation,
                    display_id: display.into(),
                    added: vec![stroke(id)],
                    removed_ids: vec![],
                },
                350,
            )
            .unwrap();
    }
    #[test]
    fn scoped_clear_preserves_other_display_pending_edits_and_history() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.ensure("b");
        add(&mut s, "a", "a1");
        add(&mut s, "b", "b1");
        let b = s.snapshot("b").unwrap();
        let updates = s.clear_display("a", 350).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].fade_out[0].id(), "a1");
        assert_eq!(s.snapshot("b").unwrap().revision, b.revision);
        assert_eq!(
            s.snapshot("b").unwrap().clear_generation,
            b.clear_generation
        );
        s.apply(
            SceneEdit {
                display_id: "b".into(),
                clear_generation: b.clear_generation,
                added: vec![stroke("b2")],
                removed_ids: vec![],
            },
            0,
        )
        .unwrap();
        assert_eq!(s.snapshot("b").unwrap().annotations.len(), 2);
        s.undo(); // b2
        let restored = s.undo(); // only a
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].scene.display_id, "a");
        assert_eq!(restored[0].scene.annotations[0].id(), "a1");
        assert_eq!(restored[0].scene.clear_generation, 1);
        assert_eq!(s.snapshot("b").unwrap().annotations[0].id(), "b1");
        s.redo(350); // clear a again
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
        let revision = s.snapshot("a").unwrap().revision;
        let empty = s.clear_display("a", 0).unwrap();
        assert_eq!(empty[0].scene.revision, revision + 1);
        assert_eq!(empty[0].scene.clear_generation, 2);
        assert!(s.clear_display("missing", 0).is_err());
        let all = s.clear(350);
        assert_eq!(all.len(), 2);
        assert_eq!(s.snapshot("a").unwrap().clear_generation, 3);
        assert_eq!(s.snapshot("b").unwrap().clear_generation, 1);
    }
    #[test]
    fn cross_display_clear_is_one_action_and_preserves_arc() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.ensure("b");
        add(&mut s, "a", "1");
        add(&mut s, "b", "2");
        let original = s.snapshot("a").unwrap().annotations[0].clone();
        assert_eq!(s.clear(350).len(), 2);
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
        let updates = s.undo();
        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|u| u.fade_duration_ms == 0));
        assert!(Arc::ptr_eq(
            &original,
            &s.snapshot("a").unwrap().annotations[0]
        ));
        assert_eq!(s.redo(350).len(), 2);
    }
    #[test]
    fn clear_then_new_stroke_and_undo_are_chronological() {
        let mut s = SceneStore::default();
        s.ensure("a");
        add(&mut s, "a", "old");
        let removed = s.clear(350);
        add(&mut s, "a", "new");
        assert_eq!(removed[0].fade_out[0].id(), "old");
        s.undo();
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
        s.undo();
        assert_eq!(s.snapshot("a").unwrap().annotations[0].id(), "old");
    }
    #[test]
    fn invalid_edit_is_atomic() {
        let mut s = SceneStore::default();
        s.ensure("a");
        add(&mut s, "a", "one");
        let mut bad = stroke("bad");
        if let Annotation::Stroke { width, .. } = &mut bad {
            *width = f64::NAN;
        }
        assert!(s
            .apply(
                SceneEdit {
                    clear_generation: 0,
                    display_id: "a".into(),
                    added: vec![bad],
                    removed_ids: vec!["one".into()]
                },
                350
            )
            .is_err());
        assert_eq!(s.snapshot("a").unwrap().annotations.len(), 1);
    }
    #[test]
    fn history_limit_does_not_remove_live_ink() {
        let mut s = SceneStore::default();
        s.ensure("a");
        for i in 0..140 {
            add(&mut s, "a", &i.to_string());
        }
        assert_eq!(s.undo.len(), MAX_HISTORY);
        for _ in 0..150 {
            s.undo();
        }
        assert_eq!(s.snapshot("a").unwrap().annotations.len(), 12);
    }
    #[test]
    fn new_edit_invalidates_redo_and_revisions_increase() {
        let mut s = SceneStore::default();
        s.ensure("a");
        add(&mut s, "a", "one");
        s.undo();
        let rev = s.snapshot("a").unwrap().revision;
        add(&mut s, "a", "two");
        assert!(s.redo(350).is_empty());
        assert!(s.snapshot("a").unwrap().revision > rev);
    }
    #[test]
    fn thousand_strokes_and_repeated_clears_remain_bounded() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.ensure("b");
        for i in 0..1000 {
            let annotation = Annotation::Stroke {
                opacity: 1.,
                id: i.to_string(),
                points: (0..100)
                    .map(|x| Point {
                        x: x as f64,
                        y: (x % 17) as f64,
                    })
                    .collect(),
                color: "#abcdef".into(),
                width: 5.,
            };
            s.apply(
                SceneEdit {
                    clear_generation: 0,
                    display_id: if i % 2 == 0 { "a" } else { "b" }.into(),
                    added: vec![annotation],
                    removed_ids: vec![],
                },
                350,
            )
            .unwrap();
        }
        for _ in 0..20 {
            assert_eq!(s.clear(350).len(), 2);
            assert_eq!(s.undo().len(), 2);
        }
        assert_eq!(
            s.snapshot("a").unwrap().annotations.len() + s.snapshot("b").unwrap().annotations.len(),
            1000
        );
        assert!(s.undo.len() <= MAX_HISTORY);
        assert!(
            s.undo
                .iter()
                .chain(s.redo.iter())
                .map(|a| a.bytes)
                .sum::<usize>()
                <= MAX_HISTORY_BYTES
        );
    }
    #[test]
    fn stale_and_duplicate_deletions_do_not_modify_scene() {
        let mut s = SceneStore::default();
        s.ensure("a");
        add(&mut s, "a", "one");
        for ids in [vec!["missing".into()], vec!["one".into(), "one".into()]] {
            assert!(s
                .apply(
                    SceneEdit {
                        clear_generation: 0,
                        display_id: "a".into(),
                        added: vec![],
                        removed_ids: ids
                    },
                    350
                )
                .is_err());
            assert_eq!(s.snapshot("a").unwrap().revision, 1);
        }
    }
    #[test]
    fn clear_barrier_discards_late_additions_and_deletions() {
        let mut s = SceneStore::default();
        s.ensure("a");
        add(&mut s, "a", "old");
        let old_generation = s.snapshot("a").unwrap().clear_generation;
        s.clear(350);
        let cleared_revision = s.snapshot("a").unwrap().revision;
        for edit in [
            SceneEdit {
                display_id: "a".into(),
                clear_generation: old_generation,
                added: vec![stroke("late")],
                removed_ids: vec![],
            },
            SceneEdit {
                display_id: "a".into(),
                clear_generation: old_generation,
                added: vec![],
                removed_ids: vec!["old".into()],
            },
        ] {
            assert!(s.apply(edit, 350).unwrap().is_empty());
        }
        assert_eq!(s.snapshot("a").unwrap().revision, cleared_revision);
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
        add(&mut s, "a", "fresh");
        assert_eq!(s.snapshot("a").unwrap().annotations[0].id(), "fresh");
        s.undo(); // The fresh edit is the newest action, not either discarded edit.
        assert!(s.snapshot("a").unwrap().annotations.is_empty());
        s.undo(); // Restores the original clear in one action; generation never rewinds.
        assert_eq!(s.snapshot("a").unwrap().annotations[0].id(), "old");
        assert_eq!(
            s.snapshot("a").unwrap().clear_generation,
            old_generation + 1
        );
    }
    #[test]
    fn empty_clear_publishes_all_scenes_without_an_undo_action() {
        let mut s = SceneStore::default();
        s.ensure("a");
        s.ensure("b");
        let updates = s.clear(350);
        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|u| u.scene.revision == 1
            && u.scene.clear_generation == 1
            && u.scene.annotations.is_empty()));
        assert!(s.undo().is_empty());
        add(&mut s, "a", "one");
        let updates = s.clear(350); // Nonempty and empty scenes both advance.
        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|u| u.scene.clear_generation == 2));
        let undo_count = s.undo.len();
        s.clear(350);
        assert_eq!(s.undo.len(), undo_count);
        let restored = s.undo();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].scene.annotations[0].id(), "one");
        assert_eq!(restored[0].scene.clear_generation, 3);
    }
    #[test]
    fn future_generation_and_missing_generation_are_rejected() {
        let mut s = SceneStore::default();
        s.ensure("a");
        assert!(s
            .apply(
                SceneEdit {
                    display_id: "a".into(),
                    clear_generation: 1,
                    added: vec![stroke("future")],
                    removed_ids: vec![]
                },
                350
            )
            .is_err());
        assert_eq!(s.snapshot("a").unwrap().revision, 0);
        assert!(serde_json::from_str::<SceneEdit>(
            r#"{"displayId":"a","added":[],"removedIds":[]}"#
        )
        .is_err());
        assert_eq!(
            serde_json::to_value(s.snapshot("a").unwrap()).unwrap()["clearGeneration"],
            0
        );
    }
    #[test]
    fn korean_text_matches_ui_length_budget() {
        let mut annotation = Annotation::Text {
            id: "korean".into(),
            x: 0.,
            y: 0.,
            text: "한".repeat(10_000),
            color: "#ffffff".into(),
            font_size: 28.,
        };
        annotation.validate().unwrap();
        if let Annotation::Text { text, .. } = &mut annotation {
            *text = "한".repeat(13_334);
        }
        assert!(annotation.validate().is_err());
    }
    #[test]
    fn settings_and_text_validation() {
        let d = AppSettings::default();
        d.validate().unwrap();
        assert_eq!(
            serde_json::from_str::<AppSettings>(&serde_json::to_string(&d).unwrap()).unwrap(),
            d
        );
        assert!(!valid_color("#abé12"));
        let a = Annotation::Text {
            id: "t".into(),
            x: 2.,
            y: 4.,
            text: "강의 자료\n설명합니다".into(),
            color: "#ffcf56".into(),
            font_size: 28.,
        };
        a.validate().unwrap();
    }
}
