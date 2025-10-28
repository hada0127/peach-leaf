<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, keymap, lineNumbers, Decoration, type DecorationSet, ViewPlugin, type ViewUpdate, WidgetType } from '@codemirror/view';
  import { EditorState, Compartment, type Range, RangeSet } from '@codemirror/state';
  import { markdown } from '@codemirror/lang-markdown';
  import { defaultKeymap, history, historyKeymap, deleteCharBackward } from '@codemirror/commands';
  import { highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, highlightActiveLine } from '@codemirror/view';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, syntaxTree } from '@codemirror/language';

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

  // Store selected image info
  let selectedImageElement: HTMLElement | null = null;
  let selectedImagePosition: { from: number; to: number } | null = null;

  // Widget for displaying images inline with resize handles
  class ImageWidget extends WidgetType {
    src: string;
    alt: string;
    width: number | null;
    from: number;
    to: number;

    constructor(src: string, alt: string, width: number | null, from: number, to: number) {
      super();
      this.src = src;
      this.alt = alt;
      this.width = width;
      this.from = from;
      this.to = to;
    }

    eq(other: ImageWidget) {
      return other.src === this.src && other.alt === this.alt && other.width === this.width;
    }

    toDOM(view: EditorView) {
      const container = document.createElement('div');
      container.className = 'cm-image-container';
      container.contentEditable = 'false';
      container.style.position = 'relative';
      container.style.display = 'inline-block';
      container.style.margin = '8px 0';

      const img = document.createElement('img');
      img.src = this.src;
      img.alt = this.alt;
      img.style.display = 'block';
      img.style.borderRadius = '4px';
      img.style.height = 'auto';

      if (this.width) {
        // 설정한 사이즈가 있으면 그 크기로
        img.style.width = `${this.width}px`;
      }
      // width가 없으면 원본 크기로 (max-width 없음)

      // Click to select
      container.addEventListener('click', (e) => {
        e.stopPropagation();
        e.preventDefault();
        console.log('[MarkdownEditor] Image clicked, selecting:', this.from, this.to);
        selectImage(container, this.from, this.to, view);
      });

      container.appendChild(img);
      return container;
    }

    ignoreEvent() {
      return false; // Allow click events
    }
  }

  // Select an image
  function selectImage(element: HTMLElement, from: number, to: number, view: EditorView) {
    console.log('[MarkdownEditor] selectImage called:', from, to);

    // Deselect previous
    if (selectedImageElement && selectedImageElement !== element) {
      console.log('[MarkdownEditor] Deselecting previous image');
      selectedImageElement.classList.remove('selected');
      removeResizeHandles(selectedImageElement);
    }

    selectedImageElement = element;
    selectedImagePosition = { from, to };
    element.classList.add('selected');
    console.log('[MarkdownEditor] Image selected, position:', selectedImagePosition);

    // Hide cursor by setting selection to empty range at image position
    // This keeps focus but hides the cursor
    view.dispatch({
      selection: { anchor: from, head: from },
      scrollIntoView: false
    });

    addResizeHandles(element, view, from, to);
  }

  // Deselect image
  function deselectImage() {
    if (selectedImageElement) {
      selectedImageElement.classList.remove('selected');
      removeResizeHandles(selectedImageElement);
      selectedImageElement = null;
      selectedImagePosition = null;
    }
  }

  // Add resize handles
  function addResizeHandles(container: HTMLElement, view: EditorView, from: number, to: number) {
    const handles = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
    const img = container.querySelector('img') as HTMLImageElement;

    handles.forEach(position => {
      const handle = document.createElement('div');
      handle.className = `resize-handle resize-handle-${position}`;
      handle.style.position = 'absolute';
      handle.style.width = '8px';
      handle.style.height = '8px';
      handle.style.background = '#4A90E2';
      handle.style.border = '1px solid white';
      handle.style.borderRadius = '50%';
      handle.style.cursor = `${position}-resize`;
      handle.style.zIndex = '10';

      // Position handles
      if (position.includes('n')) handle.style.top = '-4px';
      if (position.includes('s')) handle.style.bottom = '-4px';
      if (position.includes('w')) handle.style.left = '-4px';
      if (position.includes('e')) handle.style.right = '-4px';
      if (!position.includes('n') && !position.includes('s')) handle.style.top = 'calc(50% - 4px)';
      if (!position.includes('w') && !position.includes('e')) handle.style.left = 'calc(50% - 4px)';

      let startX: number, startY: number, startWidth: number, startHeight: number, aspectRatio: number;

      handle.addEventListener('mousedown', (e) => {
        e.stopPropagation();
        e.preventDefault();

        startX = e.clientX;
        startY = e.clientY;
        startWidth = img.offsetWidth;
        startHeight = img.offsetHeight;
        aspectRatio = startWidth / startHeight;

        const onMouseMove = (moveEvent: MouseEvent) => {
          const deltaX = moveEvent.clientX - startX;
          const deltaY = moveEvent.clientY - startY;

          let newWidth = startWidth;

          // Calculate new width based on handle position
          if (position.includes('e')) {
            newWidth = startWidth + deltaX;
          } else if (position.includes('w')) {
            newWidth = startWidth - deltaX;
          } else if (position.includes('n') || position.includes('s')) {
            newWidth = startWidth + (deltaY * aspectRatio * (position.includes('n') ? -1 : 1));
          }

          // Maintain aspect ratio and minimum size
          newWidth = Math.max(50, Math.min(newWidth, container.parentElement!.offsetWidth));

          img.style.width = `${newWidth}px`;
          img.style.height = 'auto';
        };

        const onMouseUp = () => {
          document.removeEventListener('mousemove', onMouseMove);
          document.removeEventListener('mouseup', onMouseUp);

          // Update markdown with new width
          const newWidth = img.offsetWidth;
          updateImageWidth(view, from, to, newWidth);
        };

        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
      });

      container.appendChild(handle);
    });
  }

  // Remove resize handles
  function removeResizeHandles(container: HTMLElement) {
    const handles = container.querySelectorAll('.resize-handle');
    handles.forEach(handle => handle.remove());
  }

  // Update image width in markdown
  function updateImageWidth(view: EditorView, from: number, to: number, width: number) {
    const text = view.state.doc.sliceString(from, to);
    const match = text.match(/!\[([^\]]*)\]\(([^)]+)\)/);

    if (match) {
      const alt = match[1];
      const src = match[2];
      // Store width as a comment in markdown: ![alt](src)<!-- width:123 -->
      const newText = `![${alt}](${src})<!-- width:${Math.round(width)} -->`;

      view.dispatch({
        changes: { from, to, insert: newText }
      });
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

  // Create decorations for heading sizes
  function createHeadingDecorations(view: EditorView): DecorationSet {
    const decorations: Range<Decoration>[] = [];
    const tree = syntaxTree(view.state);

    tree.iterate({
      enter: (node) => {
        const nodeName = node.name;
        let fontSize = '';

        // Match heading levels - exact same as preview
        if (nodeName === 'ATXHeading1' || nodeName === 'SetextHeading1') {
          fontSize = '1.8em';
        } else if (nodeName === 'ATXHeading2' || nodeName === 'SetextHeading2') {
          fontSize = '1.5em';
        } else if (nodeName === 'ATXHeading3') {
          fontSize = '1.3em';
        } else if (nodeName === 'ATXHeading4') {
          fontSize = '1.1em';
        } else if (nodeName === 'ATXHeading5') {
          fontSize = '1.0em';
        } else if (nodeName === 'ATXHeading6') {
          fontSize = '0.9em';
        }

        if (fontSize) {
          const deco = Decoration.line({
            attributes: {
              style: `font-size: ${fontSize};`
            }
          });
          decorations.push(deco.range(node.from));
        }
      }
    });

    return Decoration.set(decorations);
  }

  // ViewPlugin to manage heading decorations
  const headingPlugin = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = createHeadingDecorations(view);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = createHeadingDecorations(update.view);
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );

  // Create decorations for images
  function createImageDecorations(view: EditorView): DecorationSet {
    const widgets: Range<Decoration>[] = [];
    // Match ![alt](src) optionally followed by <!-- width:123 -->
    const imageRegex = /!\[([^\]]*)\]\((\.\/[^)]+)\)(?:<!--\s*width:(\d+)\s*-->)?/g;
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
      const width = match[3] ? parseInt(match[3]) : null;

      console.log('[MarkdownEditor] Found image:', { src, alt, width, from, to });

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
          widget: new ImageWidget(cachedDataUrl, alt, width, from, to),
          inclusive: true,
          block: false,
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
      lastCacheSize: number;

      constructor(view: EditorView) {
        console.log('[MarkdownEditor] ImagePlugin constructor');
        this.decorations = createImageDecorations(view);
        this.lastCacheSize = imageCache.size;
      }

      update(update: ViewUpdate) {
        const currentCacheSize = imageCache.size;
        const cacheChanged = currentCacheSize !== this.lastCacheSize;

        console.log('[MarkdownEditor] ImagePlugin update, docChanged:', update.docChanged, 'cacheChanged:', cacheChanged, 'cacheSize:', currentCacheSize);

        // Update if document changed OR cache changed (added/removed images)
        if (update.docChanged || cacheChanged) {
          console.log('[MarkdownEditor] Rebuilding image decorations');
          this.decorations = createImageDecorations(update.view);
          this.lastCacheSize = currentCacheSize;
        } else {
          console.log('[MarkdownEditor] Skipping rebuild - no changes detected');
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );

  // Create a simple marker class for atomic ranges
  class AtomicMarker {
    constructor() {}
  }

  // Provide atomic ranges separately for single-unit deletion
  const imageAtomicRanges = EditorView.atomicRanges.of(view => {
    const ranges: Range<AtomicMarker>[] = [];
    const imageRegex = /!\[([^\]]*)\]\((\.\/[^)]+)\)/g;
    const text = view.state.doc.toString();

    let match;
    while ((match = imageRegex.exec(text)) !== null) {
      const from = match.index;
      const to = from + match[0].length;
      ranges.push({ from, to, value: new AtomicMarker() });
    }

    return ranges.length > 0 ? RangeSet.of(ranges, true) : RangeSet.empty;
  });

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

    console.log('[MarkdownEditor] onMount called, filePath:', filePath);

    // Custom keymap for image selection (must be before defaultKeymap)
    const imageSelectionKeymap = keymap.of([
      {
        key: 'Delete',
        run: (view) => {
          if (selectedImagePosition) {
            console.log('[MarkdownEditor] Delete key pressed with selected image');
            const { from, to } = selectedImagePosition;
            const text = view.state.doc.sliceString(from, to);
            const match = text.match(/!\[([^\]]*)\]\((\.\/[^)]+)\)/);

            if (match) {
              const src = match[2];
              console.log('[MarkdownEditor] Removing image from cache:', src);
              imageCache.delete(src);
            }

            view.dispatch({
              changes: { from, to, insert: '' }
            });

            deselectImage();
            return true;
          }
          return false;
        }
      },
      {
        key: 'Backspace',
        run: (view) => {
          if (selectedImagePosition) {
            console.log('[MarkdownEditor] Backspace key pressed with selected image');
            const { from, to } = selectedImagePosition;
            const text = view.state.doc.sliceString(from, to);
            const match = text.match(/!\[([^\]]*)\]\((\.\/[^)]+)\)/);

            if (match) {
              const src = match[2];
              console.log('[MarkdownEditor] Removing image from cache:', src);
              imageCache.delete(src);
            }

            view.dispatch({
              changes: { from, to, insert: '' }
            });

            deselectImage();
            return true;
          }
          return false;
        }
      }
    ]);

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
        imageSelectionKeymap, // Image selection keymap BEFORE default keymap
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping, // 자동 줄바꿈 활성화
        headingPlugin, // Add heading size decorations
        imagePlugin, // Add image widget plugin
        imageAtomicRanges, // Make images deletable as single unit
        EditorView.domEventHandlers({
          paste: (event, view) => {
            console.log('[MarkdownEditor] CodeMirror paste handler triggered!');
            return handleImagePaste(event, view);
          },
          mousedown: (event, view) => {
            // Deselect image when clicking on editor content (not on image)
            const target = event.target as HTMLElement;
            if (!target.closest('.image-container')) {
              deselectImage();
            }
            return false;
          },
          keydown: (event, view) => {
            // Handle Backspace when cursor is right after an image (not selected)
            if (event.key === 'Backspace' && !selectedImagePosition) {
              const cursor = view.state.selection.main.head;
              const text = view.state.doc.toString();

              // Check if cursor is right after an image markdown (including optional width comment)
              const imageRegex = /!\[([^\]]*)\]\((\.\/[^)]+)\)(?:<!--\s*width:(\d+)\s*-->)?$/;
              const textBeforeCursor = text.substring(0, cursor);
              const match = textBeforeCursor.match(imageRegex);

              if (match) {
                event.preventDefault();
                const imageLength = match[0].length;
                const from = cursor - imageLength;
                const to = cursor;
                const src = match[2];

                console.log('[MarkdownEditor] Deleting image at cursor:', from, to);
                console.log('[MarkdownEditor] Removing image from cache:', src);
                imageCache.delete(src);

                view.dispatch({
                  changes: { from, to, insert: '' }
                });

                return true;
              }
            }

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
                    const doc = view.state.doc;
                    const line = doc.lineAt(cursor);

                    // Insert at the end of current line with newline
                    const insertPos = line.to;
                    const imageMarkdown = `\n![image](${relativePath})`;

                    view.dispatch({
                      changes: {
                        from: insertPos,
                        insert: imageMarkdown
                      },
                      selection: { anchor: insertPos + imageMarkdown.length }
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
          },
          click: (event, view) => {
            // Deselect image when clicking outside
            const target = event.target as HTMLElement;
            const imageContainer = target.closest('.cm-image-container');

            if (!imageContainer) {
              console.log('[MarkdownEditor] Click outside image, deselecting');
              deselectImage();
            } else {
              console.log('[MarkdownEditor] Click on image container');
            }
            return false;
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
            fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
            color: textColor,
            lineHeight: '1.6',
            paddingLeft: '16px',
            paddingRight: '16px',
          },
          '.cm-line': {
            lineHeight: '1.6 !important',
            padding: '0 !important',
          },
          '.cm-scroller': {
            overflow: 'auto',
          },
          // Remove borders from all markdown elements
          '.cm-line .ͼe': {
            borderBottom: 'none !important',
          },
          '.cm-line .ͼf': {
            borderBottom: 'none !important',
          },
          '.cm-line .ͼg': {
            borderBottom: 'none !important',
          },
          '.cm-line .ͼh': {
            borderBottom: 'none !important',
          },
          '.cm-line .ͼi': {
            borderBottom: 'none !important',
          },
          '.cm-line .ͼj': {
            borderBottom: 'none !important',
          },
          // Ensure all markdown syntax elements have same font size as body text
          '.cm-em, .cm-strong, .cm-strikethrough, .cm-link, .cm-code': {
            fontSize: 'inherit !important',
          },
          // Ensure heading markers (#, ##, etc) don't affect size
          '.cm-heading-mark': {
            fontSize: 'inherit !important',
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

  :global(.cm-image-container) {
    position: relative;
    display: inline-block;
    margin: 8px 0;
    vertical-align: middle;
  }

  :global(.cm-image-container.selected) {
    outline: 2px solid #4A90E2;
    outline-offset: 2px;
  }

  :global(.resize-handle) {
    position: absolute;
    width: 8px;
    height: 8px;
    background: #4A90E2;
    border: 1px solid white;
    border-radius: 50%;
    z-index: 10;
  }

  :global(.cm-image-container img) {
    display: block;
    border-radius: 4px;
    height: auto;
  }

  /* Remove underlines from headings */
  :global(.cm-line) {
    border-bottom: none !important;
  }

  :global(.cm-header-1),
  :global(.cm-header-2),
  :global(.cm-header-3),
  :global(.cm-header-4),
  :global(.cm-header-5),
  :global(.cm-header-6) {
    border-bottom: none !important;
    text-decoration: none !important;
  }

  :global(.ͼe),
  :global(.ͼf),
  :global(.ͼg),
  :global(.ͼh),
  :global(.ͼi),
  :global(.ͼj) {
    border-bottom: none !important;
    text-decoration: none !important;
  }

  /* Remove underline from all markdown heading classes */
  :global(.ͼ5),
  :global(.ͼ6),
  :global(.ͼ7),
  :global(.ͼ8),
  :global(.ͼ9),
  :global(.ͼa),
  :global(.ͼb),
  :global(.ͼc),
  :global(.ͼd),
  :global(.ͼe) {
    text-decoration: none !important;
  }

  /* Normalize font sizes for inline elements only (bold, italic, code, etc.) */
  /* Don't apply to heading lines which are controlled by headingPlugin */
  :global(.cm-line:not([style*="font-size"]) span.ͼ9) {
    font-size: inherit !important;
  }
</style>
