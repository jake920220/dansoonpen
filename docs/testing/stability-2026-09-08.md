# My Brush 전체 안정성 검증 — 2026-09-08

검사 기간: 2026-09-08 23:45–2026-09-09 00:02 KST. 문서 날짜는 조사 시작일이다.

**판정: 코드·로그·자동 검사에서 확인한 범위는 아래와 같다. 간헐적 단축키 무반응과 전체 화면의 검은 회색 덮임은 원인 미확정이며, 해결·재발 방지·실기 인수 완료로 판정하지 않는다.**

Canvas 컨텍스트 복원 후 재렌더가 예약되지 않는 결함 1건을 이벤트 주입으로 재현하고 수정했다. 등록 실패 복구·네이티브 포커스·혼합 DPI 등 남은 위험은 분리해 기록했다. 브라우저/단위 검사로 OS 입력이나 화면공유를 대신 판정하지 않았다.

## 범위와 환경

- 기준 소스: `61a62a4`, v0.4.0. 조사 worktree: `/Users/kimjunhyun/.codex/worktrees/c282/my-brush`, 브랜치 `codex/stability-audit-20260908`.
- Apple M2 / arm64, macOS 26.6.2 (25G83), Node 26.4.0, Rust 1.94.1. 고정 의존성: Tauri 2.11.5, runtime-wry 2.11.4, Tao 0.35.3, Wry 0.55.1, global-hotkey 0.8.0, global-shortcut plugin 2.3.2.
- 원본 프로젝트와 설치 앱을 변경하지 않았다. My Brush PID 74800은 19:40:58부터 실행 중이며, ScreenBrush PID 82151 및 helper PID 82156도 함께 실행 중이었다. 동시 실행만으로 충돌 원인이라고 판단하지 않는다.
- 디스크 설치 앱 `/Users/kimjunhyun/projects/my-brush/artifacts/My Brush.app`의 Info.plist는 0.4.0. 실행 파일 SHA-256: `2a9e3b0dc87399eb1180c99dfc792b47810d6678e8e51241855e05f21c8c7dcb`. 실행 중인 초기 v0.4.0의 UI 버전 표시는 인계 내용상 0.3.1일 수 있다. 이번 수정은 이 실행 세션에 적용하지 않았다.
- 지정된 사건 문서 2개, diagnostics/verification/compatibility/performance 문서, `.planning/debug/screen-share-overlay-pileup.md`, 원본 프로젝트의 사건 보존 자료를 검토했다.
- 직전 검사에서 실제 입력과 자동 입력이 겹쳤다는 인계에 따라, 재시작·기존 필기 삭제 없이 5분간 네이티브 조작하는 일정을 질문했다. 응답이 없어 입력 조작을 수행하지 않았다. CUA로 실행 앱 목록, My Brush AX 트리 및 창 캡처만 읽었다.
- CUA AX에는 단일 필기 창과 Canvas가 표시됐다. 창 단독 캡처는 흰 화면으로 반환되어 데스크톱 합성·투명도 판정 근거로 사용하지 않았다. 앱을 띄우기 위한 Raise/클릭/단축키 입력은 하지 않았다.

## 증거 보존과 로그 재집계

23:45:25 KST에 실시간 diagnostics의 JSONL 4개를 다음 Git 제외 경로로 복사하고 각 사본에 0600 권한·바이트 수·SHA-256 manifest를 기록했다.

`artifacts/stability-20260908/snapshot-20260908-234525/`

원본 JSONL과 `diagnostics.lock`은 삭제·수정하지 않았다. 복사는 동작 중인 파일의 순차 사본이므로 원자적인 파일 세트 스냅샷은 아니다. 실제 사본을 파싱·정렬한 결과, 아래 보존 구간 내부에는 잘못된 JSON 줄·중복 순번·순번 누락이 없었다. 시작 전 회전된 기록까지 완전하다는 뜻은 아니다.

