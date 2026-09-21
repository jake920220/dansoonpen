# My Brush 첫 오픈소스 공개 계획

작성: 2026-09-21. 아래 홍보 순서는 My Brush의 강의·발표 용도에 맞춘 제안이며 노출량이나 사용자 수를 보장하지 않는다. 외부 게시·저장소 생성·push는 아직 하지 않았다.

## 누구에게 무엇을 보여줄까

첫 사용자는 온라인 강사, 코드 리뷰·실시간 코딩을 설명하는 개발자, 세미나 발표자다. 핵심 소개는 **“강의 화면에 필기를 남기고, 지운 뒤 바로 이어 그릴 수 있는 오픈소스 도구”**로 잡는다. 다른 제품의 유료 기능 우회나 복제품이라는 표현보다는 직접 겪은 강의 문제와 독립 구현한 동작을 보여준다.

## 추천 순서

| 순서 | 채널 | 준비물과 목적 |
| --- | --- | --- |
| 1 | 준현님과 연결된 강사·개발자 5–10명 | 실제 강의 한 번에 사용해 보고 설치·단축키·화면공유 문제 수집. 숫자는 첫 테스트 모집 규모 제안이다. |
| 2 | GeekNews Show | 한국어 소개와 20–30초 시연, 소스 및 설치 링크. 개발 배경·현재 한계·원하는 피드백을 함께 적기. |
| 3 | Hacker News의 Show HN | 영문 README, 직접 실행 가능한 다운로드 또는 간단한 빌드 절차, 영문 데모. 기술 질문과 재현 보고에 답할 시간 확보. |
| 4 | Product Hunt | 설치가 안정되고 영문 스크린샷·짧은 영상·소개 문구가 준비된 뒤 일반 생산성 도구 사용자에게 소개. |

