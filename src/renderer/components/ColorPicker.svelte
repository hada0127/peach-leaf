<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let parentLabel = $state('');
  let currentColor = $state('#FEFCE8');

  const colors = [
    { name: 'Red Tint', hex: '#FEF2F2' },
    { name: 'Orange Tint', hex: '#FFF7ED' },
    { name: 'Amber', hex: '#FFFBEB' },
    { name: 'Yellow', hex: '#FEFCE8' },
    { name: 'Lime', hex: '#F7FEE7' },
    { name: 'Green Light', hex: '#F0FDF4' },
    { name: 'Emerald', hex: '#ECFDF5' },
    { name: 'Teal', hex: '#F0FDFA' },
    { name: 'Cyan', hex: '#ECFEFF' },
    { name: 'Sky Blue', hex: '#F0F9FF' },
    { name: 'Blue', hex: '#EEF6FF' },
    { name: 'Indigo', hex: '#EEF2FF' },
    { name: 'Violet', hex: '#F5F3FF' },
    { name: 'Purple', hex: '#FAF5FF' },
    { name: 'Fuchsia', hex: '#FDF4FF' },
    { name: 'Pink', hex: '#FDF2F8' },
    { name: 'Rose', hex: '#FFF1F2' },
    { name: 'Gray', hex: '#E8E8E8' },
  ];

  async function selectColor(color: string) {
    console.log('selectColor called with:', color);
    try {
      const { invoke } = await import('@tauri-apps/api/core');

      console.log('Color selected:', color, 'for parent:', parentLabel);

      // Call Rust command to apply color and close window
      await invoke('apply_color', {
        parentLabel: parentLabel,
        color
      });

      console.log('Color applied and window closed');
    } catch (error) {
      console.error('Error selecting color:', error);
      alert('Error: ' + error);
    }
  }

  onMount(async () => {
    console.log('ColorPicker mounted');

    // Read data from URL query parameters
    const urlParams = new URLSearchParams(window.location.search);
    const paramParentLabel = urlParams.get('parent_label');
    const paramCurrentColor = urlParams.get('current_color');

    if (paramParentLabel) {
      parentLabel = paramParentLabel;
    }
    if (paramCurrentColor) {
      currentColor = paramCurrentColor;
    }

    console.log('ColorPicker initialized from URL params:', { parentLabel, currentColor });

    // Also listen for initialization data (fallback)
    const unlisten = await listen('init-color-picker', (event: any) => {
      const data = event.payload;
      console.log('init-color-picker received:', data);
      // Rust sends parent_label and current_color (snake_case) via events
      if (data.parent_label) parentLabel = data.parent_label;
      if (data.current_color) currentColor = data.current_color;
      console.log('ColorPicker updated from event:', { parentLabel, currentColor });
    });

    // Close on window blur
    const handleBlur = async () => {
      console.log('ColorPicker blur - closing');
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('close_color_picker');
    };
    window.addEventListener('blur', handleBlur);

    return () => {
      unlisten.then(fn => fn());
      window.removeEventListener('blur', handleBlur);
    };
  });
</script>

<div class="color-picker-window">
  <div class="color-grid">
    {#each colors as color}
      <button
        class="color-item"
        class:selected={currentColor.toUpperCase() === color.hex.toUpperCase()}
        style="background-color: {color.hex}"
        onclick={() => selectColor(color.hex)}
        title={color.name}
      >
        {#if currentColor.toUpperCase() === color.hex.toUpperCase()}
          <span class="checkmark">✓</span>
        {/if}
      </button>
    {/each}
  </div>
</div>

<style>
  .color-picker-window {
    width: 100%;
    height: 100%;
    background: white;
    padding: 8px;
    box-sizing: border-box;
    display: block;
    position: relative;
  }

  .color-grid {
    display: grid;
    grid-template-columns: repeat(6, 36px);
    grid-template-rows: repeat(3, 28px);
    gap: 4px;
  }

  .color-item {
    width: 36px;
    height: 28px;
    border: 2px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .color-item:hover {
    border-color: #999;
    transform: scale(1.05);
  }

  .color-item.selected {
    border-color: #333;
    border-width: 3px;
  }

  .checkmark {
    color: #333;
    font-size: 14px;
    font-weight: bold;
    text-shadow: 0 0 3px white;
  }
</style>
