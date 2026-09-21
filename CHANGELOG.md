# Changelog

## Unreleased — 0.7.0

This is the first public-release candidate. No public release or version tag has been published yet.

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
