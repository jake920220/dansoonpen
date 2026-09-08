---
status: investigating
trigger: "아까화면공유하면서 강의진행중에 펜 쓰는데 갑자기 화면 오버레이 엄청 깔리더니 이상해지는 현상 있었거든. 특정 키랑 충돌이난건지 특정 상황에서그런건지모르지만 당황해서 브러시 앱 껐어. 원인 파악해줄수있어?"
created: 2026-09-08
updated: 2026-09-08
---

## Current Focus

hypothesis: incident trigger remains unknown; one overlay page weakens window multiplication. Confirmed adjacent risks are target/current DPI conversion, nonfocusable show warnings, and macOS non-exclusive shortcut registrations
test: correlate user's exact visual symptom, sharing app and display/key actions with existing lifecycle logs; native reproduction must be a later controlled step
expecting: distinguish compositor ghosting or share-preview feedback from actual window growth, then select a falsifiable incident hypothesis
next_action: parent obtains missing symptom details; do not relaunch or mutate the app while waiting. No product fix has been made

## Symptoms

expected: One stable overlay per display during screen sharing and drawing; ink remains until explicit deletion.
actual: During a lecture and screen sharing, many overlays/visual layers appeared suddenly; the user quit the app.
errors: No error text reported.
reproduction: Exact sharing application, display topology, key sequence and visual appearance pending.
started: Reported during real lecture using installed v0.3; no deterministic reproduction yet.

## Eliminated

- hypothesis: Every shortcut press creates another overlay for the same display.
  evidence: ensure_overlay reuses a deterministic display-ID hash label; commands are main-thread dispatched under RuntimeState mutex. Global keydown repeats are ignored until key release. Parent log review found only one overlay page35 and one control page8 during the incident process lifetime.
  timestamp: 2026-09-08
- hypothesis: Cursor movement appends unlimited DOM overlay or animation nodes.
  evidence: CursorHighlight contains fixed halo/ripple nodes; each ripple cancels the previous animation. Cursor worker allows one callback in flight. No repeated node creation in movement path.
  timestamp: 2026-09-08
- hypothesis: My Brush captures and recursively redraws desktop/share frames itself.
  evidence: Source inventory contains no screen capture or video source pipeline; canvas receives only native annotation scenes. External share-preview feedback remains a separate possibility.
  timestamp: 2026-09-08

## Evidence

- timestamp: 2026-09-08
  checked: repository and debug session inventory
  found: HEAD d1dd2c2; working tree initially clean; no .planning/debug directory or knowledge base exists.
  implication: No previously resolved pattern can be tested first. Investigate current code independently.
- timestamp: 2026-09-08
  checked: src-tauri/src/lib.rs ensure_overlay, change_mode, refresh_displays, startup, shortcut handler
  found: Stable IDs reuse a single window. Disconnected display windows are hidden but never destroyed; scene IDs remain. There is no cross-process single-instance guard. Poll refresh runs every3s and returns immediately when DisplayInfo vectors match.
  implication: Stable topology cannot multiply windows from repeated toggles alone. Display identity churn can retain hidden WebViews; duplicate processes are possible in principle but need runtime evidence.
- timestamp: 2026-09-08
  checked: src/ui/Overlay.svelte, src/canvas/renderer.ts, CursorHighlight.svelte, cursor.rs
  found: One canvas per overlay component; redraw clears the full bitmap under identity transform; scene events are revision-gated; fades have count/duration bounds; no scene/DOM multiplication on key repeat found.
  implication: Ordinary renderer accumulation is not supported by current source; native compositor issues remain untested.
- timestamp: 2026-09-08
  checked: pinned tao0.35.3 macOS window.rs728–758 and project ensure_overlay
  found: Project passes target monitor physical dimensions/position to an existing or newly created window. Tao converts Physical inputs to logical values using that window's current scale, which can differ from target display scale.
  implication: Newly created or relocated overlays crossing DPI scales can receive wrong logical geometry, potentially covering another display. Need numeric regression and eventual native mixed-DPI check.
- timestamp: 2026-09-08
  checked: parent runtime log investigation, incident PID30191
  found: Parent found one control WebContent/page8 and one overlay WebContent/page35, normal exit, no crash, and repeated makeKeyWindow on same window while canBecomeKeyWindow==NO. Old QA trace is unrelated.
  implication: Available process/page evidence contradicts actual rapidly multiplying WebViews. Window warning needs lifecycle explanation, not assumption of creation growth.
