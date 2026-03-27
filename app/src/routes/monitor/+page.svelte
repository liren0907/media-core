<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { PageContent, StatCard, Panel, StatusBadge, ProgressBar } from '$lib/components/ui';
  import { SparklineBar } from '$lib/components/data';
  import { ThroughputChart } from '$lib/components/media';
  import { formatUptime } from '$lib/utils/format';
  import {
    onSystemMetrics,
    onStreamStatus,
    onPipelineProgress,
    startMetricsStream,
    stopMetricsStream,
    getActivePipelines,
    type SystemMetricsEvent,
    type StreamStatusEvent,
    type PipelineProgressEvent,
  } from '$lib/events';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import type { SystemMetrics, StreamStats, ThroughputHistory } from '$lib/types';

  interface EventLogEntry {
    time: string;
    source: string;
    message: string;
  }

  // ── State ──

  let metrics = $state<SystemMetrics | null>(null);
  let streamStats = $state<StreamStats | null>(null);
  let throughputHistory = $state<ThroughputHistory | null>(null);
  let activePipelines = $state<string[]>([]);
  let eventLog = $state<EventLogEntry[]>([]);
  let cpuHistory: number[] = $state([0, 0, 0, 0, 0, 0, 0, 0]);

  // ── Fetch functions ──

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

  async function fetchActivePipelines() {
    try {
      activePipelines = await getActivePipelines();
    } catch (e) {
      console.error('Failed to fetch active pipelines:', e);
    }
  }

  function pushEvent(source: string, message: string) {
    const now = new Date();
    const time = now.toLocaleTimeString('en-GB', { hour12: false });
    eventLog = [{ time, source, message }, ...eventLog.slice(0, 49)];
  }


  // ── Effect: data fetching + event listeners ──

  $effect(() => {
    let unlistenMetrics: UnlistenFn | undefined;
    let unlistenStream: UnlistenFn | undefined;
    let unlistenPipeline: UnlistenFn | undefined;

    // Initial fetches
    fetchMetrics();
    fetchStreamStats();
    fetchThroughputHistory();
    fetchActivePipelines();

    // Start backend metric emission
    startMetricsStream(2000).catch(console.error);

    // Subscribe to real-time events
    onSystemMetrics((event: SystemMetricsEvent) => {
      pushEvent('system', `CPU ${event.cpuPercent.toFixed(0)}% | Mem ${event.memoryPercent.toFixed(0)}% | Disk ${event.diskPercent.toFixed(0)}% | Streams ${event.activeStreams}`);
    }).then((fn) => { unlistenMetrics = fn; });

    onStreamStatus((event: StreamStatusEvent) => {
      pushEvent('stream', `[${event.streamId}] ${event.status}${event.message ? ': ' + event.message : ''}`);
    }).then((fn) => { unlistenStream = fn; });

    onPipelineProgress((event: PipelineProgressEvent) => {
      const pct = event.progressPercent.toFixed(0);
      pushEvent('pipeline', `[${event.pipelineId}] ${event.stepName} — ${pct}%${event.hasError ? ' ERROR: ' + event.errorMessage : ''}`);
    }).then((fn) => { unlistenPipeline = fn; });

    // Polling
    const pollInterval = setInterval(() => {
      fetchMetrics();
      fetchStreamStats();
      fetchActivePipelines();
    }, 3000);

    return () => {
      clearInterval(pollInterval);
      stopMetricsStream().catch(console.error);
      unlistenMetrics?.();
      unlistenStream?.();
      unlistenPipeline?.();
    };
  });

  // ── Derived values ──

  let cpuPercent = $derived(metrics?.cpu.usagePercent ?? 0);
  let memoryPercent = $derived(metrics?.memory.usagePercent ?? 0);
  let memoryUsedGb = $derived(metrics?.memory.usedGb ?? 0);
  let diskPercent = $derived(metrics?.disk.usagePercent ?? 0);
  let diskTotalGb = $derived(metrics?.disk.totalGb ?? 0);
  let writeSpeed = $derived(metrics?.disk.writeSpeedMbps ?? 0);
  let activeCount = $derived(streamStats?.activeCount ?? 0);
  let totalCount = $derived(streamStats?.totalCount ?? 0);

  const eventSourceColors: Record<string, string> = {
    system: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
    stream: 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20',
    pipeline: 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20',
  };
