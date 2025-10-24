import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';
import { tauriAPI } from './lib/tauri';

// Expose Tauri API as window.electron for compatibility
(window as any).electron = tauriAPI;

// Svelte 5: Use mount() instead of new Component()
const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
