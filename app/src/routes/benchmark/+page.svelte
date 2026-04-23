<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import type { BenchmarkResponse, BenchmarkResultItem } from '$lib/types';
  import { getFileName } from '$lib/utils/format';

  let videoPath = $state('');
  let runs = $state(3);
  let operations = $state<Record<string, boolean>>({ context_init: true, metadata: true, frame_extraction: true });
  let isRunning = $state(false);
  let error = $state('');
  let result = $state<BenchmarkResponse | null>(null);

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

  let maxAvg = $derived(result ? Math.max(...result.results.map(r => r.averageMs)) : 1);
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
                <RunButton loading={isRunning} disabled={!videoPath} onclick={runBenchmark} />
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                <FilePicker bind:value={videoPath} label="Video File" filters={[{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov'] }]} />

                <FormField label="Runs per operation = {runs}">
                    <input type="range" bind:value={runs} min="1" max="20" step="1" class="w-full" />
                </FormField>

                <FormField label="Operations">
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
                </FormField>

                {#if error}
                    <ErrorAlert message={error} />
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
                                    <span class="text-meta w-32 truncate text-right">{item.name}</span>
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
                        <EmptyState icon="sync" message="Running benchmarks..." />
                    {:else}
                        <EmptyState icon="speed" message='Select a video and click "Run" to benchmark' />
                    {/if}
                </div>
            </Panel>
        </div>
    </div>
</PageContent>
