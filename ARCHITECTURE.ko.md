# PeachLeaf - 프로젝트 아키텍처

## 개요

PeachLeaf는 Tauri 2.x, Svelte 5, Rust로 구축된 macOS 전용 마크다운 스티커 노트 애플리케이션입니다. 네이티브 데스크톱 환경에서 마크다운 편집 기능, 다중 노트 윈도우, 영구 상태 관리를 제공합니다.

## 기술 스택

### 프론트엔드
- **프레임워크**: Svelte 5 (runes API 사용)
- **번들러**: Vite 5.4
- **에디터**: CodeMirror 6
- **마크다운 파서**: Marked 11.x
- **언어**: TypeScript

### 백엔드
- **프레임워크**: Tauri 2.9
- **언어**: Rust (Edition 2021)
- **플러그인**:
  - `tauri-plugin-clipboard-manager` 2.3.0
  - `tauri-plugin-dialog` 2.4.0

## 프로젝트 구조

```
peach-leaf/
├── .claude/                      # Claude Code 설정
│   ├── config.json               # 승인된 명령어
│   ├── instructions.md           # 프로젝트 지침
│   └── settings.local.json       # 권한 설정
├── src/                          # 프론트엔드 소스
│   └── renderer/                 # 렌더러 프로세스 (Svelte UI)
│       ├── components/           # Svelte 컴포넌트
│       │   ├── Sticker.svelte        # 메인 스티커 윈도우 컴포넌트
│       │   ├── MarkdownEditor.svelte # CodeMirror 기반 에디터
│       │   ├── MarkdownPreview.svelte # 마크다운 프리뷰 렌더러
│       │   ├── Toolbar.svelte        # 윈도우 툴바
│       │   └── ColorPicker.svelte    # 색상 선택 UI
│       ├── lib/                  # 유틸리티 라이브러리
│       │   └── tauri.ts          # Tauri API 래퍼
│       ├── App.svelte            # 루트 컴포넌트
│       └── main.ts               # 진입점
├── src-tauri/                    # 백엔드 소스 (Rust)
│   ├── src/
│   │   ├── main.rs                   # Entry point (6 lines)
│   │   ├── lib.rs                    # App initialization & composition (94 lines)
│   │   ├── models.rs                 # Data structures (25 lines)
│   │   ├── state.rs                  # State management (90 lines)
│   │   ├── menu.rs                   # Menu system (173 lines)
│   │   ├── window_manager.rs         # Window lifecycle (181 lines)
│   │   └── commands/                 # Tauri commands (352 lines total)
│   │       ├── mod.rs                # Module exports (3 lines)
│   │       ├── file.rs               # File I/O (45 lines)
│   │       ├── window.rs             # Window state (198 lines)
│   │       └── color.rs              # Color picker (104 lines)
│   ├── Cargo.toml                # Rust 의존성
│   ├── tauri.conf.json           # Tauri 설정
│   └── icons/                    # macOS 앱 아이콘
│
├── dist/                         # 빌드 출력
│   └── renderer/                 # 프론트엔드 빌드 결과물
├── node_modules/                 # NPM 의존성
├── ARCHITECTURE.md               # 프로젝트 아키텍처 문서 (영문)
├── ARCHITECTURE.ko.md            # 프로젝트 아키텍처 문서 (한글)
├── package.json                  # NPM 설정
├── tsconfig.json                 # TypeScript 설정
└── vite.config.mjs               # Vite 설정
```

## 아키텍처 다이어그램

```
┌─────────────────────────────────────────────────────────────┐
│                        사용자 인터페이스                        │
│                      (Svelte 컴포넌트)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Sticker    │  │   Markdown   │  │    Color     │      │
│  │   Window     │  │    Editor    │  │    Picker    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Tauri IPC
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Commands                           │
│                     (Rust 백엔드)                             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   File I/O   │  │   Window     │  │    Color     │      │
│  │   Commands   │  │   Commands   │  │   Commands   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    State     │  │    Window    │  │     Menu     │      │
│  │  Management  │  │   Manager    │  │    System    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                        파일 시스템                             │
│        ~/.peach-leaf/state.json                              │
│        ~/.peach-leaf/notes/*.md                              │
│        ~/.peach-leaf/notes/images/{note-id}/*.png            │
└─────────────────────────────────────────────────────────────┘
```

