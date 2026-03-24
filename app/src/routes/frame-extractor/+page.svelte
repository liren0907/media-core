<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar } from '$lib/components/ui';

  // Config state
  let videoPath = $state('');
  let mode = $state<'preview' | 'disk'>('preview');
  let strategy = $state<'every_nth' | 'first_n' | 'range' | 'keyframes'>('every_nth');
  let strategyParam = $state(30);
  let rangeStart = $state(0);
  let rangeEnd = $state(100);
  let scaleFactor = $state(1.0);

  // Disk mode config
  let outputDir = $state('');
  let extractionMode = $state('opencv_interval');

  // State
  let isProcessing = $state(false);
  let error = $state('');
  let extractedFrames: { index: number; data: string }[] = $state([]);
  let diskResult = $state<{ outputDir: string; frameCount: number } | null>(null);
  let selectedFrame: string | null = $state(null);

  async function selectVideo() {
    try {
      const file = await open({ filters: [{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov', 'webm'] }] });
      if (file) { videoPath = file as string; error = ''; }
    } catch (e) { error = `Failed to select video: ${e}`; }
  }

  async function selectOutputDir() {
    try {
      const dir = await open({ directory: true });
      if (dir) outputDir = dir as string;
    } catch (e) { error = `Failed to select directory: ${e}`; }
  }

  async function runExtraction() {
    if (!videoPath) { error = 'Select a video file first'; return; }
    isProcessing = true;
    error = '';
    extractedFrames = [];
    diskResult = null;
    selectedFrame = null;

    try {
      if (mode === 'preview') {
        const result = await invoke('execute_extract_frames_pipeline', {
          config: {
            videoPath,
            strategy,
            strategyParam: strategy === 'every_nth' || strategy === 'first_n' ? strategyParam : undefined,
            rangeStart: strategy === 'range' ? rangeStart : undefined,
            rangeEnd: strategy === 'range' ? rangeEnd : undefined,
            scaleFactor: scaleFactor < 1.0 ? scaleFactor : undefined,
          }
        });
        extractedFrames = result as { index: number; data: string }[];
      } else {
        if (!outputDir) { error = 'Select output directory'; isProcessing = false; return; }
        const result = await invoke('execute_extract_to_disk_pipeline', {
          config: {
            videoPath,
            outputDir,
            interval: strategy === 'every_nth' ? strategyParam : undefined,
            extractionMode,
          }
        });
        diskResult = result as { outputDir: string; frameCount: number };
      }
    } catch (e) {
      error = `Extraction failed: ${e}`;
    }

    isProcessing = false;
  }

  function getFileName(path: string): string {
    return path.split('/').pop() || path;
  }

  const strategyLabels: Record<string, string> = {
    every_nth: 'Every Nth Frame',
    first_n: 'First N Frames',
    range: 'Frame Range',
    keyframes: 'Keyframes Only',
  };

  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]';
</script>

<svelte:head>
  <title>Frame Extractor</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Video" icon="movie" iconColor="text-[#137fec]" value={videoPath ? getFileName(videoPath) : 'None'} />
        <StatCard label="Strategy" icon="tune" iconColor="text-purple-500" value={strategyLabels[strategy]} />
        <StatCard label="Frames" icon="photo_library" iconColor="text-orange-500" value="{String(extractedFrames.length || diskResult?.frameCount || 0)}" />
        <StatCard label="Status" icon="info" iconColor="text-green-500" value={isProcessing ? 'Processing' : extractedFrames.length > 0 || diskResult ? 'Done' : 'Ready'}>
            {#snippet extra()}<StatusBadge status={isProcessing ? 'connecting' : extractedFrames.length > 0 || diskResult ? 'active' : 'idle'} />{/snippet}
        </StatCard>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Config Panel -->
        <Panel title="Configuration" icon="tune">
            {#snippet actions()}
                <button onclick={runExtraction} disabled={isProcessing || !videoPath} class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold disabled:opacity-50 transition-colors">
                    {#if isProcessing}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{:else}<span class="material-symbols-outlined text-[14px]">play_arrow</span>{/if}
                    Extract
                </button>
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                <!-- Video selection -->
                <div class="flex flex-col gap-1">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Video File</label>
                    <div class="flex gap-1.5">
                        <input type="text" readonly value={videoPath ? getFileName(videoPath) : ''} class="{inputClass} flex-1" placeholder="Select video..." />
                        <button onclick={selectVideo} class="px-2 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors">Browse</button>
                    </div>
                </div>

                <!-- Mode toggle -->
                <div class="flex flex-col gap-1">
                    <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Mode</label>
                    <div class="flex gap-1">
                        <button onclick={() => mode = 'preview'} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {mode === 'preview' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Preview</button>
                        <button onclick={() => mode = 'disk'} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {mode === 'disk' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Export to Disk</button>
                    </div>
                </div>

                <!-- Strategy -->
                <div class="flex flex-col gap-1">
                    <label for="strategy" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Strategy</label>
                    <select id="strategy" bind:value={strategy} class="{inputClass} w-full">
                        <option value="every_nth">Every Nth Frame</option>
                        <option value="first_n">First N Frames</option>
                        <option value="range">Frame Range</option>
                        <option value="keyframes">Keyframes Only</option>
                    </select>
                </div>

                <!-- Strategy params (conditional) -->
                {#if strategy === 'every_nth'}
                    <div class="flex flex-col gap-1">
                        <label for="nth" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">N (extract every Nth frame) = {strategyParam}</label>
                        <input id="nth" type="range" bind:value={strategyParam} min="1" max="120" step="1" class="w-full" />
                    </div>
                {:else if strategy === 'first_n'}
                    <div class="flex flex-col gap-1">
                        <label for="firstn" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">N (first N frames) = {strategyParam}</label>
                        <input id="firstn" type="range" bind:value={strategyParam} min="1" max="200" step="1" class="w-full" />
                    </div>
                {:else if strategy === 'range'}
                    <div class="grid grid-cols-2 gap-2">
                        <div class="flex flex-col gap-1">
                            <label for="rangeStart" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Start</label>
                            <input id="rangeStart" type="number" bind:value={rangeStart} min="0" class="{inputClass} w-full" />
                        </div>
                        <div class="flex flex-col gap-1">
                            <label for="rangeEnd" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">End</label>
                            <input id="rangeEnd" type="number" bind:value={rangeEnd} min="1" class="{inputClass} w-full" />
                        </div>
                    </div>
                {/if}

                <!-- Scale factor -->
                {#if mode === 'preview'}
                    <div class="flex flex-col gap-1">
                        <label for="scale" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Scale = {scaleFactor.toFixed(2)}</label>
                        <input id="scale" type="range" bind:value={scaleFactor} min="0.25" max="1" step="0.05" class="w-full" />
                    </div>
                {/if}

                <!-- Disk mode options -->
                {#if mode === 'disk'}
                    <div class="flex flex-col gap-1">
                        <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Output Directory</label>
                        <div class="flex gap-1.5">
                            <input type="text" readonly value={outputDir ? getFileName(outputDir) : ''} class="{inputClass} flex-1" placeholder="Select..." />
                            <button onclick={selectOutputDir} class="px-2 py-2 bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] rounded text-[10px] font-bold transition-colors">Browse</button>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1">
                        <label for="extractionMode" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Engine</label>
                        <select id="extractionMode" bind:value={extractionMode} class="{inputClass} w-full">
                            <option value="opencv_sequential">OpenCV Sequential</option>
                            <option value="opencv_interval">OpenCV Interval</option>
                            <option value="ffmpeg">FFmpeg</option>
                            <option value="parallel">Parallel</option>
                        </select>
                    </div>
                {/if}

                {#if error}
                    <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-[10px]">{error}</div>
                {/if}
            </div>
        </Panel>

        <!-- Preview / Results -->
        <div class="lg:col-span-2 flex flex-col gap-3">
            {#if mode === 'preview'}
                <Panel title="Extracted Frames" icon="photo_library">
                    {#snippet actions()}
                        {#if extractedFrames.length > 0}
                            <span class="text-[10px] px-1.5 py-0.5 rounded bg-[#137fec]/10 text-[#137fec] font-bold">{extractedFrames.length} frames</span>
                        {/if}
                    {/snippet}
                    <div class="p-3">
                        {#if selectedFrame}
                            <div class="mb-3 relative">
                                <img src="data:image/jpeg;base64,{selectedFrame}" alt="Selected frame" class="w-full rounded border border-slate-200 dark:border-[#2a3441]" />
                                <button onclick={() => selectedFrame = null} class="absolute top-2 right-2 p-1 bg-black/60 text-white rounded hover:bg-black/80 transition-colors">
                                    <span class="material-symbols-outlined text-[16px]">close</span>
                                </button>
                            </div>
                        {/if}

                        {#if extractedFrames.length > 0}
                            <div class="grid grid-cols-4 sm:grid-cols-6 gap-1.5">
                                {#each extractedFrames as frame}
                                    <button onclick={() => selectedFrame = frame.data} class="aspect-video rounded overflow-hidden border-2 border-transparent hover:border-[#137fec] transition-colors relative group">
                                        <img src="data:image/jpeg;base64,{frame.data}" alt="Frame {frame.index}" class="w-full h-full object-cover" />
                                        <div class="absolute bottom-0 left-0 right-0 bg-black/60 text-white text-[9px] text-center font-mono py-0.5">#{frame.index}</div>
                                    </button>
                                {/each}
                            </div>
                        {:else if isProcessing}
                            <div class="flex flex-col items-center py-12 text-slate-500">
                                <span class="material-symbols-outlined text-3xl animate-spin mb-2">sync</span>
                                <p class="text-xs">Extracting frames...</p>
                            </div>
                        {:else}
                            <div class="flex flex-col items-center py-12 text-slate-500">
                                <span class="material-symbols-outlined text-3xl mb-2">photo_library</span>
                                <p class="text-xs">Select a video and click "Extract" to preview frames</p>
                            </div>
                        {/if}
                    </div>
                </Panel>
            {:else}
                <Panel title="Export Status" icon="save">
                    <div class="p-3">
                        {#if diskResult}
                            <div class="flex flex-col items-center py-8 gap-3">
                                <span class="material-symbols-outlined text-4xl text-green-500">check_circle</span>
                                <div class="text-center">
                                    <p class="text-sm font-bold text-slate-900 dark:text-white">Export Complete</p>
                                    <p class="text-xs text-slate-500 mt-1">{diskResult.frameCount} frames saved to:</p>
                                    <p class="text-xs font-mono text-slate-600 dark:text-slate-400 mt-1 break-all">{diskResult.outputDir}</p>
                                </div>
                            </div>
                        {:else if isProcessing}
                            <div class="flex flex-col items-center py-12 text-slate-500">
                                <span class="material-symbols-outlined text-3xl animate-spin mb-2">sync</span>
                                <p class="text-xs">Extracting frames to disk...</p>
                            </div>
                        {:else}
                            <div class="flex flex-col items-center py-12 text-slate-500">
                                <span class="material-symbols-outlined text-3xl mb-2">save</span>
                                <p class="text-xs">Configure and click "Extract" to save frames to disk</p>
                            </div>
                        {/if}
                    </div>
                </Panel>
            {/if}
        </div>
    </div>
</PageContent>
