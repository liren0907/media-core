<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { StatCard, Panel, StatusBadge, ProgressBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';
  import { ThroughputChart } from '$lib/components/media';
  import type { SystemMetrics, StreamStats, ThroughputHistory } from '$lib/types';
  import { formatUptime } from '$lib/utils/format';

  let currentTime = $state(new Date());

  let metrics = $state<SystemMetrics | null>(null);
  let streamStats = $state<StreamStats | null>(null);
  let throughputHistory = $state<ThroughputHistory | null>(null);
  let cpuHistory: number[] = $state([40, 60, 45, 30, 70, 45, 55, 35]);

  async function fetchMetrics() {
    try {
      metrics = await invoke('get_system_metrics');
      if (metrics) {
        cpuHistory = [...cpuHistory.slice(1), metrics.cpu.usagePercent];
      }
    } catch (e) {
      console.error('Failed to fetch system metrics:', e);
    }
  }

  async function fetchStreamStats() {
    try {
      streamStats = await invoke('get_stream_stats');
    } catch (e) {
      console.error('Failed to fetch stream stats:', e);
    }
  }

  async function fetchThroughputHistory() {
    try {
      throughputHistory = await invoke('get_throughput_history', { periodSeconds: 3600 });
    } catch (e) {
      console.error('Failed to fetch throughput history:', e);
    }
  }



  $effect(() => {
    const clockInterval = setInterval(() => {
      currentTime = new Date();
    }, 1000);

    fetchMetrics();
    fetchStreamStats();
    fetchThroughputHistory();

    const metricsInterval = setInterval(() => {
      fetchMetrics();
      fetchStreamStats();
    }, 2000);

    return () => {
      clearInterval(clockInterval);
      clearInterval(metricsInterval);
    };
  });

  // Reactive values with fallbacks
  let cpuPercent = $derived(metrics?.cpu.usagePercent ?? 0);
  let memoryUsedGb = $derived(metrics?.memory.usedGb ?? 0);
  let memoryPercent = $derived(metrics?.memory.usagePercent ?? 0);
  let diskPercent = $derived(metrics?.disk.usagePercent ?? 0);
  let diskTotalGb = $derived(metrics?.disk.totalGb ?? 0);
  let writeSpeed = $derived(metrics?.disk.writeSpeedMbps ?? 0);
</script>

    <!-- Stats Strip: 4 columns -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
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

        <StatCard label="System" icon="schedule" iconColor="text-green-500" value={currentTime.toLocaleTimeString('en-GB', { hour12: false })}>
            {#snippet extra()}
                <div class="mt-1.5 text-[10px] text-slate-500 font-mono">
                    {#if metrics}
                        Uptime {formatUptime(metrics.uptimeSeconds)}
                    {:else}
                        --:--:--
                    {/if}
                </div>
            {/snippet}
        </StatCard>
    </div>

    <!-- Throughput Chart -->
    <Panel title="Throughput" icon="show_chart">
        {#snippet actions()}
            <div class="flex items-center gap-4">
                <div class="flex items-center gap-3 text-[10px]">
                    <span class="flex items-center gap-1 text-slate-500">
                        <span class="size-1.5 rounded-full bg-[#137fec]"></span> Network
                    </span>
                    <span class="flex items-center gap-1 text-slate-500">
                        <span class="size-1.5 rounded-full bg-slate-400"></span> FPS
                    </span>
                </div>
                <select class="bg-transparent text-[10px] font-bold uppercase text-slate-500 border-none outline-none cursor-pointer hover:text-slate-700 dark:hover:text-white focus:ring-0 p-0">
                    <option>1H</option>
                    <option>24H</option>
                </select>
            </div>
        {/snippet}
        <ThroughputChart {throughputHistory} />
    </Panel>

    <!-- Active Streams Table -->
    <Panel title="Active Streams" icon="list_alt">
        {#snippet actions()}
            <div class="flex items-center gap-2">
                {#if streamStats}
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{streamStats.activeCount} / {streamStats.totalCount}</span>
                {/if}
                <a href="/multi-stream-viewer" class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">View All</a>
            </div>
        {/snippet}
        <div class="overflow-x-auto">
            <table class="w-full text-left text-xs font-mono">
                <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                    <tr>
                        <th class="px-3 py-1.5 font-medium w-32">STREAM ID</th>
                        <th class="px-3 py-1.5 font-medium">TYPE</th>
                        <th class="px-3 py-1.5 font-medium">STATUS</th>
                        <th class="px-3 py-1.5 font-medium">CODEC</th>
                        <th class="px-3 py-1.5 font-medium">LATENCY</th>
                        <th class="px-3 py-1.5 font-medium text-right w-12"></th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                    {#if streamStats && streamStats.streams.length > 0}
                        {#each streamStats.streams as stream}
                            <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                                <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">{stream.id}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400 capitalize">{stream.streamType}</td>
                                <td class="px-3 py-1.5">
                                    <StatusBadge status={stream.status} />
                                </td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.codec ?? 'N/A'}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.latencyMs ? `${stream.latencyMs}ms` : '--'}</td>
                                <td class="px-3 py-1.5 text-right">
                                    <button class="text-slate-400 hover:text-[#137fec] transition-colors">
                                        <span class="material-symbols-outlined text-[16px]">more_vert</span>
                                    </button>
                                </td>
                            </tr>
                        {/each}
                    {:else}
                        <tr>
                            <td colspan="6" class="px-3 py-4 text-center text-slate-500">
                                No active streams. <a href="/streams" class="text-[#137fec] hover:underline">Add a stream</a> to get started.
                            </td>
                        </tr>
                    {/if}
                </tbody>
            </table>
        </div>
    </Panel>
