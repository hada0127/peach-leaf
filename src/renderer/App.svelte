<script lang="ts">
  /// <reference types="svelte" />
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
  import Sticker from './components/Sticker.svelte';
  import ColorPicker from './components/ColorPicker.svelte';
  import type { StickerData } from './lib/tauri';

  let windowLabel = $state('');
  let isColorPicker = $state(false);
  let isInitialized = $state(false);

  // Svelte 5: Initialize state directly with default data
  let stickerData = $state<StickerData>({
    id: 'test',
    filePath: '/tmp/test.md',
    x: 100,
    y: 100,
    width: 400,
    height: 300,
    backgroundColor: '#FEFCE8',
    textColor: '#333333',
    mode: 'edit'
  });

  onMount(async () => {
    const currentWindow = getCurrentWindow();
    windowLabel = currentWindow.label;
    isColorPicker = windowLabel === 'color-picker';

    console.log('[App.svelte] Mounted. Window label:', windowLabel, 'isColorPicker:', isColorPicker);

    // Try to fetch saved window data from backend
    if (!isColorPicker) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const savedData = await invoke('get_window_data', { windowLabel });

        if (savedData) {
          console.log('[App.svelte] Found saved data for window:', windowLabel, savedData);

          stickerData = {
            id: savedData.id,
            filePath: savedData.file_path,
            x: savedData.x,
            y: savedData.y,
            width: savedData.width,
            height: savedData.height,
            backgroundColor: savedData.background_color,
            textColor: savedData.text_color,
            mode: savedData.mode
          };

          console.log('[App.svelte] Restored stickerData:', stickerData);
        } else {
          console.log('[App.svelte] No saved data found, using default');
          // For new windows, update the filePath to use permanent directory
          const timestamp = Date.now();
          const newId = windowLabel === 'main' ? 'main' : `note-${timestamp}`;
          stickerData.filePath = `/Users/tarucy/.peach-leaf/notes/${newId}.md`;
          stickerData.id = newId;
          console.log('[App.svelte] Created new window with file path:', stickerData.filePath);
        }
      } catch (error) {
        console.error('[App.svelte] Failed to fetch window data:', error);
      }
    }

    isInitialized = true;
  });
</script>

{#if !isInitialized}
  <div class="loading">Loading...</div>
{:else if isColorPicker}
  <ColorPicker />
{:else if stickerData}
  <Sticker data={stickerData} />
{:else}
  <div class="loading">Loading sticker data...</div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    overflow: hidden;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: #666;
  }
</style>
