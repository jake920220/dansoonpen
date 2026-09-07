# ADR 0001: Tauri와 OS별 얇은 오버레이 어댑터

상태: v0.1 구현에 채택, Windows·강의 수신 화면·장시간 성능 합격은 별도 검증 대기.

## 결정

Tauri 2/Rust가 모드, 창, 디스플레이, 단축키, 설정, 완료된 장면과 undo/redo를 관리한다. Svelte 5는 설정·도구 UI, Canvas 2D는 필기를 렌더링한다. 진행 중 포인터 샘플은 WebView 안에서 처리하고 획이 끝날 때 한 번 편집을 보낸다.

투명 오버레이는 사용한 모니터에만 만든다. 조작 모드에서는 창의 실제 커서 이벤트와 포커스를 해제하며 필기 픽셀은 남긴다. 렌더러는 정지 시 requestAnimationFrame을 예약하지 않는다. Rust 장면과 이력은 Arc로 객체를 공유하고, undo 데이터와 현재 필기의 한도를 따로 둔다.

macOS 어댑터는 AppKit의 Spaces/fullscreen 보조 창 동작과 이전 앱 포커스를 처리한다. Windows 어댑터는 foreground 복원을 처리하며 WebView2 창 생성은 동기 UI 콜백 바깥에서 실행한다. Tauri 공통 API로 해결되는 크기·위치·클릭 통과는 재구현하지 않는다.

장면 revision으로 오래된 응답을 무시한다. 전체 삭제는 별도 clearGeneration을 증가시킨다. 전역 키로 삭제하는 동안 늦게 도착한 편집은 이전 세대로 식별하여 버리고, 새 세대의 입력은 유지한다. 빈 화면 삭제도 이벤트를 보내 진행 중 편집이 취소되도록 한다.

## 판단 근거

Tauri는 OS WebView를 사용해 Chromium 런타임을 포함하지 않으므로 배포 크기에 유리하다. 하지만 WebView 프로세스와 고해상도 비트맵 비용이 있으므로 낮은 메모리·CPU 사용을 프레임워크 이름만으로 보장하지 않는다. [Tauri 아키텍처](https://v2.tauri.app/concept/architecture/)

macOS 투명 창은 Tauri의 private API 옵션을 사용한다. 이 구조는 Mac App Store 제출용이 아니며 직접 배포를 대상으로 한다. App Store가 요구되거나 실제 성능/OS 호환성이 목표에 미달하면 공통 Rust 모델을 유지하면서 네이티브 렌더러를 비교한다. [Tauri 창 API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)

## 실제 구조

- `src/shared/types.ts`: 직렬화 계약
- `src/canvas/`: 곡선·문자 렌더러, 지우개 판정, fade
- `src/ui/`: 설정과 오버레이 입력·IME
- `src/app/bridge.ts`: IPC 및 상태 구독
- `src/dev/preview.ts`: 개발 모드에만 들어가는 브라우저 미리보기
- `src-tauri/src/model.rs`: 장면·전역 이력·검증
- `src-tauri/src/lib.rs`: 명령·창·트레이·화면 연결 변경
- `src-tauri/src/platform/`: OS별 작은 보완
- `src-tauri/src/settings.rs`, `shortcut_registry.rs`: 영속 저장·단축키 트랜잭션

## 검증 책임

단위 검사는 장면 일관성·삭제 순서·이력 예산을 검사한다. 실제 클릭 통과, 포커스, IME, Spaces, DPI, 수신 화면은 하드웨어에서 검증한다. 확인 결과와 미검증 항목은 testing 문서에 구분해 남긴다. Windows 실행 환경 없이 빌드 가능성을 실기 지원으로 바꾸어 표현하지 않는다.
