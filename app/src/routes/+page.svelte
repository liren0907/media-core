<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  // Dashboard Overview - Tactical Command Style Redesign

  let currentTime = $state(new Date());

  // System metrics state
  interface SystemMetrics {
    cpu: { usagePercent: number; coreCount: number; frequencyMhz: number | null };
    memory: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number };
    disk: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number; readSpeedMbps: number | null; writeSpeedMbps: number | null };
    uptimeSeconds: number;
  }

  interface StreamStatus {
    id: string;
    name: string;
    status: string;
    streamType: string;
    codec: string | null;
    resolution: string | null;
    fps: number | null;
    bitrateKbps: number | null;
    durationSeconds: number | null;
    latencyMs: number | null;
  }

  interface StreamStats {
    activeCount: number;
    totalCount: number;
    avgLatencyMs: number;
    totalBitrateKbps: number;
    streams: StreamStatus[];
  }

  interface ThroughputPoint {
    timestamp: number;
    networkMbps: number;
    fps: number;
    cpuPercent: number;
  }

  interface ThroughputHistory {
    points: ThroughputPoint[];
    periodSeconds: number;
  }

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

  function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'active': return 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20';
      case 'idle': return 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20';
      case 'error': return 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20';
      default: return 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20';
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

<svelte:head>
	<title>Media Core Dashboard</title>
</svelte:head>