## 백엔드 모듈 책임

### 1. **models.rs** (25줄)
애플리케이션의 데이터 구조 정의.

**내보내기:**
- `StickerData`: 위치, 크기, 색상, 모드를 포함한 윈도우 상태
- `AppState`: 모든 윈도우를 포함하는 애플리케이션 상태

### 2. **state.rs** (90줄)
상태 영속화 및 파일 시스템 관리.

**함수:**
- `get_state_file_path()`: state.json 경로 반환
- `get_notes_dir()`: 노트 디렉토리 경로 반환
- `ensure_notes_dir()`: 노트 디렉토리 생성 (필요시)
- `save_app_state()`: 윈도우 상태를 JSON에 저장
- `load_app_state()`: JSON에서 윈도우 상태 로드
- `cleanup_orphaned_notes()`: state에 없는 노트 제거

**저장 위치:** `~/.peach-leaf/`
- `state.json`: 윈도우 위치, 크기, 색상, 모드
- `notes/`: 각 노트의 마크다운 파일
- `notes/images/{note-id}/`: 각 노트의 이미지 파일 (PNG 형식)

### 3. **commands/file.rs** (45줄)
프론트엔드에 노출된 파일 I/O 작업.

**명령어:**
- `read_file(file_path)`: 마크다운 파일 읽기
- `write_file(file_path, content)`: 마크다운 파일 쓰기
- `delete_note_file(note_id)`: 마크다운 파일 삭제
- `select_file()`: 파일 대화상자 (플레이스홀더)

### 4. **commands/window.rs** (198줄)
윈도우 상태 관리 및 메타데이터 저장.

**전역 상태:**
- `WINDOW_METADATA`: 윈도우 색상 및 모드를 위한 인메모리 저장소

**명령어:**
- `save_window_state()`: 모든 윈도우를 state.json에 저장
- `get_saved_state()`: 저장된 애플리케이션 상태 가져오기
- `get_window_data(window_label)`: 특정 윈도우 데이터 가져오기
- `update_window_metadata()`: 윈도우 색상/모드 업데이트
- `create_sticker_window(sticker_data)`: 새 윈도우 생성

**구현:**
- `save_window_state_impl()`: 동기 상태 저장

### 5. **commands/color.rs** (104줄)
색상 선택기 윈도우 관리.

**명령어:**
- `open_color_picker(parent_label, current_color)`: 선택기 열기
- `close_color_picker()`: 선택기 닫기
- `apply_color(parent_label, color)`: 선택한 색상 적용

**기능:**
- macOS 메뉴 바 아래에 위치
- always-on-top 모달 동작
- 윈도우별 색상 이벤트

### 6. **menu.rs** (173줄)
애플리케이션 메뉴 생성 및 이벤트 처리.

**함수:**
- `create_menu()`: 네이티브 메뉴 바 구축
- `setup_menu_handler()`: 디바운싱을 사용한 메뉴 이벤트 처리

**메뉴:**
- **PeachLeaf**: 숨기기, 종료 (macOS 전용)
- **File**: 새 노트 (⌘N), 노트 닫기 (⌘W)
- **Edit**: 실행 취소, 다시 실행, 잘라내기, 복사, 붙여넣기
- **Font**: 기본, 작게, 보통, 크게, 매우 크게
- **Color**: 색상 선택...
- **Window**: 최소화, 확대/축소
- **Help**: PeachLeaf 정보

**이벤트 처리:**
- 중복 이벤트 방지를 위한 디바운싱 (300ms)
- 포커스 인식 이벤트 라우팅
- 백엔드 처리 액션 (새 노트, 노트 닫기, 종료)

