const { app, BrowserWindow, ipcMain, dialog, globalShortcut } = require('electron');
const path = require('path');
const fs = require('fs/promises');

interface StickerData {
  id: string;
  filePath: string;
  x: number;
  y: number;
  width: number;
  height: number;
  backgroundColor: string;
  textColor: string;
  mode: 'edit' | 'preview' | 'richtext';
}

interface StoreData {
  stickers: StickerData[];
  shortcuts: Record<string, string>;
}

class SimpleStore {
  private filePath: string | null = null;
  private data: StoreData;

  constructor() {
    this.data = { stickers: [], shortcuts: {} };
  }

  private ensureInitialized() {
    if (!this.filePath) {
      this.filePath = path.join(app.getPath('userData'), 'store.json');
      this.load();
    }
  }

  private load() {
    try {
      const data = require('fs').readFileSync(this.filePath, 'utf-8');
      this.data = JSON.parse(data);
    } catch (error) {
      // File doesn't exist yet, use defaults
    }
  }

  private save() {
    try {
      require('fs').writeFileSync(this.filePath, JSON.stringify(this.data, null, 2));
    } catch (error) {
      console.error('Failed to save store:', error);
    }
  }

  get(key: string, defaultValue?: any) {
    this.ensureInitialized();
    return (this.data as any)[key] ?? defaultValue;
  }

  set(key: string, value: any) {
    this.ensureInitialized();
    (this.data as any)[key] = value;
    this.save();
  }
}

const store = new SimpleStore();

const windows = new Map<string, any>();

function createStickerWindow(stickerData: StickerData): any {
  const win = new BrowserWindow({
    width: stickerData.width || 400,
    height: stickerData.height || 300,
    x: stickerData.x,
    y: stickerData.y,
    frame: false,
    transparent: true,
    resizable: true,
    alwaysOnTop: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
  });

  const isDev = !app.isPackaged;
  if (isDev) {
    win.loadURL('http://localhost:5174');
  } else {
    win.loadFile(path.join(__dirname, '../renderer/index.html'));
  }

  win.webContents.on('did-finish-load', () => {
    win.webContents.send('init-sticker', stickerData);
  });

  win.on('close', () => {
    const bounds = win.getBounds();
    updateStickerBounds(stickerData.id, bounds);
  });

  windows.set(stickerData.id, win);
  return win;
}

function updateStickerBounds(id: string, bounds: { x: number; y: number; width: number; height: number }) {
  const stickers = store.get('stickers', []);
  const index = stickers.findIndex(s => s.id === id);
  if (index !== -1) {
    stickers[index] = { ...stickers[index], ...bounds };
    store.set('stickers', stickers);
  }
}

async function createNewSticker() {
  const result = await dialog.showOpenDialog({
    properties: ['openFile'],
    filters: [
      { name: 'Markdown', extensions: ['md', 'markdown'] },
      { name: 'All Files', extensions: ['*'] },
    ],
  });

  if (result.canceled || result.filePaths.length === 0) {
    return;
  }

  const filePath = result.filePaths[0];
  const id = Date.now().toString();

  const stickerData: StickerData = {
    id,
    filePath,
    x: 100,
    y: 100,
    width: 400,
    height: 300,
    backgroundColor: '#fff9e6',
    textColor: '#333333',
    mode: 'edit',
  };

  const stickers = store.get('stickers', []);
  stickers.push(stickerData);
  store.set('stickers', stickers);

  createStickerWindow(stickerData);
}

// IPC handlers
ipcMain.handle('read-file', async (_, filePath: string) => {
  try {
    const content = await fs.readFile(filePath, 'utf-8');
    return { success: true, content };
  } catch (error) {
    return { success: false, error: (error as Error).message };
  }
});

ipcMain.handle('write-file', async (_, filePath: string, content: string) => {
  try {
    await fs.writeFile(filePath, content, 'utf-8');
    return { success: true };
  } catch (error) {
    return { success: false, error: (error as Error).message };
  }
});

ipcMain.handle('update-sticker-config', async (_, id: string, config: Partial<StickerData>) => {
  const stickers = store.get('stickers', []);
  const index = stickers.findIndex(s => s.id === id);
  if (index !== -1) {
    stickers[index] = { ...stickers[index], ...config };
    store.set('stickers', stickers);
    return { success: true };
  }
  return { success: false };
});

ipcMain.handle('close-sticker', async (_, id: string) => {
  const win = windows.get(id);
  if (win) {
    win.close();
    windows.delete(id);
  }

  const stickers = store.get('stickers', []);
  const filtered = stickers.filter(s => s.id !== id);
  store.set('stickers', filtered);

  return { success: true };
});

ipcMain.handle('get-shortcuts', async () => {
  return store.get('shortcuts', {
    'toggle-mode': 'CommandOrControl+M',
    'new-sticker': 'CommandOrControl+N',
  });
});

ipcMain.handle('set-shortcut', async (_, action: string, shortcut: string) => {
  const shortcuts = store.get('shortcuts', {});
  shortcuts[action] = shortcut;
  store.set('shortcuts', shortcuts);
  return { success: true };
});

app.whenReady().then(() => {
  // Restore previous stickers
  const stickers = store.get('stickers', []);
  stickers.forEach(stickerData => {
    createStickerWindow(stickerData);
  });

  // If no stickers, create a new one
  if (stickers.length === 0) {
    createNewSticker();
  }

  app.on('activate', () => {
    if (windows.size === 0) {
      createNewSticker();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// Global shortcuts for creating new stickers
app.whenReady().then(() => {
  const shortcuts = store.get('shortcuts', {});
  const newStickerShortcut = shortcuts['new-sticker'] || 'CommandOrControl+Shift+N';

  globalShortcut.register(newStickerShortcut, () => {
    createNewSticker();
  });
});