</script>

<svelte:head>
  <title>Monitor</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip -->
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

    <!-- Stream Status -->
    <Panel title="Stream Status" icon="sensors">
        {#snippet actions()}
            <div class="flex items-center gap-2">
                {#if streamStats}
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{activeCount} active</span>
                {/if}
                <a href="/stream" class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">Manage</a>
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
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.name}</td>
                                <td class="px-3 py-1.5"><StatusBadge status={stream.status} /></td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400 capitalize">{stream.streamType}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.codec ?? '--'}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.resolution ?? '--'}</td>
                                <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400">{stream.latencyMs ? `${stream.latencyMs}ms` : '--'}</td>
                                <td class="px-3 py-1.5 text-right text-slate-600 dark:text-slate-400">{stream.bitrateKbps ? `${stream.bitrateKbps} kbps` : '--'}</td>
                            </tr>
                        {/each}
                    {:else}
                        <tr>
                            <td colspan="8" class="px-3 py-4 text-center text-slate-500">
                                No active streams. <a href="/stream" class="text-[#137fec] hover:underline">Start a stream</a> to monitor.
                            </td>
                        </tr>
                    {/if}
                </tbody>
            </table>
        </div>
    </Panel>

    <!-- Two-column: Event Log + Active Pipelines -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Event Log (2/3 width) -->
        <div class="lg:col-span-2">
            <Panel title="Event Log" icon="list_alt">
                {#snippet actions()}
                    <button onclick={() => { eventLog = []; }} class="text-[10px] font-bold uppercase tracking-wider text-slate-400 hover:text-slate-600 dark:hover:text-slate-200">Clear</button>
                {/snippet}
                <div class="overflow-x-auto max-h-[280px] overflow-y-auto">
                    <table class="w-full text-left text-xs font-mono">
                        <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441] sticky top-0 bg-white dark:bg-[#161e27]">
                            <tr>
                                <th class="px-3 py-1.5 font-medium w-20">TIME</th>
                                <th class="px-3 py-1.5 font-medium w-24">SOURCE</th>
                                <th class="px-3 py-1.5 font-medium">MESSAGE</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                            {#if eventLog.length > 0}
                                {#each eventLog as entry}
                                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                                        <td class="px-3 py-1.5 text-slate-500">{entry.time}</td>
                                        <td class="px-3 py-1.5">
                                            <StatusBadge status={entry.source} colorMap={eventSourceColors} />
                                        </td>
                                        <td class="px-3 py-1.5 text-slate-700 dark:text-slate-300 truncate max-w-[400px]">{entry.message}</td>
                                    </tr>
                                {/each}
                            {:else}
                                <tr>
                                    <td colspan="3" class="px-3 py-4 text-center text-slate-500">
                                        Waiting for events...
                                    </td>
                                </tr>
                            {/if}
                        </tbody>
                    </table>
                </div>
            </Panel>
        </div>

        <!-- Active Pipelines (1/3 width) -->
        <div>
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
                        <div class="text-center text-xs text-slate-500 py-4">No active pipelines</div>
                    {/if}
                </div>
            </Panel>
        </div>
    </div>

    <!-- Throughput Chart -->
    <Panel title="Throughput" icon="show_chart">
        {#snippet actions()}
            <div class="flex items-center gap-3 text-[10px]">
                <span class="flex items-center gap-1 text-slate-500">
                    <span class="size-1.5 rounded-full bg-[#137fec]"></span> Network
                </span>
                <span class="flex items-center gap-1 text-slate-500">
                    <span class="size-1.5 rounded-full bg-slate-400"></span> FPS
                </span>
                {#if metrics}
                    <span class="text-slate-400 font-mono">Uptime {formatUptime(metrics.uptimeSeconds)}</span>
                {/if}
            </div>
        {/snippet}
        <ThroughputChart {throughputHistory} gradientId="monitorGradient" />
    </Panel>
</PageContent>
