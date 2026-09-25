<script lang="ts">
  import type { StreamStats } from '$lib/types';
  import { StatCard, ProgressBar } from '$lib/components/ui';

  interface Props {
    streamStats: StreamStats | null;
    latencyPercent: number;
    healthStatus: string;
  }

  let { streamStats, latencyPercent, healthStatus }: Props = $props();
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-3">
    <StatCard label="Active Streams" icon="cast_connected" iconColor="text-[#137fec]" value="{streamStats?.activeCount ?? 0} / {streamStats?.totalCount ?? 0}">
        {#snippet extra()}
            <div class="mt-1.5 text-[10px] font-mono {streamStats && streamStats.activeCount > 0 ? 'text-green-500' : 'text-slate-500'}">
                {streamStats && streamStats.activeCount > 0 ? `${streamStats.activeCount} active` : 'No active streams'}
            </div>
        {/snippet}
    </StatCard>

    <StatCard label="Avg Latency" icon="speed" iconColor="text-orange-500" value="{streamStats?.avgLatencyMs?.toFixed(0) ?? '--'}ms">
        {#snippet extra()}
            <ProgressBar percent={latencyPercent} color={latencyPercent > 50 ? 'bg-orange-500' : 'bg-[#137fec]'} />
        {/snippet}
    </StatCard>

    <StatCard label="HLS Status" icon="playlist_play" iconColor="text-green-500" value={healthStatus}>
        {#snippet extra()}
            <div class="mt-1.5 text-meta">
                {streamStats && streamStats.totalBitrateKbps > 0 ? `${(streamStats.totalBitrateKbps / 1000).toFixed(1)} Mbps` : 'Ready'}
            </div>
        {/snippet}
    </StatCard>
</div>