| 항목 | 이전 장애 조사 세션 재검사 | 현재 실행 세션의 보존된 구간 |
| --- | --- | --- |
| 세션 | `1788839887220-66259` | `1788864058793-74800` |
| KST 구간 | 12:58:07.220–13:44:07.214 | 20:01:37.708–23:45:24.011 |
| 레코드 / 순번 | 4,000 / 1–4,000 | 7,950 / 791–8,740 |
| 구간 내부 누락·중복 / reported dropped | 0 / 0 | 0 / 0 |
| session.start / exit | 있음 / 있음 | 둘 다 없음: 시작은 회전, 세션은 실행 중 |
| 수신 누름 / 놓음 | 162 / 162 | 1 / 1 |
| 동작 / dispatch 대응 | 토글 97·삭제 65, 모두 대응 | 삭제 1, 대응 |
| 수신→dispatch 최대 | 1ms | 0ms |
| 네이티브 probe | 275회, 최대 126ms | 1,342회, 최대 631ms |
| probe pending / session_busy | 0 / 0 | 0 / 0 |
| probe의 모드 | draw 72·interact 203 | interact 1,342 |
| Draw 진입 후 네이티브 포커스 | 72회, 최대 137ms | 진입 기록 없음 |
| 필기 heartbeat 최대 간격 | 10.137초 | 11.954초 |
| 필기 Canvas | 2560×1440 | 2560×1440 |
| 관측된 필기 창 | 생성 전 0 → 생성 후 1 | 계속 1 |
| 앱 오류·panic·명령 실패·context loss 기록 | 없음 | 없음 |

현재 구간에서 `canBecomeKey=false`, `ignoresMouse=true`와 interact 상태가 probe마다 일치했다. 같은 창 frame과 scale=1, opaque=false가 유지됐다. 필기 객체 수는 1→0이고 실제 삭제 키 이벤트가 있었다. 자동 검사가 만든 삭제가 아니다. 이 구간은 장시간 그리기·전환·삭제 스트레스 테스트가 아니다.

이전 세션의 13:29:10.683 `focused=false` / Draw 유지와 13:29:11.618 Alt+X 정상 처리를 다시 확인했다. 이후 WebView `document.hasFocus()`와 네이티브 key 상태의 차이는 기존 조사 결론을 유지한다. `native key`와 `document.hasFocus()`를 같은 의미로 취급하지 않는다.

원본 macOS hotkeys 통합 로그 21,425개 항목에서 `canBecomeKeyWindow` 경고 72개를 다시 확인했다. `processDidTerminate`, `gpuProcessExited`, `unresponsive` 문자열은 없었다. 문자열 부재는 WebKit/WindowServer 장애 전체의 부재를 증명하지 않는다. 첫 사건 errors/lifecycle 자료에는 조사 범위 전체로 경고가 각각 12개 있어, 해당 PID·시간대를 제한한 기존 사건 기록 11회와 구분했다.

집계 산출물: `current-audit.json`, `hotkeys-audit.json`, `audit_logs.py` (모두 `artifacts/stability-20260908/`). 이전 `audit-summary.json`의 4,000개·162쌍·97/65 집계와 일치했다. 전체 로그·필기·오류 원문을 보고서에 싣지 않는다.

**관측 한계:** 앱에 도착하지 않은 물리 키는 확인할 수 없다. `dropped=0`은 진단 큐의 기록 상태이며 키 수신 보증이 아니다. 10초 probe 사이의 짧은 이상도 놓칠 수 있다. 숨겨진 설정창은 이전 구간에서 heartbeat 간격이 최대 223.041초여서, 설정창 heartbeat만으로 필기창 정지를 판단해서는 안 된다.

## 실제 실행한 검사

