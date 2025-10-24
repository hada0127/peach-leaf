import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';

// Svelte 5: Use mount() instead of new Component()
const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;
