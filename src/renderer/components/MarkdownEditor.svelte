<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorView, keymap, lineNumbers } from '@codemirror/view';
  import { EditorState, Compartment } from '@codemirror/state';
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
