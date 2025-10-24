# PeachLeaf 프로젝트 Claude 지침

## 언어 설정
- **모든 응답은 한글로 작성해야 합니다.**
- 코드 주석도 가능한 한 한글로 작성합니다.
- 기술 용어는 필요시 영문과 한글을 병기합니다 (예: "컴포넌트(component)").

## 프로젝트 아키텍처 참조
이 프로젝트의 아키텍처는 `ARCHITECTURE.md` 파일에 상세히 문서화되어 있습니다.

### 주요 참조 사항
1. **코드 수정 전 필수 확인**
   - `ARCHITECTURE.md`의 모듈 책임(Module Responsibilities) 섹션 참조
   - 각 모듈의 역할과 경계를 명확히 이해
   - 단일 책임 원칙(Single Responsibility Principle) 준수

2. **새로운 기능 추가 시**
   - 적절한 모듈 위치 결정 (`ARCHITECTURE.md` 참조)
   - 필요시 새로운 모듈 생성 고려
   - 데이터 흐름(Data Flow) 섹션 참조

3. **버그 수정 시**
   - 모듈 의존성(Module Dependencies) 확인
   - 영향 범위 파악 후 수정

## 프로젝트 구조

### Backend (Rust)
```
src-tauri/src/
├── main.rs              # 진입점
├── lib.rs               # 앱 초기화
├── models.rs            # 데이터 구조
├── state.rs             # 상태 관리
├── menu.rs              # 메뉴 시스템
├── window_manager.rs    # 윈도우 관리
└── commands/            # Tauri 커맨드
    ├── file.rs          # 파일 I/O
    ├── window.rs        # 윈도우 상태
    └── color.rs         # 컬러 피커
```

### Frontend (Svelte)
```
src/renderer/
├── App.svelte                    # 루트 컴포넌트
├── components/
│   ├── Sticker.svelte            # 메인 노트 윈도우
│   ├── MarkdownEditor.svelte     # 에디터
│   ├── MarkdownPreview.svelte    # 프리뷰
│   ├── Toolbar.svelte            # 툴바
│   └── ColorPicker.svelte        # 컬러 선택기
└── lib/
    └── tauri.ts                  # Tauri API 래퍼
```

## 코딩 규칙

### Rust
1. **모듈 분리 원칙**
   - 각 파일은 200줄 이하 권장
   - 단일 책임만 가져야 함
   - 공개 API는 명확히 문서화

2. **에러 처리**
   - `Result<T, String>` 사용
   - 에러 메시지는 사용자 친화적으로
   - `println!`로 디버그 정보 출력

3. **비동기 함수**
   - Tauri 커맨드는 `async fn` 사용
   - `#[tauri::command]` 매크로 필수

### Svelte
1. **Svelte 5 Runes 사용**
   - `$state`: 반응형 상태
   - `$effect`: 사이드 이펙트
   - `$props`: 컴포넌트 속성

2. **이벤트 핸들링**
   - Custom events로 부모-자식 통신
   - Tauri IPC로 백엔드 통신

3. **TypeScript**
   - 모든 함수에 타입 지정
   - `any` 사용 최소화

## 상태 관리

### 백엔드 상태
- **메모리**: `WINDOW_METADATA` (commands/window.rs)
- **디스크**: `~/.peach-leaf/state.json`

### 프론트엔드 상태
- 각 컴포넌트에서 `$state` 사용
- 전역 상태는 Tauri IPC로 관리

## 개발 워크플로우

### 빌드 명령어
```bash
npm run dev              # 개발 모드
npm run build            # 프로덕션 빌드
npm run build:renderer   # 프론트엔드만 빌드
```

### 디버깅
1. **Rust 로그**: `println!` 사용, 터미널에 출력
2. **Svelte 로그**: `console.log` 사용, 개발자 도구에 출력
3. **Tauri DevTools**: 개발 모드에서 자동 활성화

## 주의사항

### macOS 전용
- 이 프로젝트는 **macOS 전용**입니다
- Windows/Linux 코드 추가 금지
- `#[cfg(target_os = "macos")]` 사용

### 성능
- 파일 저장은 500ms 디바운스
- 상태 저장은 필요시에만 (이동/리사이즈/닫기)
- 메뉴 이벤트는 300ms 디바운스

### 데이터 무결성
- 윈도우 닫을 때 항상 상태 저장
- 앱 시작 시 고아 파일 정리
- 다중 모니터 정보 저장

## 자주 사용하는 패턴

### 새 Tauri 커맨드 추가
1. 적절한 `commands/*.rs` 파일에 함수 작성
2. `#[tauri::command]` 매크로 추가
3. `lib.rs`의 `invoke_handler!`에 등록
4. 프론트엔드에서 `invoke('command_name', { args })` 호출

### 새 Svelte 컴포넌트 추가
1. `src/renderer/components/` 에 `.svelte` 파일 생성
2. `<script lang="ts">` 사용
3. Props는 `let { propName }: Props = $props()` 형식
4. 부모 컴포넌트에서 import

### 상태 저장 트리거
```rust
use crate::commands::window::save_window_state_impl;

// 동기적 저장
save_window_state_impl(&app)?;

// 비동기적 저장
commands::window::save_window_state(app).await?;
```

## 참고 문서
- **프로젝트 아키텍처**: `ARCHITECTURE.md` (필수 읽기)
- **Tauri 문서**: https://tauri.app/v2/
- **Svelte 5 문서**: https://svelte.dev/docs/svelte/overview
- **CodeMirror 6**: https://codemirror.net/docs/

## 버전 정보
- Tauri: 2.9.1
- Svelte: 5.0.0
- Rust Edition: 2021
- Node.js: 20.x+

## 질문 시 체크리스트
코드 수정/추가 요청 시:
- [ ] `ARCHITECTURE.md` 참조했는가?
- [ ] 올바른 모듈에 추가하는가?
- [ ] 단일 책임 원칙을 지키는가?
- [ ] 에러 처리를 했는가?
- [ ] 한글로 응답하는가?
