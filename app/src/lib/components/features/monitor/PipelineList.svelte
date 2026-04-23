<script lang="ts">
  import { Panel, StatusBadge } from '$lib/components/ui';

  interface Props {
    activePipelines: string[];
  }

  let { activePipelines }: Props = $props();
</script>

<Panel title="Pipelines" icon="conversion_path">
    {#snippet actions()}
        <span class="text-[10px] px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-500 font-bold">{activePipelines.length}</span>
    {/snippet}
    <div class="p-3">
        {#if activePipelines.length > 0}
            <div class="flex flex-col gap-2">
                {#each activePipelines as pipelineId}
                    <div class="flex items-center justify-between px-2 py-1.5 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                        <span class="text-xs font-mono text-slate-700 dark:text-slate-300 truncate">{pipelineId}</span>
                        <StatusBadge status="active" />
                    </div>
                {/each}
            </div>
        {:else}
            <div class="text-center text-caption py-4">No active pipelines</div>
        {/if}
    </div>
</Panel>
