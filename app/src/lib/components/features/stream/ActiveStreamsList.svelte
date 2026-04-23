<script lang="ts">
  import type { StreamStats } from '$lib/types';
  import { Panel, StatusBadge } from '$lib/components/ui';

  interface Props {
    streamStats: StreamStats | null;
  }

  let { streamStats }: Props = $props();
</script>

{#if streamStats && streamStats.streams.length > 0}
    <Panel title="Active Streams" icon="sensors">
        <div class="p-3 flex flex-col gap-1.5">
            {#each streamStats.streams as stream}
                <div class="flex items-center justify-between p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                    <div class="flex items-center gap-2 min-w-0">
                        <StatusBadge status={stream.status} />
                        <div class="min-w-0">
                            <p class="text-xs font-medium text-slate-900 dark:text-white truncate">{stream.name}</p>
                            <p class="text-meta">{stream.codec ?? '?'} · {stream.resolution ?? 'N/A'}</p>
                        </div>
                    </div>
                    {#if stream.latencyMs}
                        <span class="text-meta shrink-0">{stream.latencyMs}ms</span>
                    {/if}
                </div>
            {/each}
        </div>
    </Panel>
{/if}
