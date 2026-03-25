<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar } from '$lib/components/ui';
  import type { BenchmarkResponse, BenchmarkResultItem } from '$lib/types';

  let videoPath = $state('');
  let runs = $state(3);
  let operations = $state<Record<string, boolean>>({ context_init: true, metadata: true, frame_extraction: true });
  let isRunning = $state(false);
  let error = $state('');
  let result = $state<BenchmarkResponse | null>(null);

  async function selectVideo() {
    try {
      const file = await open({ filters: [{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov'] }] });
      if (file) { videoPath = file as string; error = ''; }
    } catch (e) { error = `Failed: ${e}`; }
  }

  async function runBenchmark() {
    if (!videoPath) { error = 'Select a video file first'; return; }
    const selectedOps = Object.entries(operations).filter(([, v]) => v).map(([k]) => k);
    if (selectedOps.length === 0) { error = 'Select at least one operation'; return; }

    isRunning = true;
    error = '';
    result = null;

    try {
      result = await invoke('run_benchmark', {
        config: { videoPath, operations: selectedOps, runs }
      });
    } catch (e) {
      error = `Benchmark failed: ${e}`;
    }

    isRunning = false;
  }

  function getFileName(path: string): string { return path.split('/').pop() || path; }

  let maxAvg = $derived(result ? Math.max(...result.results.map(r => r.averageMs)) : 1);

  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]';
</script>

<svelte:head>
  <title>Benchmark</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Video" icon="movie" iconColor="text-[#137fec]" value={videoPath ? getFileName(videoPath) : 'None'} />
        <StatCard label="Operations" icon="tune" iconColor="text-purple-500" value="{String(Object.values(operations).filter(v => v).length)}" />
        <StatCard label="Fastest" icon="bolt" iconColor="text-green-500" value={result?.fastest || '--'} />
        <StatCard label="Total Time" icon="timer" iconColor="text-orange-500" value={result ? `${result.totalMs.toFixed(0)}ms` : '--'} />
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Config -->
        <Panel title="Configuration" icon="tune">
            {#snippet actions()}
                <button onclick={runBenchmark} disabled={isRunning || !videoPath} class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold disabled:opacity-50 transition-colors">
                    {#if isRunning}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{:else}<span class="material-symbols-outlined text-[14px]">play_arrow</span>{/if}
                    Run
                </button>
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                <div class="flex flex-col gap-1">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Video File</label>
                    <div class="flex gap-1.5">
                        <input type="text" readonly value={videoPath ? getFileName(videoPath) : ''} class="{inputClass} flex-1" placeholder="Select video..." />
                        <button onclick={selectVideo} class="px-2 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors">Browse</button>
                    </div>
                </div>

                <div class="flex flex-col gap-1">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Runs per operation = {runs}</label>
                    <input type="range" bind:value={runs} min="1" max="20" step="1" class="w-full" />
                </div>

                <div class="flex flex-col gap-1.5">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Operations</label>
                    {#each [
                        { key: 'context_init', label: 'Context Init', desc: 'Source access time' },
                        { key: 'metadata', label: 'Metadata Extraction', desc: 'Video metadata parsing' },
                        { key: 'frame_extraction', label: 'Frame Extraction', desc: '10 frames via OpenCV' },
                    ] as op}
                        <label class="flex items-center gap-2 p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] cursor-pointer text-xs">
                            <input type="checkbox" bind:checked={operations[op.key]} class="rounded border-slate-300 dark:border-slate-600 text-[#137fec] w-3.5 h-3.5" />
                            <div class="flex-1">
                                <span class="text-slate-900 dark:text-white font-medium">{op.label}</span>
                                <span class="text-[10px] text-slate-500 ml-1">{op.desc}</span>
                            </div>
                        </label>
                    {/each}
                </div>

                {#if error}
                    <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-[10px]">{error}</div>
                {/if}
            </div>
        </Panel>

        <!-- Results -->
        <div class="lg:col-span-2 flex flex-col gap-3">
            <Panel title="Results" icon="speed">
                <div class="p-3">
                    {#if result && result.results.length > 0}
                        <!-- Bar chart -->
                        <div class="flex flex-col gap-2 mb-4">
                            {#each result.results as item}
                                <div class="flex items-center gap-3">
                                    <span class="text-[10px] font-mono text-slate-500 w-32 truncate text-right">{item.name}</span>
                                    <div class="flex-1 h-5 bg-slate-100 dark:bg-[#1f2937] rounded overflow-hidden relative">
                                        <div class="h-full rounded transition-all {item.name === result.fastest ? 'bg-green-500' : item.name === result.slowest ? 'bg-orange-500' : 'bg-[#137fec]'}" style="width: {(item.averageMs / maxAvg) * 100}%"></div>
                                        <span class="absolute inset-y-0 right-2 flex items-center text-[10px] font-mono font-bold text-slate-700 dark:text-slate-300">{item.averageMs.toFixed(1)}ms</span>
                                    </div>
                                </div>
                            {/each}
                        </div>

                        <!-- Table -->
                        <table class="w-full text-xs font-mono">
                            <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                                <tr>
                                    <th class="px-3 py-1.5 font-medium text-left">OPERATION</th>
                                    <th class="px-3 py-1.5 font-medium text-right">AVG</th>
                                    <th class="px-3 py-1.5 font-medium text-right">MIN</th>
                                    <th class="px-3 py-1.5 font-medium text-right">MAX</th>
                                    <th class="px-3 py-1.5 font-medium text-right">STD DEV</th>
                                    <th class="px-3 py-1.5 font-medium text-right">RUNS</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                                {#each result.results as item}
                                    <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50">
                                        <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">
                                            {item.name}
                                            {#if item.name === result.fastest}<span class="ml-1 text-green-500 text-[10px]">fastest</span>{/if}
                                            {#if item.name === result.slowest}<span class="ml-1 text-orange-500 text-[10px]">slowest</span>{/if}
                                        </td>
                                        <td class="px-3 py-1.5 text-right">{item.averageMs.toFixed(2)}ms</td>
                                        <td class="px-3 py-1.5 text-right text-green-600 dark:text-green-400">{item.minMs.toFixed(2)}ms</td>
                                        <td class="px-3 py-1.5 text-right text-orange-600 dark:text-orange-400">{item.maxMs.toFixed(2)}ms</td>
                                        <td class="px-3 py-1.5 text-right text-slate-500">{item.stdDevMs.toFixed(2)}ms</td>
                                        <td class="px-3 py-1.5 text-right">{item.runs}</td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    {:else if isRunning}
                        <div class="flex flex-col items-center py-12 text-slate-500">
                            <span class="material-symbols-outlined text-3xl animate-spin mb-2">sync</span>
                            <p class="text-xs">Running benchmarks...</p>
                        </div>
                    {:else}
                        <div class="flex flex-col items-center py-12 text-slate-500">
                            <span class="material-symbols-outlined text-3xl mb-2">speed</span>
                            <p class="text-xs">Select a video and click "Run" to benchmark</p>
                        </div>
                    {/if}
                </div>
            </Panel>
        </div>
    </div>
</PageContent>
