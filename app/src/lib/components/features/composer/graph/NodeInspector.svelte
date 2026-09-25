<script lang="ts">
  /**
   * Right-hand inspector.
   *
   * Every node is selectable, including sources and outputs — a canvas where only
   * some cards respond to a click is a guessing game. When there is nothing to tune
   * it says so.
   *
   * This is also where extra inputs are wired. Edges are never dragged: a port that
   * still needs a supplier offers a list of the nodes that produce it.
   */
  import { Icon } from '$lib/components/ui';
  import type { GraphSlot } from '$lib/types';
  import type { GraphController } from '../graph-controller.svelte';
  import ParamField from './ParamField.svelte';

  interface Props {
    graph: GraphController;
    selectedId: string;
    onremove: (nodeId: string) => void;
  }

  let { graph, selectedId, onremove }: Props = $props();

  const SLOT_LABEL: Record<GraphSlot, string> = {
    source: 'Media source',
    metadata: 'Metadata',
    analysis: 'Analysis report',
    extractedFrames: 'In-memory frames',
    hlsResult: 'HLS output',
    annotationResult: 'Annotation output',
    videoProcessResult: 'Frames on disk',
    processResult: 'File processing output',
  };

  const meta = $derived(graph.metaOfNode(selectedId));
  const values = $derived(graph.paramsOf(selectedId));
  const parents = $derived(graph.parentsOf(selectedId));

  /** Everything upstream of `selectedId`, used to tell a satisfied port from an open one. */
  const upstreamSlots = $derived.by(() => {
    const found = new Set<GraphSlot>();
    const seen = new Set<string>();
    const stack = [...parents];
    while (stack.length) {
      const id = stack.pop()!;
      if (seen.has(id)) continue;
      seen.add(id);
      for (const write of graph.metaOfNode(id)?.writes ?? []) found.add(write.slot);
      stack.push(...graph.parentsOf(id));
    }
    return found;
  });

  function descendants(nodeId: string): Set<string> {
    const out = new Set<string>();
    const stack = [nodeId];
    while (stack.length) {
      const id = stack.pop()!;
      for (const child of graph.childrenOf(id)) {
        if (!out.has(child)) {
          out.add(child);
          stack.push(child);
        }
      }
    }
    return out;
  }

  /** Nodes that write `slot` and can legally be wired in without making a cycle. */
  function candidatesFor(slot: GraphSlot): { id: string; label: string }[] {
    const blocked = descendants(selectedId);
    return graph.nodes
      .filter((node) => node.id !== selectedId)
      .filter((node) => !blocked.has(node.id))
      .filter((node) => !parents.includes(node.id))
      .filter((node) => graph.metaOf(node.ref)?.writes.some((w) => w.slot === slot))
      .map((node) => ({ id: node.id, label: graph.labelOfNode(node.id) }));
  }

  function supplierOf(slot: GraphSlot): string | null {
    for (const parent of parents) {
      if (graph.metaOfNode(parent)?.writes.some((w) => w.slot === slot)) {
        return graph.labelOfNode(parent);
      }
    }
    return null;
  }

  function canInherit(fallbackFrom: string | undefined): boolean {
    if (!fallbackFrom) return false;
    // Currently the only inheritable output is the frames-on-disk directory.
    return upstreamSlots.has('videoProcessResult');
  }
</script>

