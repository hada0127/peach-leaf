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

    console.log('App.svelte mounted. Window label:', windowLabel, 'isColorPicker:', isColorPicker);

    // Listen for init-sticker event to restore saved window data (BEFORE setting isInitialized)
    if (!isColorPicker) {
      await listen('init-sticker', (event: any) => {
        const data = event.payload;
        console.log('[App.svelte] Received init-sticker event:', data);

        stickerData = {
          id: data.id,
          filePath: data.file_path,
          x: data.x,
          y: data.y,
          width: data.width,
          height: data.height,
          backgroundColor: data.background_color,
          textColor: data.text_color,
          mode: data.mode
        };

        console.log('[App.svelte] Updated stickerData:', stickerData);
      });

      // For new windows (not restored), update the filePath to use permanent directory
      if (stickerData.filePath.startsWith('/tmp/')) {
        const timestamp = Date.now();
        const newId = windowLabel === 'main' ? 'main' : `note-${timestamp}`;
        stickerData.filePath = `/Users/tarucy/.peach-leaf/notes/${newId}.md`;
        stickerData.id = newId;
        console.log('[App.svelte] Updated new window file path to:', stickerData.filePath);
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
