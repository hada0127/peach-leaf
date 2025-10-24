const electron = require('electron');
console.log('Type:', typeof electron);
console.log('Keys:', Object.keys(electron || {}).slice(0, 20));

if (electron && electron.app) {
  console.log('SUCCESS: electron.app exists!');
  electron.app.quit();
} else {
  console.log('FAIL: electron is:', electron);
}
