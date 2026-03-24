<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label: string;
    icon: string;
    iconColor?: string;
    value: string;
    sub?: string;
    extra?: Snippet;
  }

  let { label, icon, iconColor = 'text-slate-500', value, sub, extra }: Props = $props();
</script>

<div class="bg-white dark:bg-[#161e27] rounded-lg border border-slate-200 dark:border-[#2a3441] p-3">
  <div class="flex items-center justify-between mb-2">
    <h3 class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</h3>
    <span class="material-symbols-outlined text-[16px] {iconColor}">{icon}</span>
  </div>
  <div class="flex items-baseline gap-2">
    <span class="text-xl font-bold font-display text-slate-900 dark:text-white">{value}</span>
    {#if sub}
      <span class="text-[10px] font-mono text-slate-500">{sub}</span>
    {/if}
  </div>
  {#if extra}
    {@render extra()}
  {/if}
</div>
