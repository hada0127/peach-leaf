# PeachLeaf

A beautiful, lightweight markdown sticky notes application for macOS.

![PeachLeaf](https://img.shields.io/badge/platform-macOS-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue)

## Overview

PeachLeaf is a native macOS application that brings the simplicity of sticky notes with the power of markdown. Create multiple floating note windows, customize their appearance, and organize your thoughts with markdown formatting and inline images.

## Features

### 📝 Markdown Editing
- Full markdown syntax support with live preview
- Syntax highlighting with CodeMirror 6
- Switch between edit and preview modes
- Auto-save with 500ms debounce

### 🖼️ Image Support
- Paste images directly from clipboard (⌘V)
- Drag and drop images into notes
- Inline image preview while editing
- Resize images with 8-directional handles
- Maintain aspect ratio during resize
- Images stored locally with your notes

### 🎨 Customization
- Choose from predefined color palettes
- Separate background and text colors
- Persistent color settings per note
- Font size adjustment (5 levels)

### 🪟 Window Management
- Create multiple independent note windows
- Drag windows anywhere on screen
- Resize notes to your preference
- Multi-monitor support with position persistence
- Auto-restore windows on app launch

### ⌨️ Keyboard Shortcuts
- `⌘N` - Create new note
- `⌘W` - Close current note
- `⌘M` - Toggle edit/preview mode
- `⌘V` - Paste image from clipboard
- `⌘Z` / `⌘⇧Z` - Undo / Redo
- `Delete` / `Backspace` - Delete selected image

## Installation

### Download
Download the latest release from the [Releases](https://github.com/yourusername/peach-leaf/releases) page.

### From Source

**Prerequisites:**
- Node.js 18+ and npm
- Rust 1.70+
- Xcode Command Line Tools (macOS)

**Install and Run:**

```bash
# Clone the repository
git clone https://github.com/yourusername/peach-leaf.git
cd peach-leaf

# Install dependencies
npm install

# Run in development mode
npm run dev

# Build for production
npm run build
```

## Usage

### Creating Notes

1. **First Launch**: A default note window will appear
2. **New Note**: Press `⌘N` or select `File → New Note` from the menu
3. **Start Typing**: Click anywhere in the note to start editing

### Working with Images

#### Adding Images
- **Paste**: Copy an image and press `⌘V` in the editor
- **Drag & Drop**: Drag image files directly into the note

#### Resizing Images
1. Click on an image to select it
2. Drag any of the 8 resize handles (corners or edges)
3. The image will maintain its aspect ratio

#### Deleting Images
- Click on an image to select it
- Press `Delete` or `Backspace`

### Customizing Colors

1. Select `Color → Choose Color...` from the menu
2. Click on a color palette
3. The color picker will close automatically

### Switching Modes

- **Edit Mode**: Full markdown editing with syntax highlighting
- **Preview Mode**: Rendered markdown view
- Toggle: Press `⌘M` or click the mode button in toolbar

### Font Sizes

Select `Font` from the menu and choose:
- Default (16px)
- Small (14px)
- Medium (18px)
- Large (20px)
- Extra Large (24px)

## File Storage

All data is stored locally in your home directory:

```
~/.peach-leaf/
├── state.json                    # Window positions and settings
└── notes/
    ├── note-1234567890.md        # Note content
    └── images/
        └── note-1234567890/      # Images for this note
            ├── image-1234567891.png
            └── image-1234567892.png
```

### Markdown Format

Notes are stored as plain markdown files with optional width comments for images:

```markdown
# My Note

Some text here.

![image](./images/note-1234567890/image-1234567891.png)<!-- width:600 -->

More text.
```

## Building from Source

### Development

```bash
# Run with hot reload
npm run dev

# Build frontend only
npm run build:renderer
```

### Production Build

```bash
# Build the complete application
npm run build

# The output will be in src-tauri/target/release/bundle/
```

## Troubleshooting

### Notes Don't Restore on Launch
- Check `~/.peach-leaf/state.json` exists
- Verify file permissions
- Try restarting the app

### Images Not Displaying
- Ensure images are in `~/.peach-leaf/notes/images/`
- Check markdown syntax: `![alt](./images/note-id/image.png)`
- Verify image files exist on disk

### App Won't Launch
- Check macOS version (requires macOS 10.15+)
- Verify app is not in quarantine: `xattr -d com.apple.quarantine /path/to/PeachLeaf.app`

## Technology Stack

- **Frontend**: Svelte 5, TypeScript, Vite
- **Editor**: CodeMirror 6
- **Backend**: Tauri 2.9, Rust
- **Markdown**: Marked 11.x
- **Styling**: CSS with Svelte scoped styles

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/)
- Markdown editing powered by [CodeMirror](https://codemirror.net/)
- Markdown rendering by [Marked](https://marked.js.org/)

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/yourusername/peach-leaf/issues).

---

Made with ❤️ for macOS
