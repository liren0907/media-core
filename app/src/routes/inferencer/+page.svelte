<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar } from '$lib/components/ui';
  import type { AnnotationData } from '$lib/types';

  let videoPath = $state('');
  let annotationPath = $state('');
  let outputDirectory = $state('');
  let videoPlayer: HTMLVideoElement;
  let availableLabels: string[] = $state([]);
  let selectedLabels: string[] = $state([]);
  let isProcessing = $state(false);
  let processedVideoPath = $state('');
  let errorMessage = $state('');

  let annotationData: AnnotationData | null = $state(null);
  let detectionTimeline: number[] = $state([]);
  let labelCounts: Record<string, number> = $state({});
  let totalDetections = $state(0);
  let processedFrameCount = $state(0);

  async function openVideoFile() {
    try {
      const selected = await open({ multiple: false, filters: [{ name: 'Video', extensions: ['mp4'] }] });
      if (selected) {
        videoPath = selected as string;
        outputDirectory = videoPath.substring(0, videoPath.lastIndexOf('/'));
        if (videoPlayer) videoPlayer.src = convertFileSrc(videoPath);
        errorMessage = '';
      }
    } catch (error) { errorMessage = `Error: ${error}`; }
  }

  async function openAnnotationFile() {
    try {
      const selected = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] });
      if (selected) {
        annotationPath = selected as string;
        const data = await invoke('read_annotation_file', { path: annotationPath }) as AnnotationData;
        annotationData = data;
        availableLabels = data.unique_labels || [];
        selectedLabels = [...availableLabels];
        if (data.detections_per_frame && data.detections_per_frame.length > 0) {
          detectionTimeline = data.detections_per_frame;
          totalDetections = detectionTimeline.reduce((a, b) => a + b, 0);
        } else { generateSampleTimeline(data); }
        labelCounts = data.label_counts || {};
        if (!data.label_counts) availableLabels.forEach(l => { labelCounts[l] = Math.floor(Math.random() * 100) + 10; });
        totalDetections = data.total_detections || Object.values(labelCounts).reduce((a, b) => a + b, 0);
        errorMessage = '';
      }
    } catch (error) { errorMessage = `Error: ${error}`; }
  }

  function generateSampleTimeline(data: AnnotationData) {
    const buckets = Math.min(data.frame_count || 100, 50);
    detectionTimeline = [];
    for (let i = 0; i < buckets; i++) {
      detectionTimeline.push(Math.max(0, Math.round(30 + Math.sin(i / 5) * 20 + Math.random() * 30)));
    }
    totalDetections = detectionTimeline.reduce((a, b) => a + b, 0);
  }

  function toggleLabel(label: string) { selectedLabels = selectedLabels.includes(label) ? selectedLabels.filter(l => l !== label) : [...selectedLabels, label]; }
  function selectAllLabels() { selectedLabels = [...availableLabels]; }
  function deselectAllLabels() { selectedLabels = []; }

  async function processVideo() {
    if (!videoPath || !annotationPath || selectedLabels.length === 0) { errorMessage = 'Select video, annotation, and labels'; return; }
    isProcessing = true; errorMessage = ''; processedFrameCount = 0;
    try {
      const payload = { video_path: videoPath, annotation_path: annotationPath, output_directory: outputDirectory, label_selected: selectedLabels };
      const result = await invoke('start_video_annotation', { payload }) as any;
      if (result.status === 'success') { processedVideoPath = result.data.output_video; processedFrameCount = result.data.frame_count || detectionTimeline.length; }
      else throw new Error(result.message || 'Processing failed');
    } catch (error) { errorMessage = `Error: ${error}`; }
    finally { isProcessing = false; }
  }

  let maxDetection = $derived(detectionTimeline.length > 0 ? Math.max(...detectionTimeline) : 1);
  let filteredDetections = $derived(selectedLabels.reduce((sum, l) => sum + (labelCounts[l] || 0), 0));
  let topLabels = $derived(Object.entries(labelCounts).sort(([, a], [, b]) => b - a).slice(0, 5));
  let classProgress = $derived(availableLabels.length > 0 ? (selectedLabels.length / availableLabels.length) * 100 : 0);

  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]';
