# My Brush

강의 화면 위에 선과 텍스트를 남기는 macOS·Windows용 화면 필기 도구입니다. 필기는 직접 지우기 전까지 유지되며, 앱 조작 모드에서는 뒤쪽 프로그램을 그대로 사용할 수 있습니다.

**v0.1은 로컬 검증 단계입니다.** OS별 실제 검증 범위는 [호환성 기록](docs/testing/compatibility.md)을 확인하세요. Windows 설치·실기 검증과 회의 수신 화면 검증을 완료하기 전까지 두 OS의 정식 지원을 선언하지 않습니다.

## 포함 기능

- 자유곡선 펜, 기본 12px·1–32px 굵기, 원형 빠른 색상과 색상환·밝기·HEX 선택
- 한글·영문 여러 줄 텍스트, 획 단위 지우개
- 직접 지우기 전까지 유지, 전체 삭제 시 350ms 페이드, 애니메이션 줄이기
- 실행 취소·다시 실행, 여러 모니터의 전체 삭제를 한 번에 복구
- 키 조합을 직접 눌러 전역 단축키 변경·충돌 안내, 트레이에서 입력 복구·설정·종료
- 모니터 선택, 연결 해제된 화면의 필기를 실행 세션 안에서 보관
- 로컬 설정 저장. 계정·서버·화면 녹화 기능 없음

화살표·도형·형광펜·포인터 강조·세션 저장·이미지 내보내기는 고도화 단계입니다.

## 개발 실행

