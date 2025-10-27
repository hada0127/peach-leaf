<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, keymap, lineNumbers, Decoration, type DecorationSet, ViewPlugin, type ViewUpdate, WidgetType } from '@codemirror/view';
  import { EditorState, Compartment, type Range } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine } from '@codemirror/view';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language';

  interface Props {
    content: string;
    textColor: string;
    fontSize?: number;
    filePath?: string;
    onchange?: (event: CustomEvent<string>) => void;
    oneditorready?: (view: EditorView) => void;
  }

  let { content, textColor, fontSize = 11, filePath = '', onchange, oneditorready }: Props = $props();

  console.log('[MarkdownEditor] Component created, filePath:', filePath);

  let editorContainer = $state<HTMLDivElement>();
  let editorView: EditorView | null = null;

  // Image cache for loaded images
  const imageCache = new Map<string, string>();

  // Widget for displaying images
  class ImageWidget extends WidgetType {
    src: string;
    alt: string;

    constructor(src: string, alt: string) {
      super();
      this.src = src;
      this.alt = alt;
    }

    toDOM() {
      const wrap = document.createElement('span');
      wrap.className = 'cm-image-widget';
      const img = document.createElement('img');
      img.src = this.src;
      img.alt = this.alt;
      img.style.maxWidth = '100%';
      img.style.display = 'block';
      img.style.margin = '4px 0';
      img.style.borderRadius = '4px';
      wrap.appendChild(img);
      return wrap;
    }
  }

  // Load image as data URL
  async function loadImageAsDataUrl(relativePath: string): Promise<string | null> {
    if (!filePath) return null;

    // Check cache first
    if (imageCache.has(relativePath)) {
      return imageCache.get(relativePath)!;
    }

    try {
      const noteDir = filePath.substring(0, filePath.lastIndexOf('/'));
      const absolutePath = `${noteDir}/${relativePath.substring(2)}`;

      const { invoke } = await import('@tauri-apps/api/core');
      const dataUrl = await invoke<string>('read_image_as_data_url', {
        imagePath: absolutePath
      });

      // Cache the result
      imageCache.set(relativePath, dataUrl);
      return dataUrl;
    } catch (error) {
      console.error('[MarkdownEditor] Failed to load image:', relativePath, error);
      return null;
    }
  }

  // Create decorations for images
  function createImageDecorations(view: EditorView): DecorationSet {
    const widgets: Range<Decoration>[] = [];
    const imageRegex = /!\[([^\]]*)\]\((\.\/[^)]+)\)/g;
    const text = view.state.doc.toString();

    console.log('[MarkdownEditor] Creating image decorations, text length:', text.length);

    let match;
    let matchCount = 0;
    while ((match = imageRegex.exec(text)) !== null) {
      matchCount++;
      const from = match.index;
      const to = from + match[0].length;
      const alt = match[1];
      const src = match[2];

      console.log('[MarkdownEditor] Found image:', { src, alt, from, to });

      // Load image asynchronously if not cached
      const cachedDataUrl = imageCache.get(src);
      if (!cachedDataUrl) {
        console.log('[MarkdownEditor] Image not cached, loading:', src);
        loadImageAsDataUrl(src).then((dataUrl) => {
          if (dataUrl) {
            console.log('[MarkdownEditor] Image loaded, triggering update:', src);
            // Trigger a full decoration rebuild by dispatching an empty transaction
            view.dispatch({});
          }
        });
      } else {
        console.log('[MarkdownEditor] Image cached, creating widget:', src);
        // Replace the markdown with an image widget
        const deco = Decoration.replace({
          widget: new ImageWidget(cachedDataUrl, alt),
        });
        widgets.push(deco.range(from, to));
      }
    }

    console.log('[MarkdownEditor] Found', matchCount, 'images,', widgets.length, 'widgets created');
    return Decoration.set(widgets);
  }

  // ViewPlugin to manage image decorations
  const imagePlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        console.log('[MarkdownEditor] ImagePlugin constructor');
        this.decorations = createImageDecorations(view);
      }

      update(update: ViewUpdate) {
        // Always update decorations, even for empty transactions
        // This is needed because images load asynchronously
        console.log('[MarkdownEditor] ImagePlugin update, docChanged:', update.docChanged);
        this.decorations = createImageDecorations(update.view);
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );

  // Handle image paste
  function handleImagePaste(event: ClipboardEvent, view: EditorView): boolean {
    console.log('[MarkdownEditor] Paste event triggered');

    if (!filePath) {
      console.log('[MarkdownEditor] No filePath, skipping');
      return false;
    }

    const items = event.clipboardData?.items;
    if (!items) {
      console.log('[MarkdownEditor] No clipboard items');
      return false;
    }

    console.log('[MarkdownEditor] Checking clipboard items:', items.length);

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      console.log('[MarkdownEditor] Item', i, 'type:', item.type);

      // Check if it's an image
      if (item.type.startsWith('image/')) {
        console.log('[MarkdownEditor] Image detected!');
        event.preventDefault();

        const file = item.getAsFile();
        if (!file) {
          console.log('[MarkdownEditor] Could not get file');
          continue;
        }

        console.log('[MarkdownEditor] Processing image file:', file.name, file.type, file.size);

        // Read image as base64
        const reader = new FileReader();
        reader.onload = async (e) => {
          console.log('[MarkdownEditor] FileReader loaded');
          const base64Data = e.target?.result as string;
          const base64 = base64Data.split(',')[1]; // Remove data:image/...;base64, prefix

          // Generate unique filename
          const timestamp = Date.now();
          const ext = file.type.split('/')[1];
          const filename = `image-${timestamp}.${ext}`;

          console.log('[MarkdownEditor] Saving image:', filename);

          try {
            // Save image via Tauri command
            const { invoke } = await import('@tauri-apps/api/core');
            const relativePath = await invoke<string>('save_pasted_image', {
              notePath: filePath,
              imageData: base64,
              imageName: filename
            });

            console.log('[MarkdownEditor] Image saved to:', relativePath);

            // Insert markdown image syntax at cursor position
            const cursor = view.state.selection.main.head;
            const imageMarkdown = `![image](${relativePath})`;

            view.dispatch({
              changes: {
                from: cursor,
                insert: imageMarkdown
              },
              selection: { anchor: cursor + imageMarkdown.length }
            });

            console.log('[MarkdownEditor] Image pasted and saved:', relativePath);
          } catch (error) {
            console.error('[MarkdownEditor] Failed to save pasted image:', error);
          }
        };
        reader.readAsDataURL(file);

        return true;
      }
    }

    console.log('[MarkdownEditor] No image found in clipboard');
    return false;
  }

  // Use onMount instead of $effect to prevent re-initialization
  onMount(() => {
    if (!editorContainer) return;

    const startState = EditorState.create({
      doc: content,
      extensions: [
        // basicSetup without lineNumbers
        highlightSpecialChars(),
        history(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        syntaxHighlighting(defaultHighlightStyle),
        bracketMatching(),
        // lineNumbers(), // 줄번호 제거
        highlightActiveLine(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping, // 자동 줄바꿈 활성화
        imagePlugin, // Add image widget plugin
        EditorView.domEventHandlers({
          paste: (event, view) => {
            console.log('[MarkdownEditor] CodeMirror paste handler triggered!');
            return handleImagePaste(event, view);
          },
          keydown: (event, view) => {
            // Detect Cmd+V or Ctrl+V
            if ((event.metaKey || event.ctrlKey) && event.key === 'v') {
              event.preventDefault(); // Prevent default paste

              if (!filePath) {
                return true;
              }

              // Handle paste asynchronously without blocking
              (async () => {
                // Try to read image from clipboard using native Tauri API (no permission popup!)
                try {
                  const { invoke } = await import('@tauri-apps/api/core');
                  const relativePath = await invoke<string | null>('read_clipboard_image', {
                    notePath: filePath
                  });

                  if (relativePath) {
                    // Image was found and saved
                    console.log('[MarkdownEditor] Image pasted from clipboard:', relativePath);

                    const cursor = view.state.selection.main.head;
                    const imageMarkdown = `![image](${relativePath})`;

                    view.dispatch({
                      changes: {
                        from: cursor,
                        insert: imageMarkdown
                      },
                      selection: { anchor: cursor + imageMarkdown.length }
                    });

                    return;
                  }
                } catch (error) {
                  console.error('[MarkdownEditor] Native clipboard read failed:', error);
                }

                // No image found, paste text using clipboard-manager plugin
                try {
                  const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
                  const text = await readText();

                  if (text) {
                    const selection = view.state.selection.main;
                    const from = selection.from;
                    const to = selection.to;

                    view.dispatch({
                      changes: { from, to, insert: text },
                      selection: { anchor: from + text.length }
                    });

                    console.log('[MarkdownEditor] Text pasted from clipboard');
                  }
                } catch (error) {
                  console.error('[MarkdownEditor] Failed to paste text:', error);
                }
              })();

              return true; // We handled it
            }

            return false; // Don't prevent default for other keys
          }
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newContent = update.state.doc.toString();
            const event = new CustomEvent('change', { detail: newContent });
            onchange?.(event);
          }
        }),
        EditorView.theme({
          '&': {
            height: '100%',
            fontSize: `${fontSize}px`,
          },
          '&.cm-focused': {
            outline: 'none',
          },
          '.cm-content': {
            fontFamily: 'Monaco, Menlo, monospace',
            color: textColor,
          },
          '.cm-scroller': {
            overflow: 'auto',
          },
        }),
      ],
    });

    editorView = new EditorView({
      state: startState,
      parent: editorContainer,
    });

    console.log('[MarkdownEditor] Editor initialized');

    // Notify parent that editor is ready
    if (oneditorready) {
      oneditorready(editorView);
    }

    return () => {
      if (editorView) {
        editorView.destroy();
        editorView = null;
      }
    };
  });

  // Update editor content when prop changes
  $effect(() => {
    if (editorView && content !== editorView.state.doc.toString()) {
      const transaction = editorView.state.update({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: content
        }
      });
      editorView.dispatch(transaction);
    }
  });

  // Note: textColor and fontSize changes require component re-mount
  // This is handled automatically by {#key mode} in parent component
</script>

<div class="editor-container" bind:this={editorContainer}></div>

<style>
  .editor-container {
    height: 100%;
    width: 100%;
  }

  :global(.cm-editor) {
    height: 100%;
  }
</style>
