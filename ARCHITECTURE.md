# PeachLeaf - Project Architecture

## Overview

PeachLeaf는 Tauri 2.x, Svelte 5, Rust로 구축된 macOS 전용 마크다운 스티커 노트 애플리케이션입니다. 네이티브 데스크톱 환경에서 마크다운 편집 기능, 다중 노트 윈도우, 영구 상태 관리를 제공합니다.

## Technology Stack

### Frontend
- **Framework**: Svelte 5 (with new runes API)
- **Bundler**: Vite 5.4
- **Editor**: CodeMirror 6
- **Markdown Parser**: Marked 11.x
- **Language**: TypeScript

### Backend
- **Framework**: Tauri 2.9
- **Language**: Rust (Edition 2021)
- **Plugins**:
  - `tauri-plugin-clipboard-manager` 2.3.0
  - `tauri-plugin-dialog` 2.4.0

## Project Structure

```
peach-leaf/
├── .claude/                      # Claude Code 설정
│   ├── config.json               # 승인된 명령어
│   ├── instructions.md           # 프로젝트 지침 (한글 응답, 아키텍처 참조)
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
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri configuration
│   └── icons/                    # macOS app icons
│
├── dist/                         # Build output
│   └── renderer/                 # 프론트엔드 빌드 결과물
├── node_modules/                 # NPM dependencies
├── ARCHITECTURE.md               # 프로젝트 아키텍처 문서 (이 파일)
├── package.json                  # NPM configuration
├── tsconfig.json                 # TypeScript configuration
└── vite.config.mjs               # Vite configuration
```

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                         │
│                      (Svelte Components)                      │
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
│                     (Rust Backend)                            │
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
│                      File System                              │
│        ~/.peach-leaf/state.json                              │
│        ~/.peach-leaf/notes/*.md                              │
└─────────────────────────────────────────────────────────────┘
```

## Backend Module Responsibilities

### 1. **models.rs** (25 lines)
Data structure definitions for the application.

**Exports:**
- `StickerData`: Window state including position, size, colors, mode
- `AppState`: Application state containing all windows

### 2. **state.rs** (90 lines)
State persistence and file system management.

**Functions:**
- `get_state_file_path()`: Returns path to state.json
- `get_notes_dir()`: Returns path to notes directory
- `ensure_notes_dir()`: Creates notes directory if needed
- `save_app_state()`: Persists window state to JSON
- `load_app_state()`: Loads window state from JSON
- `cleanup_orphaned_notes()`: Removes notes not in state

**Storage Location:** `~/.peach-leaf/`
- `state.json`: Window positions, sizes, colors, modes
- `notes/`: Markdown files for each note

### 3. **commands/file.rs** (45 lines)
File I/O operations exposed to the frontend.

**Commands:**
- `read_file(file_path)`: Read markdown file
- `write_file(file_path, content)`: Write markdown file
- `delete_note_file(note_id)`: Delete markdown file
- `select_file()`: File dialog (placeholder)

### 4. **commands/window.rs** (198 lines)
Window state management and metadata storage.

**Global State:**
- `WINDOW_METADATA`: In-memory storage for window colors and modes

**Commands:**
- `save_window_state()`: Save all windows to state.json
- `get_saved_state()`: Retrieve saved application state
- `get_window_data(window_label)`: Get specific window data
- `update_window_metadata()`: Update window color/mode
- `create_sticker_window(sticker_data)`: Create new window

**Implementation:**
- `save_window_state_impl()`: Synchronous state saving

### 5. **commands/color.rs** (104 lines)
Color picker window management.

**Commands:**
- `open_color_picker(parent_label, current_color)`: Open picker
- `close_color_picker()`: Close picker
- `apply_color(parent_label, color)`: Apply selected color

**Features:**
- Positioned below macOS menu bar
- Modal behavior with always-on-top
- Window-specific color events

### 6. **menu.rs** (173 lines)
Application menu creation and event handling.

**Functions:**
- `create_menu()`: Build native menu bar
- `setup_menu_handler()`: Handle menu events with debouncing

**Menus:**
- **PeachLeaf**: Hide, Quit (macOS only)
- **File**: New Note (⌘N), Close Note (⌘W)
- **Edit**: Undo, Redo, Cut, Copy, Paste
- **Font**: Default, Small, Medium, Large, Extra Large
- **Color**: Choose Color...
- **Window**: Minimize, Zoom
- **Help**: About PeachLeaf

**Event Handling:**
- Debouncing (300ms) to prevent duplicate events
- Focus-aware event routing
- Backend-handled actions (new note, close note, quit)

### 7. **window_manager.rs** (181 lines)
Window lifecycle management.

**Functions:**
- `create_main_window()`: Create initial window
- `restore_window(sticker_data)`: Restore saved window
- `create_new_note_backend()`: Create new note window

**Features:**
- Multi-monitor support
- Monitor position tracking
- Window metadata population
- Automatic state saving after creation

### 8. **lib.rs** (94 lines)
Application entry point and module composition.

**Responsibilities:**
- Register Tauri commands
- Initialize plugins
- Setup menu system
- Restore saved windows on startup
- Handle window lifecycle events

**Event Handling:**
- Prevent exit on window close
- Auto-save state on window destroy

## Frontend Component Hierarchy

```
App.svelte
├── ColorPicker.svelte              (Color selection UI)
└── Sticker.svelte                  (Note window)
    ├── Toolbar.svelte              (Window controls)
    ├── MarkdownEditor.svelte       (Edit mode)
    └── MarkdownPreview.svelte      (Preview mode)
```

### Component Responsibilities

#### **App.svelte**
- Determines window type (color picker vs note)
- Loads saved window data
- Routes to appropriate component

#### **Sticker.svelte**
- Main note window logic
- Mode switching (edit/preview)
- File loading/saving via Tauri IPC (`invoke('read_file')`, `invoke('write_file')`)
- Drag-and-drop window movement
- Menu event handling
- Keyboard shortcuts (⌘M for mode toggle)

#### **MarkdownEditor.svelte**
- CodeMirror 6 integration
- Markdown syntax highlighting
- Auto-save on content change
- Undo/Redo support

#### **MarkdownPreview.svelte**
- Render markdown to HTML
- Custom styling
- Font size adjustment

#### **Toolbar.svelte**
- Mode toggle button
- Close button
- Drag handle

#### **ColorPicker.svelte**
- Predefined color palette
- Click-outside to close
- Apply color on selection

## Data Flow

### 1. Window Creation
```
Menu (⌘N)
  → menu.rs::handle_menu_event
  → window_manager.rs::create_new_note_backend
  → Create window + emit init-sticker event
  → App.svelte receives event
  → Sticker.svelte renders
```

### 2. Content Editing
```
User types in MarkdownEditor
  → handleContentChange event
  → Sticker.svelte::saveFile (debounced 500ms)
  → commands::write_file
  → state.rs::save_app_state
```

### 3. Color Change
```
Menu (Color → Choose Color...)
  → menu.rs → emit open_color_picker event
  → Sticker.svelte::openColorPicker
  → commands::open_color_picker
  → ColorPicker.svelte opens
  → User selects color
  → commands::apply_color
  → emit color-selected-{window_id}
  → Sticker.svelte updates backgroundColor
  → commands::update_window_metadata
  → commands::save_window_state
```

### 4. Window Close
```
Menu (⌘W) or Close Button
  → Sticker.svelte::handleClose
  → Check for content
  → Show confirmation dialog if needed
  → commands::delete_note_file
  → window.close()
  → RunEvent::WindowEvent::Destroyed
  → commands::save_window_state
```

### 5. Application Startup
```
main.rs::main
  → lib.rs::run
  → state::load_app_state
  → state::cleanup_orphaned_notes
  → window_manager::restore_window (for each saved window)
  → menu::create_menu
  → menu::setup_menu_handler
```

## State Management

### Backend State
- **In-Memory**: `WINDOW_METADATA` (HashMap of window colors/modes)
- **Persistent**: `~/.peach-leaf/state.json`

### State Structure
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

### Frontend State (Svelte Runes)
- `$state`: Reactive state variables
- `$effect`: Side effects on state changes
- `$props`: Component properties

## Key Features

### 1. Multi-Monitor Support
- Saves monitor name, position, and size
- Restores windows to correct monitor
- Falls back gracefully if monitor not found

### 2. Auto-Save
- Content saved 500ms after typing stops
- Window state saved on:
  - Window move/resize
  - Color change
  - Mode change
  - Window close

### 3. Orphan File Cleanup
- Runs on application startup
- Removes `.md` files not referenced in `state.json`
- Prevents disk space waste

### 4. Mode Switching
- **Edit Mode**: CodeMirror editor with syntax highlighting
- **Preview Mode**: Rendered markdown
- Toggle with ⌘M or toolbar button

### 5. Keyboard Shortcuts
- ⌘N: New Note
- ⌘W: Close Note
- ⌘M: Toggle Edit/Preview Mode
- ⌘Z: Undo
- ⌘⇧Z: Redo
- ⌘X/C/V: Cut/Copy/Paste

## Platform-Specific Behavior

### macOS Only
- Uses `macOSPrivateApi: true` for transparent windows
- Global menu bar integration
- Color picker positioned below menu bar (y=25px)
- Keyboard shortcuts follow macOS conventions

### Build Targets
- **macOS**: DMG, APP bundle
- **Windows**: Removed (not supported)
- **Linux**: Removed (not supported)

## Configuration Files

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

## Development Commands

```bash
# Development mode with hot reload
npm run dev

# Build frontend only
npm run build:renderer

# Build full application (frontend + backend)
npm run build

# Run Tauri commands
npm run tauri dev
npm run tauri build
```

## Module Dependencies

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

## Future Improvements

### Potential Enhancements
1. **Search Functionality**: Full-text search across all notes
2. **Tags/Categories**: Organize notes with tags
3. **Export**: Export to PDF, HTML
4. **Themes**: Dark mode, custom themes
5. **Sync**: Cloud sync via WebDAV, Dropbox, etc.
6. **Shortcuts**: Global hotkeys to show/hide notes
7. **Rich Text**: Tables, checkboxes, images
8. **Note Linking**: Wiki-style links between notes

### Code Quality
1. **Unit Tests**: Add tests for state management
2. **Integration Tests**: Test window lifecycle
3. **Error Handling**: More robust error messages
4. **Logging**: Structured logging with levels
5. **Performance**: Lazy loading for large notes

## License

MIT License

## Author

PeachLeaf Contributors
