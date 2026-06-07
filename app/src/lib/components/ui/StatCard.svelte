<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

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
    <h3 class="text-stat-label">{label}</h3>
    <Icon name={icon} class="text-[16px] {iconColor}" />
  </div>
  <div class="flex items-baseline gap-2">
    <span class="text-stat-value">{value}</span>
    {#if sub}
      <span class="text-meta">{sub}</span>
    {/if}
  </div>
  {#if extra}
    {@render extra()}
  {/if}
</div>
