<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import MarkdownEditor from './MarkdownEditor.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import Toolbar from './Toolbar.svelte';

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

  function handleClose() {
    if (window.electron) {
      window.electron.closeSticker(data.id);
    } else {
      console.log('Would close sticker');
    }
  }

  function startDrag(e: MouseEvent) {
    console.log('startDrag called, target:', e.target);

    // 버튼 클릭은 드래그로 처리하지 않음
    const isButton = (e.target as HTMLElement).closest('button');
    if (isButton) {
      console.log('Button clicked, not dragging');
      return;
    }

    const toolbar = (e.target as HTMLElement).closest('.toolbar');
    console.log('toolbar found:', toolbar);
    if (toolbar) {
      console.log('Starting drag');
      isDragging = true;
      dragStartX = e.screenX;
      dragStartY = e.screenY;
      e.preventDefault(); // 기본 동작 방지
    }
  }

  async function handleDrag(e: MouseEvent) {
    if (isDragging && e.screenX !== 0 && e.screenY !== 0) {
      const deltaX = e.screenX - dragStartX;
      const deltaY = e.screenY - dragStartY;
      dragStartX = e.screenX;
      dragStartY = e.screenY;

      console.log('Moving by:', deltaX, deltaY);

      // Tauri API 사용
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      const position = await currentWindow.outerPosition();
      await currentWindow.setPosition({
        x: position.x + deltaX,
        y: position.y + deltaY
      });
    }
  }

  function stopDrag() {
    isDragging = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    console.log('Key pressed:', e.key, 'metaKey:', e.metaKey, 'ctrlKey:', e.ctrlKey);
    if ((e.metaKey || e.ctrlKey) && e.key === 'm') {
      console.log('Mode toggle shortcut detected!');
      e.preventDefault();
      toggleMode();
    }
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
        file_path: tempFilePath,
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
  function handleMenuEvent(menuId: string) {
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
  role="application"
  tabindex="0"
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
        <MarkdownEditor {content} {textColor} {fontSize} onchange={handleContentChange} />
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
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 12px;
  }

  .mode-indicator {
    background: #ff9800;
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: bold;
    margin-bottom: 8px;
    display: inline-block;
  }
</style>
