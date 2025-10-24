import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface StickerData {
  id: string;
  filePath: string;
  x: number;
  y: number;
  width: number;
  height: number;
  backgroundColor: string;
  textColor: string;
  mode: 'edit' | 'preview';
}

export const tauriAPI = {
  readFile: async (filePath: string): Promise<{ success: boolean; content?: string; error?: string }> => {
    try {
      const content = await invoke<string>('read_file', { filePath });
      return { success: true, content };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  writeFile: async (filePath: string, content: string): Promise<{ success: boolean; error?: string }> => {
    try {
      await invoke('write_file', { filePath, content });
      return { success: true };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  },

  selectFile: async (): Promise<string | null> => {
    try {
      const result = await invoke<string | null>('select_file');
      return result;
    } catch (error) {
      console.error('File selection error:', error);
      return null;
    }
  },

  onInitSticker: (callback: (data: StickerData) => void) => {
    listen<StickerData>('init-sticker', (event) => {
      callback(event.payload);
    });
  },

  updateStickerConfig: async (id: string, config: Partial<StickerData>) => {
    // Store locally for now - can be enhanced with Tauri store plugin
    console.log('Update sticker config:', id, config);
    return { success: true };
  },

  closeSticker: async (id: string) => {
    console.log('Close sticker:', id);
    return { success: true };
  },

  getShortcuts: async () => {
    return {
      'toggle-mode': 'CommandOrControl+M',
      'new-sticker': 'CommandOrControl+N',
    };
  },

  setShortcut: async (action: string, shortcut: string) => {
    console.log('Set shortcut:', action, shortcut);
    return { success: true };
  },
};

// Expose to window for compatibility
if (typeof window !== 'undefined') {
  (window as any).tauri = tauriAPI;
}
