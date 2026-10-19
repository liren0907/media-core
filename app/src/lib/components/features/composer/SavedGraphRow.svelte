<script lang="ts" module>
  /** "14:02" for today, "Oct 18 14:02" for any other day. */
  export function savedTime(ms: number): string {
    const date = new Date(ms);
    const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (date.toDateString() === new Date().toDateString()) return time;
    return `${date.toLocaleDateString([], { month: 'short', day: 'numeric' })} ${time}`;
  }
</script>

<script lang="ts">
  /**
   * One row of the Open ▾ list.
   *
   * A component of its own so the gallery can show the row /composer really uses:
   * in a browser the list never loads, so the gallery is the only place to see it.
   * For the same reason it carries its own button styling instead of the editor's
   * `tool-btn`, which exists only while GraphEditor is loaded.
   */
  import { Icon } from '$lib/components/ui';
  import type { GraphListItem } from '$lib/types';

  interface Props {
    item: GraphListItem;
    /** The graph on the canvas was opened from (or last saved to) this file. */
    open: boolean;
    onopen: () => void;
    ondelete: () => void;
  }

  let { item, open, onopen, ondelete }: Props = $props();
</script>

<div
  class="flex items-center gap-1 pr-1 {open
    ? 'bg-blue-50 dark:bg-[#137fec]/10'
    : 'hover:bg-slate-50 dark:hover:bg-[#1f2937]'}"
>
  <button
    disabled={!!item.broken}
    onclick={onopen}
    title={item.broken ?? item.reason ?? ''}
    class="flex-1 min-w-0 text-left px-2 py-1.5 flex flex-col gap-0.5 {item.broken
      ? 'cursor-not-allowed'
      : ''}"
  >
    <span class="flex items-center gap-1 text-[11px] font-medium text-slate-900 dark:text-white">
      <span class="truncate {item.broken ? 'text-slate-400' : ''}">
        {item.broken ? `${item.id.slice(0, 8)}.json` : item.label}
      </span>
      {#if !item.broken && !item.valid}
        <Icon name="warning" class="text-[12px] shrink-0 text-amber-500" />
      {/if}
      {#if open}
        <span class="text-[9px] font-bold text-[#137fec] shrink-0">OPEN</span>
      {/if}
    </span>
    <span class="text-[10px] text-slate-500">
      {#if item.broken}
        Can't read · {savedTime(item.modifiedMs)}
      {:else}
        {item.nodeCount} node{item.nodeCount === 1 ? '' : 's'} · {savedTime(item.modifiedMs)}
      {/if}
    </span>
  </button>
  <button
    onclick={ondelete}
    class="shrink-0 flex items-center px-2 py-1 rounded text-slate-500 transition-colors
           hover:bg-slate-100 dark:hover:bg-gray-800"
    title="Delete"
    aria-label="Delete {item.broken ? 'unreadable file' : item.label}"
  >
    <Icon name="delete" class="text-[13px]" />
  </button>
</div>
