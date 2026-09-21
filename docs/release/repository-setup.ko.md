# 첫 GitHub 공개 준비

검토일: 2026-09-21. 공개 이름은 **DansoonPen(단순펜)**, 권장 저장소 이름은 **dansoonpen**으로 확정했다. 원격 저장소·공개 릴리스는 아직 생성하지 않았다. GitHub 계정이 정해지면 실제 URL을 연결한다.

## 참고한 개인 계정의 오픈소스 도구

README 구조를 참고했으며 코드·이미지·소개 문구를 가져오지 않았다.

| 프로젝트 | 참고한 구성 | 이 프로젝트에 반영 |
| --- | --- | --- |
| [DrawPen](https://github.com/DmytroVasin/DrawPen) | 짧은 소개, 설치, 단축키 표, 알려진 한계 | 처음 방문한 사람이 실행과 조작 방법을 먼저 찾도록 배치 |
| [gInk](https://github.com/geovens/gInk) | 발표 중 방해하지 않는 사용 목적, 빠른 사용법, 번역 기여 안내 | 강의 흐름과 핵심 동작을 먼저 설명하고 번역 파일 연결 |
| [MarkerOn](https://github.com/ifer47/markeron) | 플랫폼별 설치 안내, 빠른 시작, 전역/도구 단축키 구분 | 지원 상태를 표로 표시하고 실제 등록되지 않은 패키지 매니저 명령은 넣지 않음 |

## 이름 검토

이름의 기억하기 쉬움·용도 전달·검색 구별성을 평가했다. 검색은 상표 등록 여부나 특정 국가에서의 사용 권한을 보증하지 않는다.

사용자 최종 선택: **DansoonPen**. 정확한 영문·한글 이름 검색에서 동명 앱을 확인하지 못했다. 이는 제한적인 검색 결과이며 사용 권리 확인은 아니다. 아래 후보는 결정 과정의 비교 기록이다.

| 후보 | 관찰 | 판단 |
| --- | --- | --- |
| My Brush | [동명 Android 그림 앱](https://play.google.com/store/apps/details?id=info.miusoft.mybrush), [유사한 macOS MyBrushes](https://apps.apple.com/us/app/mybrushes-sketch-paint-design/id676028900?mt=12) 확인 | 친숙하지만 같은 분야 검색 구별성이 약함 |
| EasyBrush | [Easy-Brush 캔버스 브러시 엔진](https://github.com/DQLean/Easy-Brush) 확인 | 같은 드로잉 분야의 기존 프로젝트와 혼동될 수 있어 비추천 |
| MePen | 이번 웹/GitHub 검색에서 동명의 화면 필기 앱은 찾지 못함. [다른 업종의 MEPEN](https://aktau.mepen.kz/about/) 등 사용 사례는 존재 | 두 사용자 제안 중 추천. `MePen — Screen annotations for teaching`처럼 용도 병기 |
| LectureInk | 강의 필기 용도를 명확히 드러내는 대안. 이번 검색에서 동명 앱은 확인하지 못함 | 강의 외 발표·일상 사용으로 확장하면 다소 좁게 들릴 수 있음 |

검색 시 사용한 표기: My Brush, MyBrushes, EasyBrush, easy-brush, MePen, LectureInk, DansoonPen, 단순펜. 검색 결과에 없다는 이유로 도메인·패키지 이름·상표가 사용 가능하다고 단정하지 않는다. GitHub 저장소 이름은 계정별로 구분되므로 전 세계에서 유일할 필요는 없다.

이름 변경 시 표시 이름·README·패키지 파일명을 맞추되, 기존 설치의 설정 경로를 바꾸는 앱 식별자 변경은 별도 마이그레이션 없이 수행하지 않는다.

## 저장소를 만들 때

1. GitHub 소유 계정을 확정한다. 앱 이름은 `DansoonPen`, 저장소 이름은 `dansoonpen`을 사용한다.
2. GitHub에서 **빈 저장소**를 만든다. 로컬에 이미 README·LICENSE·.gitignore와 커밋 이력이 있으므로 새 저장소 생성 화면의 자동 추가 옵션은 선택하지 않는다.
3. 첫 공개 전 검토를 계속하려면 일단 Private으로 만들고, 공개할 소스·이력을 확인한 후 Public으로 전환할 수 있다. Private 저장소는 일반 사용자가 다운로드할 수 없다.
4. 실제 생성된 저장소 URL을 확인해 origin을 연결하고 로컬 main을 push한다. 이 문서는 명령 실행이나 공개 승인을 대신하지 않는다.
5. About 설명, Topics, Issues, 라이선스 인식, 기본 브랜치를 확인한다. 존재하지 않는 홈페이지·설치 파일 링크는 넣지 않는다.

About 제안:

> Persistent screen annotations for teaching and presentations. Built with Tauri, Rust and Svelte. Korean and English UI.

Topics 제안: `screen-annotation`, `drawing`, `teaching`, `presentation`, `macos`, `windows`, `tauri`, `rust`, `svelte`.

Windows topic은 구현 분야를 나타낼 수 있지만 README와 릴리스에서는 실기 검증 상태를 계속 구분한다.

## 처음 올리기 전에

- [ ] 최종 앱 이름·저장소 URL을 README, NOTICE, 앱 정보 화면과 일치시킨다.
- [ ] 커밋 작성자 이름·이메일이 공개되어도 되는지 확인한다. 이메일을 바꾸려면 이후 커밋 설정과 기존 이력 수정의 범위를 먼저 구분한다.
- [ ] 현재 파일뿐 아니라 Git 이력의 비밀정보·개인 경로·내부 기록을 검토한다. `.gitignore` 추가만으로 과거 커밋이 사라지지는 않는다.
- [ ] `.planning/`과 `docs/testing/`의 공개 범위를 검토한다. 원본 로컬 기록을 지우거나 기존 이력을 임의로 재작성하지 않는다.
- [ ] LICENSE, NOTICE, 의존성 고지, 폰트 라이선스가 함께 있는지 확인한다.
- [ ] 앱 상태와 맞는 한국어·영어 README, 정상 상대 링크, 시연 이미지를 확인한다.
- [ ] `artifacts/`, 빌드 결과, 로컬 설정·진단 로그·인증서는 커밋하지 않는다.
- [ ] 별도 작업 폴더의 변경 보존과 통합 결과를 확인한다. 공개하지 않는 로컬 백업은 `artifacts/`에 둔다.

라이선스는 현재 Apache-2.0이다. 재배포 시 관련 고지 보존 의무가 있지만, 수정본 앱 메인 화면의 상시 저작자 표시나 수정본 소스 공개 의무까지 만드는 라이선스는 아니다. 원저작자 표시는 NOTICE와 앱 정보에 유지한다. [Apache-2.0 4조](https://www.apache.org/licenses/LICENSE-2.0)

## 소스 공개와 설치 파일 배포

소스 저장소를 Public으로 바꾸는 것과 GitHub Releases에 설치 파일을 올리는 것은 별도 단계다. 설치 파일은 리포지토리에 직접 커밋하지 않고, 검증된 버전의 Release 자산으로 제공한다.

첫 릴리스는 Pre-release로 시작하고 다음 자료를 함께 준비한다.

- 해당 소스 커밋을 가리키는 버전 태그와 변경 내역.
- 실제 검증된 OS/CPU용 앱 패키지. macOS에서 Windows 설치를 검증했다고 표시하지 않는다.
- 각 파일의 SHA-256, 설치·첫 실행 안내, 서명/공증 상태.
- 필기 영구 저장 없음, 화면공유 방식, 미검증 환경 등 사용에 직접 영향을 주는 한계.
- 민감한 화면이 없는 짧은 실제 시연. 모의 UI를 실제 동작 검증 영상처럼 소개하지 않는다.

Homebrew·WinGet·자동 업데이트 링크는 해당 배포 경로를 실제로 등록하고 검증한 뒤 추가한다. GitHub Pages와 별도 도메인은 첫 공개의 필수 조건이 아니다.

## 공개 이후

[홍보 계획과 게시 초안](open-source-launch.ko.md)을 사용한다. 먼저 소규모 실제 강의 피드백을 받고 GeekNews Show, Show HN 순으로 소개하는 것을 권한다. 요청을 전부 기능으로 구현하기보다 설치 실패·입력 무응답·필기 손실을 우선 처리한다.
