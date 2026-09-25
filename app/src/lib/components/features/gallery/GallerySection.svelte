<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Icon } from '$lib/components/ui';

  interface Props {
    /** Anchor id — must match the entry in the page's section index. */
    id: string;
    title: string;
    description?: string;
    /** Optional header link, e.g. to the component's own gallery page. */
    link?: { href: string; label: string };
    children: Snippet;
  }

  let { id, title, description, link, children }: Props = $props();
</script>

<section {id} class="scroll-mt-4">
  <div class="rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] overflow-hidden">
    <div class="px-4 py-3 border-b border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#1f2937]/50 flex items-start justify-between gap-3">
      <div>
        <h2 class="text-card-title text-sm">{title}</h2>
        {#if description}
          <p class="text-caption mt-0.5">{description}</p>
        {/if}
      </div>
      {#if link}
        <a href={link.href} class="shrink-0 inline-flex items-center gap-1 mt-0.5 text-nav text-[#137fec] hover:underline">
          {link.label}
          <Icon name="arrow_forward" class="text-[12px]" />
        </a>
      {/if}
    </div>
    <div class="p-4 flex flex-col gap-5">
      {@render children()}
    </div>
  </div>
</section>
