<script lang="ts">
  /**
   * One card. Its size comes from layout.ts and is written as an inline style — the
   * coordinate engine assumes exact heights, so CSS must not hold a second opinion.
   */
  import { Icon } from '$lib/components/ui';
  import type { GraphCategory } from '$lib/types';

  interface Props {
    label: string;
    category: GraphCategory;
    summary: string;
    width: number;
    height: number;
    selected: boolean;
    /** Currently executing — the run progress highlight. */
    active: boolean;
    requiresFfmpeg: boolean;
    onselect: () => void;
  }

  let {
    label,
    category,
    summary,
    width,
    height,
    selected,
    active,
    requiresFfmpeg,
    onselect,
  }: Props = $props();

  // Tailwind cannot see class names composed at run time, so every variant is spelled
  // out in full here. (The app has been bitten by this before: a `${color}/20` built
  // in a template produced a utility that was never generated, and the element came
  // out transparent.)
  const ACCENT: Record<GraphCategory, string> = {
    source: 'bg-emerald-500',
    extract: 'bg-sky-500',
    analysis: 'bg-violet-500',
    convert: 'bg-amber-500',
    annotate: 'bg-pink-500',
    process: 'bg-teal-500',
    report: 'bg-slate-500',
  };
</script>

<button
  onclick={onselect}
  style="width: {width}px; height: {height}px;"
  class="node group text-left rounded-md border bg-white dark:bg-[#161e27] overflow-hidden transition-colors
    {selected
    ? 'border-[#137fec] ring-1 ring-[#137fec]'
    : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}
    {active ? 'node-active' : ''}"
>
  <div class="flex items-center gap-1.5 px-2 h-[34px]">
    <span class="shrink-0 w-1.5 h-1.5 rounded-full {ACCENT[category]}"></span>
    <span class="flex-1 min-w-0 truncate text-[11px] font-medium text-slate-900 dark:text-white">
      {label}
    </span>
    {#if requiresFfmpeg}
      <Icon name="warning" class="shrink-0 text-[12px] text-amber-500" />
    {/if}
  </div>

  {#if summary}
    <!-- Fixed height, never wraps: params are not a relayout trigger, so a second line
         here would leave every connection pointing at the wrong place. -->
    <div
      class="px-2 h-[20px] flex items-center border-t border-slate-100 dark:border-[#232c38]
             text-[10px] text-slate-500 dark:text-slate-400 whitespace-nowrap overflow-hidden text-ellipsis"
    >
      <span class="truncate">{summary}</span>
    </div>
  {/if}
</button>

<style>
  /* Class + timeout rather than a CSS animation: HMR and a hidden browser pane both
     freeze animations mid-frame, which leaves the highlight stuck on. */
  .node-active {
    border-color: #137fec;
    box-shadow: 0 0 0 3px rgb(19 127 236 / 0.25);
  }
</style>
