// Svelte 5 Runes global type declarations
// This file helps TypeScript recognize Svelte 5's runes

declare global {
  /**
   * Svelte 5 $state rune - creates reactive state
   */
  function $state<T>(initial: T): T;
  function $state<T>(): T | undefined;

  /**
   * Svelte 5 $derived rune - creates derived/computed values
   */
  function $derived<T>(expression: T): T;

  /**
   * Svelte 5 $effect rune - runs side effects
   */
  function $effect(fn: () => void | (() => void)): void;

  /**
   * Svelte 5 $props rune - declares component props
   */
  function $props<T>(): T;

  /**
   * Svelte 5 $bindable rune - creates bindable props
   */
  function $bindable<T>(initial?: T): T;

  /**
   * Svelte 5 $inspect rune - debugging tool
   */
  function $inspect(...values: any[]): void;
}

export {};
