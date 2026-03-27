<script lang="ts">
  import { StatCard, ProgressBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';
  import type { SystemMetrics, StreamStats } from '$lib/types';

  interface Props {
    metrics: SystemMetrics | null;
    streamStats: StreamStats | null;
    cpuHistory: number[];
  }

  let { metrics, streamStats, cpuHistory }: Props = $props();

  let cpuPercent = $derived(metrics?.cpu.usagePercent ?? 0);
  let memoryPercent = $derived(metrics?.memory.usagePercent ?? 0);
  let memoryUsedGb = $derived(metrics?.memory.usedGb ?? 0);
  let diskPercent = $derived(metrics?.disk.usagePercent ?? 0);
  let diskTotalGb = $derived(metrics?.disk.totalGb ?? 0);
  let writeSpeed = $derived(metrics?.disk.writeSpeedMbps ?? 0);
  let activeCount = $derived(streamStats?.activeCount ?? 0);
  let totalCount = $derived(streamStats?.totalCount ?? 0);
</script>

<div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
    <StatCard label="Streams" icon="cell_tower" iconColor="text-[#137fec]" value="{activeCount} / {totalCount}">
        {#snippet extra()}
            <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                {#if streamStats}
                    {streamStats.avgLatencyMs.toFixed(0)}ms avg · {(streamStats.totalBitrateKbps / 1000).toFixed(1)} Mbps
                {:else}
                    --
                {/if}
            </div>
        {/snippet}
    </StatCard>

    <StatCard label="CPU" icon="memory" iconColor="text-[#137fec]" value="{cpuPercent.toFixed(0)}%" sub={metrics ? `${metrics.cpu.coreCount}C` : ''}>
        {#snippet extra()}
            <SparklineBar values={cpuHistory} />
        {/snippet}
    </StatCard>

    <StatCard label="Memory" icon="developer_board" iconColor="text-orange-500" value="{memoryPercent.toFixed(0)}%">
        {#snippet extra()}
            <ProgressBar percent={memoryPercent} color="bg-orange-500" />
            <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                {memoryUsedGb.toFixed(1)} / {metrics?.memory.totalGb?.toFixed(1) ?? '0'} GB
            </div>
        {/snippet}
    </StatCard>

    <StatCard label="Disk" icon="hard_drive" iconColor="text-purple-500" value="{diskPercent.toFixed(0)}%" sub="{diskTotalGb.toFixed(0)} GB">
        {#snippet extra()}
            <ProgressBar percent={diskPercent} color="bg-purple-500" />
            <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                {writeSpeed > 0 ? `${writeSpeed.toFixed(0)} MB/s write` : 'Idle'}
            </div>
        {/snippet}
    </StatCard>
</div>
