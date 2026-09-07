# 2026-09-08 구현 인수 기록

상태: **v0.1 코드와 macOS 로컬 테스트 패키지 준비 완료. 양 OS 정식 지원 검증은 미완료.**

## 통과한 검사

| 검사 | 결과 |
| --- | --- |
| `npm run check` | Svelte 오류0·경고0 |
| `npm test` | 2개 파일, 14개 테스트 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --offline --all-targets -- -D warnings` | 통과 |
| `cargo test --offline` | 17개 테스트 통과, 0.84초 |
| `npm run tauri build -- --bundles app` | 최종 macOS arm64 release 빌드 성공, 2분02초 |
| `node scripts/generate-notices.mjs --check` | 잠금 파일·로컬 원문 일치, 303개 의존성 고지 |
| 독립 아키텍처 검토 | APPROVE, [검토 기록](architecture-review.md) |
| 실제 macOS 핵심 흐름 | 준현님이 투명 필기·Esc 후 필기 유지 및 뒤 앱 조작 정상 확인 |
| 번들 법적 고지 | 세 파일의 SHA-256이 소스 원본과 일치 |
| 로컬 패키지 | macOS 포장 스크립트 구문·실행 및 ad-hoc 서명 검증 통과 |

Rust 최종 검사는 실제 번들 리소스를 사용했다. 초기 리소스 준비 전 검사에서 썼던 테스트용 override를 최종 검증에는 사용하지 않았다. 빌드 중 생성한 브라우저 미리보기 adapter는 production JS에 포함하지 않는다.

## 전달물

- `artifacts/My-Brush-0.1.0-macOS-arm64.zip`: 로컬 검증용 macOS 앱. ad-hoc 서명이며 Developer ID 공증본은 아니다.
- `artifacts/My-Brush-0.1.0-source.zip`: 기능·문서 커밋을 포함한 소스 사본. Windows PC에서 압축을 풀고 README의 준비 절차와 `scripts/verify-windows.ps1`을 실행한다.
- `artifacts/SHA256SUMS.txt`: 전달 ZIP의 체크섬.
- 바이너리와 임시 로그는 Git에 넣지 않는다. 리모트·push·공개 업로드는 수행하지 않았다.

## 다음 필수 검증

1. Windows 11 PC에서 빌드·설치 후 동일한 필기/유지/조작/삭제 흐름을 확인한다.
2. 실제 macOS/Windows 한글 IME, 전역 키, 전체화면 슬라이드, 배율이 다른 모니터·연결 변경을 확인한다.
3. Zoom 또는 Meet의 모니터 전체 공유를 다른 수신 기기에서 확인하고 빔프로젝터를 시험한다.
4. 장시간 유지·100회 전환과 귀속 WebView를 포함한 CPU/메모리·공유 부하 측정을 한다.

자세한 시나리오는 [호환성 기록](compatibility.md)과 [성능 문서](performance.md)를 따른다. 이 결과를 먼저 반영한 뒤 도형·형광펜 등 고도화 우선순위를 정한다.
