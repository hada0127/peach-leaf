<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
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
  let editorView: EditorView | null = null;
  let saveTimeout: number | null = null;
  let lastMenuEventTime = 0;
  let isProcessingMenuEvent = false;
  let pressedKeys = new Set<string>();

  // Drag state
  let isDragging = $state(false);
  let dragStartX = $state(0);
  let dragStartY = $state(0);
  let initialWindowX = $state(0);
  let initialWindowY = $state(0);
  let lastDragTime = 0;


  // Watch for props changes and update local state
  $effect(() => {
    console.log('[Sticker] Props changed:', data);
    backgroundColor = data.backgroundColor;
    textColor = data.textColor;
    mode = data.mode;
    // Reload file when filePath changes
    if (data.filePath) {
      console.log('[Sticker] File path changed to:', data.filePath);
      loadFile();
    }
  });

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
      // Also save window state after content changes
      await saveWindowState();
      saveTimeout = null;
    }, 500);
  }

  async function saveWindowState() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_window_state');
      console.log('[Sticker] Window state saved');
    } catch (error) {
      console.error('[Sticker] Failed to save window state:', error);
    }
  }

  function handleContentChange(event: CustomEvent<string>) {
    content = event.detail;
    saveFile();
  }

  async function toggleMode() {
    console.log('toggleMode called, current mode:', mode);

    // 편집 모드와 미리보기 모드만 토글
    mode = mode === 'edit' ? 'preview' : 'edit';

    console.log('Mode changed to:', mode);

    // Update backend metadata with new mode and save state
    try {
      const { invoke } = await import('@tauri-apps/api/core');

      await invoke('update_window_metadata', {
        windowLabel: data.id,
        backgroundColor: null,
        mode: mode
      });
      console.log(`[${data.id}] Updated backend metadata with mode:`, mode);

      // Save window state immediately after mode change
      await saveWindowState();
    } catch (error) {
      console.error('Failed to update window metadata:', error);
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
      console.log(`[${data.id}] handleClose called`);
      console.log(`[${data.id}] raw content:`, JSON.stringify(content));

      // Check if there's content in the note
      const trimmedContent = content.trim();

      console.log(`[${data.id}] trimmedContent:`, JSON.stringify(trimmedContent));
      console.log(`[${data.id}] content length:`, trimmedContent.length);

      // The default content is "# New Note" (after trimming)
      // Consider the note empty if it only contains the default heading
      const isDefaultContent = trimmedContent === '' ||
                               trimmedContent === '# New Note';

      const hasRealContent = !isDefaultContent;

      console.log(`[${data.id}] isDefaultContent:`, isDefaultContent);
      console.log(`[${data.id}] hasRealContent:`, hasRealContent);

      // If there's real content, show confirmation dialog
      if (hasRealContent) {
        const { confirm } = await import('@tauri-apps/plugin-dialog');
        const shouldClose = await confirm('This note has content. Are you sure you want to close it?', {
          title: 'Close Note',
          kind: 'warning'
        });

        if (!shouldClose) {
          return; // User cancelled, don't close
        }
      }

      // Delete the note file
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('delete_note_file', { noteId: data.id });
      console.log(`[${data.id}] Note file deleted`);

      // Close the window (state will be saved automatically by backend after window is destroyed)
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      await currentWindow.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  }

  // Drag functions
  async function startDrag(e: MouseEvent) {
    const toolbar = (e.target as HTMLElement).closest('.toolbar');
    if (toolbar) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      const position = await currentWindow.outerPosition();

      // Use physical coordinates directly
      initialWindowX = position.x;
      initialWindowY = position.y;

      isDragging = true;
      dragStartX = e.screenX;
      dragStartY = e.screenY;
      lastDragTime = Date.now();
    }
  }

  async function handleDrag(e: MouseEvent) {
    if (isDragging && e.screenX !== 0 && e.screenY !== 0) {
      const deltaX = e.screenX - dragStartX;
      const deltaY = e.screenY - dragStartY;

      const newX = initialWindowX + deltaX;
      const newY = initialWindowY + deltaY;

      const { getCurrentWindow, PhysicalPosition } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      const newPosition = new PhysicalPosition(newX, newY);
      await currentWindow.setPosition(newPosition);

      lastDragTime = Date.now();
    }
  }

  function stopDrag() {
    if (isDragging) {
      isDragging = false;
      // Debounced save - only save after drag has been idle for 100ms
      setTimeout(() => {
        if (!isDragging && Date.now() - lastDragTime > 100) {
          saveWindowState();
        }
      }, 100);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // 키를 누르고 있는 동안 중복 실행 방지
    const keyCombo = `${e.metaKey || e.ctrlKey ? 'cmd-' : ''}${e.key.toLowerCase()}`;

    // 이미 눌려있는 키면 무시 (키 반복 방지)
    if (pressedKeys.has(keyCombo)) {
      e.preventDefault();
      return;
    }

    pressedKeys.add(keyCombo);

    // Cmd+M: 모드 전환 - 이것만 preventDefault
    if ((e.metaKey || e.ctrlKey) && e.key === 'm') {
      e.preventDefault();
    }
    // Cmd+N, Cmd+W, Cmd+Q는 메뉴 시스템이 처리하도록 그냥 통과
    // 다른 모든 키는 기본 동작 허용 (CodeMirror가 처리)
  }

  function handleKeyup(e: KeyboardEvent) {
    const keyCombo = `${e.metaKey || e.ctrlKey ? 'cmd-' : ''}${e.key.toLowerCase()}`;

    // 키를 뗐을 때 실행
    if (pressedKeys.has(keyCombo)) {
      pressedKeys.delete(keyCombo);

      // Cmd+M: 모드 전환
      if ((e.metaKey || e.ctrlKey) && e.key === 'm') {
        e.preventDefault();
        toggleMode();
      }
    }
  }

  // 컬러 피커 열기
  async function openColorPicker() {
    console.log('openColorPicker called');
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      console.log('Opening color picker for window:', data.id, 'with color:', backgroundColor);

      await invoke('open_color_picker', {
        parentLabel: data.id,
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

    // Debounce: 300ms 이내의 중복 이벤트 무시
    const now = Date.now();
    if (now - lastMenuEventTime < 300) {
      console.log('Ignoring duplicate menu event');
      return;
    }
    lastMenuEventTime = now;

    // 이미 처리 중인 경우 무시
    if (isProcessingMenuEvent) {
      console.log('Already processing menu event');
      return;
    }
    isProcessingMenuEvent = true;

    try {
      // New Note is now handled in backend, skip here
      if (menuId === 'new_note') {
        console.log('new_note handled in backend, skipping frontend');
        return;
      }

      // Color Picker is now handled in backend via open_color_picker_event

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
      // Close note is now handled in backend via window-specific event
    } finally {
      // 처리 완료 후 플래그 리셋
      isProcessingMenuEvent = false;
    }
  }

  let unlistenMenu: (() => void) | null = null;
  let unlistenColorSelected: (() => void) | null = null;
  let unlistenCloseNote: (() => void) | null = null;
  let unlistenOpenColorPicker: (() => void) | null = null;
  let unlistenResized: (() => void) | null = null;
  let unlistenMoved: (() => void) | null = null;

  onMount(async () => {
    loadFile();
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('keyup', handleKeyup);
    document.addEventListener('mousemove', handleDrag);
    document.addEventListener('mouseup', stopDrag);

    // Tauri 메뉴 이벤트 리스닝
    unlistenMenu = await listen('menu', (event) => {
      handleMenuEvent(event.payload as string);
    });

    // 윈도우 정보 가져오기
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const currentWindow = getCurrentWindow();
    const windowLabel = currentWindow.label;

    console.log(`[${data.id}] Window label: ${windowLabel}`);
    console.log(`[${data.id}] Initial data:`, data);

    // Props로 받은 배경색 적용
    if (data.backgroundColor) {
      backgroundColor = data.backgroundColor;
      console.log(`[${data.id}] Applied backgroundColor from props:`, data.backgroundColor);
    }

    // 윈도우별 컬러 선택 이벤트 리스닝
    unlistenColorSelected = await listen(`color-selected-${data.id}`, async (event: any) => {
      const eventData = event.payload;
      console.log(`[${data.id}] Color event received:`, eventData);

      backgroundColor = eventData.color;
      console.log(`[${data.id}] Color applied:`, eventData.color);

      // Update backend metadata with new color and save state
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_window_metadata', {
          windowLabel: data.id,
          backgroundColor: eventData.color,
          mode: null
        });
        console.log(`[${data.id}] Updated backend metadata with color:`, eventData.color);

        // Save window state immediately after color change
        await saveWindowState();
      } catch (error) {
        console.error(`[${data.id}] Failed to update window metadata:`, error);
      }
    });

    console.log(`[${data.id}] Listening for close_note_${data.id}`);
    console.log(`[${data.id}] Listening for open_color_picker_${data.id}`);
    console.log(`[${data.id}] Listening for color-selected-${data.id}`);

    unlistenCloseNote = await listen(`close_note_${data.id}`, () => {
      console.log(`[${data.id}] Received close_note event for this window`);
      handleClose();
    });

    // 색상 선택기 열기 이벤트 리스닝 (백엔드에서 포커스된 윈도우에만 전송)
    unlistenOpenColorPicker = await listen(`open_color_picker_${data.id}`, () => {
      console.log(`[${data.id}] Received open_color_picker event for this window`);
      openColorPicker();
    });

    // Listen for window resize events
    unlistenResized = await currentWindow.onResized(async () => {
      console.log(`[${data.id}] Window resized, saving state...`);
      await saveWindowState();
    });

    // Listen for window move events - but don't save during drag
    unlistenMoved = await currentWindow.onMoved(async () => {
      if (!isDragging) {
        console.log(`[${data.id}] Window moved, saving state...`);
        await saveWindowState();
      }
    });
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    window.removeEventListener('keyup', handleKeyup);
    document.removeEventListener('mousemove', handleDrag);
    document.removeEventListener('mouseup', stopDrag);
    if (unlistenMenu) unlistenMenu();
    if (unlistenColorSelected) unlistenColorSelected();
    if (unlistenCloseNote) unlistenCloseNote();
    if (unlistenOpenColorPicker) unlistenOpenColorPicker();
    if (unlistenResized) unlistenResized();
    if (unlistenMoved) unlistenMoved();
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
