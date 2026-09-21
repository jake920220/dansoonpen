# My Brush

**English** | [한국어](README.ko.md)

Draw on your screen while teaching, presenting or explaining code. Your annotations stay until you clear them. Switch back to your apps without losing your notes.

My Brush is an independently built, open-source desktop app using Tauri, Rust and Svelte. No account, server or screen recording is required.

**v0.6 is in pre-release validation.** macOS Apple Silicon builds and selected native workflows have been tested. Windows support is implemented but has **not yet been validated on a Windows PC**. See [compatibility](docs/testing/compatibility.md) and [verification](docs/testing/verification.md) for the exact scope; historical testing documents are currently in Korean.

## Get started

There is no public download URL yet. Build from source using the instructions below. Once a public repository and release are available, the release should include platform-specific installers, checksums and installation instructions.

1. Open My Brush, select a display and choose **Start drawing**.
2. Use **Option+Z** on macOS or **Alt+Shift+Z** on Windows to switch between drawing and interacting with your apps. Your annotations stay on screen.
3. Use **Option+X / Alt+Shift+X**, or the trash button, to clear annotations. Your current tool and toolbar remain active.
4. Open **설정 / Settings → 언어 / Language** to choose **English** or **한국어**. The setting is saved and applies immediately to the control window, toolbar and tray menu. Existing installations start in Korean. Your annotations and saved preset names are never translated.
5. When sharing, choose your **entire display**. Sharing a single app window may not include the annotation overlay.

Local macOS ZIPs use an ad-hoc signature. They are not Developer ID signed or notarized distribution builds. Installation on other Macs remains a release-validation task.

## Features

- Pen with adjustable width, quick colors and a color wheel; new-install defaults are yellow and 12px.
- Independent eraser diameter (16–128px, default 48px), quick size choices and actual-size previews.
- Highlighter with independent width/opacity, and arrows with Shift snapping to 45°.
- Multiline text with bundled Nanum Gothic, default 40px. Click existing text with the text tool to edit it, move it and change its color or size.
- Undo/redo, gentle manual clearing, and recovery of the last clear before further edits.
- Movable, collapsible toolbar with a compact view and **More tools**. Its layout is remembered per display.
- Three editable presets, separate saved defaults and current-tool settings.
- Return to drawing with the saved default **pen**, regardless of the previously selected tool.
- Hide/show annotations without deleting them, clear one display or all displays, and optional cursor/click highlights.
- Global shortcut recording and conflict feedback. Recovery controls in the macOS menu bar / Windows tray.
- Korean and English interface, including tooltips and accessibility labels.

## Shortcuts

Global bindings can be changed in Settings. The following are defaults; customized bindings take precedence.

| Action | macOS | Windows |
| --- | --- | --- |
| Draw ↔ interact | Option+Z | Alt+Shift+Z |
| Clear all, keep current tool | Option+X | Alt+Shift+X |
| Hide / show annotations | Option+V | Alt+Shift+V |
| Open settings while My Brush is active | Cmd+, | Ctrl+, |
| Return to apps, keep annotations | Esc | Esc |
| Undo / redo | Cmd+Z / Cmd+Shift+Z | Ctrl+Z / Ctrl+Shift+Z |
| Pen / eraser / text | P / E / T | P / E / T |
| Arrow / highlighter | A / H | A / H |
| Presets 1–3 | Shift+1–3 | Shift+1–3 |
| Quick colors / tool size | 1–6 / [ ] | 1–6 / [ ] |
| Cursor highlight while drawing | C | C |

Tool shortcuts do not intercept text input. In the text editor, Enter finishes editing, Shift+Enter inserts a line break and Esc cancels. IME composition takes priority; comprehensive native Korean IME validation is still pending.

To move existing text, select **Text**, click the annotation and drag its handle. Arrow keys move the focused handle by 1px; Shift+arrow moves it by 10px. Content, position and style changes are one undoable edit. Finishing with empty text deletes that annotation.

Toolbar changes affect the current tool. Settings changes update saved starting defaults and the current tool. Presets save tool, color, pen width, eraser diameter, text size and highlighter width/opacity. Use **Default pen** to restore your saved defaults.

