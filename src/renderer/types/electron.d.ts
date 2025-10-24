export interface ElectronAPI {
  readFile: (filePath: string) => Promise<{ success: boolean; content?: string; error?: string }>;
  writeFile: (filePath: string, content: string) => Promise<{ success: boolean; error?: string }>;
  updateStickerConfig: (id: string, config: any) => Promise<{ success: boolean }>;
  closeSticker: (id: string) => Promise<{ success: boolean }>;
  getShortcuts: () => Promise<Record<string, string>>;
  setShortcut: (action: string, shortcut: string) => Promise<{ success: boolean }>;
  onInitSticker: (callback: (data: any) => void) => void;
}

declare global {
  interface Window {
    electron: ElectronAPI;
  }
}