### 7. **window_manager.rs** (181줄)
윈도우 생명주기 관리.

**함수:**
- `create_main_window()`: 초기 윈도우 생성
- `restore_window(sticker_data)`: 저장된 윈도우 복원
- `create_new_note_backend()`: 새 노트 윈도우 생성

**기능:**
- 멀티 모니터 지원
- 모니터 위치 추적
- 윈도우 메타데이터 채우기
- 생성 후 자동 상태 저장

### 8. **lib.rs** (94줄)
애플리케이션 진입점 및 모듈 구성.

**책임:**
- Tauri 명령어 등록
- 플러그인 초기화
- 메뉴 시스템 설정
- 시작 시 저장된 윈도우 복원
- 윈도우 생명주기 이벤트 처리

**이벤트 처리:**
- 윈도우 닫기 시 종료 방지
- 윈도우 파괴 시 상태 자동 저장

## 프론트엔드 컴포넌트 계층구조

```
App.svelte
├── ColorPicker.svelte              (색상 선택 UI)
└── Sticker.svelte                  (노트 윈도우)
    ├── Toolbar.svelte              (윈도우 컨트롤)
    ├── MarkdownEditor.svelte       (편집 모드)
    └── MarkdownPreview.svelte      (미리보기 모드)
```

### 컴포넌트 책임

#### **App.svelte**
- 윈도우 타입 결정 (색상 선택기 vs 노트)
- 저장된 윈도우 데이터 로드
- 적절한 컴포넌트로 라우팅

#### **Sticker.svelte**
- 메인 노트 윈도우 로직
- 모드 전환 (편집/미리보기)
- Tauri IPC를 통한 파일 로드/저장 (`invoke('read_file')`, `invoke('write_file')`)
- 드래그 앤 드롭 윈도우 이동
- 메뉴 이벤트 처리
- 키보드 단축키 (⌘M으로 모드 토글)

#### **MarkdownEditor.svelte**
- CodeMirror 6 통합
- 마크다운 문법 강조
- 콘텐츠 변경 시 자동 저장
- 실행 취소/다시 실행 지원
- **이미지 관리**:
  - 클립보드에서 이미지 붙여넣기 (PNG로 변환, `./images/{note-id}/`에 저장)
  - 드래그 앤 드롭 이미지 지원
  - 위젯 시스템을 사용한 인라인 이미지 미리보기
  - 8방향 핸들을 사용한 이미지 크기 조정
  - 이미지 선택 및 삭제 (Delete/Backspace)
  - 마크다운 주석을 통한 이미지 너비 영속화 (`<!-- width:XXX -->`)
  - 로드된 이미지를 위한 이미지 캐시 (data URL)
  - 스마트 삭제: 마크다운 참조 삭제 시 캐시에서 이미지 제거

#### **MarkdownPreview.svelte**
- 마크다운을 HTML로 렌더링
- 커스텀 스타일링
- 글꼴 크기 조정

#### **Toolbar.svelte**
- 모드 토글 버튼
- 닫기 버튼
- 드래그 핸들

#### **ColorPicker.svelte**
- 사전 정의된 색상 팔레트
- 외부 클릭으로 닫기
- 선택 시 색상 적용

## 데이터 흐름

### 1. 윈도우 생성
```
메뉴 (⌘N)
  → menu.rs::handle_menu_event
  → window_manager.rs::create_new_note_backend
  → 윈도우 생성 + init-sticker 이벤트 발생
  → App.svelte가 이벤트 수신
  → Sticker.svelte 렌더링
```

### 2. 콘텐츠 편집
```
사용자가 MarkdownEditor에서 입력
  → handleContentChange 이벤트
  → Sticker.svelte::saveFile (500ms 디바운스)
  → commands::write_file
  → state.rs::save_app_state
```

