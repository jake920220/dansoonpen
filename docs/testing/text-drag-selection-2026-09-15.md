# 텍스트 편집 중 오버레이 선택 방지 — v0.6.2

준현님은 텍스트 입력 기능에서 의도하지 않은 드래그 후 화면 전체 오버레이가 푸른빛 또는 짙은 회색으로 보이는 현상을 보고했다. 기존 오버레이는 캔버스 입력만 `preventDefault`로 막고, 텍스트 편집창의 제목·안내문을 비롯한 DOM 영역에는 WebKit의 기본 텍스트 선택과 드래그가 열려 있었다.

v0.6.2는 오버레이 문서에만 `user-select: none`과 WebKit 드래그 방지를 적용한다. 텍스트 입력란과 입력 컨트롤에서는 글자 선택을 계속 허용한다. 선택 가능하지 않은 영역의 `selectstart`와 모든 기본 `dragstart`를 취소하고, 텍스트 이동 시작과 창 포커스 이탈 때 문서 선택 범위를 정리한다. 전용 텍스트 이동 손잡이의 포인터 캡처와 위치 변경은 유지한다.

## 검증

- `npm run check`: Svelte 오류·경고 0
- `npm test`: 프런트엔드 55개 통과
- `cargo test --locked`: Rust 52개 통과
- `cargo fmt --check`, strict Clippy 통과
- macOS arm64 v0.6.2 release 앱 빌드와 ad-hoc 서명 검증 통과
- 실제 설치 앱에서 텍스트 입력란의 일부 글자 선택이 유지되는 것을 확인했다.
- 선택된 글자가 있는 상태에서 편집창 제목을 화면 바깥 방향으로 드래그했을 때 선택 표시가 사라지고 화면 전체 선택이 생기지 않았다.
- 전용 `텍스트 이동` 손잡이로 편집창 위치가 계속 이동했다. 검증용 글은 저장하지 않고 취소했다.

이 경로는 보고된 색 변화와 일치하는 구체적인 재현 후보지만, 과거 장시간 사용 사건 당시의 화면 선택 상태가 기록되지 않았으므로 그 사건의 유일한 원인으로 확정하지 않는다.

설치 앱은 `/Users/kimjunhyun/projects/my-brush/artifacts/My Brush.app`, 이전 v0.6.1 앱은 `/Users/kimjunhyun/projects/my-brush/artifacts/backups/My Brush 0.6.1 before 0.6.2 20260915-0943.app`에 보존했다. 로컬 패키지는 `artifacts/My-Brush-0.6.2-macOS-arm64.zip`이며 SHA-256은 `00fc1bd6dc57af23c038953fb19f47869e151c75419666a022cd87a4fa088310`이다.
