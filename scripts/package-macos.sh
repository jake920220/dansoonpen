#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != Darwin ]]; then
  echo 'macOS에서 실행하세요.' >&2
  exit 1
fi
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != '--skip-build' ) ]]; then
  echo '사용법: bash scripts/package-macos.sh [--skip-build]' >&2
  exit 1
fi
brush_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$brush_root"
if [[ "${1:-}" != '--skip-build' ]]; then
  npm run tauri build -- --bundles app
fi

brush_version="$(node -p 'JSON.parse(require("fs").readFileSync("package.json", "utf8")).version')"
brush_arch="$(uname -m)"
brush_bundle='src-tauri/target/release/bundle/macos/DansoonPen.app'
mkdir -p artifacts
brush_package_dir="$(mktemp -d "$brush_root/artifacts/.macos-package.XXXXXX")"
trap 'rm -rf -- "$brush_package_dir"' EXIT
ditto "$brush_bundle" "$brush_package_dir/DansoonPen.app"
# Local testing signature only; this is not Developer ID signing or notarization.
codesign --force --deep --sign - "$brush_package_dir/DansoonPen.app"
codesign --verify --deep --strict "$brush_package_dir/DansoonPen.app"
brush_archive="$brush_root/artifacts/DansoonPen-$brush_version-macOS-$brush_arch.zip"
ditto -c -k --sequesterRsrc --keepParent "$brush_package_dir/DansoonPen.app" "$brush_archive"
shasum -a 256 "$brush_archive"
echo "로컬 검증용 패키지: $brush_archive"