### 3. 색상 변경
```
메뉴 (Color → Choose Color...)
  → menu.rs → open_color_picker 이벤트 발생
  → Sticker.svelte::openColorPicker
  → commands::open_color_picker
  → ColorPicker.svelte 열림
  → 사용자가 색상 선택
  → commands::apply_color
  → color-selected-{window_id} 이벤트 발생
  → Sticker.svelte가 backgroundColor 업데이트
  → commands::update_window_metadata
  → commands::save_window_state
```

### 4. 윈도우 닫기
```
메뉴 (⌘W) 또는 닫기 버튼
  → Sticker.svelte::handleClose
  → 콘텐츠 확인
  → 필요시 확인 대화상자 표시
  → commands::delete_note_file
  → window.close()
  → RunEvent::WindowEvent::Destroyed
  → commands::save_window_state
```

### 5. 애플리케이션 시작
```
main.rs::main
  → lib.rs::run
  → state::load_app_state
  → state::cleanup_orphaned_notes
  → window_manager::restore_window (저장된 각 윈도우에 대해)
  → menu::create_menu
  → menu::setup_menu_handler
```

### 6. 이미지 붙여넣기/삽입
```
사용자가 이미지 붙여넣기 (⌘V)
  → MarkdownEditor.svelte::handleImagePaste
  → convertImageToBlob (클립보드)
  → saveImageFile (Tauri FS API)
  → 마크다운 삽입: ![image](./images/{note-id}/{timestamp}.png)
  → ImagePlugin이 위젯 생성
  → loadImageAsDataUrl (Tauri FS API)
  → imageCache가 data URL 저장
  → 인라인 이미지 미리보기 렌더링
```

### 7. 이미지 크기 조정
```
사용자가 크기 조정 핸들 드래그
  → ImageWidget 크기 조정 이벤트
  → 새로운 크기 계산 (종횡비 유지)
  → updateImageWidth(view, from, to, width)
  → 마크다운 업데이트: ![image](path)<!-- width:XXX -->
  → 새로운 범위로 selectedImagePosition 업데이트
  → ImagePlugin이 새로운 크기로 재구성
```

### 8. 이미지 삭제
```
사용자가 이미지를 선택하고 Delete/Backspace 누름
  → imageSelectionKeymap이 키 가로채기
  → 마크다운에서 이미지 src 추출
  → imageCache.delete(src)
  → 마크다운 범위 삭제 (width 주석 포함)
  → ImagePlugin 재구성
  → 이미지 파일은 디스크에 유지 (실행 취소 지원)

또는

사용자가 마크다운 텍스트를 수동으로 삭제
  → MarkdownEditor가 문서 변경 감지
  → ImagePlugin이 캐시된 이미지와 현재 문서 비교
  → 삭제된 이미지를 캐시에서 제거
  → 삭제된 이미지의 위젯은 렌더링되지 않음
```

## 상태 관리

### 백엔드 상태
- **인메모리**: `WINDOW_METADATA` (윈도우 색상/모드의 HashMap)
- **영구적**: `~/.peach-leaf/state.json`

### 상태 구조
```json
{
  "windows": [
    {
      "id": "note-1234567890",
      "file_path": "/Users/user/.peach-leaf/notes/note-1234567890.md",
      "x": 150,
      "y": 150,
      "width": 400,
      "height": 300,
      "background_color": "#FEFCE8",
      "text_color": "#333333",
      "mode": "edit",
      "monitor_name": "Built-in Retina Display",
      "monitor_position": [0, 0],
      "monitor_size": [3024, 1964]
    }
  ]
}
```

### 프론트엔드 상태 (Svelte Runes)
- `$state`: 반응형 상태 변수
- `$effect`: 상태 변경에 대한 부수 효과
- `$props`: 컴포넌트 속성

