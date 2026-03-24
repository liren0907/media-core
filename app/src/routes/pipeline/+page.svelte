<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar } from '$lib/components/ui';
  import {
    executeBatchPipeline,
    getActivePipelines,
    cancelPipeline,
    onPipelineProgress,
    onPipelineComplete,
    type PipelineProgressEvent,
    type PipelineCompleteEvent,
    type BatchProcessConfig,
    type BatchProcessResult,
    type BatchItemResult,
  } from '$lib/events';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  // Config state
  let selectedFiles: string[] = $state([]);
  let operation = $state<'metadata' | 'motion' | 'hls' | 'extract'>('metadata');
  let parallel = $state(false);
  let isRunning = $state(false);
  let error = $state('');

  // Progress state
  let activePipelines: string[] = $state([]);
  let currentProgress = $state<PipelineProgressEvent | null>(null);

  // Results state
  let batchResult = $state<BatchProcessResult | null>(null);
  let completionEvent = $state<PipelineCompleteEvent | null>(null);

  async function selectFiles() {
    try {
      const files = await open({
        multiple: true,
        filters: [
          { name: 'Media', extensions: ['mp4', 'avi', 'mkv', 'mov', 'webm', 'jpg', 'jpeg', 'png', 'bmp'] },
        ],
      });
      if (files) {
        selectedFiles = Array.isArray(files) ? files as string[] : [files as string];
        error = '';
      }
    } catch (e) {
      error = `File selection failed: ${e}`;
    }
  }

  function removeFile(index: number) {
    selectedFiles = selectedFiles.filter((_, i) => i !== index);
  }

  function clearFiles() {
    selectedFiles = [];
  }

  async function runBatch() {
    if (selectedFiles.length === 0) {
      error = 'Select files first';
      return;
    }

    isRunning = true;
    error = '';
    batchResult = null;
    completionEvent = null;
    currentProgress = null;

    try {
      const config: BatchProcessConfig = {
        filePaths: selectedFiles,
        operation,
        parallel,
      };
      batchResult = await executeBatchPipeline(config);
    } catch (e) {
      error = `Batch processing failed: ${e}`;
    }

    isRunning = false;
  }

  async function handleCancel(pipelineId: string) {
    try {
      await cancelPipeline(pipelineId);
    } catch (e) {
      console.error('Failed to cancel pipeline:', e);
    }
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }

  // Event listeners
  $effect(() => {
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;

    // Poll active pipelines
    const fetchPipelines = async () => {
      try { activePipelines = await getActivePipelines(); } catch { /* ignore */ }
    };
    fetchPipelines();
    const pollInterval = setInterval(fetchPipelines, 3000);

    // Subscribe to progress events
    onPipelineProgress((event) => {
      currentProgress = event;
    }).then((fn) => { unlistenProgress = fn; });

    onPipelineComplete((event) => {
      completionEvent = event;
      currentProgress = null;
      fetchPipelines();
    }).then((fn) => { unlistenComplete = fn; });

    return () => {
      clearInterval(pollInterval);
      unlistenProgress?.();
      unlistenComplete?.();
    };
  });

  let progressPercent = $derived(currentProgress?.progressPercent ?? 0);

  const operationLabels: Record<string, string> = {
    metadata: 'Metadata Extraction',
    motion: 'Motion Detection',
    hls: 'HLS Conversion',
    extract: 'Frame Extraction',
  };
</script>

