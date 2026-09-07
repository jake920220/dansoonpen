# v0.1 bridge contract

`src/shared/types.ts` is the frontend contract. Rust serializes camelCase fields. Native state and scene history are authoritative. Frontend renders locally; send one edit at pointer-up or text commit, never IPC per point.

Commands (Tauri invoke):
- `get_state` -> AppState
- `set_mode { mode }` -> AppState
- `select_display { displayId }` -> AppState
- `update_settings { settings }` -> AppState; merge only supplied fields into current native settings, validate/persist and re-register shortcuts transactionally. Palette and shortcut edits in separate windows do not replace unrelated fields.
- `capture_shortcut { active }` -> void; control window only. Suspend the app's global shortcuts while its focused recorder captures a key combination. Restore on exit, native focus loss or control close.
- `get_scene { displayId }` -> SceneSnapshot
- `apply_edit { edit: SceneEdit }` -> SceneSnapshot; add/remove as one undoable edit
- `clear_all` -> void; remove existing annotations from all displays in one undoable action
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

V0.1 includes only pen, eraser, text. Shapes/highlighter/cursor effects are deferred. UI labels Korean, shortcuts use Alt mapped to Option on macOS. All commands validate caller permissions/arguments and errors are user-visible with native interact fallback where input can be trapped.