GeekNews의 Show는 서비스·오픈소스를 알리는 별도 공간이다. [GeekNews Show](https://news.hada.io/show)

Show HN은 다른 사람이 직접 실행하거나 사용해볼 수 있는 제작물을 대상으로 하므로, 소개글만 있고 제품을 시험할 수 없는 상태는 피한다. [Show HN 가이드](https://news.ycombinator.com/showhn.html)

Product Hunt는 소개 페이지와 갤러리 등 출시 자료를 미리 준비하도록 안내한다. 첫날 모든 채널에 동시에 올리기보다 초기 피드백을 반영해 순차 공개하는 것을 권한다. [Product Hunt 준비 안내](https://www.producthunt.com/launch/preparing-for-launch)

사용자가 이미 모여 있는 곳에서 문제와 해결을 설명하고, 다운로드·문서로 이어지게 한다. 이 접근은 GitHub Open Source Guides의 사용자 찾기 안내와도 맞는다. [Finding Users](https://opensource.guide/finding-users/)

## 공개 전 실제로 준비할 것

- [x] Apache-2.0 LICENSE, 원저작자 NOTICE, 의존성 및 폰트 고지 유지.
- [x] 한국어·영어 README와 앱 언어 선택 구현.
- [x] 기여 안내와 버그·기능 제안 템플릿 준비.
- [ ] v0.6.3 별도 작업본과 공개할 main의 기준 통합 확인. 실행 앱보다 기능이 적은 버전을 공개하지 않기.
- [ ] GitHub 소유 계정·저장소 이름 확정 후 NOTICE·앱 정보·README에 검증된 URL 기입. 지금은 가짜 링크를 넣지 않는다.
- [ ] 현재 파일뿐 아니라 Git 이력의 토큰·개인 경로·내부 메모·작성자 이메일을 확인. 개인 경로는 비밀키와 다르지만 공개 의도를 확인할 자료다. 이력을 임의로 삭제하거나 재작성하지 않는다.
- [ ] macOS 다른 기기에서 다운로드·설치·첫 실행 검사, Developer ID 서명·공증 여부와 한계를 릴리스에 명시.
- [ ] Windows는 실제 PC 검증 전까지 실험적/미검증으로 표기. 빌드 성공을 사용성 확인으로 대체하지 않는다.
- [ ] 화면공유 수신 기기·프로젝터·한글 조합·물리 단축키 반복·장시간 사용 검사.
- [ ] 민감한 강의 자료가 없는 20–30초 데모와 설정창 스크린샷 준비.
- [ ] GitHub 저장소 공개·첫 릴리스·홍보 게시를 각각 실행할 때 최종 URL과 자료 확인.

소스 공개와 일반 사용자용 앱 배포는 나눠 진행할 수 있다. 소스를 먼저 공개하더라도 README에서 현재 설치 방법과 미검증 범위를 분명히 알린다. 공개 직후 안정성이 확인되지 않은 앱을 “강의에서 문제없이 동작”한다고 홍보하지 않는다.

## 25초 시연 구성

- 0–5초: 강의 슬라이드 위에 펜으로 표시.
- 5–10초: 앱 조작 모드로 바꾸고 슬라이드를 조작해도 필기 유지.
- 10–15초: 휴지통으로 필기만 삭제하고 같은 도구로 바로 다시 작성.
- 15–20초: 기존 텍스트를 클릭해 수정·이동.
- 20–25초: 한국어↔영어 선택과 저장소·다운로드 안내.

## 한국어 게시 초안

제목: **Show GN: My Brush — 강의 화면에 필기를 남기는 오픈소스 도구**

강의를 하면서 화면에 그린 표시가 설명 도중 사라지거나, 지울 때마다 드로잉 모드가 풀리는 것이 불편해서 My Brush를 만들었습니다.

필기는 직접 지울 때까지 유지되고, 다른 앱을 조작할 때도 화면에 남습니다. 휴지통은 그림만 지우며 도구와 메뉴 독을 유지합니다. 펜·형광펜·화살표·재편집 가능한 텍스트, 실행 취소, 한국어·영어 UI를 제공합니다. Tauri/Rust/Svelte로 만들었고 계정이나 서버 연결 없이 사용합니다.

현재 macOS에서 검증을 진행하고 있으며 Windows는 실제 PC 확인이 남아 있습니다. 단축키와 화면 오버레이의 간헐적 문제도 계속 조사 중입니다. 강의·발표 중 사용해보신 분들의 설치 환경과 재현 가능한 피드백을 받고 싶습니다.

게시 전 여기에 확정된 소스·설치·시연 링크와 해당 릴리스의 실제 검증 범위를 넣습니다.

## English draft

Title: **Show HN: My Brush — persistent screen annotations for teaching**

I teach and often need to mark up my screen while explaining code or slides. I built My Brush so annotations stay until I clear them, and clearing doesn't interrupt drawing by dismissing the toolbar.

It includes a pen, highlighter, arrows, editable text, undo/redo and a Korean/English interface. It is built with Tauri, Rust and Svelte and works without an account or server.

macOS validation is ongoing; Windows implementation still needs testing on real hardware. Intermittent hotkey and overlay issues are under investigation. I'd especially appreciate reproducible feedback from people who teach or present with screen sharing.

Before posting, add the actual repository, download and demo URLs, and update the tested-platform statement to match that release.

## 첫 공개 이후 무엇을 볼까

별 수보다 다운로드 후 설치 성공, 실제 강의 한 번 사용, 재사용 의향, 동일 장애 재현 여부를 먼저 본다. 첫 1–2주는 이슈에 재현 환경을 요청하고 해결한 내용을 릴리스 노트로 남긴다. 추가 기능보다 설치 마찰과 입력·포커스 안정성 개선을 우선한다.

## 출처 표시의 정확한 의미

현재 Apache-2.0은 재배포 시 관련 저작권·NOTICE·라이선스 고지 보존 등의 의무를 둔다. 수정본의 메인 화면에 항상 원저작자 이름을 띄우거나 수정본 소스를 공개하도록 요구하지는 않는다. My Brush의 정보 화면에는 원저작자를 표시한다. [Apache-2.0 원문, 특히 4조](https://www.apache.org/licenses/LICENSE-2.0)

공개 저장소에는 README·기여 방법·라이선스 등 기본 문서를 함께 제공하는 것이 좋다. [Starting an Open Source Project](https://opensource.guide/starting-a-project/)
