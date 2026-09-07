# v0.1 아키텍처 검토

- 검토일: 2026-09-08
- 최종 판정: **APPROVE — 코드 및 브리지 계약 검토 승인**
- 범위: `src/`, `src-tauri/src/`, Tauri 설정·capability·플랫폼 코드, 브리지 계약, 관련 테스트 소스
- 방법: 독립 검토 에이전트의 읽기 전용 코드 검토 및 수정 후 재검토. 구현 파일과 Git 상태는 변경하지 않았다.

현재 코드에서 릴리스를 막을 추가적인 확정 결함은 발견하지 못했다. 이 판정은 실제 OS 입력·포커스·IME·화면공유 지원 확인을 의미하지 않는다. 네이티브 검증 결과는 별도의 실행 기록으로 남겨야 한다.

## 확인한 구조와 근거

| 영역 | 확인 내용 | 근거 |
| --- | --- | --- |
| 장면 소유권 | Rust가 장면과 전역 undo/redo를 소유하고 프런트엔드는 획 완료·텍스트 확정 시 편집을 전송한다. 포인트별 IPC와 시간 경과 삭제는 없다. | `src-tauri/src/model.rs`, `src/ui/Overlay.svelte`, `docs/development/bridge-contract.md` |
| 전체 삭제 순서 | `clearGeneration`을 필수 계약으로 사용한다. 빈 장면을 포함한 모든 알려진 화면의 세대와 revision이 증가한다. 이전 세대의 늦은 추가·삭제는 무시하고 미래 세대는 거부한다. | `SceneStore::clear`, `SceneStore::apply`, `SceneSnapshot`, `SceneEdit` |
| 삭제 후 새 필기 | 프런트엔드는 세대 증가 시 진행 중 편집과 이전 낙관적 표시를 취소한다. 새 편집은 제출 시점 세대를 캡처한다. fade는 분리된 객체 스냅샷만 렌더링한다. | `Overlay.svelte`의 `acceptScene`, `submit`; `CanvasRenderer.fadeOut` |
| 삭제와 실행 취소 | 전체 삭제는 확정된 객체들을 하나의 기록으로 남긴다. 빈 삭제는 undo 기록을 만들지 않는다. undo는 삭제 세대를 되돌리지 않는다. 미완성 제스처 취소는 undo 복구 대상이 아니다. | `SceneStore::clear`, `undo`; 브리지 계약 |
| 설정 | 프런트엔드 변경을 누적·직렬화하고 저장 중 설정 폼을 잠근다. Rust는 입력을 검증하고 단축키 변경 실패를 복구하며 임시 파일을 flush한 뒤 설정을 교체한다. | `Overlay.svelte`, `Control.svelte`, `settings.rs`, `shortcut_registry.rs` |
| 한글 | IME 키 입력을 일반 명령과 구분한다. 조합 중 확정은 blur 후 이벤트 처리 기회를 주고 DOM의 실제 값을 사용한다. UI 10,000 UTF-16 단위를 수용하도록 native 제한을 40,000 UTF-8 바이트로 맞추고 편집창 제거 전 검사한다. | `Overlay.svelte`의 `commitText`, `textKey`; `Annotation::validate` |
| 입력 복구 | 모드 전환 시 overlay 입력을 먼저 해제한다. 실패 시 click-through와 비포커스 설정을 재시도하고 필요한 창을 숨긴다. 설정 창은 포커스 복원 실패에도 접근 가능하게 한다. | `lib.rs`의 `release_inputs`, `change_mode`, `open_control` |
| 권한 | capability는 상태 이벤트 listen/unlisten만 허용한다. custom command는 native에서 창 역할과 화면 소유권을 검사한다. 배포 CSP는 원격 스크립트·일반 네트워크 연결을 허용하지 않는다. | `capabilities/desktop.json`, `tauri.conf.json`, `lib.rs` |
| 자원 사용 | renderer는 변경 및 fade 시에만 프레임을 예약한다. fade 객체·포인트·기간, native 장면 및 히스토리 크기를 제한한다. 히스토리는 Arc로 객체를 공유한다. | `renderer.ts`, `model.rs` |
| 다중 화면 | 물리 화면 크기를 native 창에 적용하고 프런트엔드는 논리 좌표와 DPR로 렌더링한다. 연결이 끊어진 화면 장면은 유지하고 입력을 해제한 뒤 창을 숨긴다. | `refresh_displays`, `ensure_overlay`, `platform/*`, `CanvasRenderer.resize` |

