# v0.2 bridge contract

`src/shared/types.ts` is the frontend contract. Rust serializes camelCase fields. Native state and scene history are authoritative. Frontend renders locally; send one edit at pointer-up or text commit, never IPC per point.

Commands (Tauri invoke):
- `get_state` -> AppState
- `set_mode { mode }` -> AppState
- `select_display { displayId }` -> AppState
- `update_settings { settings }` -> AppState; merge only supplied fields into current native settings, validate/persist and re-register shortcuts transactionally. Default color/width/textSize edits also patch the current brush; unrelated settings leave it intact. Writes are transactional: a persistence failure does not apply the brush patch.
- `update_brush { brush }` -> AppState; validate and merge current tool fields in native memory, shared by all overlays. Never writes startup defaults.
- `update_preset { index, name?, brush? }` -> AppState; validate and persist a single preset slot without replacing other slots or current tool.
- `capture_shortcut { active }` -> void; control window only. Suspend the app's global shortcuts while its focused recorder captures a key combination. Restore on exit, native focus loss or control close.
- `get_scene { displayId }` -> SceneSnapshot
- `apply_edit { edit: SceneEdit }` -> SceneSnapshot; add/remove as one undoable edit
- `clear_all` -> void; remove existing annotations from all displays in one undoable action, then leave draw mode and release input/restore prior app focus immediately while the fade continues. Advance clear generation before publishing the mode change. The global clear shortcut and tray call the same native routine. If already interacting, preserve current app focus.
- `undo` / `redo` -> void; global chronological undo, including all-monitor clear
- `show_control` -> void
- `quit_app` -> void

Events:
- `brush-state`: AppState, emitted after native state transitions/settings/displays changes
- `brush-open-settings`: void, targeted at control to open the settings tab from the tray, overlay or native Cmd+, menu item
- `brush-scene`: SceneUpdate; broadcast, frontend filters displayId. Clear emits removed objects as fadeOut and empty current scene. Undo emits restored scene with zero fade. Late fade must never erase new objects.

Windows:
- `control`: index.html?view=control, onboarding/settings. Normal opaque window.
- `overlay-*`: index.html?view=overlay&display=ENCODED_ID. Transparent, borderless display-sized webview. Initially nonfocusable + cursor events ignored. Only active draw display receives input; inactive overlays remain click-through.

Each SceneSnapshot and SceneEdit carries `clearGeneration`. Native clear increments it even for empty scenes and emits a higher revision for every known display. An edit from an older generation is intentionally ignored; future generations are rejected. UI abandons pending strokes/text and optimistic markers on generation advance, while new-generation edits survive. A global clear cancels an unfinished gesture; undo restores completed native objects.

Native maintains per-display scenes, revision, bounded global history with shared annotation storage where possible. On display unplug hide overlay without discarding scene; restore when known display returns. Scene changes must not show inactive/disconnected windows. Frontend calls get_state/get_scene AFTER registering listeners, and ignores older revisions.

V0.2 adds arrows and highlighter strokes. `stroke.opacity` defaults to 1 for old strokes; highlighters use width 8–64 and opacity 0.1–0.8. `arrow` contains start/end/color/width, with shared frontend geometry for rendering and erasing. AppState.brush is transient; settings v3 contains 3 persistent presets and adds defaults when loading v1/v2 files. Highlighter fields on older preset brushes deserialize to 24px/0.32. Toolbar location is a separate local WebView UI preference keyed by display ID. Cursor effects and other shapes remain deferred. UI labels Korean, shortcuts use Alt mapped to Option on macOS. All commands validate caller permissions/arguments and errors are user-visible with native interact fallback where input can be trapped.

## 필기 임시 숨김

`AppState.annotationsVisible`은 세션 표시 상태다. `toggle_annotations`는 주석과 undo/redo를 건드리지 않는다. 숨기기는 입력을 해제하고, 복원은 앱 포커스를 바꾸지 않는다. 그리기 진입은 표시 상태를 복원한다. 설정의 `visibilityShortcut`(macOS Option+V / Windows Alt+Shift+V)은 빈 문자열로 해제할 수 있다. 이전 설정에 같은 조합을 이미 배정했다면 새 단축키만 비활성으로 마이그레이션한다. 숨겨진 동안 clear/undo가 이루어지면 최신 장면이 복원된다.

## 선택한 화면 삭제

`clear_current`는 현재 선택한 모니터만 수동 fade로 지우고 앱 조작으로 복귀한다. 다른 모니터의 장면·revision·clearGeneration에는 영향을 주지 않는다. clearGeneration은 **모니터별**로 관리하며 전체 삭제는 각 모니터 값을 각각 증가시킨다. 전체 삭제는 계속 하나의 undo이고, 선택 삭제의 undo/redo는 해당 모니터만 변경한다. 빈 장면의 삭제도 진행 중 획을 무효화할 세대 이벤트를 보낸다.
