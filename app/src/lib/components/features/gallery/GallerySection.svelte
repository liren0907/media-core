<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Anchor id — must match the entry in the page's section index. */
    id: string;
    title: string;
    description?: string;
    children: Snippet;
  }

  let { id, title, description, children }: Props = $props();
</script>

<section {id} class="scroll-mt-4">
  <div class="rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] overflow-hidden">
    <div class="px-4 py-3 border-b border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#1f2937]/50">
      <h2 class="text-card-title text-sm">{title}</h2>
      {#if description}
        <p class="text-caption mt-0.5">{description}</p>
      {/if}
    </div>
    <div class="p-4 flex flex-col gap-5">
      {@render children()}
    </div>
  </div>
</section>
