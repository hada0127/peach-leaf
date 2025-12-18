<script lang="ts">
  import { marked } from 'marked';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    content: string;
    textColor: string;
    fontSize?: number;
    filePath?: string;
  }

  let { content, textColor, fontSize = 11, filePath = '' }: Props = $props();

  // Cache for loaded image data URLs
  let imageCache = $state<Map<string, string>>(new Map());
  let cacheVersion = $state(0); // Used to trigger re-renders

  // Load images asynchronously and cache them as data URLs
  $effect(() => {
    if (!content || !filePath) return;

    const imageRegex = /!\[.*?\]\((\.\/[^)]+)\)/g;
    const matches = Array.from(content.matchAll(imageRegex));
    const imagePaths = matches.map(m => m[1]);

    if (imagePaths.length === 0) return;

    const noteDir = filePath.substring(0, filePath.lastIndexOf('/'));

    // Load images in parallel
    Promise.all(
      imagePaths.map(async (relativePath) => {
        // Skip if already cached
        if (imageCache.has(relativePath)) return;

        try {
          // Convert relative path to absolute
          const absolutePath = `${noteDir}/${relativePath.substring(2)}`;

          // Call Tauri command to get data URL
          const dataUrl = await invoke<string>('read_image_as_data_url', {
            imagePath: absolutePath
          });

          console.log('[MarkdownPreview] Loaded image:', relativePath);

          // Cache the data URL
          imageCache.set(relativePath, dataUrl);
        } catch (error) {
          console.error('[MarkdownPreview] Failed to load image:', relativePath, error);
        }
      })
    ).then(() => {
      // Trigger re-render by updating cache version
      cacheVersion++;
    });
  });

  // Svelte 5: Use $derived for computed values
  let html = $derived.by(() => {
    // Access cacheVersion to trigger re-render when images are loaded
    const _ = cacheVersion;

    // Custom renderer to use cached data URLs
    const renderer = new marked.Renderer();
    const originalImage = renderer.image.bind(renderer);

    renderer.image = (args) => {
      const href = typeof args === 'object' ? args.href : args;
      const title = typeof args === 'object' ? args.title : '';
      const text = typeof args === 'object' ? args.text : '';

      // If it's a relative path starting with ./ and we have it cached
      if (href && href.startsWith('./')) {
        const cachedDataUrl = imageCache.get(href);

        if (cachedDataUrl) {
          // Find width comment after this image: <!-- width:123 -->
          // Simple approach: search for the path followed by width comment
          const escapedHref = href.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          const widthRegex = new RegExp(`\\(${escapedHref}\\)<!--\\s*width:(\\d+)\\s*-->`);
          const widthMatch = content.match(widthRegex);
          const width = widthMatch ? parseInt(widthMatch[1]) : null;

          console.log('[MarkdownPreview] Image:', href, 'Width:', width);

          const widthStyle = width ? ` style="width: ${width}px; height: auto;"` : '';
          return `<img src="${cachedDataUrl}" alt="${text}" title="${title || ''}"${widthStyle} />`;
        } else {
          // Show placeholder while loading
          return `<img src="" alt="${text}" title="${title || ''}" style="display:none;" />`;
        }
      }

      // For other images (http/https), use original renderer
      return originalImage({ href, title, text });
    };

    return marked(content, { breaks: true, gfm: true, renderer });
  });
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
    margin: 0;
    padding: 0;
  }

  .preview :global(h2) {
    font-size: 1.5em;
    margin: 0;
    padding: 0;
  }

  .preview :global(h3) {
    font-size: 1.3em;
    margin: 0;
    padding: 0;
  }

  .preview :global(h4) {
    font-size: 1.1em;
    margin: 0;
    padding: 0;
  }

  .preview :global(h5) {
    font-size: 1.0em;
    margin: 0;
    padding: 0;
  }

  .preview :global(h6) {
    font-size: 0.9em;
    margin: 0;
    padding: 0;
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

  /* Table column alignment - marked library adds these styles */
  .preview :global(th[align="left"]),
  .preview :global(td[align="left"]) {
    text-align: left;
  }

  .preview :global(th[align="center"]),
  .preview :global(td[align="center"]) {
    text-align: center;
  }

  .preview :global(th[align="right"]),
  .preview :global(td[align="right"]) {
    text-align: right;
  }

  .preview :global(img) {
    max-width: 100%;
    height: auto;
  }

  /* 프린트 스타일 */
  @media print {
    .preview {
      overflow: visible !important;
      height: auto !important;
    }
  }
</style>
