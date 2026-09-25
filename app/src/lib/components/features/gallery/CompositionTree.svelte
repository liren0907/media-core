<script lang="ts" module>
  /** One node in a page's component-composition tree. */
  export interface CompositionNode {
    /** Component name as written in the source. */
    name: string;
    /** That component's own gallery page, when it has one. */
    href?: string;
    /** Usage context — count, source location, which snippet it fills. */
    note?: string;
    children?: CompositionNode[];
  }
</script>

<script lang="ts">
  import { Icon } from '$lib/components/ui';

  interface Props {
    nodes: CompositionNode[];
  }

  let { nodes }: Props = $props();
</script>

{#snippet tree(items: CompositionNode[])}
  <ul class="flex flex-col gap-1.5">
    {#each items as node, i (i)}
      <li class="flex flex-col gap-1.5">
        <div class="flex items-center gap-2 flex-wrap">
          {#if node.href}
            <a
              href={node.href}
              title="Open {node.name} in the gallery"
              class="inline-flex items-center gap-1 rounded-md border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] px-2 py-0.5 text-code text-[#137fec] hover:border-[#137fec] transition-colors"
            >
              {node.name}
              <Icon name="arrow_forward" class="text-[11px]" />
            </a>
          {:else}
            <span class="inline-flex items-center rounded-md border border-slate-200 dark:border-[#2a3441] bg-slate-100/70 dark:bg-[#0d1117] px-2 py-0.5 text-code text-slate-600 dark:text-slate-300">
              {node.name}
            </span>
          {/if}
          {#if node.note}<span class="text-meta">{node.note}</span>{/if}
        </div>
        {#if node.children?.length}
          <div class="ml-3 border-l border-slate-200 dark:border-[#2a3441] pl-3">
            {@render tree(node.children)}
          </div>
        {/if}
      </li>
    {/each}
  </ul>
{/snippet}

{@render tree(nodes)}
