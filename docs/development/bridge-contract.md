# v0.6 bridge contract

`src/shared/types.ts` is the frontend contract. Rust serializes camelCase fields. Native state and scene history are authoritative. Frontend renders locally; send one edit at pointer-up or text commit, never IPC per point.

Commands (Tauri invoke):
- `get_state` -> AppState
- `set_mode { mode }` -> AppState
- `update_settings { settings }` -> AppState; merge only supplied fields into current native settings, validate/persist and re-register shortcuts transactionally. Default color/width/textSize edits also patch the current brush; unrelated settings leave it intact. Writes are transactional: a persistence failure does not apply the brush patch.
- `update_brush { brush, brushGeneration }` -> AppState; validate and merge current tool fields in native memory, shared by all overlays. Never writes startup defaults. Ignore writes from earlier drawing entries using AppState.brushGeneration. A successful interact→draw transition advances this generation and restores the saved default pen; draw→draw focus changes preserve the current brush.
- `update_preset { index, name?, brush? }` -> AppState; validate and persist a single preset slot without replacing other slots or current tool.
- `capture_shortcut { active }` -> void; control window only. Suspend the app's global shortcuts while its focused recorder captures a key combination. Restore on exit, native focus loss or control close.
- `get_scene { displayId }` -> SceneSnapshot
- `apply_edit { edit: SceneEdit }` -> SceneSnapshot; add/remove as one undoable edit
- `clear_all` -> void; remove existing annotations from all displays in one undoable action, preserving mode, brush, toolbar and native input/focus throughout the fade. Advance clear generation to invalidate pending edits; new-generation strokes survive. The global clear shortcut and tray call the same native routine. Empty clears also preserve the current mode.
- `undo_clear { token }` -> void; restore the issued clear only while its history revision is unchanged. Preserve mode/focus and reject after any new edit/undo/redo.
- `undo` / `redo` -> void; global chronological undo, including all-monitor clear
- `show_control` -> void
- `quit_app` -> void

Events:
- `brush-state`: AppState, emitted after native state transitions/settings/displays changes
- `brush-open-settings`: void, targeted at control to open the settings tab from the tray, overlay or native Cmd+, menu item
- `brush-scene`: SceneUpdate; broadcast, frontend filters displayId. Clear emits removed objects as fadeOut and empty current scene. Undo emits restored scene with zero fade. Late fade must never erase new objects.

Windows:
- `control`: index.html?view=control, onboarding/settings. Normal opaque window.
- `overlay-*`: index.html?view=overlay&display=ENCODED_ID. Transparent, borderless display-sized webview. Initially nonfocusable + cursor events ignored. Every connected overlay receives input in draw mode; all overlays are click-through in interact mode. There is no selected display or activeDisplayId. Keyboard focus initially goes to the display under the pointer (primary fallback); a first click on another display draws immediately.

Each SceneSnapshot and SceneEdit carries `clearGeneration`. Native clear increments it even for empty scenes and emits a higher revision for every known display. An edit from an older generation is intentionally ignored; future generations are rejected. UI abandons pending strokes/text and optimistic markers on generation advance, while new-generation edits survive. A global clear cancels an unfinished gesture; undo restores completed native objects.

Native maintains per-display scenes, revision, bounded global history with shared annotation storage where possible. On display unplug hide overlay without discarding scene; restore when known display returns. Scene changes must not show disconnected windows. Reconnecting a display while drawing enables its overlay without resetting the brush or stealing focus. Losing every display exits drawing without deleting scenes. Frontend calls get_state/get_scene AFTER registering listeners, and ignores older revisions.

V0.2 adds arrows and highlighter strokes. `stroke.opacity` defaults to 1 for old strokes; highlighters use width 8–64 and opacity 0.1–0.8. `arrow` contains start/end/color/width, with shared frontend geometry for rendering and erasing. AppState.brush is transient; settings v3 contains 3 persistent presets and adds defaults when loading v1/v2 files. Highlighter fields on older preset brushes deserialize to 24px/0.32. Toolbar location is a separate local WebView UI preference keyed by display ID. Cursor effects and other shapes remain deferred. UI labels Korean, shortcuts use Alt mapped to Option on macOS. All commands validate caller permissions/arguments and errors are user-visible with native interact fallback where input can be trapped.

## 필기 임시 숨김

`AppState.annotationsVisible`은 세션 표시 상태다. `toggle_annotations`는 주석과 undo/redo를 건드리지 않는다. 숨기기는 입력을 해제하고, 복원은 앱 포커스를 바꾸지 않는다. 그리기 진입은 표시 상태를 복원한다. 설정의 `visibilityShortcut`(macOS Option+V / Windows Alt+Shift+V)은 빈 문자열로 해제할 수 있다. 이전 설정에 같은 조합을 이미 배정했다면 새 단축키만 비활성으로 마이그레이션한다. 숨겨진 동안 clear/undo가 이루어지면 최신 장면이 복원된다.

## 이 화면 삭제

