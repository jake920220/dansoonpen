# 화면 드로잉 앱 조사

조사일: 2026-09-08 (한국 시간). 공식 제품 설명·개발 문서·법령·사용자 리뷰를 구분해 기록했다. 앱 실행 비교, 실제 성능 측정, 특허·상표의 포괄적 권리조사는 수행하지 않았다.

## 1. 제품과 사용자 피드백

| 제품 | 확인한 기능 / 과금 구분 | My Brush에 반영할 제안 |
| --- | --- | --- |
| ScreenBrush | 공식 App Store 설명에 Freeze Mode, Ghost Mode, 텍스트, 도형, 선택 삭제, 스냅샷, 커서 강조, 숫자 배지가 있다. 무료+인앱결제 구조다. 현재 기능별 무료/유료 경계 전체는 공식 공개 자료로 확정하지 못했다. 무료판의 1~2초 자동 소멸과 유지 기능의 유료 제한은 준현님의 사용 경험이다. | 유지가 기본인 펜, 조작 모드와 지우기의 분리, 단계적으로 숫자 배지·스냅샷 추가. [공식 설명](https://apps.apple.com/us/app/screenbrush/id1233965871?mt=12) |
| Presentify | 유료 Mac 앱. 펜·형광펜·텍스트·도형, 화이트보드, 커서 강조, 스포트라이트·확대 제공. | 작은 도구막대, 빠른 모드 전환, 한눈에 고르는 펜 프리셋. [공식 사이트](https://presentifyapp.com/) |
| Epic Pen | 공식 가이드에서 Pro 전용으로 사용자 지정 색상, 도형, 텍스트, Fading Ink, 화이트/블랙보드, 커서 강조를 명시한다. Fading Ink는 지정 시간이 지나면 소멸하는 기능이다. | 색상·텍스트·도형은 유용하다. 시간제 소멸은 후속 선택 기능으로 두고, 첫 버전은 사용자 명령으로만 삭제한다. [공식 가이드](https://epicpen.com/userguide), [기능 소개](https://epicpen.com/features) |
| gInk / ppInk | Windows 오픈소스. gInk는 클릭 통과·여러 모니터·필압·펜 프리셋을 제공한다. ppInk는 도형·텍스트·숫자·편집·페이딩 등으로 확장한다. 저장소에서 MIT 표기를 확인했다. | 색+굵기+불투명도를 묶은 프리셋과 숫자 배지를 참고한다. 코드를 가져오면 실제 파일·DLL·이미지의 개별 라이선스를 재확인한다. [gInk](https://github.com/geovens/gInk), [ppInk](https://github.com/pubpub-zz/ppInk), [ppInk 사용법](https://pubpub-zz.github.io/ppInk/) |
| ZoomIt | Microsoft 공식 설명은 전역 단축키, 확대, 드로잉, 텍스트, 쉬는 시간 타이머, 녹화·캡처를 소개한다. 현재 페이지에는 Mac 다운로드 링크도 있어 Windows 전용이라고 단정하지 않는다. Mac판의 기능 동등성은 별도 확인이 필요하다. | 단축키와 화이트보드는 참고하고 확대·녹화·타이머는 첫 버전에서 제외한다. [Microsoft 공식 설명](https://learn.microsoft.com/en-us/sysinternals/downloads/zoomit) |

### 실제 리뷰에서 얻은 요구사항

- Presentify의 2025-10-29 리뷰는 단축키로 빠르게 주석을 그리는 흐름을 높이 평가한다. 2021-11-30 리뷰는 빠른 색상 선택 수와 단축키 지우개를 요구한다. [미국 App Store 리뷰](https://apps.apple.com/us/app/presentify-screen-annotation/id1507246666?mt=12&platform=mac&see-all=reviews)
- Presentify의 2022-10-30 리뷰는 주석을 그리면서 슬라이드를 넘기거나 뒤쪽 콘텐츠를 조작하는 Interactive Mode를 긍정적으로 평가한다. [캐나다 App Store 리뷰](https://apps.apple.com/ca/app/presentify-screen-annotation/id1507246666?mt=12&platform=mac&see-all=reviews)
- ScreenBrush의 2025-02-12 리뷰는 고정된 그림과 새로 그리는 임시 그림을 함께 사용하고 싶다는 요구를 제기한다. 같은 리뷰의 멈춤 경험은 개별 사용자 보고이며 재현된 현재 결함으로 취급하지 않는다. [ScreenBrush 리뷰](https://apps.apple.com/us/app/screenbrush/id1233965871?mt=12&platform=mac&see-all=reviews)
- 위 리뷰는 기능의 가치를 찾는 정성적 근거다. 대표성 있는 설문이나 시장 점유율·인기 순위 근거는 아니다. 오래된 피드백을 현재 제품의 미해결 문제로 단정하지 않는다.

우선순위 판단: **그림 유지, 앱 조작 복귀, 간결한 단축키, 펜 프리셋, 실행 취소**를 먼저 만든다. 추가 가치가 높은 후보는 **숫자 배지, 고정 펜과 임시 레이저의 병용, 커서 강조, 화이트보드**다.

## 2. Tauri 적합성

Tauri 2는 구현 후보로 적합하지만, 이 앱에 필요한 모든 동작을 공통 웹 코드만으로 보장할 수 있다는 의미는 아니다. 창 관리와 OS 통합을 초기 검증 대상으로 둔다.

| 항목 | 공식 근거와 판단 |
| --- | --- |
| 배포 크기 | Tauri는 macOS에서 WKWebView, Windows에서 WebView2를 사용한다. 브라우저 엔진을 앱마다 묶지 않는 구조여서 배포 크기에 유리한 출발점이다. Windows WebView2 런타임 설치 여부도 배포 설계에 포함해야 한다. [Tauri WebView 문서](https://v2.tauri.app/reference/webview-versions/) |
| Electron과 비교 | Electron은 Chromium과 Node.js를 함께 제공한다. 엔진 일관성이 장점이고 런타임 포함 비용이 있다. 따라서 패키지 크기 관점에서 Tauri를 우선하되, 실제 메모리·CPU 우열은 같은 기능의 release 빌드로 측정해야 한다. [Electron 소개](https://www.electronjs.org/docs/latest/) |
| 투명·최상단·클릭 통과 | Tauri에 transparent, alwaysOnTop, setIgnoreCursorEvents 등이 있다. 그러나 CSS의 pointer-events 설정만으로 다른 앱까지 클릭이 통과하지는 않는다. 창 차원의 입력 처리가 필요하다. [Tauri Window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/) |
| macOS 투명 WebView | Tauri 문서는 macOS 투명 창에 macOSPrivateApi 설정이 필요하며 이 경로는 Mac App Store 승인을 받을 수 없다고 명시한다. GitHub 직접 배포를 계획할 수 있지만, OS 업데이트 호환성과 서명·공증을 따로 검증한다. 네이티브 캔버스로 바꾸면 이 의존성을 피할 여지를 별도 검토한다. [같은 API의 WindowOptions](https://v2.tauri.app/reference/javascript/api/namespacewindow/#windowoptions) |
| 전역 단축키 | 공식 global-shortcut 플러그인이 있고 눌림·해제 이벤트를 구분한다. 다른 앱이 쓰는 단축키와 충돌할 수 있다. macOS와 Windows의 modifier 키 차이, 한글 입력 중 처리를 검증한다. [플러그인](https://v2.tauri.app/plugin/global-shortcut/), [API](https://v2.tauri.app/reference/javascript/global-shortcut/) |
| 전체화면·가상 데스크톱 | Tauri의 visibleOnAllWorkspaces는 Windows에서 미지원이다. 최상단 옵션만으로 macOS 전체화면 Space·Keynote·모든 가상 데스크톱 동작을 확정하지 않는다. macOS는 AppKit 창/패널 동작 보완을 검증한다. [Tauri Window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/) |
| Windows 포커스 | Win32는 WS_EX_NOACTIVATE·WS_EX_TOOLWINDOW·WS_EX_TOPMOST 같은 창 동작을 제공한다. 텍스트 입력에는 실제 키보드 포커스가 필요하므로 비활성 오버레이와 입력 상태를 구분해야 한다. WS_EX_TRANSPARENT만 설정하면 모든 클릭 문제가 해결된다고 가정하지 않는다. [Microsoft 창 스타일](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles) |

성능상 주의: 작은 설치 파일이 작은 실행 메모리를 보장하지 않는다. 3840×2160 RGBA 버퍼 하나만 해도 33,177,600바이트, 약 31.6MiB다. 두 모니터, 여러 캔버스, WebView·GPU 버퍼가 비용을 늘린다. 따라서 모니터별 필요 시 창 생성, 불필요한 복수 버퍼 제한, 정지 상태 재렌더링 중단, 과도한 점 데이터 억제가 핵심이다. 이 수치는 픽셀 버퍼 산술이며 제품의 측정 결과가 아니다.

### 화면공유

Zoom은 전체 화면과 선택한 앱 창 공유를 구분한다. 별도 프로세스의 오버레이가 특정 앱 창 공유에 포함된다고 보장할 수 없다는 것이 이 프로젝트의 설계 판단이다. 준현님은 **모니터 전체 공유·빔프로젝터 우선**을 선택했다. [Zoom 공식 설명](https://www.zoom.com/en/products/virtual-meetings/features/screen-sharing/)

전체 화면 공유에서도 회의 앱의 캡처 방식에 따라 차이가 생길 수 있으므로 수강생 역할의 수신 화면에서 검증한다. 로컬 화면에 보이는 것만으로 합격 처리하지 않는다. 오버레이를 캡처에서 숨기는 content protection 옵션은 사용 목적에 맞지 않는다.

앱 창 공유까지 반드시 지원하려면 캡처·합성, OBS 등의 별도 출력 경로, 회의 앱 통합을 검토해야 한다. 이번 범위에 포함하지 않는다.

## 3. 라이선스와 법적 검토

다음은 개발 방향을 정하기 위한 일반 정보다. 특정 제품에 대한 비침해 법률 의견이나 개별 권리조사를 완료했다는 뜻은 아니다.

### 기능을 참고해 독립적으로 개발하기

한국 저작권법 제101조의2는 프로그램 언어·규약·해법을 프로그램 저작권 보호에서 제외한다. 미국 저작권청도 아이디어·방법·시스템과 구체적 표현을 구분한다. 이에 비추어 **화면 위에 그리기, 사용자가 지울 때까지 유지하기, 단축키로 지우기 같은 기능을 직접 구현하는 방향은 가능하다고 판단**한다. 타 앱의 유료 기능이라는 사실 자체가 독립 구현을 금지하는 근거는 아니다. [한국 저작권법 제101조의2 등, 시행 2026-08-11](https://law.go.kr/LSW/lsLawLinkInfo.do?chrClsCd=010202&lsJoLnkSeq=1017054983), [미국 저작권청](https://www.copyright.gov/help/faq/faq-protect.html)

다만 코드를 직접 작성했다는 사실만으로 모든 권리 문제가 사라지는 것은 아니다. 프로그램 코드·아이콘·설명·창작적 UI 표현을 그대로 복제하지 않고 독자적인 이름과 화면 구성을 만든다. 제3자 이미지·폰트·코드는 허용된 라이선스 범위에서 사용한다. 공개된 기능 설명과 정상 사용 경험에서 요구사항을 추출하는 방식으로 진행한다.

상표·특허·부정경쟁 문제는 저작권과 별개의 검토 대상이다. 특히 특허는 단순히 코드를 베꼈는지만으로 판단하지 않는다. 미국 특허청은 특허의 배타적 권리가 제작·사용·판매 등에도 미친다고 설명한다. 이번 조사로 관련 특허가 없거나 “My Brush”라는 이름이 사용 가능하다고 확정하지 않는다. 이름·로고를 공개하기 전 사용 예정 시장에서 확인하고, 특정 권리 충돌이 발견되면 그 쟁점에 한해 전문가 검토를 받는다. [USPTO](https://www.uspto.gov/patents/basics/essentials), [한국 부정경쟁방지법](https://law.go.kr/법령/부정경쟁방지및영업비밀보호에관한법률)

### 프로젝트 라이선스 선택

| 후보 | 사용·수정·상업 활용 | 재배포 시 핵심 조건 | 선택 기준 |
| --- | --- | --- | --- |
| MIT | 허용, 비공개 파생물·판매 가능 | 저작권·허가 고지 유지 | 원저작자 저작권 고지를 보존하는 간단한 선택지 |
| Apache-2.0 | 허용, 비공개 파생물·판매 가능 | 라이선스·관련 고지 유지, 변경 표시, 해당 시 NOTICE 처리 | 원본 출처 고지와 수정 사실 표시를 함께 관리하려는 현재 요구의 권장안 |
| GPL-3.0 | 상업 이용·판매 가능 | GPL 적용 저작물의 배포 시 해당 조건에 따른 소스 제공·라이선스 유지 | 배포되는 파생물도 자유 소프트웨어로 유지하길 원하는 경우 |

[MIT 원문](https://opensource.org/license/mit), [Apache-2.0 원문](https://www.apache.org/licenses/LICENSE-2.0.html), [GPLv3 원문](https://www.gnu.org/licenses/gpl.en.html)

준현님은 원저작자와 원본 출처가 본인임을 표시하도록 의무화하고 싶다고 답했다. 이를 **원저작자·출처 고지 보존 요구**로 반영하며, 수정본 소스 공개 의무까지 확정된 것으로 해석하지 않는다.

현재 권장안은 **Apache-2.0 + NOTICE**다. LICENSE는 표준 원문을 유지하고 NOTICE에 원저작자 저작권 고지와 원본 프로젝트 정보를 넣는다. 공개 저장소 URL과 저작권자 표시 이름은 정해진 뒤 기입한다. Apache-2.0 제4조에 따라 수정한 파일에는 변경 사실을 표시하고, 배포되는 파생물에는 관련 NOTICE 출처 고지의 읽을 수 있는 사본을 포함해야 한다. 고지 위치는 함께 배포하는 NOTICE, 소스·문서, 해당 고지를 표시하는 앱 화면 중 허용되는 방식으로 선택할 수 있다. **모든 파생 앱의 메인 화면이나 홍보물에 이름을 항상 노출하도록 강제하는 조건은 아니다.** [Apache-2.0 제4조](https://www.apache.org/licenses/LICENSE-2.0.html)

MIT도 저작권·허가 고지 보존을 요구하므로 원저작자 이름을 남기는 목적에 사용할 수 있다. 이번 권장안은 명시적인 NOTICE 처리와 변경 사실 표시 규정을 함께 활용하려는 선택이다. 두 라이선스 모두 수정본의 비공개·상업 배포를 허용한다. 원본 저작권 표시는 기여자가 새로 작성한 변경분의 저작권까지 준현님에게 자동 귀속시킨다는 뜻은 아니다. [MIT 원문](https://opensource.org/license/mit), [Apache-2.0 제4조](https://www.apache.org/licenses/LICENSE-2.0.html)

Apache-2.0의 특허 조항은 세상의 모든 제3자 특허로부터 보호한다는 뜻은 아니다. 비상업적 이용만 허용하는 제한은 OSI가 정의하는 오픈소스와 맞지 않는다. [OSI 정의](https://opensource.org/osd)

GitHub에 공개해 놓기만 하면 일반적인 이용·수정·배포 허가가 자동으로 생기는 것은 아니다. 선택한 LICENSE를 넣고 실행 파일 배포에도 의존성 고지를 포함한다. Tauri 자체의 MIT/Apache-2.0 이중 라이선스와 최종 앱의 모든 의존성 라이선스는 별개의 확인 항목이다. [GitHub 라이선스 안내](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository), [Tauri 저장소](https://github.com/tauri-apps/tauri)

배포 준비물 제안: LICENSE, 원본 프로젝트의 NOTICE, 의존성의 THIRD_PARTY_NOTICES, CONTRIBUTING, 개발·빌드 안내, 기능 데모, 지원 OS·제약 표, 오프라인 처리 설명. 자체 앱은 ‘정보 / 라이선스’ 화면에도 원저작자와 원본 저장소를 표시한다. 이 UI 선택을 표준 라이선스가 모든 파생물에 강제하는 의무와 혼동하지 않는다. 패키지와 사용한 소스·자산에 맞춰 실제 고지를 생성한다. 아직 의존성이 없으므로 라이선스 감사를 통과했다고 표시하지 않는다.

## 4. GitHub 다운로드 배포

소스 저장소의 Releases에 실행 파일을 첨부할 수 있다. 저장소에서 코드를 받는 것과 설치 파일을 받는 경로를 README에서 분리한다. 장래 산출물은 macOS DMG와 Windows 설치 EXE, 체크섬, 변경 내역, 라이선스 고지다. [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)

- macOS: Apple은 App Store 밖 직접 배포와 Developer ID 서명·공증 경로를 제공한다. 일반 사용자 배포의 기본안은 서명·공증된 DMG다. App Store 심사와 공증은 서로 다르며 공증 성공을 미리 보장하지 않는다. [Apple 배포 안내](https://developer.apple.com/macos/distribution/)
- Apple Developer Program은 조사일 기준 연 99 USD이며 지역별 현지 가격이 다를 수 있다. 적격 비영리·교육·정부 기관의 감면은 별도 요건이 있고 개인 오픈소스 프로젝트라는 이유로 자동 면제되지 않는다. [Apple 가입 안내](https://developer.apple.com/programs/enroll/)
- Windows: 코드 서명은 신원 확인과 배포 신뢰에 도움이 되지만 SmartScreen 경고 제거를 무조건 보장하지 않는다. SmartScreen은 파일·앱의 평판도 평가한다. 서명 공급자·비용은 배포 시점에 비교한다. [Microsoft SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/), [Tauri 서명 방법](https://v2.tauri.app/distribute/sign/windows/)
- 서명·공증이 준비되지 않은 초기 빌드는 실험용임을 표시하고 설치 검증 범위를 설명한다. 운영체제 보안 기능을 일괄 끄는 설치 절차는 기본 배포 안내로 삼지 않는다.
- 다운로드 서버 운영 없이 GitHub Releases를 사용할 수 있지만, 서명·실제 OS 검증 비용까지 없어지는 것은 아니다. 현재는 준현님의 요청대로 리모트 연결·push·공개 릴리스를 진행하지 않는다.

## 5. 조사 결론과 남은 확인

Tauri 2 + TypeScript Canvas를 기술 실험의 출발점으로 추천한다. macOS/Windows 네이티브 보완 경계를 처음부터 마련하고, 성능·전체화면·포커스·한글 입력을 두 OS에서 통과한 뒤 확정한다. 준현님은 Windows PC 직접 테스트가 가능하다고 답했다.

출처 표시 요구는 확정되었으며 Apache-2.0 + NOTICE가 현재 권장안이다. 최종 라이선스 적용, 저작권자 표시 이름·원본 저장소 URL, 최소 지원 OS·CPU 아키텍처, 실제 강의 앱 조합, 서명 비용 지출 여부는 후속 확정 항목이다. 이 항목들은 현재의 요구사항 정리와 검증 계획 작성을 막지 않는다.