- timestamp: 2026-09-08
  checked: change_mode order and tao0.35.3 set_visible/is_focusable
  found: release_inputs sets focusable=false, then change_mode calls show before setting focusable=true. Tao show calls make_key_and_order_front_sync; canBecomeKeyWindow returns focusable. refresh_displays also shows inactive nonfocusable windows.
  implication: Warning is reproducible from source order and can occur during normal draw entry. It does not establish failure or explain many visual layers by itself.
- timestamp: 2026-09-08
  checked: headless reproduction artifacts/incident-20260908/geometry-repro via cargo run --offline
  found: With target Retina physical position5120,968 and size2940x1912 atscale2 but current window scale1, pinned dpi0.1.2 conversions produce logical position5120,968 and size2940x1912 instead of2560,484 and1470x956. Passing logical geometry derived with target scale passes12 arithmetic combinations of1x/2x andpositive/negative origins.
  implication: A target/current DPI conversion defect is demonstrated without GUI. This is not native window placement verification and not proof it triggered the incident. Parent logs show only one overlay and no evidence of Retina selection.
- timestamp: 2026-09-08
  checked: npm test -- src/canvas/renderer.test.ts src/app/shortcuts.test.ts
  found: 14 tests in2 files pass, including full bitmap clear, bounded fades, restoration without duplicate ink, idle scheduling and Korean physical shortcuts.
  implication: Covered pure renderer paths do not reproduce accumulation. Browser/unit tests cannot rule out WindowServer/WebKit compositor or full-screen behavior.
- timestamp: 2026-09-08
  checked: global-hotkey macOS registration and Apple SDK CarbonEvents.h15395–15476
  found: Library calls RegisterEventHotKey with options0. SDK documents non-exclusive hotkeys: multiple applications may register the same key combination and receive notifications. A non-exclusive registration also succeeds when an exclusive registration exists but does not receive notifications until the exclusive holder releases it.
  implication: Successful shortcut registration cannot establish absence of other-app conflicts onmacOS. Another app reacting to OptionZ/X is possible; incident app/key evidence is still missing. Do not change defaults based on this alone.
- timestamp: 2026-09-08
  checked: tray accelerators and global handler routes
  found: Tray native menu items and Carbon global registrations use the same settings keys; callbacks are independent. Global callback has pressed-ID suppression; tray callback does not share it. The Carbon handler returns noErr and the tray menu is separate from the normal app menu.
  implication: One-physical-key double delivery is not proven by source alone; it needs native event-source tracing. Even dual callbacks reuse the same window label and would change mode/visibility, not multiply overlay WebViews.
- timestamp: 2026-09-08
  checked: parent incident binary and memory evidence
  found: Installed executable matches v0.3 release archive. Parent found no GPU crash/context-loss/processDidTerminate records; overlay footprint samples69–164MB are not full process-tree RSS or an OOM diagnosis.
  implication: No evidence supports a stale debug binary or recorded WebContent crash as incident cause. Missing logs cannot exclude compositor corruption.

## Resolution

root_cause: Incident cause not confirmed. Mixed-DPI input conversion and show/focus ordering are concrete adjacent defects; non-exclusive hotkey conflict detection is a platform limitation. None yet explains the reported visual pileup with available evidence.
fix: None; product behavior left unchanged per diagnosis-only scope.
verification: Headless geometry reproduction shows mismatch and12 candidate conversion cases pass;14 existing TS renderer/shortcut tests pass. GUI app was never launched, input injected or settings changed by this investigation.
files_changed: []

## Proposed follow-up, not applied

1. macOS geometry: compute target display logical size/position before calling Tauri, or place against an identified NSScreen atomically. Verify new overlay placement and DPI changes on both screens, with negative origins and full-screen presentation transitions. Arithmetic proof is not sufficient native verification.
2. Visibility/focus: use nonactivating native orderFrontRegardless for pass-through overlay visibility; only make drawing overlay focusable/key deliberately. AppKit documentation endpoint: https://developer.apple.com/documentation/appkit/nswindow/orderfrontregardless(). Confirm focus restoration and warning absence on repeated draw/interact/monitor refresh.
3. Diagnostics: record bounded mode command source, stable window IDs, actual frame/scale and display topology changes. This can distinguish window creation, geometric drift and visual-only recursion on recurrence. Do not log typed lecture content or screenshots.
4. Shortcut conflicts: explain macOS non-exclusive behavior accurately; consider an explicit conflict detection strategy only after investigating user-selected bindings. Changing to exclusive hotkeys has policy/interoperability tradeoffs and does not by itself diagnose this incident.

## Investigation outcome

INCONCLUSIVE for the reported incident. Confidence high in the two source-level native risks and API non-exclusivity; confidence low that any one is the incident root cause. User clarification and eventual controlled native reproduction are needed. Current available logs support one native overlay page, not a growing set.
