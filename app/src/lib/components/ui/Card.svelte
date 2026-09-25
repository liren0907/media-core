<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    padding?: 'normal' | 'tight' | 'none';
    children: Snippet;
  }

  let { padding = 'normal', children }: Props = $props();

  const padClass = { normal: 'p-3', tight: 'p-2', none: '' };
</script>

<div class="bg-white dark:bg-[#161e27] rounded-lg border border-slate-200 dark:border-[#2a3441] {padClass[padding]}">
  {@render children()}
</div>