### MarkdownEditor 이미지 상태
- **imageCache**: `Map<string, string>` - 이미지 경로를 data URL로 매핑
- **selectedImageElement**: `HTMLElement | null` - 현재 선택된 이미지 DOM 요소
- **selectedImagePosition**: `{from: number, to: number} | null` - 선택된 이미지 마크다운 범위
- **ImagePlugin decorations**: 이미지를 인라인으로 렌더링하기 위한 동적 위젯
- **이미지 위젯 생명주기**:
  1. `![alt](path)` 또는 `![alt](path)<!-- width:XXX -->`에 대한 마크다운 파싱
  2. Tauri FS API를 통해 이미지 로드
  3. imageCache에 data URL 캐시
  4. ImageWidget으로 Decoration.replace() 생성
  5. 선택 시 크기 조정 핸들과 함께 img 요소 렌더링

## 이미지 시스템 아키텍처

### 개요
MarkdownEditor.svelte는 CodeMirror 6의 확장 API를 사용하여 포괄적인 이미지 관리 시스템을 구현합니다. 이미지는 파일로 저장되고 마크다운에서 참조되며, 인라인 미리보기는 커스텀 위젯으로 렌더링됩니다.

### 핵심 구성 요소

#### 1. **ImageWidget (WidgetType)**
이미지 미리보기를 인라인으로 렌더링하는 커스텀 CodeMirror 위젯.

**기능:**
- Tauri FS API를 통해 이미지를 로드하고 data URL로 변환
- 컨테이너 래퍼와 함께 `<img>` 요소 렌더링
- 마크다운 주석을 통한 너비 지정 지원
- 크기 조정 시 종횡비 유지
- CSS 클래스로 시각적 선택 상태 추가

**생명주기:**
```typescript
constructor(src, alt, width) → toDOM() → eq() → destroy()
```

#### 2. **ImagePlugin (ViewPlugin)**
이미지 데코레이션 생명주기 및 캐시 동기화 관리.

**책임:**
- 이미지 문법에 대한 마크다운 파싱: `![alt](path)` 및 `![alt](path)<!-- width:XXX -->`
- 각 이미지에 대해 `Decoration.replace()` 생성
- 이미지 추가/제거를 감지하기 위한 캐시 크기 추적
- 문서 또는 캐시 변경 시에만 데코레이션 재구성
- 에디터 뷰에 데코레이션 제공

**최적화:**
- 불필요한 재구성을 피하기 위해 마지막 캐시 크기 캐싱
- `docChanged` 또는 캐시 크기 변경 시에만 재구성 트리거

#### 3. **이미지 캐시**
로드된 이미지를 data URL로 저장하는 `Map<string, string>`.

**목적:**
- 반복적인 파일 시스템 읽기 방지
- 즉각적인 위젯 렌더링 가능
- 문서 상태와 동기화
- 삭제된 이미지 감지

**캐시 작업:**
- `imageCache.set(src, dataUrl)`: 로드된 이미지 저장
- `imageCache.get(src)`: 캐시된 이미지 검색
- `imageCache.delete(src)`: 삭제 시 제거
- `imageCache.size`: 변경 감지를 위한 추적

#### 4. **선택 시스템**
크기 조정 및 삭제 작업을 위한 선택된 이미지 추적.

**상태:**
- `selectedImageElement`: DOM 요소 참조
- `selectedImagePosition`: `{from, to}` 마크다운 범위

**동작:**
- 이미지 클릭 → 선택, 크기 조정 핸들 추가, 커서 숨김
- 외부 클릭 → 선택 해제, 핸들 제거
- 블러 없이 커서 위치 지정 (키보드 이벤트 유지)

#### 5. **크기 조정 시스템**
종횡비 보존과 함께 8방향 크기 조정 핸들.

**핸들 위치:**
- 모서리: `nw, ne, sw, se` (대각선 크기 조정)
- 가장자리: `n, s, e, w` (방향 크기 조정)

