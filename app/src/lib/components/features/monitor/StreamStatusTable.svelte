<script lang="ts">
  import { Panel, StatusBadge } from '$lib/components/ui';
  import type { StreamStats } from '$lib/types';

  interface Props {
    streamStats: StreamStats | null;
  }

  let { streamStats }: Props = $props();

  let activeCount = $derived(streamStats?.activeCount ?? 0);
</script>

<Panel title="Stream Status" icon="sensors">
    {#snippet actions()}
        <div class="flex items-center gap-2">
            {#if streamStats}
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{activeCount} active</span>
            {/if}
            <a href="/streams" class="text-stat-label text-status-info hover:text-blue-400">Manage</a>
        </div>
    {/snippet}
    <div class="overflow-x-auto">
        <table class="w-full text-left text-xs font-mono">
            <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                <tr>
                    <th class="px-3 py-1.5 font-medium w-28">ID</th>
                    <th class="px-3 py-1.5 font-medium">NAME</th>
                    <th class="px-3 py-1.5 font-medium">STATUS</th>
                    <th class="px-3 py-1.5 font-medium">TYPE</th>
                    <th class="px-3 py-1.5 font-medium">CODEC</th>
                    <th class="px-3 py-1.5 font-medium">RESOLUTION</th>
                    <th class="px-3 py-1.5 font-medium">LATENCY</th>
                    <th class="px-3 py-1.5 font-medium text-right">BITRATE</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                {#if streamStats && streamStats.streams.length > 0}
                    {#each streamStats.streams as stream}
                        <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                            <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">{stream.id}</td>
                            <td class="px-3 py-1.5 text-muted">{stream.name}</td>
                            <td class="px-3 py-1.5"><StatusBadge status={stream.status} /></td>
                            <td class="px-3 py-1.5 text-muted capitalize">{stream.streamType}</td>
                            <td class="px-3 py-1.5 text-muted">{stream.codec ?? '--'}</td>
                            <td class="px-3 py-1.5 text-muted">{stream.resolution ?? '--'}</td>
                            <td class="px-3 py-1.5 text-muted">{stream.latencyMs ? `${stream.latencyMs}ms` : '--'}</td>
                            <td class="px-3 py-1.5 text-right text-muted">{stream.bitrateKbps ? `${stream.bitrateKbps} kbps` : '--'}</td>
                        </tr>
                    {/each}
                {:else}
                    <tr>
                        <td colspan="8" class="px-3 py-4 text-center text-slate-500">
                            No active streams. <a href="/streams" class="text-[#137fec] hover:underline">Start a stream</a> to monitor.
                        </td>
                    </tr>
                {/if}
            </tbody>
        </table>
    </div>
</Panel>