| 검사 | 결과와 검증 범위 |
| --- | --- |
| `npm ci` | 성공, lock 유지. npm audit 출력 0 vulnerabilities. 취약점 전체/제품 보안 인증으로 해석하지 않음 |
| 수정 전 `npm test` | 8개 파일·43개 통과 |
| 수정 전 `cargo test --locked --manifest-path src-tauri/Cargo.toml` | 37개 통과 |
| 신규 Canvas 회귀, 수정 전 | 13개 중 신규 2개 실패·기존 11개 통과: 복원 이벤트 후 재렌더 0회 |
| 신규 Canvas 회귀, 수정 후 | 13개 전부 통과 |
| 최종 `npm test` | 8개 파일·45개 통과 |
| 최종 Rust 전체 테스트 | 40개 통과, 추가한 단축키 실패 2개·두 화면 반복 시나리오 1개 포함 |
| `npm run check` | Svelte 오류 0·경고 0 |
| `npm run build` | 프로덕션 JS/CSS 및 2,054,744바이트 Nanum Gothic 생성 |
| `cargo fmt ... -- --check` | 통과 |
| `cargo clippy --locked ... --all-targets -- -D warnings` | 통과 |
| `node scripts/generate-notices.mjs --check` | 잠금 의존성과 로컬 고지 원문 일치 |
| `npm run tauri build -- --no-bundle --ci` | 성공, optimized release 컴파일 4분 29초. Mach-O arm64 실행 파일 확인. 설치 앱 교체·추가 `.app` 생성 없음 |
| 설치 앱 read-only 검사 | `codesign --verify --deep --strict` 통과. LICENSE·NOTICE·THIRD_PARTY_NOTICES·Nanum Gothic OFL 네 리소스가 소스와 일치 |
| 스크립트 구문 | macOS 포장 스크립트 `bash -n`, 측정 Python AST 파싱 통과. Windows PowerShell 실행은 안 함 |
| 혼합 DPI 계산 재현 | 이전 재현 소스를 worktree artifacts로 복사해 `cargo run --locked --offline` 실행. 기존 계산 차이 재현·후보 계산 12개 통과. GUI 검증 아님 |
| 5분 자원 관측 | 기존 측정 스크립트로 실행 중인 메인 PID만 관측. 전체 앱/유휴 성능 판정 아님 |

### 회귀 검사의 의미

- `src/canvas/renderer.test.ts`: 실제 `CanvasRenderer`에 복원 이벤트를 보낸다. 편집 없이 기존 필기 재렌더, DPR/전체 bitmap clear, 최신 장면·진행 중 획 유지, 만료 fade 재등장 방지, 복원 이벤트 병합 및 dispose 후 예약 없음 검사. Canvas/GPU·OS 자체의 장애를 강제한 시험은 아니다.
- `src-tauri/src/shortcut_registry.rs`: 실제 `replace`에 실패하는 Registry를 주입한다. 새 키 추가→이전 키 일부 해제→해제 실패→롤백 등록도 실패한 상태에서 추적값과 가짜 OS 등록값이 일치하고, 장애 해제 후 복원되는지 확인했다. 캡처 후 두 번째 키 복원 실패 시 부분 등록이 남지 않고 재시도 가능한지도 검사했다. 실제 Carbon/Windows API 실패의 원자성까지 이 fake가 보장하지 않는다.
- `src-tauri/src/model.rs`: 100회 반복해 A 텍스트 작성·A 화면 삭제·늦은 A IME 확정 거부·B의 진행 중 편집 수용·A 새 필기·undo/redo를 확인했다. 각 화면 내용과 이력 한도를 실제 모델로 검증한다. 모델에 문자열을 전달한 검사는 실제 IME 조합 검사가 아니다.
- 기존 1,000획·100,000점, 다중 화면 clear barrier, 같은 ID 텍스트 교체의 순서·undo/redo, 지우개 220ms fade, 설정 v4 기본값 이전·사용자 지정값 보존 검사도 전체 실행에 포함된다. 신규 프런트엔드/네이티브 컴포넌트 통합 검사를 했다고 확대하지 않는다.

### 자원 관측

`scripts/measure-macos.py`로 PID 74800의 300.067초를 관측했다. 메인 프로세스 단일 코어 기준 평균 CPU 0.730%, RSS 31.922–39.969 MiB. 사용자 활동을 통제하지 않았고 검사 빌드가 병행됐다. WebKit/GPU PID 귀속을 검증하지 않아 합산하지 않았다. **전체 앱 유휴 CPU 목표 달성, 메모리 누수 없음, 강의 부하 합격을 의미하지 않는다.** 원자료: `main-process-5min.json`.

## 확인된 결함과 수정

### B1 · P2 · Canvas 컨텍스트 복원 후 기존 필기가 재표시되지 않음 — 수정

