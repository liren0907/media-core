<script lang="ts">
  import type { Snippet } from 'svelte';
  import CodeBlock from './CodeBlock.svelte';

  interface Props {
    /** Optional caption for this specific variant. */
    label?: string;
    /** Optional usage snippet rendered below the live demo. */
    code?: string;
    children: Snippet;
  }

  let { label, code, children }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  {#if label}
    <span class="text-stat-label">{label}</span>
  {/if}
  <!-- Live demo "stage": a recessed surface so bordered components read clearly -->
  <div class="rounded-lg bg-slate-100/70 dark:bg-[#0d1117] p-4 flex flex-wrap items-center gap-3">
    {@render children()}
  </div>
  {#if code}
    <CodeBlock {code} />
  {/if}
</div>