**크기 조정 흐름:**
1. 사용자가 핸들 드래그
2. 원래 위치에서 델타 계산
3. 새로운 너비 계산 (종횡비 유지)
4. width 주석으로 마크다운 업데이트
5. 새로운 범위로 `selectedImagePosition` 업데이트 (주석 포함)
6. 플러그인이 새로운 크기로 위젯 재구성

#### 6. **삭제 시스템**
선택 기반 및 커서 기반의 두 가지 삭제 메커니즘.

**선택 기반 삭제:**
- 높은 우선순위의 커스텀 keymap (`defaultKeymap` 이전)
- 이미지 선택 시 Delete/Backspace 가로채기
- width 주석을 포함한 전체 범위 삭제
- 캐시에서 제거, 플러그인 재구성 트리거

**커서 기반 삭제:**
- 정규식 패턴이 커서 위치의 이미지 마크다운 감지
- Delete: 이미지 이전 커서 → 앞으로 매칭
- Backspace: 이미지 이후 커서 → 뒤로 매칭
- 정규식에서 선택적 width 주석 처리

**정규식 패턴:**
```typescript
// 앞으로 (Delete)
/^!\[([^\]]*)\]\((\.\/[^)]+)\)(?:<!--\s*width:(\d+)\s*-->)?/

// 뒤로 (Backspace)
/!\[([^\]]*)\]\((\.\/[^)]+)\)(?:<!--\s*width:(\d+)\s*-->)?$/
```

#### 7. **붙여넣기 시스템**
파일 변환이 포함된 클립보드 이미지 처리.

**흐름:**
1. ⌘V keydown 이벤트 가로채기
2. 클립보드 항목 읽기
3. 첫 번째 이미지 타입 항목 찾기
4. blob을 PNG로 변환
5. 타임스탬프로 고유한 파일명 생성
6. 이미지 디렉토리 생성: `./images/{note-id}/`
7. Tauri FS API를 통해 파일 저장
8. 현재 줄 끝에 마크다운 삽입
9. 플러그인이 자동으로 로드하고 렌더링

**파일 명명:**
```
image-{timestamp}.png
```

**디렉토리 구조:**
```
~/.peach-leaf/notes/
  note-1234567890.md
  images/
    note-1234567890/
      image-1234567891.png
      image-1234567892.png
```

### 마크다운 형식

#### 기본 이미지
```markdown
![image](./images/note-1234567890/image-1234567891.png)
```

#### 너비가 있는 이미지
```markdown
![image](./images/note-1234567890/image-1234567891.png)<!-- width:600 -->
```

### 성능 고려사항

- **지연 로딩**: FS API를 통해 필요 시 이미지 로드
- **캐싱**: 반복 읽기를 피하기 위해 data URL 캐싱
- **재구성 조절**: 실제 변경 시에만 플러그인 재구성
- **위젯 동등성**: `eq()` 메서드가 불필요한 재렌더링 방지

### 브라우저 호환성

- 이미지 붙여넣기를 위해 `navigator.clipboard.read()` 사용
- `clipboard-read` 권한 필요 (Tauri에서 부여)
- 이미지 렌더링을 위한 data URL (광범위하게 지원됨)

## 주요 기능

### 1. 멀티 모니터 지원
- 모니터 이름, 위치, 크기 저장
- 올바른 모니터로 윈도우 복원
- 모니터를 찾을 수 없을 경우 우아하게 폴백

### 2. 자동 저장
- 입력 중지 500ms 후 콘텐츠 저장
- 윈도우 상태 저장:
  - 윈도우 이동/크기 조정
  - 색상 변경
  - 모드 변경
  - 윈도우 닫기

### 3. 고아 파일 정리
- 애플리케이션 시작 시 실행
- `state.json`에서 참조되지 않는 `.md` 파일 제거
- 디스크 공간 낭비 방지

### 4. 모드 전환
- **편집 모드**: 문법 강조가 있는 CodeMirror 에디터
- **미리보기 모드**: 렌더링된 마크다운
- ⌘M 또는 툴바 버튼으로 토글

