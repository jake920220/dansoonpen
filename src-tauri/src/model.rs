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
            Self::Stroke { id, .. } | Self::Text { id, .. } => id,
        }
    }
    pub fn bytes(&self) -> usize {
        128 + self.id().len()
            + match self {
                Self::Stroke { points, .. } => points.len() * 16,
                Self::Text { text, .. } => text.len(),
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
                ..
            } => {
                !points.is_empty()
                    && points.len() <= 100_000
                    && points.iter().all(|p| coordinate(p.x) && coordinate(p.y))
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
fn coordinate(n: f64) -> bool {
    finite_range(n, -100_000., 100_000.)
}
fn finite_range(n: f64, min: f64, max: f64) -> bool {
    n.is_finite() && n >= min && n <= max
}
pub fn valid_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    pub version: u32,
    pub color: String,
    pub width: f64,
    pub text_size: f64,
    pub quick_colors: Vec<String>,
    pub toggle_shortcut: String,
    pub clear_shortcut: String,
    pub reduce_motion: bool,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 2,
            color: "#ffcf56".into(),
            width: 12.,
            text_size: 28.,
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
            reduce_motion: false,
        }
    }
}
impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 2
            || !valid_color(&self.color)
            || !finite_range(self.width, 1., 32.)
            || !finite_range(self.text_size, 8., 144.)
            || self.quick_colors.len() != 6
            || !self.quick_colors.iter().all(|s| valid_color(s))
            || self.toggle_shortcut.len() > 80
            || self.clear_shortcut.len() > 80
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
    pub version: Option<u32>,
    pub color: Option<String>,
    pub width: Option<f64>,
    pub text_size: Option<f64>,
    pub quick_colors: Option<Vec<String>>,
    pub toggle_shortcut: Option<String>,
    pub clear_shortcut: Option<String>,
    pub reduce_motion: Option<bool>,
}
impl SettingsPatch {
    pub fn apply(self, current: &AppSettings) -> AppSettings {
        AppSettings {
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
pub struct AppState {
    pub mode: Mode,
    pub active_display_id: Option<String>,
    pub displays: Vec<DisplayInfo>,
    pub settings: AppSettings,
    pub revision: u64,
    pub error: Option<String>,
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
    clear_generation: u64,
    scenes: HashMap<String, Scene>,
    undo: VecDeque<Action>,
    redo: Vec<Action>,
}
impl SceneStore {
    pub fn ensure(&mut self, id: &str) {
        self.scenes.entry(id.into()).or_default();
    }
    pub fn snapshot(&self, id: &str) -> Result<SceneSnapshot, String> {
        let s = self.scenes.get(id).ok_or("알 수 없는 화면입니다.")?;
        Ok(SceneSnapshot {
            display_id: id.into(),
            clear_generation: self.clear_generation,
            revision: s.revision,
            annotations: s.items.clone(),
        })
    }
    fn commit(&mut self, changes: Vec<Change>, fade: u32) -> Vec<SceneUpdate> {
        if changes.is_empty() {
            return vec![];
        }
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
                        clear_generation: self.clear_generation,
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
        // A clear is also a barrier for edits already in flight from a webview.
        // Discard old additions AND deletions before resolving their object IDs.
        if edit.clear_generation < self.clear_generation {
            return Ok(vec![]);
        }
        if edit.clear_generation > self.clear_generation {
            return Err("필기 세대가 현재 화면보다 앞섭니다. 화면을 다시 불러오세요.".into());
        }
        if edit.added.len() > 1000 || edit.removed_ids.len() > 100_000 {
            return Err("한 번에 편집할 수 있는 주석 수를 초과했습니다.".into());
        }
        let scene = self
            .scenes
            .get(&edit.display_id)
            .ok_or("알 수 없는 화면입니다.")?;
        let mut ids: HashSet<String> = scene.items.iter().map(|a| a.id().into()).collect();
        let removed: HashSet<&str> = edit.removed_ids.iter().map(String::as_str).collect();
        if removed.len() != edit.removed_ids.len() || !removed.iter().all(|id| ids.contains(*id)) {
            return Err("삭제 대상이 오래되었거나 중복되었습니다. 화면을 다시 불러오세요.".into());
        }
        // IDs cannot be reused, including as replacements in the same edit.
        for a in &edit.added {
            a.validate()?;
            if !ids.insert(a.id().into()) {
                return Err("중복된 주석 ID입니다.".into());
            }
        }
        let mut after: Vec<_> = scene
            .items
            .iter()
            .filter(|a| !removed.contains(a.id()))
            .cloned()
            .collect();
        after.extend(edit.added.into_iter().map(Arc::new));
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
        self.clear_generation += 1;
        // Empty native scenes may still have uncommitted strokes/text in the webview.
        // They need a fresh revision and generation event, but no empty undo action.
        let empty_displays: Vec<_> = self
            .scenes
            .iter()
            .filter(|(_, scene)| scene.items.is_empty())
            .map(|(id, _)| id.clone())
            .collect();
        let changes = self
            .scenes
            .iter()
            .filter(|(_, s)| !s.items.is_empty())
            .map(|(id, s)| Change {
                display: id.clone(),
                before: s.items.clone(),
                after: vec![],
            })
            .collect();
        let mut updates = self.commit(changes, fade);
        for id in empty_displays {
            let scene = self.scenes.get_mut(&id).expect("known empty scene");
            scene.revision += 1;
            updates.push(SceneUpdate {
                scene: SceneSnapshot {
                    display_id: id,
                    clear_generation: self.clear_generation,
                    revision: scene.revision,
                    annotations: vec![],
                },
                fade_out: vec![],
                fade_duration_ms: fade,
            });
        }
        updates
    }
    pub fn undo(&mut self) -> Vec<SceneUpdate> {
        if let Some(a) = self.undo.pop_back() {
            let updates = self.install(&a.changes, true, 0);
            self.redo.push(a);
            updates
        } else {
            vec![]
        }
    }
    pub fn redo(&mut self, fade: u32) -> Vec<SceneUpdate> {
        if let Some(a) = self.redo.pop() {
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
    use super::*;
    fn stroke(id: &str) -> Annotation {
        Annotation::Stroke {
            id: id.into(),
            points: vec![Point { x: 1., y: 2. }],
            color: "#abcdef".into(),
            width: 5.,
        }
    }
    fn add(store: &mut SceneStore, display: &str, id: &str) {
        let generation = store.clear_generation;
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
