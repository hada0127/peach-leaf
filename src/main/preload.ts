const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  readFile: (filePath: string) => ipcRenderer.invoke('read-file', filePath),
  writeFile: (filePath: string, content: string) =>
    ipcRenderer.invoke('write-file', filePath, content),
  updateStickerConfig: (id: string, config: any) =>
    ipcRenderer.invoke('update-sticker-config', id, config),
  closeSticker: (id: string) => ipcRenderer.invoke('close-sticker', id),
  getShortcuts: () => ipcRenderer.invoke('get-shortcuts'),
  setShortcut: (action: string, shortcut: string) =>
    ipcRenderer.invoke('set-shortcut', action, shortcut),
  onInitSticker: (callback: (data: any) => void) => {
    ipcRenderer.on('init-sticker', (_, data) => callback(data));
  },
});