검증한 도구: Node.js 26.4, npm 11.17, Rust/Cargo 1.94. macOS 최소 설정은 13.0이며, Windows의 첫 검증 대상은 Windows 11 x64입니다. macOS에는 Xcode Command Line Tools, Windows에는 Visual Studio C++ Build Tools와 WebView2가 필요합니다. [Tauri 사전 준비](https://v2.tauri.app/start/prerequisites/)

```sh
npm ci
npm run tauri dev
```

`npm run dev`만 실행하면 브라우저 UI 미리보기가 열립니다. 실제 화면 오버레이·전역 키·클릭 통과를 검사하려면 반드시 Tauri 앱을 실행하세요.

```sh
npm run check
npm test
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## 설치 파일 만들기

각 OS에서 빌드합니다. macOS에서 Windows 설치 파일을 만든 것으로 간주하지 않습니다.

```sh
# 최초 고지 재생성 시 모든 대상의 dependency cache 확보
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
node scripts/generate-notices.mjs

# macOS .app
npm run tauri build -- --bundles app

# 로컬 검증용 ad-hoc 서명과 ZIP 포장까지
bash scripts/package-macos.sh

# Windows 설치 프로그램 (Windows 터미널에서)
npm run tauri build -- --bundles nsis
```

macOS 결과는 `src-tauri/target/release/bundle/macos/My Brush.app`, 로컬 서명 ZIP은 `artifacts/`, Windows 결과는 `src-tauri/target/release/bundle/nsis/`에 생성됩니다. 로컬 ZIP의 ad-hoc 서명은 Developer ID 서명·공증이 아닙니다. 공개 배포 전에는 서명·공증과 새 기기 설치를 따로 검증해야 합니다. 현재 리모트·자동 업로드·업데이트 서버는 없습니다.

## 사용법

앱을 열고 그릴 화면을 선택한 뒤 **그리기 시작**을 누르세요. macOS 메뉴 막대 또는 Windows 트레이의 펜 아이콘에서도 설정과 복구 메뉴를 열 수 있습니다.

트레이 메뉴 오른쪽에는 OS 형식에 맞는 단축키가 표시됩니다. 그리기·전체 지우기는 저장한 단축키를 따르며 설정을 바꾸면 즉시 갱신됩니다. 앱 조작 복귀는 Esc, macOS 종료는 ⌘Q로 표시합니다. 필기 창 또는 My Brush 설정 창이 활성화된 상태에서 ⌘, (Windows Ctrl+,)로 설정 탭을 바로 엽니다. 그리던 내용은 유지됩니다.

| 동작 | macOS | Windows |
| --- | --- | --- |
| 그리기 ↔ 앱 조작 | Option+Z | Alt+Shift+Z |
| 전체 지우기 | Option+X | Alt+Shift+X |
| 설정 열기 (My Brush 활성화 시) | Cmd+, | Ctrl+, |
| 그림을 유지하고 앱 조작 | Esc | Esc |
| 실행 취소 / 다시 실행 | Cmd+Z / Cmd+Shift+Z | Ctrl+Z / Ctrl+Shift+Z |
| 펜 / 지우개 / 텍스트 | P / E / T | P / E / T |
| 빠른 색 / 굵기 | 1–6 / [ ] | 1–6 / [ ] |

텍스트 편집 중에는 도구 문자키가 입력을 가로채지 않습니다. Enter는 입력 완료, Shift+Enter는 줄바꿈, Esc는 편집 취소입니다. 한글 조합 중에는 조합 처리를 우선합니다.

설정의 색상·굵기·글자 크기·애니메이션은 자동 저장되며 다음 필기부터 적용됩니다. 이미 그린 내용은 바뀌지 않습니다. 기본 색상은 노란색 `#ffcf56`이며 6색 구성은 유지합니다. 원형 색상을 누르면 현재 펜 색상을 바꾸고, 무지개 버튼을 누르면 색상환이 열립니다. 색상환의 방향키 좌우는 색조, 상하는 채도를 조절합니다. 밝기·HEX 입력과 ‘빠른 색상 N에 저장’도 사용할 수 있습니다.

단축키는 입력칸을 클릭하고 원하는 조합을 누른 뒤 놓고 **단축키 적용**을 누르세요. Esc는 입력 취소, Tab은 다음 항목으로 이동합니다. 입력 중에는 My Brush의 전역 단축키를 잠시 해제하고 입력 종료·창 닫기·포커스 이탈 시 복구합니다. 다른 앱이 선점해 등록에 실패하면 이전 조합으로 복원하고 오류를 알립니다. OS·다른 앱의 모든 단축키 충돌을 사전에 판별할 수는 없습니다. macOS Option 조합은 특수문자 입력과 겹칠 수 있으므로 해당 문자를 자주 쓰면 조합을 바꾸세요. Windows의 Alt+Z는 [NVIDIA 오버레이](https://www.nvidia.com/en-us/geforce/guides/gfecnt/geforce-experience-shadowplay-is-now-share/)와 겹쳐 기본 조합에서 피했습니다.

이전 설정은 자동으로 읽습니다. 기존 기본 굵기 5px는 12px로, 이전 기본 단축키 두 개를 그대로 쓰던 경우에는 위 조합으로 갱신합니다. 개별 변경한 색상·굵기·단축키는 유지합니다.

화면공유에서는 **모니터 전체**를 선택하세요. 특정 앱 창만 공유하면 별도 오버레이가 수신자에게 전달되지 않을 수 있습니다. 그림은 화면 좌표에 고정되며, 뒤 앱의 스크롤이나 슬라이드를 따라 이동하지 않습니다.

## 범위와 한계

- 그림은 앱 실행 중 메모리에만 보관합니다. 앱을 완전히 종료하면 사라집니다. 창 닫기는 트레이로 숨기며, 종료는 앱 정보나 트레이 메뉴에서 합니다.
- 한 번에 한 모니터에 그립니다. 다른 화면의 그림은 유지됩니다. 모니터 경계를 넘는 하나의 획은 지원하지 않습니다.
- 지우개는 픽셀 일부가 아니라 맞닿은 획·텍스트 객체 전체를 지웁니다.
- 전체 삭제의 실행 취소는 완료된 필기를 복구합니다. 전역 삭제 순간 작성 중인 획·텍스트는 취소하며, 늦게 도착한 편집 때문에 다시 나타나지 않도록 삭제 세대를 검사합니다.
- 실행 취소는 최대 128개 작업이며 메모리 예산에 따라 더 적을 수 있습니다. 이력이 줄어들어도 현재 보이는 필기를 자동 삭제하지 않습니다. 현재 장면 데이터 64MiB를 초과하는 새 편집은 오류로 안내합니다.
- 매우 큰 삭제 장면은 fade 임시 데이터 제한(2,000개 객체·200,000점 비용) 때문에 일부가 즉시 사라질 수 있습니다. 삭제와 실행 취소의 대상은 그대로 유지됩니다.
- 한 텍스트는 최대 10,000 UTF-16 단위, 한 획은 최대 100,000점입니다. 긴 획은 먼저 저장하고 다시 그리도록 안내합니다.
- UAC/보안 데스크톱, DRM 화면, 게임 독점 전체화면은 보장 범위가 아닙니다.

## 오픈소스와 출처

원저작자: **김준현**. My Brush의 코드는 [Apache License 2.0](LICENSE)으로 제공합니다. 재배포 시 라이선스와 관련 저작권·출처 고지를 보존하고 변경한 파일을 표시해야 합니다. [NOTICE](NOTICE)는 원본 프로젝트의 출처를 담습니다. 원본 저장소 URL은 실제 공개 후 기록합니다.

Apache-2.0은 수정본 소스 공개나 앱 메인 화면의 상시 저작자 표시를 요구하지 않습니다. 자체 앱은 정보 화면에서도 원저작자를 표시합니다. 의존성에는 각자의 라이선스가 적용되며 [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt)를 소스와 배포물에 함께 제공합니다.

이 프로젝트는 ScreenBrush의 코드·아이콘·제품 이름을 사용하지 않고 요구 기능을 독립적으로 구현했습니다. 타 제품과의 제휴를 의미하지 않습니다. 프로젝트 이름과 공개 배포에 관한 최종 상표·특허 검토를 완료했다고 주장하지 않습니다.

설계 배경은 [기술 결정](docs/adr/0001-overlay-stack.md), 검증은 [호환성](docs/testing/compatibility.md)·[성능](docs/testing/performance.md), 기여 방법은 [CONTRIBUTING.md](CONTRIBUTING.md)에 정리합니다.