`clear_current`는 명령을 보낸 필기 창의 모니터만 수동 fade로 지우고 현재 모드·도구·입력 포커스를 유지한다. 다른 모니터의 장면·revision·clearGeneration에는 영향을 주지 않는다. 제어창의 clear_current 호출은 거부하며 트레이에서는 커서가 있는 화면을 찾는다. clearGeneration은 **모니터별**로 관리하며 전체 삭제는 각 모니터 값을 각각 증가시킨다. 전체 삭제는 계속 하나의 undo이고, 선택 삭제의 undo/redo는 해당 모니터만 변경한다. 빈 장면의 삭제도 진행 중 획을 무효화할 세대 이벤트를 보낸다.

## 커서 강조

`AppState.cursorEnabled`은 기본 꺼짐인 세션 상태이며 `toggle_cursor`로 변경한다. `settings.cursor`의 color/size(24–96)/showClicks만 저장한다. 연결된 모든 화면의 오버레이를 재사용하며 앱 조작·필기 숨김 중에도 커서는 표시할 수 있다. 입력 포커스나 주석·undo에는 영향이 없다.

`brush-cursor`는 각 화면의 창에 개별 전달하는 창 내부 논리 좌표, visible, clicks, sequence다. 비활성 시 worker는 조건변수에서 대기한다. 활성 시 최대 약30Hz로 위치를 읽고 값이 바뀔 때만 전송하며 네이티브 콜백을 중첩 적재하지 않는다. CanvasRenderer 대신 작은 DOM 레이어만 움직이고 클릭 애니메이션은 최대 하나다. 화면을 벗어나면 좌표는0/숨김으로 정규화한다.

macOS는 NSWindow의 `mouseLocationOutsideOfEventStream`과 `CGEventSourceCounterForEventType`(현재 세션의 왼쪽 클릭 횟수)을 사용한다. 이벤트 주입·이벤트 탭·화면 캡처·키 입력 수집은 없다. AppKit의 창 좌표를 사용해 서로 다른 배율의 바탕화면 원점 환산을 피한다. Windows는 GetCursorPos와 GetAsyncKeyState의 현재 왼쪽 버튼 상태를 읽는다. Windows의 빠른 클릭 누락·좌표·입력은 실제 PC 검증을 기다린다.

참고: [Apple 창 내부 마우스 위치](https://developer.apple.com/documentation/appkit/nswindow/mouselocationoutsideofeventstream), [Apple Quartz 이벤트 함수](https://developer.apple.com/documentation/coregraphics/core-graphics-functions), [Microsoft GetCursorPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos).

AppState.feedback carries a native-completed action ID, message and creation timestamp. The UI expires notices after 3.5 seconds without another IPC or window. Interact notices remain click-through; clear recovery is available from the existing tray/control, or toolbar while drawing. clearUndoToken is invalidated by scene history changes.

## macOS 다중 화면 배치와 입력

화면의 물리 좌표를 **대상 화면의 배율**로 나눠 AppKit 논리 좌표로 변환하고 기본 화면의 논리 높이를 기준으로 y축을 뒤집는다. NSWindow 전체 프레임을 메인 스레드에서 동기 적용하여 이전 화면의 배율이나 비동기 위치 변경에 의존하지 않는다. 화면 표시는 orderFrontRegardless로 포커스를 바꾸지 않고, 그리기 진입 시 모든 창의 입력을 연 뒤 NSApplication 자체 활성화 → 대상 창 key 지정 → WebView 포커스 순서로 처리한다. 앱 조작 복귀는 모든 창의 입력을 해제한다. Windows 경로는 실제 PC에서 별도 확인해야 한다.

## 화면 사이 도구 패널 이동 (v0.6.1)

`get_toolbar`와 `brush-toolbar`는 현재 표시 화면과 저장할 위치(선호 화면, 0–1 정규화 x/y, 접힘·더보기), revision을 공유한다. 도구 패널은 한 화면에만 표시한다. `begin_toolbar_drag {drag:{token,offsetX,offsetY,width,height}}`는 현재 패널 창에 드래그 소유권을 부여하고, `move_toolbar {token,point,finish}`는 포인터 이벤트의 원본 창 내부 좌표를 대상 모니터 좌표로 환산해 이동한다. macOS의 논리 바탕화면과 Windows의 물리 바탕화면을 구분하며 비동기 처리 시 현재 마우스 위치를 다시 읽지 않는다. 이동 IPC는 프레임 단위로 합치고 한 번에 하나만 처리한다. 원래 손잡이 DOM은 투명 상태로 유지해 다른 모니터까지 pointer capture가 이어진다.

finish 때만 `settings.toolbar`를 원자 저장하며 모드 종료·화면 구성 변경 때 드래그를 취소한다. 오래된 token과 다른 창의 이동은 무시한다. `update_toolbar {position}`은 현재 패널의 키보드 이동·접힘·더보기만 저장한다. 펜 설정 등 다른 설정 변경은 toolbar를 보존한다. 저장한 화면이 끊기면 기본 화면으로 임시 이동하고 재연결 때 선호 화면을 복원한다. 텍스트 편집창의 색·크기 도구는 해당 텍스트 옆에 유지한다.