{#if !meta}
  <div class="p-4 text-xs text-slate-500">Select a node to see its settings.</div>
{:else}
  <div class="flex flex-col divide-y divide-slate-200 dark:divide-[#2a3441]">
    <!-- Identity -->
    <div class="p-3 flex flex-col gap-1">
      <div class="flex items-start justify-between gap-2">
        <h3 class="text-section-title text-slate-900 dark:text-white">{meta.label}</h3>
        <span
          class="shrink-0 text-[9px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#1f2937] text-slate-500"
        >
          {meta.kind}
        </span>
      </div>
      <p class="text-[11px] leading-snug text-slate-500 dark:text-slate-400">{meta.description}</p>
      <code class="text-[10px] text-slate-400 dark:text-slate-500 font-mono">{meta.id}</code>
      {#if meta.requiresFfmpeg}
        <p class="text-[10px] text-amber-600 dark:text-amber-400 flex items-center gap-1">
          <Icon name="warning" class="text-[12px]" /> Requires ffmpeg on PATH
        </p>
      {/if}
    </div>

    <!-- Inputs -->
    {#if meta.reads.length > 0}
      <div class="p-3 flex flex-col gap-2">
        <h4 class="text-stat-label">Inputs</h4>
        {#each meta.reads as read (read.slot)}
          {@const supplier = supplierOf(read.slot)}
          {@const satisfiedByParam =
            read.satisfiedByParam !== undefined &&
            values[read.satisfiedByParam] !== undefined &&
            values[read.satisfiedByParam] !== ''}
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-1.5 text-[11px]">
              {#if supplier}
                <Icon name="check_circle" class="text-[13px] text-emerald-500" />
                <span class="text-slate-600 dark:text-slate-300">{SLOT_LABEL[read.slot]}</span>
                <span class="text-slate-400">← {supplier}</span>
              {:else if satisfiedByParam}
                <Icon name="check_circle" class="text-[13px] text-emerald-500" />
                <span class="text-slate-600 dark:text-slate-300">{SLOT_LABEL[read.slot]}</span>
                <span class="text-slate-400">← set manually</span>
              {:else if read.optional}
                <span class="w-[13px] text-center text-slate-400 leading-none">–</span>
                <span class="text-slate-500">{SLOT_LABEL[read.slot]}</span>
                <span class="text-slate-400">optional</span>
              {:else}
                <Icon name="warning" class="text-[13px] text-amber-500" />
                <span class="text-slate-600 dark:text-slate-300">{SLOT_LABEL[read.slot]}</span>
              {/if}
            </div>

            {#if !supplier}
              {@const candidates = candidatesFor(read.slot)}
              {#if candidates.length > 0}
                <select
                  value=""
                  onchange={(e) => {
                    const id = e.currentTarget.value;
                    if (id) graph.connect(id, selectedId);
                    e.currentTarget.value = '';
                  }}
                  class="text-[11px] bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-2 py-1 text-slate-600 dark:text-slate-300"
                >
                  <option value="">Connect an input…</option>
                  {#each candidates as candidate (candidate.id)}
                    <option value={candidate.id}>{candidate.label}</option>
                  {/each}
                </select>
              {:else if !read.optional && !satisfiedByParam}
                <p class="text-[10px] text-slate-400 pl-[18px]">
                  Nothing in this graph produces it{read.satisfiedByParam
                    ? ` — fill in "${read.satisfiedByParam}" below`
                    : ''}.
                </p>
              {/if}
            {:else}
              <button
                onclick={() => {
                  const parent = parents.find((p) =>
                    graph.metaOfNode(p)?.writes.some((w) => w.slot === read.slot)
                  );
                  if (parent) graph.disconnect(parent, selectedId);
                }}
                class="self-start text-[10px] text-slate-400 hover:text-red-500 pl-[18px]"
              >
                Disconnect
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Parameters -->
    <div class="p-3 flex flex-col gap-3">
      <h4 class="text-stat-label">Parameters</h4>
      {#if meta.params.length === 0}
        <p class="text-[11px] text-slate-400">Nothing to configure on this node.</p>
      {:else}
        {#each meta.params as spec (spec.key)}
          <ParamField
            {spec}
            value={values[spec.key]}
            canInherit={canInherit(spec.fallbackFrom)}
            onchange={(v) => graph.setParam(selectedId, spec.key, v)}
          />
        {/each}
      {/if}
    </div>

    <!-- Removal -->
    <div class="p-3">
      <button
        onclick={() => onremove(selectedId)}
        class="w-full flex items-center justify-center gap-1 px-2 py-1.5 rounded text-[11px] font-bold text-red-600 dark:text-red-400 border border-red-200 dark:border-red-900/50 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors"
      >
        <Icon name="delete" class="text-[14px]" /> Remove node
      </button>
    </div>
  </div>
{/if}