### 5. 키보드 단축키
- ⌘N: 새 노트
- ⌘W: 노트 닫기
- ⌘M: 편집/미리보기 모드 토글
- ⌘Z: 실행 취소
- ⌘⇧Z: 다시 실행
- ⌘X/C/V: 잘라내기/복사/붙여넣기
- ⌘V: 클립보드에서 이미지 붙여넣기 (에디터에서)
- Delete/Backspace: 선택된 이미지 삭제

## 플랫폼별 동작

### macOS 전용
- 투명 윈도우를 위해 `macOSPrivateApi: true` 사용
- 전역 메뉴 바 통합
- 메뉴 바 아래에 색상 선택기 위치 (y=25px)
- 키보드 단축키가 macOS 규칙을 따름

### 빌드 타겟
- **macOS**: DMG, APP 번들
- **Windows**: 지원되지 않음
- **Linux**: 지원되지 않음

## 설정 파일

### tauri.conf.json
```json
{
  "identifier": "com.peachleaf.app",
  "bundle": {
    "targets": ["dmg", "app"],
    "icon": ["icons/*.png", "icons/icon.icns"]
  },
  "app": {
    "macOSPrivateApi": true
  }
}
```

### package.json
```json
{
  "name": "peach-leaf",
  "version": "1.0.0",
  "description": "macOS markdown sticky notes app built with Tauri",
  "keywords": ["markdown", "sticky-notes", "tauri", "macos"],
  "scripts": {
    "dev": "tauri dev",
    "dev:renderer": "vite",
    "build": "tauri build",
    "build:renderer": "vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@codemirror/lang-markdown": "^6.2.4",
    "@codemirror/state": "^6.5.2",
    "@codemirror/theme-one-dark": "^6.1.2",
    "@codemirror/view": "^6.38.6",
    "@tauri-apps/api": "^2.9.0",
    "@tauri-apps/plugin-clipboard-manager": "^2.3.0",
    "@tauri-apps/plugin-dialog": "^2.4.0",
    "codemirror": "^6.0.1",
    "marked": "^11.1.1"
  }
}
```

## 개발 명령어

```bash
# 핫 리로드가 있는 개발 모드
npm run dev

# 프론트엔드만 빌드
npm run build:renderer

# 전체 애플리케이션 빌드 (프론트엔드 + 백엔드)
npm run build

# Tauri 명령어 실행
npm run tauri dev
npm run tauri build
```

## 모듈 의존성

```
lib.rs
  ├── models
  ├── state
  ├── commands
  │   ├── file
  │   ├── window (depends on: state, models)
  │   └── color
  ├── menu (depends on: window_manager)
  └── window_manager (depends on: state, commands::window)
```

## 향후 개선사항

### 잠재적 개선사항
1. **검색 기능**: 모든 노트에 대한 전체 텍스트 검색
2. **태그/카테고리**: 태그로 노트 정리
3. **내보내기**: PDF, HTML로 내보내기
4. **테마**: 다크 모드, 커스텀 테마
5. **동기화**: WebDAV, Dropbox 등을 통한 클라우드 동기화
6. **단축키**: 노트 표시/숨기기를 위한 전역 핫키
7. **리치 텍스트**: 테이블, 체크박스
8. **노트 링킹**: 노트 간 위키 스타일 링크
9. **이미지 개선사항**:
   - GIF, JPEG, WebP 형식 지원 (현재 PNG만 지원)
   - 이미지 압축 옵션
   - 배치 이미지 작업
   - 이미지 캡션 편집

### 코드 품질
1. **단위 테스트**: 상태 관리에 대한 테스트 추가
2. **통합 테스트**: 윈도우 생명주기 테스트
3. **에러 처리**: 더 강력한 에러 메시지
4. **로깅**: 레벨이 있는 구조화된 로깅
5. **성능**: 큰 노트에 대한 지연 로딩

## 라이선스

MIT License

## 저자

PeachLeaf Contributors
