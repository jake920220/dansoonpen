# Windows 설치 파일 첫 배포 검사

2026-09-21. 사용자의 실제 Windows PC 테스트를 위해 x64 NSIS 설치 파일을 준비했다. 실제 사용자 입력 검증과 CI 빌드 검증을 구분한다.

## v0.7.0에서 발견한 문제

[첫 Windows 실행](https://github.com/jake920220/dansoonpen/actions/runs/35571956637)에서 프런트엔드 59개 테스트는 통과했다. Rust는 52개 중 49개가 통과하고 진단 로그의 저장·회전 관련 3개가 실패했다.

진단 로그의 잠금 파일을 append-only로 열고 `File::lock()`을 호출하면 Windows에서 접근 거부가 발생했다. 쓰기 worker가 중단되어 진단 기록이 남지 않을 수 있었다. 읽기 권한을 함께 열도록 수정했다. [Rust File::lock의 Windows 제약](https://doc.rust-lang.org/std/fs/struct.File.html#method.lock)을 참고했다. 로그 내용·보관량·필기 동작은 바꾸지 않았다.

실패한 테스트를 제외하지 않고 같은 테스트로 재검사한다. 이미 공개된 v0.7.0 태그는 바꾸지 않으며 수정본을 v0.7.1로 구분한다.

## 검증 범위

- macOS: 수정 후 Rust 전체 54개·Clippy·fmt·Svelte 검사 통과, arm64 release 빌드 및 ZIP의 버전·라이선스 4종 일치 확인.
- Windows: [수정 후 CI](https://github.com/jake920220/dansoonpen/actions/runs/35572499600)에서 프런트엔드 검사·테스트, Rust 테스트·Clippy, NSIS 빌드, 무인 설치와 실행 파일·라이선스 리소스 확인을 수행한다.
- Windows 결과는 아래 완료 기록에 추가한다. CI의 설치 검사는 화면 위 필기·포커스·IME·화면공유 수신을 검증하지 않는다.
- 실행 파일은 코드 서명 인증서가 없는 테스트 배포본이다. macOS는 ad-hoc 서명이며 Apple 공증은 수행하지 않았다.

실제 PC 확인은 [Windows v0.7.1 체크리스트](windows-v0.7.1.md)를 따른다.

## 완료 기록

수정 후 Windows CI 전체 성공. 프런트엔드 59개, Rust 52개 테스트 통과, fmt·Svelte·Clippy 통과. Windows Server 2022 실행 환경에서 NSIS 설치 프로그램의 무인 설치 종료 코드 0과 실행 파일 버전 0.7.1, LICENSE·NOTICE·THIRD_PARTY_NOTICES.txt·Nanum Gothic OFL 설치를 확인했다.

- 앱 소스 커밋: `82466d7718daab5138b24a0723d7835537a6d989`.
- Windows 설치 파일: `DansoonPen-0.7.1-Windows-x64-setup.exe` (2,884,546 bytes), Authenticode `NotSigned`.
- Windows SHA-256: `760a01f65af204f5d6aa4e9eae733508a0946e07e6876ac86306e649b9b3f65c`.
- macOS ZIP SHA-256: `81038dfebe10507c125f723250dd04b3eed35a59a6e24b2aa6599f23783a2e86`.
- Actions 아티팩트를 내려받아 Windows 설치 파일의 SHA-256과 PE 형식을 확인했다.

Windows 10/11 실제 PC, 한글 IME, 투명 오버레이 입력, 여러 화면 배율, 화면공유 수신 결과는 아직 받지 않았다.