<div class="flex h-[calc(100vh-64px)] overflow-hidden bg-slate-50 dark:bg-[#0d1117] text-slate-800 dark:text-slate-200 font-sans">
    
    <!-- Main Content Area (Left/Center) -->
    <div class="flex-1 flex flex-col min-w-0">
        
        <!-- Command Header -->
        <div class="h-16 px-6 border-b border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] flex items-center justify-between shrink-0">
            <div class="flex items-center gap-4">
                <div class="flex flex-col">
                    <h1 class="text-lg font-bold tracking-tight uppercase font-display text-slate-900 dark:text-white">Media Command</h1>
                    <span class="text-[10px] tracking-widest text-slate-500 uppercase">System Overview</span>
                </div>
            </div>
            
            <div class="font-mono text-xl font-medium tracking-wider text-slate-700 dark:text-slate-300">
                {currentTime.toLocaleTimeString('en-GB', { hour12: false })}
            </div>

            <div class="flex items-center gap-3">
                <a href="/stream" class="flex items-center gap-2 px-3 py-1.5 bg-[#137fec] hover:bg-blue-600 text-white text-xs font-bold uppercase tracking-wider rounded transition-colors">
                    <span class="material-symbols-outlined text-[16px]">add_circle</span>
                    Add Stream
                </a>
                <button class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-[#283039] text-xs font-bold uppercase tracking-wider rounded transition-colors">
                    <span class="material-symbols-outlined text-[16px]">download</span>
                    Report
                </button>
            </div>
        </div>

        <!-- Scrollable Content -->
        <div class="flex-1 overflow-auto p-6 flex flex-col gap-6">
            
            <!-- Stats Grid -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <!-- CPU Card -->
                <div class="bg-white dark:bg-[#161e27] rounded-lg border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm relative overflow-hidden group">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-[#137fec]/5 rounded-bl-full -mr-6 -mt-6 transition-transform group-hover:scale-110"></div>
                    <div class="flex justify-between items-start relative z-10 mb-4">
                        <div>
                            <h3 class="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1">CPU Load</h3>
                            <div class="flex items-baseline gap-2">
                                <span class="text-3xl font-bold font-display text-slate-900 dark:text-white">{cpuPercent.toFixed(0)}%</span>
                                {#if metrics}
                                    <span class="text-xs font-mono text-slate-500">{metrics.cpu.coreCount} cores</span>
                                {/if}
                            </div>
                        </div>
                        <div class="size-8 rounded bg-slate-100 dark:bg-[#1f2937] flex items-center justify-center text-[#137fec]">
                            <span class="material-symbols-outlined text-[20px]">memory</span>
                        </div>
                    </div>
                    <!-- Micro Chart -->
                    <div class="h-8 w-full flex items-end gap-1">
                        {#each cpuHistory as value, i}
                            <div 
                                class="w-full rounded-sm transition-all {i === cpuHistory.length - 1 ? 'bg-[#137fec] shadow-[0_0_8px_rgba(19,127,236,0.5)]' : 'bg-[#137fec]/20 group-hover:bg-[#137fec]/40'}"
                                style="height: {value}%"
                            ></div>
                        {/each}
                    </div>
                </div>

                <!-- Memory Card -->
                <div class="bg-white dark:bg-[#161e27] rounded-lg border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm relative overflow-hidden group">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-orange-500/5 rounded-bl-full -mr-6 -mt-6 transition-transform group-hover:scale-110"></div>
                    <div class="flex justify-between items-start relative z-10 mb-4">
                        <div>
                            <h3 class="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1">Memory Usage</h3>
                            <div class="flex items-baseline gap-2">
                                <span class="text-3xl font-bold font-display text-slate-900 dark:text-white">{memoryPercent.toFixed(0)}%</span>
                                {#if memoryPercent > 80}
                                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-orange-500/10 text-orange-500 font-bold uppercase">High</span>
                                {/if}
                            </div>
                        </div>
                        <div class="size-8 rounded bg-slate-100 dark:bg-[#1f2937] flex items-center justify-center text-orange-500">
                            <span class="material-symbols-outlined text-[20px]">developer_board</span>
                        </div>
                    </div>
                    <div class="w-full bg-slate-100 dark:bg-[#1f2937] rounded-full h-1.5 mt-auto overflow-hidden">
                        <div class="bg-orange-500 h-1.5 rounded-full shadow-[0_0_8px_rgba(249,115,22,0.5)] transition-all" style="width: {memoryPercent}%"></div>
                    </div>
                    <div class="mt-2 text-[10px] text-slate-500 font-mono">
                        {memoryUsedGb.toFixed(1)} GB / {metrics?.memory.totalGb?.toFixed(1) ?? '0'} GB
                    </div>
                </div>

                <!-- RAM Card -->
                <div class="bg-white dark:bg-[#161e27] rounded-lg border border-slate-200 dark:border-[#2a3441] p-6 shadow-sm relative overflow-hidden group">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-purple-500/5 rounded-bl-full -mr-6 -mt-6 transition-transform group-hover:scale-110"></div>
                    <div class="flex justify-between items-start relative z-10 mb-4">
                        <div>
                            <h3 class="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1">Available RAM</h3>
                            <div class="flex items-baseline gap-2">
                                <span class="text-3xl font-bold font-display text-slate-900 dark:text-white">{metrics?.memory.availableGb?.toFixed(1) ?? '0'}</span>
                                <span class="text-xs font-mono text-slate-500">GB</span>
                            </div>
                        </div>
                        <div class="size-8 rounded bg-slate-100 dark:bg-[#1f2937] flex items-center justify-center text-purple-500">
                            <span class="material-symbols-outlined text-[20px]">storage</span>
                        </div>
                    </div>
                    <div class="flex gap-1 mt-auto">
                        <div class="h-1.5 rounded-sm shadow-[0_0_8px_rgba(168,85,247,0.5)] bg-purple-500 transition-all" style="flex: {memoryPercent}"></div>
                        <div class="h-1.5 rounded-sm bg-slate-100 dark:bg-[#1f2937]" style="flex: {100 - memoryPercent}"></div>
                    </div>
                </div>
            </div>

            <!-- Main Chart Section -->
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <!-- Large Chart -->
                <div class="lg:col-span-2 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] shadow-sm flex flex-col">
                    <div class="px-6 py-4 border-b border-slate-200 dark:border-[#2a3441] flex justify-between items-center bg-slate-50 dark:bg-[#1f2937]/50">
                        <div class="flex items-center gap-2">
                            <span class="material-symbols-outlined text-slate-500 text-[18px]">show_chart</span>
                            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300">Throughput Analytics</h3>
                        </div>
                        <div class="flex items-center gap-4">
                            <div class="flex items-center gap-2 text-[10px]">
                                <span class="flex items-center gap-1.5 text-slate-500">
                                    <span class="size-1.5 rounded-full bg-[#137fec]"></span> Network
                                </span>
                                <span class="flex items-center gap-1.5 text-slate-500">
                                    <span class="size-1.5 rounded-full bg-slate-400"></span> FPS
                                </span>
                            </div>
                            <select class="bg-transparent text-[10px] font-bold uppercase text-slate-500 border-none outline-none cursor-pointer hover:text-slate-700 dark:hover:text-white focus:ring-0 p-0">
                                <option>1H</option>
                                <option>24H</option>
                            </select>
                        </div>
                    </div>
                    
                    <div class="p-6 relative w-full h-[250px]">
                        <!-- Chart SVG -->
                        <svg class="w-full h-full" preserveAspectRatio="none" viewBox="0 0 800 250">
                            <defs>
                                <linearGradient id="chartGradient" x1="0" x2="0" y1="0" y2="1">
                                    <stop offset="0%" stop-color="#137fec" stop-opacity="0.2"></stop>
                                    <stop offset="100%" stop-color="#137fec" stop-opacity="0"></stop>
                                </linearGradient>
                            </defs>
                            <!-- Grid Lines -->
                            <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="200" y2="200" opacity="0.5"></line>
                            <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="150" y2="150" opacity="0.5"></line>
                            <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="100" y2="100" opacity="0.5"></line>
                            <line stroke="#2a3441" stroke-dasharray="2 4" stroke-width="1" x1="0" x2="800" y1="50" y2="50" opacity="0.5"></line>
                            <!-- Primary Line (Area) -->
                            {#if throughputHistory && throughputHistory.points.length > 0}
                                {@const points = throughputHistory.points}
                                {@const pathData = points.map((p, i) => {
                                    const x = (i / (points.length - 1)) * 800;
                                    const y = 220 - (p.networkMbps / 30 * 170);
                                    return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
                                }).join(' ')}
                                <path d="{pathData} V 250 H 0 Z" fill="url(#chartGradient)"></path>
                                <path d="{pathData}" fill="none" stroke="#137fec" stroke-width="2"></path>
                            {:else}
                                <path d="M0 200 C 100 180, 200 220, 300 150 S 500 80, 600 120 S 700 60, 800 80 V 250 H 0 Z" fill="url(#chartGradient)"></path>
                                <path d="M0 200 C 100 180, 200 220, 300 150 S 500 80, 600 120 S 700 60, 800 80" fill="none" stroke="#137fec" stroke-width="2"></path>
                            {/if}
                            <!-- Secondary Line -->
                            <path d="M0 180 C 120 190, 250 160, 350 170 S 550 150, 650 140 S 750 160, 800 150" fill="none" opacity="0.5" stroke="#94a3b8" stroke-dasharray="5 5" stroke-width="2"></path>
                        </svg>
                        <!-- X Axis Labels -->
                        <div class="flex justify-between mt-2 text-[10px] text-slate-500 font-mono uppercase tracking-wider">
                            <span>{new Date(Date.now() - 3600000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                            <span>{new Date(Date.now() - 2700000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                            <span>{new Date(Date.now() - 1800000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                            <span>{new Date(Date.now() - 900000).toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                            <span>{currentTime.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}</span>
                        </div>
                    </div>
                </div>

                <!-- Storage Status Panel -->
                <div class="lg:col-span-1 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] shadow-sm flex flex-col">
                    <div class="px-6 py-4 border-b border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#1f2937]/50">
                        <div class="flex items-center gap-2">
                            <span class="material-symbols-outlined text-slate-500 text-[18px]">hard_drive</span>
                            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300">Storage</h3>
                        </div>
                    </div>
                    
                    <div class="p-6 flex flex-col items-center justify-center flex-1">
                        <div class="relative size-40">
                            <svg class="size-full rotate-[-90deg]" viewBox="0 0 36 36">
                                <!-- Background Circle -->
                                <path class="text-slate-100 dark:text-slate-700" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-width="2"></path>
                                <!-- Value Circle -->
                                <path class="text-[#137fec]" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" fill="none" stroke="currentColor" stroke-dasharray="{diskPercent}, 100" stroke-width="2"></path>
                            </svg>
                            <div class="absolute inset-0 flex flex-col items-center justify-center text-slate-900 dark:text-white">
                                <span class="text-3xl font-bold font-display">{diskPercent.toFixed(0)}%</span>
                                <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400">USED</span>
                            </div>
                        </div>
                        <div class="w-full flex flex-col gap-3 mt-6">
                            <div class="flex justify-between items-center p-3 rounded bg-slate-50 dark:bg-[#1f2937] border border-slate-100 dark:border-[#2a3441]">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-slate-500">Total</span>
                                <span class="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">{diskTotalGb.toFixed(0)} GB</span>
                            </div>
                            <div class="flex justify-between items-center p-3 rounded bg-slate-50 dark:bg-[#1f2937] border border-slate-100 dark:border-[#2a3441]">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-slate-500">Write Spd</span>
                                <span class="text-xs font-mono font-bold text-slate-700 dark:text-slate-200">{writeSpeed > 0 ? writeSpeed.toFixed(0) : '--'} MB/s</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Active Tasks Table -->
            <div class="rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#161e27] shadow-sm overflow-hidden">
                <div class="px-4 py-2 border-b border-slate-200 dark:border-[#2a3441] flex justify-between items-center bg-slate-50 dark:bg-[#1f2937]/50">
                    <div class="flex items-center gap-2">
                        <span class="material-symbols-outlined text-slate-500 text-[18px]">list_alt</span>
                        <h3 class="text-xs font-bold uppercase tracking-wider text-slate-600 dark:text-slate-300">Active Streams</h3>
                        {#if streamStats}
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{streamStats.activeCount} / {streamStats.totalCount}</span>
                        {/if}
                    </div>
                    <a href="/multi-stream-viewer" class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">View All</a>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left text-xs font-mono">
                        <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                            <tr>
                                <th class="px-4 py-2 font-medium w-32">STREAM ID</th>
                                <th class="px-4 py-2 font-medium">TYPE</th>
                                <th class="px-4 py-2 font-medium">STATUS</th>
                                <th class="px-4 py-2 font-medium">CODEC</th>
                                <th class="px-4 py-2 font-medium">LATENCY</th>
                                <th class="px-4 py-2 font-medium text-right w-16"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                            {#if streamStats && streamStats.streams.length > 0}
                                {#each streamStats.streams as stream}
                                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                                        <td class="px-4 py-2 text-slate-900 dark:text-white font-bold">{stream.id}</td>
                                        <td class="px-4 py-2 text-slate-600 dark:text-slate-400 capitalize">{stream.streamType}</td>
                                        <td class="px-4 py-2">
                                            <span class="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold border uppercase tracking-wide {getStatusColor(stream.status)}">
                                                {stream.status}
                                            </span>
                                        </td>
                                        <td class="px-4 py-2 text-slate-600 dark:text-slate-400">{stream.codec ?? 'N/A'}</td>
                                        <td class="px-4 py-2 text-slate-600 dark:text-slate-400">{stream.latencyMs ? `${stream.latencyMs}ms` : '--'}</td>
                                        <td class="px-4 py-2 text-right">
                                            <button class="text-slate-400 hover:text-[#137fec] transition-colors">
                                                <span class="material-symbols-outlined text-[16px]">more_vert</span>
                                            </button>
                                        </td>
                                    </tr>
                                {/each}
                            {:else}
                                <tr>
                                    <td colspan="6" class="px-4 py-8 text-center text-slate-500">
                                        No active streams. <a href="/stream" class="text-[#137fec] hover:underline">Add a stream</a> to get started.
                                    </td>
                                </tr>
                            {/if}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
</div>