## 검토 중 발견하여 해결한 결함

1. 최신 clear/undo 이벤트 이후 늦은 편집 응답이 낙관적 표식을 제거해도 재렌더링하지 않아 그림이 남던 문제: 응답 처리 후 항상 장면을 다시 렌더링한다.
2. 빠른 색상·굵기 변경 중 debounce가 앞선 변경을 버리던 문제: partial 변경을 누적하고 저장을 직렬화한다.
3. 설정 저장 중 새 입력이 응답으로 덮어써지던 문제: 저장 중 fieldset을 비활성화한다.
4. 네이티브 모드 전환 후 지우개 원이 남던 문제: 모드 이탈·blur에서 명시적으로 숨긴다.
5. 비동기 작업 후 display select의 `currentTarget`을 읽던 문제: 이벤트 처리 시 화면 ID를 캡처한다.
6. 네이티브 전체 삭제 후 진행 중 획·텍스트가 뒤늦게 살아나던 문제: 필수 삭제 세대와 빈 장면 이벤트로 순서를 일치시킨다.
7. 한글 입력의 UTF-16/UTF-8 제한 차이로 UI에서 허용한 텍스트가 거부되던 문제: 제한을 맞추고 프런트엔드 사전 검증을 추가한다.

## 검증 근거와 범위

통합 담당 에이전트가 최종 수정본에서 다음 결과를 보고했다. 검토 에이전트는 중복 빌드를 실행하지 않았으며 관련 테스트의 소스와 주장 범위를 확인했다.

- Rust 테스트 17개 통과, `cargo fmt` 및 엄격한 clippy 통과. 최종 확인은 실제 번들 리소스를 사용했다.
- 프런트엔드 테스트 14개 통과, Svelte 검사 경고 0개, production build 통과.
- 삭제 세대 테스트는 늦은 추가·삭제 무시, 새 세대 편집 보존, 빈 장면 이벤트, 누락·미래 세대 거부, undo 후 세대 유지, 한글 10,000자 허용을 다룬다.
- renderer 테스트는 idle 상태의 프레임 중단, DPR, 새 그림과 fade 분리, 겹친 fade, undo 복원, reduce motion, 자원 해제를 다룬다.

## 실제 환경 검증이 남은 항목

- macOS 실제 앱의 외부 앱 클릭 전달, 이전 앱 포커스 복원, 전체 화면·Spaces·트레이 복구.
- macOS 한국어 IME 조합 중 Enter·Esc·완료 버튼·모드 전환·전역 전체 삭제의 실제 이벤트 순서와 마지막 글자 보존.
- 실제 다중 모니터의 서로 다른 배율, 좌표, 선택 전환, 연결 해제·재연결 후 필기 유지.
- Windows 빌드 및 WebView2의 투명 창, click-through, 전경 창 복귀, 전역 단축키, 화면 배율과 다중 모니터.
- 화면공유 제품별 모니터 전체 공유와 특정 앱 창 공유의 실제 동작.

이 항목들은 브라우저 미리보기나 단위 테스트로 대체하지 않으며, 특히 Windows와 화면공유 조합을 검증 완료로 표시하지 않는다. 전체 삭제는 UI가 새 세대를 수신한 시점을 경계로 미완성 제스처를 취소한다는 계약을 실제 입력에서도 확인해야 한다.
