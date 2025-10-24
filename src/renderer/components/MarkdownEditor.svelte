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
    onchange?: (event: CustomEvent<string>) => void;
    oneditorready?: (view: EditorView) => void;
  }

  let { content, textColor, fontSize = 11, onchange, oneditorready }: Props = $props();

  let editorContainer = $state<HTMLDivElement>();
  let editorView: EditorView | null = null;

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
