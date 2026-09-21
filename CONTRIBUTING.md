# Contributing

**English** | [한국어](CONTRIBUTING.ko.md)

Bug reports, translation improvements and testing on real Windows/macOS hardware are welcome. Public repository links will be added once the repository exists.

## Development

Follow the setup and validation commands in [README.md](README.md). Keep changes focused on one feature or fix, explain the behavior before and after, and include meaningful regression tests. Keep the app lightweight: avoid unnecessary dependencies, continuous rendering and per-pointer-sample IPC.

Native input, focus, transparency, display scaling and global shortcuts must be tested in the desktop app. Browser previews cannot establish native support. Validate screen sharing on the receiving device as well as the sender. Clearly label environments that have not been tested.

## Translations

The interface supports `ko` and `en`. Korean source messages are the fallback; English translations live in `src/locales/en.json`. Svelte components use the reactive `t` store, and native menus read the same catalog. Preserve all placeholders, keyboard shortcuts and accessibility labels. Do not translate or rewrite annotation contents or user-authored preset names. Run `npm test`, `npm run check` and the native tests after changes.

The language setting is a persisted settings field, not a drawing tool option. A language-only update must preserve annotations, current tool, presets, shortcuts and mode. Resetting drawing defaults must not unexpectedly reset the selected language.

## Reports and patches

Describe expected and actual behavior, reproduction steps, OS/app version and monitor resolution/scaling. Include a timestamp for diagnostic-log correlation. Review logs and screenshots for private information before attaching them. Do not post credentials or private lecture material.

Use focused commits. This repository's current maintainers use Conventional Commit subjects with a Korean description and a bullet list of details in the body. Review staged files and validation results before committing. Remote configuration, push and public releases are separate maintainer actions.

## Licensing

Contributions must be code you can distribute under Apache-2.0. Do not copy another product's code, icons or paid assets without permission. Preserve applicable LICENSE and NOTICE information and mark modifications as required by the license. No copyright assignment or additional contribution license is requested.

When changing dependencies, regenerate notices with `node scripts/generate-notices.mjs` and verify with `node scripts/generate-notices.mjs --check`. Review any dependency-specific source-distribution or modification-notice obligations.
