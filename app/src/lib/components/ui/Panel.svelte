<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    icon?: string;
    actions?: Snippet;
    children: Snippet;
  }

  let { title, icon, actions, children }: Props = $props();
</script>

<div class="rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] overflow-hidden">
  <div class="px-4 py-2 border-b border-slate-200 dark:border-[#2a3441] flex justify-between items-center bg-slate-50 dark:bg-[#1f2937]/50">
    <div class="flex items-center gap-2">
      {#if icon}
        <span class="material-symbols-outlined text-slate-500 text-[16px]">{icon}</span>
      {/if}
      <h3 class="text-section-title">{title}</h3>
    </div>
    {#if actions}
      {@render actions()}
    {/if}
  </div>
  {@render children()}
</div>
