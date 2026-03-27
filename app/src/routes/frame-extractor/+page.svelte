<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar, ErrorAlert, EmptyState, RunButton, FormField } from '$lib/components/ui';
  import { FilePicker, DirPicker } from '$lib/components/form';
  import { getFileName } from '$lib/utils/format';
  import { inputClass } from '$lib/utils/styles';

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

  const strategyLabels: Record<string, string> = {
    every_nth: 'Every Nth Frame',
    first_n: 'First N Frames',
    range: 'Frame Range',
    keyframes: 'Keyframes Only',
  };

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
                <RunButton loading={isProcessing} disabled={!videoPath} label="Extract" onclick={runExtraction} />
            {/snippet}
            <div class="p-3 flex flex-col gap-3">
                <!-- Video selection -->
                <FilePicker bind:value={videoPath} label="Video File" filters={[{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov', 'webm'] }]} />

                <!-- Mode toggle -->
                <FormField label="Mode">
                    <div class="flex gap-1">
                        <button onclick={() => mode = 'preview'} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {mode === 'preview' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Preview</button>
                        <button onclick={() => mode = 'disk'} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {mode === 'disk' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Export to Disk</button>
                    </div>
                </FormField>

                <!-- Strategy -->
                <FormField label="Strategy">
                    <select id="strategy" bind:value={strategy} class="{inputClass} w-full">
                        <option value="every_nth">Every Nth Frame</option>
                        <option value="first_n">First N Frames</option>
                        <option value="range">Frame Range</option>
                        <option value="keyframes">Keyframes Only</option>
                    </select>
                </FormField>

                <!-- Strategy params (conditional) -->
                {#if strategy === 'every_nth'}
                    <FormField label="N (extract every Nth frame) = {strategyParam}">
                        <input id="nth" type="range" bind:value={strategyParam} min="1" max="120" step="1" class="w-full" />
                    </FormField>
                {:else if strategy === 'first_n'}
                    <FormField label="N (first N frames) = {strategyParam}">
                        <input id="firstn" type="range" bind:value={strategyParam} min="1" max="200" step="1" class="w-full" />
                    </FormField>
                {:else if strategy === 'range'}
                    <div class="grid grid-cols-2 gap-2">
                        <FormField label="Start">
                            <input id="rangeStart" type="number" bind:value={rangeStart} min="0" class="{inputClass} w-full" />
                        </FormField>
                        <FormField label="End">
                            <input id="rangeEnd" type="number" bind:value={rangeEnd} min="1" class="{inputClass} w-full" />
                        </FormField>
                    </div>
                {/if}

                <!-- Scale factor -->
                {#if mode === 'preview'}
                    <FormField label="Scale = {scaleFactor.toFixed(2)}">
                        <input id="scale" type="range" bind:value={scaleFactor} min="0.25" max="1" step="0.05" class="w-full" />
                    </FormField>
                {/if}

                <!-- Disk mode options -->
                {#if mode === 'disk'}
                    <DirPicker bind:value={outputDir} label="Output Directory" />
                    <FormField label="Engine">
                        <select id="extractionMode" bind:value={extractionMode} class="{inputClass} w-full">
                            <option value="opencv_sequential">OpenCV Sequential</option>
                            <option value="opencv_interval">OpenCV Interval</option>
                            <option value="ffmpeg">FFmpeg</option>
                            <option value="parallel">Parallel</option>
                        </select>
                    </FormField>
                {/if}

                {#if error}
                    <ErrorAlert message={error} />
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
                            <EmptyState icon="photo_library" message="Select a video and click &quot;Extract&quot; to preview frames" />
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
                            <EmptyState icon="save" message="Configure and click &quot;Extract&quot; to save frames to disk" />
                        {/if}
                    </div>
                </Panel>
            {/if}
        </div>
    </div>
</PageContent>
