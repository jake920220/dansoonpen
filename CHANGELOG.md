# Changelog

## 0.7.1 — 2026-09-21 (Pre-release)

- Added a Windows x64 NSIS test installer, automated Windows checks, and a real-device test checklist.
- Fixed Windows diagnostic log writes failing with access denied: the log lock handle now includes read access, as required for Windows file locking. Existing rotation and persistence tests cover the failure.
- Windows installation, launch, and basic use passed user testing. Detailed Korean IME, mixed-DPI, screen-sharing, and extended-session checks remain open.
- [Downloads and release notes](https://github.com/jake920220/dansoonpen/releases/tag/v0.7.1).

## 0.7.0 — 2026-09-21 (Pre-release)

First public pre-release. [Release notes and macOS Apple Silicon download](https://github.com/jake920220/dansoonpen/releases/tag/v0.7.0). The app is ad-hoc signed, without Developer ID signing or Apple notarization.

### Added

- Korean and English interface selection, saved across app sessions.
- English and Korean usage guides, contribution notes, and issue templates.

### Changed

- The project is now **DansoonPen**, previously developed as My Brush. The application identifier is retained so existing settings continue to load.
- Integrated the existing v0.6.3 work on drawing across connected displays, a shared toolbar that moves between monitors, and cursor highlighting across displays.
- Kept the macOS input/focus recovery and text-selection overlay fixes from that work.
- Language changes and drawing-default resets preserve toolbar preferences.

### Validation

- macOS Apple Silicon is the current native validation platform.
- Windows implementation is present; real-device validation is still pending.
- See [verification](docs/testing/integration-2026-09-21.md) for automated checks, observed native behavior, and remaining gaps.