- 위치: `src/canvas/renderer.ts:23`, `:31`, `:104`; 기존 진단 전용 관찰은 `src/app/diagnostics.ts`의 `observeCanvasDiagnostics`.
- 트리거: 필기가 있는 상태에서 마지막 RAF가 끝난 뒤, 2D 컨텍스트의 bitmap이 소실·복원되고 추가 편집/resize/fade가 없음.
- 재현: `npm test -- src/canvas/renderer.test.ts`. 장면을 한 번 그린 뒤 `contextlost`/`contextrestored`를 주입하고 다음 RAF의 draw 결과를 확인한다.
- 실제 결과: 수정 전 신규 2개 실패. 기존 코드는 복원 이벤트를 진단에만 남기고 프레임을 예약하지 않았다. 수정 후 장면을 재렌더하고 기존 색·좌표·DPR을 유지한다.
- 수정: 컨텍스트 복원 리스너가 기존 단일 RAF 예약 함수를 호출하고 dispose가 리스너를 제거한다. commit `d9fe0d1`.
- 근거: [HTML Standard의 context loss/restoration 절차](https://html.spec.whatwg.org/multipage/webappapis.html#context-lost-steps)와 [2D 컨텍스트 reset](https://html.spec.whatwg.org/multipage/canvas.html#reset-the-rendering-context-to-its-default-state)은 bitmap/렌더 상태 초기화와 복원 이벤트를 규정한다. 이벤트 복원 후 재그리기는 앱의 책임이다.
- 한계: 실제 WebKit context loss가 이번 사건 때 발생했다는 기록은 없다. 엔진이 복원 이벤트를 보내지 않거나 WebContent 자체가 정지·종료되면 이 수정만으로 복구하지 못한다. 검은 회색 오버레이와 단축키 미수신의 원인 수정으로 발표하지 않는다.

## 코드상 위험 — 네이티브 재현과 구분

P1은 발현 시 입력/사용에 큰 영향을 줄 가능성이 있어 다음 재현의 우선순위가 높은 항목이다. P2는 제한된 트리거·복구 가능성이 있는 항목이다. 아래 심각도는 발현 확률이나 사건 원인 확신도를 뜻하지 않는다. 별도 표시가 없으면 네이티브 제품 코드는 수정하지 않았다.

### R1 · P1 · 캡처 종료 중 등록 복원 실패 후 재시도 생략

- 위치: `src-tauri/src/lib.rs:670` (`end_shortcut_capture`), 특히 `:672`; `src-tauri/src/shortcut_registry.rs:30` (`replace`).
- 트리거/재현 절차: 설정 입력칸을 선택해 전역 키를 해제한 뒤, 다른 프로세스의 독점 등록 등으로 기존 키 재등록이 실패하도록 한다. Esc/blur로 종료를 시도하고 충돌을 해제한 뒤 다시 `capture(false)` 또는 창 닫기를 수행한다.
- 코드상 결과: `capturing_shortcut=false`를 복원 성공 전에 설정한다. 복원이 실패해 빈 등록 목록으로 롤백되면 다음 종료 호출은 `if`를 통과하지 않아 재등록을 시도하지 않는다. 설정 저장이나 캡처 새 진입·종료가 필요할 수 있다.
- 실제 검사: Registry 수준에서 복원 실패→빈 목록→명시적 재시도 성공은 새 테스트로 확인했다. 호출자 전체·OS 등록 충돌 재현은 안 했다. 현재/이전 로그에는 해당 등록 실패가 없다.
- 권고: 복원 성공 후에만 캡처 완료로 확정하거나 복원 대기 상태를 별도로 두고, 창 blur/close에서 재시도되도록 호출자 회귀 검사를 먼저 작성한다. 오류 후에도 트레이/설정 복구 경로가 남는지 실제 앱으로 확인한다.

### R2 · P1 · 목표 모니터와 현재 창의 배율 차이

- 위치: `src-tauri/src/lib.rs:193`의 `set_size(PhysicalSize)`와 `:196`의 `set_position`; Tao 0.35.3 macOS `window.rs:728`, `:755`.
- 트리거/재현 절차: 배율 1 창을 배율 2 대상 화면의 물리 위치·크기로 배치. 복사한 `geometry-repro`에서 실제 고정 dpi 0.1.2 변환을 실행했다.
- 실제 산술 결과: 요청 (5120,968), 2940×1912가 현재 scale=1로 논리 좌표 (5120,968), 2940×1912로 해석된다. 목표 scale=2 기준 기대값은 (2560,484), 1470×956이다. 목표 배율로 변환한 후보는 양/음 원점을 포함한 12개 산술 사례 통과.
- 한계: 실제 창 생성 시 초기 scale·비동기 이동·DPI 전환 이후 frame까지 재현한 것이 아니다. 현재 보존 구간의 필기창은 계속 기본 화면 scale=1이고 잘못된 frame이 관측되지 않았다.
- 권고: macOS 대상 화면의 논리 위치/크기 적용 방식을 준비하고, 실제 두 화면에서 생성·이동·정밀 좌표·재연결·음수 원점·Spaces를 확인한 뒤 적용한다. Windows physical 좌표 경로를 함께 바꾸지 않는다.

### R3 · P2 · 표시 시 비활성 창에 키 창 요청

- 위치: `src-tauri/src/lib.rs:238` (`change_mode`), show 단계 `:254`; 비활성 창 표시 경로 `refresh_displays:457`, `toggle_cursor_highlight:291`.
- 트리거: `release_inputs`로 focusable=false인 창에 `show()`한 다음 true로 바꿈. Tao `set_visible(true)`는 `make_key_and_order_front_sync` 호출.
- 실제 근거: 과거 세션의 경고 72회와 해당 순서가 일치한다. 72회 모두 이후 137ms 안에 네이티브 포커스를 얻었다. 이번 자동 조작으로 재발시킨 경고는 아니다.
- 권고: 입력용 표시 순서와 비입력용 표시 방식을 구분해 준비하고, 경고 제거뿐 아니라 뒤 앱 복귀·커서 전용 표시·모니터 갱신에서 포커스를 빼앗지 않는지 검증한다. 현재 경고를 장애 원인으로 확정하지 않는다.

### R4 · P2 · 키업 누락 시 다음 누름이 반복으로 처리됨

- 위치: `src-tauri/src/lib.rs:982`의 Released 처리와 `:991`의 `pressed.insert`.
- 트리거/절차: Pressed 뒤 Released를 전달하지 않고 다시 Pressed를 전달한다.
- 코드상 결과: 다음 Pressed가 `repeat`로 무시된다. 이어서 정상 Released가 도착하면 latch가 풀리고 그 다음 Pressed는 처리된다. 영구 잠금이라고 단정하지 않는다.
- 실제 근거: 이전 162쌍·현재 1쌍 모두 키업이 있고 repeat 무시도 없다. OS에서 키업을 잃는 상황은 재현하지 못했다.
- 권고: 실제 물리 키 홀드·메뉴 열기·포커스 이동·캡처 진입/취소를 섞어 수신 시퀀스를 확인한다. 근거 없이 timeout으로 pressed를 비우면 홀드 중 재토글될 수 있으므로 먼저 OS 키 상태와 대조할 방안을 검증한다.

### R5 · P2 · Draw 상태로 네이티브 포커스가 다른 앱에 있을 때 토글 의미

- 위치: `src-tauri/src/lib.rs:999` 부근 전역 토글, `:1169` 이후 window event 처리; `src-tauri/src/platform/macos.rs:10`의 PreviousFocus.
- 절차: Draw → 다른 앱으로 포커스 이동 → Alt+Z 한 번.
- 코드상 결과: 토글은 네이티브 key 여부 대신 저장된 Draw 모드를 보고 Interact로 전환한다. 사용자가 ‘다시 그리기’를 기대하면 한 번 더 눌러야 하는 상황이 가능하다. Draw 중 다른 앱으로 이동한 뒤 돌아올 때 캡처한 이전 앱이 오래된 값일 가능성도 검증해야 한다.
- 실제 근거: 과거 13:29:10 포커스 이탈은 확인했지만 다음 키는 Alt+X였으므로 이 토글 시나리오를 입증하지 않는다.
- 권고: CUA 5분 조율 시 첫 재현 항목으로 두고, 현재 앱·모드·네이티브 key·토글 의도를 함께 확인한다. 포커스 이탈만으로 필기를 삭제하거나 모드를 임의 변경하지 않는다.

### R6 · P1 · 메인 스레드/IPC 정지 시 같은 경로의 복구도 대기

- 위치: `src-tauri/src/lib.rs:48`, `:63`, `:205`; `src-tauri/src/diagnostics.rs:261`; `src/ui/Overlay.svelte:278`.
- 트리거/절차: Draw 중 네이티브 작업이나 applyEdit IPC가 완료되지 않는 상황을 격리된 검사 앱에서 주입한다.
- 코드상 결과: macOS 명령·트레이·전역 단축키 동작 모두 메인 스레드 경로를 공유한다. `native_command`의 `rx.recv()`에는 timeout이 없고, UI Esc의 `action`도 기존 edits·설정 저장 완료를 기다린다. watchdog은 별도 스레드에서 pending을 기록하지만 자동 입력 복구는 하지 않는다.
- 정상 방어: 전환 실패 시 입력 해제를 재시도하고 Interact/오류 상태를 발행한다. `release_inputs`는 한 창 실패에도 나머지 창을 시도하고 해당 창 hide를 시도한다. 단, hide 실패는 무시되어 모든 API가 실패하면 실제 해제 보증이 없다.
- 실제 근거: 보존한 두 구간에 pending/busy/해제 실패가 없다. 실행 중인 사용자 앱을 고의로 멈추는 fault injection은 안 했다.
- 권고: 별도 테스트 앱에서 IPC 대기와 네이티브 멈춤을 구분해 탈출 경로를 시험한다. timeout만 붙이면 늦게 도착한 명령이 다시 Draw로 바꿀 수 있으므로 작업 취소/유효성까지 설계한 뒤 적용한다. 현재 진단 기능을 watchdog 자동 복구라고 안내하지 않는다.

### R7 · P2 · 메인 스레드의 동기 저장과 모니터 조회 적체

- 위치: `src-tauri/src/lib.rs:591` 이후 설정 저장 경로, `:1147` 이후 3초 모니터 조회; `src-tauri/src/settings.rs:77`.
- 트리거: 느린 디스크/fsync 또는 네이티브 작업 지연 중 반복 설정 변경·모니터 조회.
- 코드상 결과: macOS에서는 Session mutex를 가진 메인 스레드가 `sync_all`을 기다린다. 모니터 조회는 3초마다 새 작업을 보내고 미완료 요청의 수를 제한하지 않는다. 반면 커서 poll과 watchdog은 요청 1개만 진행하도록 제한되어 있다.
- 실제 근거: 본 로그 구간에서 적체·오류를 확인하지 못했다. 단순 `run_on_main_thread`를 무조건 다음 턴 큐라고 가정하지 않았다. 고정 runtime-wry의 `send_user_message:235`는 메인 스레드 호출이면 즉시 실행한다. Tao Focused 이벤트는 `AppState::queue_event:370`에서 큐에 넣으므로, focus 이벤트의 즉시 재진입으로 mutex가 반드시 교착된다고 단정할 근거도 없다.
- 권고: 지연 저장을 주입한 격리 검사와 명령 순서 추적을 먼저 수행하고, 필요 시 저장과 창 조작의 잠금 범위·모니터 조회 동시 요청 수를 제한한다.

### R8 · P2 · 단일 인스턴스와 모니터 ID 변경의 자원 보존

- 위치: `src-tauri/src/lib.rs:148` 창 재사용, `:457` 화면 갱신, `:957`의 `run`; `src-tauri/src/settings.rs:73`의 고정 `.tmp` 저장 경로.
- 절차: 별도 복사 실행 파일 동시 실행 또는 디스플레이 ID가 달라지는 연결 변경을 반복.
- 코드상 결과: 같은 화면 label은 기존 창을 재사용하지만 연결 해제된 창은 hide만 하고 삭제하지 않는다. 알려진 화면/SceneStore도 보존한다. 프로세스 단일 실행 guard는 없으며 diagnostics의 파일 잠금은 로그 쓰기용이다. 여러 인스턴스가 동시에 설정 저장 시 같은 임시 파일을 사용할 수 있다.
- 실제 근거: 현재 My Brush 메인 프로세스는 하나이며 단축키 사건 구간에서도 창 증가를 확인하지 못했다. 두 번째 앱을 실행하거나 화면 연결을 바꾸지 않았다.
- 권고: 화면의 필기 보존 정책을 유지하면서 숨긴 WebView 수와 scene bytes를 관측한다. 단일 인스턴스 및 설정 쓰기 경쟁은 격리 테스트 후 별도 수정 범위로 다룬다.

### R9 · P2 · 전역 키와 메뉴 가속키의 독립 경로

- 위치: `src-tauri/src/lib.rs:837` 이후 트레이 키, `:888` 이후 메뉴 처리, `:973` 이후 전역 handler.
- 절차: 앱/다른 앱/트레이 메뉴가 각각 활성인 상태에서 같은 물리 키 한 번을 보내 출처별 로그를 비교.
- 코드상 결과: 전역 콜백만 pressed 억제를 공유한다. tray draw는 Draw 지정, 전역 키는 toggle이라 두 출처가 같은 입력으로 발생할 경우 처리 순서에 따라 결과가 달라질 수 있다.
- 실제 근거: 한 물리 입력이 두 경로 모두에 전달된 증거는 없다. global-hotkey 0.8.0의 Carbon 등록 옵션은 0이다. 기존 SDK 조사처럼 등록 성공만으로 타 앱 충돌 부재를 보장하지 않는다.
- 권고: 등록 성공을 충돌 검사 합격으로 표현하지 말고, ScreenBrush 공존 상태에서 동의받은 짧은 재현을 수행한다. 기본키·등록 독점성·메뉴 제거를 추측으로 바꾸지 않는다.

### R10 · P1 · Windows 작업 실행 순서 실기 미확인

- 위치: `src-tauri/src/lib.rs:59`의 Windows `thread::spawn`, `:973` 전역 handler, `src-tauri/src/platform/windows.rs:18` 포커스 복귀.
- 트리거/절차: Windows에서 빠른 Pressed/Released·토글/삭제·설정 변경을 반복하고 received/dispatched/action 순서를 비교.
- 코드상 결과: 각 네이티브 작업을 별도 스레드로 보내므로 mutex는 동시 실행을 막지만 원래 수신 순서의 FIFO까지 보장하지 않는다. 키업이 키다운보다 먼저 처리되는 등의 순서 위험이 있다. `SetForegroundWindow`도 실패를 반환할 수 있으며 코드가 이를 오류로 취급한다.
- 실제 결과: Windows 실행 환경이 없어 빌드·UI·입력·순서 모두 미검증. macOS 검사 및 모델의 직렬 호출 테스트로 대신 합격 처리하지 않았다.
- 권고: `scripts/verify-windows.ps1` 및 물리 입력 시나리오를 Windows 11에서 수행하고, 순서 반전이 재현되면 WebView 생성 제약을 유지하는 단일 작업 실행 방식을 검토한다.

## v0.4.0 기능별 결과와 남은 실기 매트릭스

| 대상 | 이번에 확인한 내용 | 미완료 / 다음 절차 |
| --- | --- | --- |
| 지우개 220ms·서버 에코 | 렌더러 fade 시간·중복 활성 fade·undo 테스트 통과. Overlay의 erased/pendingRemovals 필터 경로 검토 | 실제 펜 입력·긴 지우기·IPC 응답 순서·220ms가 지난 뒤 에코의 컴포넌트 통합 재현 |
| 전체 삭제와 새 필기 | 실제 모델 barrier·다중 화면·100회 추가 시나리오 통과 | 포인터 down 중 Alt+X → 새 필기 → up을 실제 OS 입력으로 검사 |
| 텍스트 재편집 | 같은 ID·레이어 순서·한 번의 undo/redo·오래된 수정 거부 테스트 통과 | 기존 글 2번째 줄 클릭·편집 취소·빈 내용 완료·다른 앱 전환 |
| 한글 IME | byte 길이·물리 도구 키 단위 검사, blur/조합 종료 대기 코드 검토 | 두벌식 마지막 받침·Enter/Shift+Enter/Esc·조합 중 전역 삭제·포커스 이동 |
| Nanum Gothic/기본 40px | 폰트 해시 `76f45ef4a6bcff344c837c95a7dcc26e017e38b5846d5ae0cdcb5b86be2e2d31`, 번들 산출·OFL·v4 이전 검사 통과 | 현재 네이티브 WebView의 실제 font load 및 확정 전후 baseline. 로드 실패 catch 후 state 연결 코드는 존재하나 실제 실패 주입은 안 함 |
| 캡처 취소·닫기 | Control의 capture Promise 직렬화, Recorder blur/destroy, 네이티브 control blur/close 복원 경로 검토 | CUA로 입력칸 → Esc/Tab/창 닫기/다른 앱 → 키 정상 복귀, 등록 실패 분기 R1 |
| 단축키/포커스 | 과거 162쌍 처리 재확인, 현재 창 AX·probe 상태 읽기 | 실제 홀드·100회 전환·20회 지연 p95·Draw에서 다른 앱 이동. 자동 키 조율 응답 대기 |
| 투명도/검은 회색 화면 | 소스 transparent/no shadow, Wry `drawsBackground=false` 경로 및 native opaque=false 확인 | 실제 장애의 발생 시각·직전 키·공유 앱·본인 화면/수신 화면 차이 필요. CUA 창 캡처는 합성 판단 불가 |
| 다중 모니터 | 구성 2560×1440@1 + 2940×1912@2, 계산 차이 재현 | 혼합 DPI 실제 frame·정밀 좌표·음수 원점·연결 해제/재연결 |
| Spaces/전체 화면 | macOS collection behavior/level 25 코드 확인 | Spaces 왕복·전체 화면 슬라이드·프로젝터 입력/포커스. macOS 다른 버전·Intel 미검증 |
| 장시간 자원 | 현존 로그 3시간 44분 구간 및 메인 PID 5분 관측 | 30분 이상 실제 필기·다중 화면·귀속 WebView/GPU 합산. 화면공유 부하와 수신 프레임 별도 측정 |
| Windows/공유 수신 | 관련 코드·검사 스크립트 검토 | Windows 11 실기, 공유 수신 기기/Zoom/Meet/기타 사용 앱 모두 미검증 |

## 다음 우선순위

1. 조율된 5분 네이티브 검사: 기존 필기 수·모드·active display를 확인하고, Alt+Z 왕복·다른 앱으로 포커스 이동·캡처 Esc/창 닫기를 실시한다. 사용자 입력이 섞이면 즉시 중단한다. 단축키 수신/출처와 네이티브 key를 같은 시간대에 대응한다.
2. R1 등록 복원 실패의 호출자 회귀 및 복구 상태 수정. R3 표시 순서 후보와 R2 혼합 DPI 배치 후보는 실제 두 화면 검증과 함께 기능 단위로 다룬다.
3. 기존 필기 삭제·앱 재시작이 허용된 별도 검사 시간에 빠른 삭제/새 필기·IME·IPC 실패·컨텍스트/프로세스 손실을 시험한다. 실행 앱 교체는 이 보고서의 수정 커밋만으로 자동 수행하지 않는다.
4. Windows 실기와 실제 공유 수신 화면, 전체 앱 자원 측정을 완료해야 정식 안정성 인수 여부를 판단할 수 있다.

## 재실행과 산출물

저장소 루트에서 실행한다. 아래는 로컬 검증 명령이며 앱 실행·설치·push를 하지 않는다.

```sh
npm ci
npm test
npm run check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
node scripts/generate-notices.mjs --check
npm run tauri build -- --no-bundle --ci
```

이 worktree에 보존된 증거를 다시 집계하려면:

```sh
python3 artifacts/stability-20260908/audit_logs.py artifacts/stability-20260908/snapshot-20260908-234525 --output artifacts/stability-20260908/current-audit.json
python3 artifacts/stability-20260908/audit_logs.py /Users/kimjunhyun/projects/my-brush/artifacts/incident-20260908-hotkeys --output artifacts/stability-20260908/hotkeys-audit.json
cargo run --locked --offline --manifest-path artifacts/stability-20260908/geometry-repro/Cargo.toml
```

주요 검사 로그는 `artifacts/stability-20260908/`의 `baseline-*`, `context-recovery-before.log`, `context-recovery-after.log`, `final-frontend.log`, `expanded-rust.log`, `final-svelte.log`, `final-fmt.log`, `final-clippy.log`, `notices.log`, `release-build.log`, `geometry.log`다. `evidence-manifest.json`에 이 산출물들의 바이트 수와 SHA-256을 보존했다. 회귀 테스트 커밋은 `2b89278`이다. 원자료·바이너리는 Git에 포함하지 않는다. 새로운 앱 번들·리모트 연결·push·공개 릴리스를 만들지 않았다.

### 빌드 완료 기록

`npm run tauri build -- --no-bundle --ci` 종료 코드 0. optimized release 컴파일은 4분 29초였다. 생성물 `src-tauri/target/release/my-brush`는 `Mach-O 64-bit executable arm64`이며 SHA-256은 `ccad50c615f72a514f41ff1ca5928134f196fc753cc91781ebfcf7ec161276a6`이다. 실제 실행·설치·bundle 서명은 수행하지 않았으므로 새 수정의 네이티브 런타임 검증으로 간주하지 않는다. 설치 앱의 기존 서명 검사는 별도 항목이다.
