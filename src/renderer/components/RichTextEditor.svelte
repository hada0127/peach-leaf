<script lang="ts">
  import { marked } from 'marked';

  interface Props {
    content: string;
    textColor: string;
    onchange?: (event: CustomEvent<string>) => void;
  }

  let { content, textColor, onchange }: Props = $props();

  let editorDiv = $state<HTMLDivElement>();
  let isUpdating = $state(false);

  function handleInput() {
    if (!isUpdating && editorDiv) {
      const text = editorDiv.innerText;
      const event = new CustomEvent('change', { detail: text });
      onchange?.(event);
    }
  }

  // Svelte 5: Use $effect to update editor when content changes
  $effect(() => {
    if (editorDiv && content !== editorDiv.innerText) {
      isUpdating = true;
      editorDiv.innerHTML = marked(content, { breaks: true, gfm: true }) as string;
      isUpdating = false;
    }
  });
</script>

<div
  class="richtext-editor"
  contenteditable="true"
  bind:this={editorDiv}
  oninput={handleInput}
  style="color: {textColor}"
  role="textbox"
  tabindex="0"
></div>

<style>
  .richtext-editor {
    height: 100%;
    overflow: auto;
    font-size: 14px;
    line-height: 1.6;
    outline: none;
    padding: 0;
  }

  .richtext-editor:focus {
    outline: none;
  }

  .richtext-editor :global(h1) {
    font-size: 1.8em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
    border-bottom: 2px solid rgba(0, 0, 0, 0.1);
  }

  .richtext-editor :global(h2) {
    font-size: 1.5em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  }

  .richtext-editor :global(h3) {
    font-size: 1.3em;
    margin-top: 0.5em;
    margin-bottom: 0.5em;
  }

  .richtext-editor :global(code) {
    background: rgba(0, 0, 0, 0.05);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: Monaco, Menlo, monospace;
    font-size: 0.9em;
  }

  .richtext-editor :global(pre) {
    background: rgba(0, 0, 0, 0.05);
    padding: 12px;
    border-radius: 4px;
    overflow-x: auto;
  }

  .richtext-editor :global(blockquote) {
    border-left: 4px solid rgba(0, 0, 0, 0.2);
    padding-left: 12px;
    margin-left: 0;
    color: rgba(0, 0, 0, 0.6);
  }

  .richtext-editor :global(a) {
    color: #0066cc;
  }
</style>
