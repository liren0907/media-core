<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Panel } from '$lib/components/ui';
  import { ThroughputChart } from '$lib/components/media';
  import { formatUptime } from '$lib/utils/format';
  import { MonitorStats, StreamStatusTable, EventLog, PipelineList } from '$lib/components/features/monitor';
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

  // ── Constants ──

  const eventSourceColors: Record<string, string> = {
    system: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
    stream: 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20',
    pipeline: 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20',
  };
</script>

    <MonitorStats {metrics} {streamStats} {cpuHistory} />
    <StreamStatusTable {streamStats} />
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <div class="lg:col-span-2">
            <EventLog {eventLog} {eventSourceColors} onclear={() => { eventLog = []; }} />
        </div>
        <div>
            <PipelineList {activePipelines} />
        </div>
    </div>
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
