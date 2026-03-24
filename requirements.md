# git-ghosts — 도깨비 유령 탐색기

Git 저장소에서 "유령"을 찾아내는 CLI 도구. 삭제됐지만 흔적이 남은 코드, 좀비 브랜치, 고아 커밋을 탐지하고 시각화합니다.

## 기능 요구사항

1. **Ghost Code Detection**: `git log --diff-filter=D`로 삭제된 파일 목록 추출. 삭제 시점, 삭제한 커밋, 원래 크기 표시
2. **Zombie Branch Detection**: 마지막 커밋이 N일 이상 지난 브랜치를 "좀비"로 분류. 기본 30일, 설정 가능
3. **Orphan Commit Detection**: `git fsck --unreachable`로 어디에도 연결 안 된 고아 커밋 탐지
4. **Ghost Summary Report**: 터미널에 컬러 테이블로 유령 요약 출력 (삭제 파일 수, 좀비 브랜치 수, 고아 커밋 수)
5. **CLI**: `git-ghosts scan` (전체 스캔), `git-ghosts report` (요약), `git-ghosts clean --dry-run` (좀비 브랜치 정리 프리뷰)

## 비기능 요구사항

- Rust 2021 edition, clap CLI
- git2 crate for git operations (libgit2 바인딩)
- colored/termcolor for terminal output
- 대형 저장소(10만 커밋)에서도 10초 이내 스캔
