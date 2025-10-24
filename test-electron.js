const electron = require('electron');

console.log('electron:', Object.keys(electron));
console.log('app:', electron.app);
console.log('ipcMain:', electron.ipcMain);
console.log('BrowserWindow:', electron.BrowserWindow);
