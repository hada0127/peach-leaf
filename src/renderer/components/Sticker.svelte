<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import Toolbar from './Toolbar.svelte';
  import type { EditorView } from '@codemirror/view';

  interface Props {
    data: {
      id: string;
      filePath: string;
      backgroundColor: string;
      textColor: string;
      mode: 'edit' | 'preview';
    };
  }

  let { data }: Props = $props();

  let content = $state('');
  let mode = $state<'edit' | 'preview'>(data.mode);
  let backgroundColor = $state(data.backgroundColor);
  let textColor = $state(data.textColor);
  let fontSize = $state(11); // Default font size
  let isDragging = $state(false);
  let dragStartX = $state(0);
  let dragStartY = $state(0);
  let windowX = $state(0);
  let windowY = $state(0);
  let scaleFactor = $state(1);
  let editorView: EditorView | null = null;
  let saveTimeout: number | null = null;

  async function loadFile() {
    if (!window.electron) {
      // For testing without Electron
      content = '# Test Markdown\n\nThis is a test note.\n\n- Item 1\n- Item 2\n\n**Bold text** and *italic text*.';
      return;
    }

    const result = await window.electron.readFile(data.filePath);
    if (result.success && result.content !== undefined) {
      content = result.content;
    }
  }

  async function saveFile() {
    if (!window.electron) {
      console.log('Would save:', content);
      return;
    }

    if (saveTimeout) {
      clearTimeout(saveTimeout);
    }

    saveTimeout = window.setTimeout(async () => {
      await window.electron.writeFile(data.filePath, content);
      saveTimeout = null;
    }, 500);
  }

  function handleContentChange(event: CustomEvent<string>) {
    content = event.detail;
    saveFile();
  }

  function toggleMode() {
    console.log('toggleMode called, current mode:', mode);

    // 편집 모드와 미리보기 모드만 토글
    mode = mode === 'edit' ? 'preview' : 'edit';

    console.log('Mode changed to:', mode);

    if (window.electron) {
      window.electron.updateStickerConfig(data.id, { mode });
    }
  }

  function handleColorChange(event: CustomEvent<{ bg: string; text: string }>) {
    backgroundColor = event.detail.bg;
    textColor = event.detail.text;
    if (window.electron) {
      window.electron.updateStickerConfig(data.id, {
        backgroundColor,
        textColor,
      });
    }
  }

  async function handleClose() {
    try {
      // Check if there's content in the note
      const trimmedContent = content.trim();

      // If there's content, show confirmation dialog
      if (trimmedContent.length > 0) {
        const { confirm } = await import('@tauri-apps/plugin-dialog');
        const shouldClose = await confirm('This note has content. Are you sure you want to close it?', {
          title: 'Close Note',
          kind: 'warning'
        });

        if (!shouldClose) {
          return; // User cancelled, don't close
        }
      }

      // Close the window
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      await currentWindow.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  }

  async function startDrag(e: MouseEvent) {
    // 버튼 클릭은 드래그로 처리하지 않음
    const isButton = (e.target as HTMLElement).closest('button');
    if (isButton) {
      return;
    }

    const toolbar = (e.target as HTMLElement).closest('.toolbar');
    if (toolbar) {
      // 현재 윈도우 위치 가져오기
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      const position = await currentWindow.outerPosition();

      // Retina 디스플레이를 위한 scale factor 가져오기
      scaleFactor = await currentWindow.scaleFactor();
      windowX = position.x;
      windowY = position.y;

      isDragging = true;
      dragStartX = e.screenX;
      dragStartY = e.screenY;
      e.preventDefault();
    }
  }

  function handleDrag(e: MouseEvent) {
    if (isDragging && e.screenX !== 0 && e.screenY !== 0) {
      // screenX/screenY는 논리적 픽셀이므로 scaleFactor를 곱해 물리적 픽셀로 변환
      const deltaX = (e.screenX - dragStartX) * scaleFactor;
      const deltaY = (e.screenY - dragStartY) * scaleFactor;

      windowX += deltaX;
      windowY += deltaY;

      dragStartX = e.screenX;
      dragStartY = e.screenY;

      // 비동기로 윈도우 위치 업데이트 (await 하지 않아 부드러운 드래그)
      updateWindowPosition(windowX, windowY);
    }
  }

  async function updateWindowPosition(x: number, y: number) {
    try {
      const { getCurrentWindow, PhysicalPosition } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      const newPosition = new PhysicalPosition(Math.round(x), Math.round(y));
      await currentWindow.setPosition(newPosition);
    } catch (error) {
      console.error('Failed to update window position:', error);
    }
  }

  function stopDrag() {
    isDragging = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    // Cmd+M: 모드 전환 - 다른 키는 모두 통과시킴
    if ((e.metaKey || e.ctrlKey) && e.key === 'm') {
      e.preventDefault();
      toggleMode();
    }
    // 다른 모든 키는 기본 동작 허용 (CodeMirror가 처리)
  }

  // 컬러 피커 열기
  async function openColorPicker() {
    console.log('openColorPicker called');
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { getCurrentWindow } = await import('@tauri-apps/api/window');

      const currentWindow = getCurrentWindow();
      const windowLabel = currentWindow.label;

      console.log('Opening color picker for window:', windowLabel, 'with color:', backgroundColor);

      await invoke('open_color_picker', {
        parentLabel: windowLabel,
        currentColor: backgroundColor
      });

      console.log('Color picker opened successfully');
    } catch (error) {
      console.error('Failed to open color picker:', error);
      alert('Failed to open color picker: ' + error);
    }
  }

  // 새 노트 생성
  async function createNewNote() {
    try {
      // Tauri API import
      const { invoke } = await import('@tauri-apps/api/core');

      // 랜덤 ID와 임시 파일 경로 생성
      const newId = `note-${Date.now()}`;
      const tempFilePath = `/tmp/${newId}.md`;

      // 새 창 위치를 기존 창들과 겹치지 않게 랜덤 오프셋 추가
      const randomOffset = Math.floor(Math.random() * 100) + 50;

      // 새 스티커 데이터
      const newStickerData = {
        id: newId,
        file_path: tempFilePath,  // snake_case for Rust
        x: 150 + randomOffset,
        y: 150 + randomOffset,
        width: 400,
        height: 300,
        background_color: '#FEFCE8',  // snake_case for Rust
        text_color: '#333333',        // snake_case for Rust
        mode: 'edit',
      };

      // 빈 파일 생성
      await invoke('write_file', {
        filePath: tempFilePath,  // camelCase for Tauri auto-conversion
        content: '# New Note\n\n'
      });

      // 새 창 생성
      await invoke('create_sticker_window', { stickerData: newStickerData });

      console.log('New note created successfully');
    } catch (error) {
      console.error('Failed to create new note:', error);
    }
  }

  // 메뉴 이벤트 핸들러
  async function handleMenuEvent(menuId: string) {
    console.log('Menu event received:', menuId);

    // New Note
    if (menuId === 'new_note') {
      console.log('Handling new_note');
      createNewNote();
      return;
    }

    // Color Picker 열기
    if (menuId === 'open_color_picker') {
      console.log('Handling open_color_picker');
      openColorPicker();
      return;
    }

    // Edit 메뉴
    if (menuId === 'undo' && editorView) {
      const { undo } = await import('@codemirror/commands');
      undo(editorView);
      return;
    }
    if (menuId === 'redo' && editorView) {
      const { redo } = await import('@codemirror/commands');
      redo(editorView);
      return;
    }
    if (menuId === 'cut' && editorView) {
      const selection = editorView.state.selection.main;
      if (!selection.empty) {
        const text = editorView.state.sliceDoc(selection.from, selection.to);
        const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
        await writeText(text);
        editorView.dispatch({
          changes: { from: selection.from, to: selection.to, insert: '' }
        });
      }
      return;
    }
    if (menuId === 'copy' && editorView) {
      const selection = editorView.state.selection.main;
      if (!selection.empty) {
        const text = editorView.state.sliceDoc(selection.from, selection.to);
        const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
        await writeText(text);
      }
      return;
    }
    if (menuId === 'paste' && editorView) {
      try {
        const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
        const text = await readText();
        const selection = editorView.state.selection.main;
        const from = selection.from;
        const to = selection.to;
        const insertLength = text.length;

        editorView.dispatch({
          changes: { from, to, insert: text },
          selection: { anchor: from + insertLength }
        });
      } catch (error) {
        console.error('Failed to paste:', error);
      }
      return;
    }

    // Font size
    if (menuId === 'font_small') {
      fontSize = 12;
    }
    else if (menuId === 'font_medium') {
      fontSize = 14;
    }
    else if (menuId === 'font_large') {
      fontSize = 16;
    }
    else if (menuId === 'font_xlarge') {
      fontSize = 18;
    }
    else if (menuId === 'font_default') {
      fontSize = 11;
    }
    // Window
    else if (menuId === 'minimize') {
      console.log('Minimize window');
    }
    else if (menuId === 'zoom') {
      console.log('Zoom window');
    }
    // Close
    else if (menuId === 'close_note') {
      handleClose();
    }
  }

  onMount(async () => {
    loadFile();
    document.addEventListener('mousemove', handleDrag);
    document.addEventListener('mouseup', stopDrag);
    window.addEventListener('keydown', handleKeydown);

    // Tauri 메뉴 이벤트 리스닝
    const unlistenMenu = listen('menu', (event) => {
      handleMenuEvent(event.payload as string);
    });

    // 컬러 선택 이벤트 리스닝
    const unlistenColorSelected = listen('color-selected', (event: any) => {
      const data = event.payload;
      console.log('Color event received:', data);

      backgroundColor = data.color;
      console.log('Color applied:', data.color);
    });

    return () => {
      document.removeEventListener('mousemove', handleDrag);
      document.removeEventListener('mouseup', stopDrag);
      window.removeEventListener('keydown', handleKeydown);
      unlistenMenu.then(fn => fn());
      unlistenColorSelected.then(fn => fn());
    };
  });
</script>

<div
  class="sticker"
  style="background-color: {backgroundColor}; color: {textColor};"
  onmousedown={startDrag}
>
  <Toolbar
    {mode}
    {textColor}
    ontoggle-mode={toggleMode}
    onclose={handleClose}
  />

  <div class="content" style="font-size: {fontSize}px;">
    {#key mode}
      {#if mode === 'edit'}
        <MarkdownEditor
          {content}
          {textColor}
          {fontSize}
          onchange={handleContentChange}
          oneditorready={(view) => { editorView = view; }}
        />
      {:else}
        <MarkdownPreview {content} {textColor} {fontSize} />
      {/if}
    {/key}
  </div>
</div>

<style>
  .sticker {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 12px;
  }
</style>