A clear cancels unfinished strokes/text, preserves newly started strokes after the clear, and leaves the current mode unchanged. **Restore last clear** is available in the toolbar, tray or control window until a later edit or undo/redo invalidates it. The notice itself is click-through while interacting with other apps.

## Build from source

Tools used for local validation: Node.js 26, Rust/Cargo 1.94. macOS minimum deployment target: 13.0. Windows's first manual-validation target is Windows 11 x64.

Install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/): Xcode Command Line Tools on macOS; Microsoft C++ Build Tools, the Rust MSVC toolchain and WebView2 on Windows.

```sh
npm ci
npm run tauri dev
```

`npm run dev` starts a development-only browser preview. It cannot validate desktop transparency, global hotkeys, OS focus or click-through behavior.

```sh
npm run check
npm test
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Build each platform on that platform:

```sh
# Prepare dependency caches and bundled notices before packaging.
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
node scripts/generate-notices.mjs

# macOS app
npm run tauri build -- --bundles app

# macOS local ad-hoc signed ZIP
bash scripts/package-macos.sh

# Windows installer — run on Windows
npm run tauri build -- --bundles nsis
```

The macOS bundle is created in `src-tauri/target/release/bundle/macos/`; local ZIPs in `artifacts/`; Windows installers in `src-tauri/target/release/bundle/nsis/`. Windows testers can use `scripts/verify-windows.ps1`. A successful build does not establish native behavior or screen-sharing compatibility.

## Privacy, limits and known issues

- Drawings live in memory and disappear when you quit. There is no session persistence or image export yet. Closing the control window hides it; **Quit app** or the tray menu exits.
- Settings and bounded diagnostics are stored locally. No account, analytics upload or screen recording is implemented. Logs exclude annotation text and screen images; review any log before sharing it.
- Draw on one display at a time; annotations on other displays are preserved. One stroke cannot cross display boundaries. Annotations stay at screen coordinates rather than following a scrolling app or a slide.
- The eraser removes entire touched strokes, arrows or text objects, not individual pixels.
- Undo history is bounded by 128 operations and a memory budget. Dropping old history does not erase visible annotations. Scene data has a 64MiB limit; one stroke has a 100,000-point limit and text has a 10,000 UTF-16-unit limit. Large clears may skip part of the animation to bound temporary memory.
- Intermittent hotkey failures and a reported gray overlay are under investigation. Their root cause and full resolution have not been established. If they recur, record the time, OS, display arrangement and exact steps; see [diagnostics](docs/testing/diagnostics.md).
- Windows native behavior, screen-sharing receiver output, projectors, mixed-DPI precision, Spaces/fullscreen combinations and long lecture sessions still require validation. UAC/security desktops, DRM and exclusive fullscreen games are outside the current guarantees.

## Contribute

Bug reports, reproducible test cases, Windows testing and translation improvements are welcome. Include OS/app versions, display scaling and the steps to reproduce. Screenshots and logs should be stripped of private teaching material or other sensitive information.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development expectations. Translation source messages are Korean; English translations live in [`src/locales/en.json`](src/locales/en.json). Keep placeholders such as `{0}` unchanged, and never translate annotation contents or user-authored preset names. The native menu uses the same catalog. Run the tests after changing translations.

## License and attribution

Original author: **김준현**. Code is distributed under the [Apache License 2.0](LICENSE). Retain the license and applicable copyright/attribution notices when redistributing, and mark modified files as required by that license. [NOTICE](NOTICE) identifies the original project; its official repository URL will be added when the public repository exists.

Apache-2.0 does not require publishing a fork's source code or permanently displaying the original author's name on the main app screen. My Brush includes author attribution in About.

The unmodified bundled Nanum Gothic font is separately licensed under [SIL OFL 1.1](src/assets/fonts/nanum-gothic/OFL.txt). Its [source and checksums](src/assets/fonts/nanum-gothic/SOURCE.json), original license, and dependency notices in [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt) accompany distributions.

My Brush is independently implemented and is not affiliated with ScreenBrush. It does not use ScreenBrush code, icons or branding. This statement is not a completed trademark or patent clearance review.
