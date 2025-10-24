<script lang="ts">
  import { marked } from 'marked';

  interface Props {
    content: string;
    textColor: string;
    fontSize?: number;
  }

  let { content, textColor, fontSize = 11 }: Props = $props();

  // Svelte 5: Use $derived for computed values
  let html = $derived(marked(content, { breaks: true, gfm: true }));
</script>

<div class="preview" style="color: {textColor}; font-size: {fontSize}px;">
  {@html html}
</div>

<style>
  .preview {
    height: 100%;
    overflow: auto;
    line-height: 1.6;
  }

  .preview :global(h1) {
    font-size: 1.8em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
    border-bottom: 2px solid rgba(0, 0, 0, 0.1);
  }

  .preview :global(h2) {
    font-size: 1.5em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .preview :global(h3) {
    font-size: 1.3em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
  }

  .preview :global(code) {
    background: rgba(0, 0, 0, 0.05);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: Monaco, Menlo, monospace;
    font-size: 0.9em;
  }

  .preview :global(pre) {
    background: rgba(0, 0, 0, 0.05);
    padding: 12px;
    border-radius: 4px;
    overflow-x: auto;
  }

  .preview :global(pre code) {
    background: none;
    padding: 0;
  }

  .preview :global(blockquote) {
    border-left: 4px solid rgba(0, 0, 0, 0.2);
    padding-left: 12px;
    margin-left: 0;
    color: rgba(0, 0, 0, 0.6);
  }

  .preview :global(a) {
    color: #0066cc;
    text-decoration: none;
  }

  .preview :global(a:hover) {
    text-decoration: underline;
  }

  .preview :global(ul),
  .preview :global(ol) {
    padding-left: 24px;
  }

  .preview :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
  }

  .preview :global(th),
  .preview :global(td) {
    border: 1px solid rgba(0, 0, 0, 0.2);
    padding: 6px 12px;
    text-align: left;
  }

  .preview :global(th) {
    background: rgba(0, 0, 0, 0.05);
    font-weight: bold;
  }

  .preview :global(img) {
    max-width: 100%;
    height: auto;
  }
</style>
