<script lang="ts">
  /// <reference types="svelte" />
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
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

  onMount(() => {
    const currentWindow = getCurrentWindow();
    windowLabel = currentWindow.label;
    isColorPicker = windowLabel === 'color-picker';
    isInitialized = true;
    console.log('App.svelte mounted. Window label:', windowLabel, 'isColorPicker:', isColorPicker);
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