<svelte:head>
  <title>Pipeline Manager</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Files" icon="folder_open" iconColor="text-[#137fec]" value="{String(selectedFiles.length)}" sub="selected" />
        <StatCard label="Operation" icon="settings" iconColor="text-purple-500" value={operationLabels[operation]} />
        <StatCard label="Active Jobs" icon="conversion_path" iconColor="text-orange-500" value="{String(activePipelines.length)}" />
        <StatCard label="Progress" icon="trending_up" iconColor="text-green-500" value="{progressPercent.toFixed(0)}%">
            {#snippet extra()}
                <ProgressBar percent={progressPercent} />
            {/snippet}
        </StatCard>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Config Panel -->
        <Panel title="Batch Configuration" icon="settings">
            {#snippet actions()}
                <button onclick={runBatch} disabled={isRunning || selectedFiles.length === 0} class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold disabled:opacity-50 transition-colors">
                    {#if isRunning}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{:else}<span class="material-symbols-outlined text-[14px]">play_arrow</span>{/if}
                    Run
                </button>
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                <!-- File selection -->
                <div class="flex flex-col gap-1">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Files</label>
                    <button onclick={selectFiles} class="w-full py-2 border-2 border-dashed border-slate-200 dark:border-[#2a3441] rounded text-xs text-slate-500 hover:text-[#137fec] hover:border-[#137fec] transition-colors">
                        + Select Files
                    </button>
                    {#if selectedFiles.length > 0}
                        <div class="flex flex-col gap-1 max-h-[200px] overflow-y-auto mt-1">
                            {#each selectedFiles as file, i}
                                <div class="flex items-center justify-between px-2 py-1 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] text-[10px] font-mono">
                                    <span class="truncate max-w-[180px]" title={file}>{getFileName(file)}</span>
                                    <button onclick={() => removeFile(i)} class="text-slate-400 hover:text-red-500 transition-colors">
                                        <span class="material-symbols-outlined text-[14px]">close</span>
                                    </button>
                                </div>
                            {/each}
                        </div>
                        <button onclick={clearFiles} class="text-[10px] text-slate-400 hover:text-red-500 self-end">Clear all</button>
                    {/if}
                </div>

                <!-- Operation -->
                <div class="flex flex-col gap-1">
                    <label for="operation" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Operation</label>
                    <select id="operation" bind:value={operation} class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]">
                        <option value="metadata">Metadata Extraction</option>
                        <option value="motion">Motion Detection</option>
                        <option value="hls">HLS Conversion</option>
                        <option value="extract">Frame Extraction</option>
                    </select>
                </div>

                <!-- Parallel toggle -->
                <label class="flex items-center gap-2 cursor-pointer">
                    <div class="relative inline-flex items-center">
                        <input type="checkbox" bind:checked={parallel} class="sr-only peer">
                        <div class="w-9 h-5 bg-slate-200 rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#137fec]"></div>
                    </div>
                    <span class="text-xs text-slate-700 dark:text-slate-300">Parallel Processing</span>
                </label>

                {#if error}
                    <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-[10px]">{error}</div>
                {/if}
            </div>
        </Panel>

        <!-- Active Jobs + Results -->
        <div class="lg:col-span-2 flex flex-col gap-3">
            <!-- Progress -->
            {#if currentProgress}
                <Panel title="Current Job" icon="pending">
                    <div class="p-3 flex flex-col gap-2">
                        <div class="flex items-center justify-between text-xs">
                            <span class="font-mono text-slate-500">{currentProgress.pipelineId.slice(0, 8)}...</span>
                            <StatusBadge status={currentProgress.hasError ? 'error' : 'active'} />
                        </div>
                        <ProgressBar percent={currentProgress.progressPercent} />
                        <div class="flex justify-between text-[10px] text-slate-500 font-mono">
                            <span>{currentProgress.stepName}</span>
                            <span>{currentProgress.stepIndex + 1} / {currentProgress.totalSteps}</span>
                        </div>
                        {#if currentProgress.message}
                            <p class="text-[10px] text-slate-500">{currentProgress.message}</p>
                        {/if}
                    </div>
                </Panel>
            {/if}

            <!-- Active Pipelines -->
            <Panel title="Active Pipelines" icon="conversion_path">
                {#snippet actions()}
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-500 font-bold">{activePipelines.length}</span>
                {/snippet}
                <div class="p-3">
                    {#if activePipelines.length > 0}
                        <div class="flex flex-col gap-1.5">
                            {#each activePipelines as pipelineId}
                                <div class="flex items-center justify-between px-2 py-1.5 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                                    <div class="flex items-center gap-2">
                                        <StatusBadge status="active" />
                                        <span class="text-xs font-mono text-slate-700 dark:text-slate-300 truncate max-w-[200px]">{pipelineId}</span>
                                    </div>
                                    <button onclick={() => handleCancel(pipelineId)} class="px-2 py-0.5 text-[10px] font-bold text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors">Cancel</button>
                                </div>
                            {/each}
                        </div>
                    {:else}
                        <div class="text-center text-xs text-slate-500 py-4">No active pipelines</div>
                    {/if}
                </div>
            </Panel>

            <!-- Results -->
            <Panel title="Results" icon="task_alt">
                <div class="p-3">
                    {#if batchResult}
                        <div class="flex flex-col gap-3">
                            <!-- Summary -->
                            <div class="grid grid-cols-3 gap-2">
                                <div class="p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] text-center">
                                    <div class="text-lg font-bold text-slate-900 dark:text-white">{batchResult.total}</div>
                                    <div class="text-[10px] text-slate-500">Total</div>
                                </div>
                                <div class="p-2 rounded bg-green-50 dark:bg-green-900/10 border border-green-200 dark:border-green-900/30 text-center">
                                    <div class="text-lg font-bold text-green-600 dark:text-green-400">{batchResult.succeeded}</div>
                                    <div class="text-[10px] text-green-600 dark:text-green-400">Succeeded</div>
                                </div>
                                <div class="p-2 rounded bg-red-50 dark:bg-red-900/10 border border-red-200 dark:border-red-900/30 text-center">
                                    <div class="text-lg font-bold text-red-600 dark:text-red-400">{batchResult.failed}</div>
                                    <div class="text-[10px] text-red-600 dark:text-red-400">Failed</div>
                                </div>
                            </div>

                            <!-- Individual results -->
                            <table class="w-full text-xs font-mono">
                                <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
                                    <tr>
                                        <th class="px-3 py-1.5 font-medium text-left">FILE</th>
                                        <th class="px-3 py-1.5 font-medium text-left">STATUS</th>
                                        <th class="px-3 py-1.5 font-medium text-left">DETAILS</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                                    {#each batchResult.results as item}
                                        <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50">
                                            <td class="px-3 py-1.5 truncate max-w-[200px]" title={item.filePath}>{getFileName(item.filePath)}</td>
                                            <td class="px-3 py-1.5"><StatusBadge status={item.success ? 'active' : 'error'} /></td>
                                            <td class="px-3 py-1.5 text-slate-500 truncate max-w-[200px]">{item.error || 'OK'}</td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        </div>
                    {:else if completionEvent}
                        <div class="text-center py-4">
                            <div class="text-xs text-green-600 dark:text-green-400 font-bold">Batch Complete</div>
                            <div class="text-[10px] text-slate-500 mt-1">{completionEvent.succeeded}/{completionEvent.total} succeeded</div>
                        </div>
                    {:else}
                        <div class="text-center text-xs text-slate-500 py-6">Run a batch to see results here</div>
                    {/if}
                </div>
            </Panel>
        </div>
    </div>
</PageContent>