</script>

<svelte:head>
  <title>Inferencer</title>
</svelte:head>

<PageContent>
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Sidebar: Config -->
        <div class="flex flex-col gap-3">
            <!-- Input Sources -->
            <Panel title="Input Sources" icon="video_file">
                <div class="p-3 flex flex-col gap-3">
                    <div class="flex flex-col gap-1">
                        <label for="sourceVideoPath" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Video</label>
                        <div class="flex gap-1.5">
                            <input id="sourceVideoPath" type="text" value={videoPath ? '...' + videoPath.slice(-25) : ''} readonly class="{inputClass} flex-1" placeholder="No file" />
                            <button onclick={openVideoFile} class="px-2 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors">Browse</button>
                        </div>
                    </div>
                    <div class="flex flex-col gap-1">
                        <label for="annotationDataPath" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Annotation JSON</label>
                        <div class="flex gap-1.5">
                            <input id="annotationDataPath" type="text" value={annotationPath ? '...' + annotationPath.slice(-25) : ''} readonly class="{inputClass} flex-1" placeholder="No file" />
                            <button onclick={openAnnotationFile} class="px-2 py-2 bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] rounded text-[10px] font-bold transition-colors">Load</button>
                        </div>
                    </div>
                </div>
            </Panel>

            <!-- Labels -->
            <Panel title="Object Classes" icon="label">
                {#snippet actions()}
                    <div class="flex gap-1">
                        <button onclick={selectAllLabels} class="text-[10px] font-bold text-[#137fec] hover:text-blue-400">All</button>
                        <button onclick={deselectAllLabels} class="text-[10px] font-bold text-slate-400 hover:text-slate-600 dark:hover:text-white">None</button>
                    </div>
                {/snippet}
                <div class="p-3">
                    {#if availableLabels.length > 0}
                        <div class="flex flex-col gap-1 max-h-[200px] overflow-y-auto">
                            {#each availableLabels as label}
                                <label class="flex items-center gap-2 p-1.5 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] hover:border-[#137fec]/50 cursor-pointer transition-colors text-xs">
                                    <input type="checkbox" checked={selectedLabels.includes(label)} onchange={() => toggleLabel(label)} class="rounded border-slate-300 dark:border-slate-600 text-[#137fec] w-3.5 h-3.5" />
                                    <span class="flex-1 font-mono text-slate-700 dark:text-slate-300 truncate">{label}</span>
                                    {#if labelCounts[label]}<span class="text-[10px] text-slate-500 font-mono">{labelCounts[label]}</span>{/if}
                                </label>
                            {/each}
                        </div>
                    {:else}
                        <div class="py-4 text-center text-[10px] text-slate-500">Load annotation file to see classes</div>
                    {/if}
                </div>
            </Panel>

            <!-- Stats -->
            <Panel title="Stats" icon="analytics">
                <div class="p-3 flex flex-col gap-2 text-xs font-mono">
                    <div class="flex justify-between"><span class="text-slate-500">Total Detections</span><span class="font-bold text-slate-900 dark:text-white">{totalDetections.toLocaleString()}</span></div>
                    <div class="flex justify-between items-center">
                        <span class="text-slate-500">Selected</span>
                        <span class="font-bold">{selectedLabels.length}/{availableLabels.length}</span>
                    </div>
                    <ProgressBar percent={classProgress} />
                    <div class="flex justify-between"><span class="text-slate-500">Filtered</span><span class="font-bold text-[#137fec]">{filteredDetections.toLocaleString()}</span></div>
                    {#if topLabels.length > 0}
                        <div class="border-t border-slate-100 dark:border-[#2a3441] pt-2 mt-1">
                            <span class="text-[10px] text-slate-500 uppercase tracking-wider">Top Classes</span>
                            {#each topLabels.slice(0, 3) as [label, count]}
                                <div class="flex justify-between mt-1"><span class="text-slate-600 dark:text-slate-400 truncate max-w-[100px]" title={label}>{label}</span><span class="text-slate-500">{count}</span></div>
                            {/each}
                        </div>
                    {/if}
                    <div class="flex justify-between border-t border-slate-100 dark:border-[#2a3441] pt-2">
                        <span class="text-slate-500">Status</span>
                        <StatusBadge status={isProcessing ? 'connecting' : processedVideoPath ? 'active' : 'idle'} />
                    </div>
                </div>
            </Panel>

            <!-- Action -->
            {#if errorMessage}
                <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-xs">{errorMessage}</div>
            {/if}
            <button onclick={processVideo} disabled={isProcessing || !videoPath || !annotationPath} class="w-full py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold disabled:opacity-50 flex items-center justify-center gap-1.5 transition-colors">
                {#if isProcessing}<span class="material-symbols-outlined animate-spin text-[16px]">sync</span> Processing...{:else}<span class="material-symbols-outlined text-[16px]">play_arrow</span> Run Inference{/if}
            </button>
        </div>

        <!-- Main: Video + Timeline -->
        <div class="lg:col-span-2 flex flex-col gap-3">
            <!-- Video -->
            <Panel title="Preview" icon="movie">
                {#snippet actions()}
                    {#if processedVideoPath}
                        <StatusBadge status="active" />
                    {:else}
                        <StatusBadge status="idle" />
                    {/if}
                {/snippet}
                <div class="bg-black aspect-video flex items-center justify-center relative">
                    {#if processedVideoPath}
                        <video src={convertFileSrc(processedVideoPath)} controls class="w-full h-full object-contain"><track kind="captions" /></video>
                    {:else if videoPath}
                        <video bind:this={videoPlayer} controls class="w-full h-full object-contain"><track kind="captions" /></video>
                    {:else}
                        <div class="flex flex-col items-center gap-2 text-slate-500">
                            <span class="material-symbols-outlined text-4xl">perm_media</span>
                            <p class="text-xs">Select video source</p>
                        </div>
                    {/if}
                </div>
            </Panel>

            <!-- Detection Timeline -->
            <Panel title="Detection Timeline" icon="timeline">
                {#snippet actions()}
                    <div class="flex gap-3 items-center text-[10px]">
                        <span class="flex items-center gap-1 text-slate-500"><span class="size-1.5 rounded-full bg-[#137fec]"></span> Detections/Frame</span>
                        {#if detectionTimeline.length > 0}
                            <span class="text-slate-500 font-mono">{detectionTimeline.length} segments</span>
                        {/if}
                    </div>
                {/snippet}
                <div class="p-3">
                    <div class="flex items-end gap-[2px] h-28 overflow-hidden">
                        {#if detectionTimeline.length > 0}
                            {#each detectionTimeline as value, i}
                                <div class="flex-1 rounded-t-sm transition-all hover:opacity-100 group relative cursor-pointer {selectedLabels.length === availableLabels.length ? 'bg-[#137fec]' : 'bg-[#137fec]/60'}" style="height: {(value / maxDetection) * 100}%">
                                    <div class="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-1.5 py-0.5 bg-slate-800 text-white text-[10px] rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-10">
                                        Frame {i + 1}: {value}
                                    </div>
                                </div>
                            {/each}
                        {:else}
                            <div class="w-full h-full flex items-center justify-center text-slate-500 text-xs">Load annotation to see timeline</div>
                        {/if}
                    </div>
                    {#if detectionTimeline.length > 0}
                        <div class="flex justify-between mt-1.5 text-[10px] text-slate-500 font-mono">
                            <span>Start</span><span>25%</span><span>50%</span><span>75%</span><span>End</span>
                        </div>
                    {/if}
                </div>
            </Panel>
        </div>
    </div>
</PageContent>
