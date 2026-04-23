<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { Panel, StatCard, ErrorAlert } from '$lib/components/ui';

  let videoSrc = $state("");
  let videoPath = $state("");
  let annotationPath = $state("");
  let outputPath = $state("");
  let labels: string[] = $state([]);
  let selectedLabels: string[] = $state([]);
  let totalFrames = $state(0);
  let totalObjects = $state(0);
  let isProcessing = $state(false);
  let errorMessage = $state("");
  let annotationResponse: any = $state(null);
  let videoResult: any = $state("");

  function formatVideoInfo(jsonStr: string) {
    try {
      const data = typeof jsonStr === 'string' ? JSON.parse(jsonStr) : jsonStr;
      if (!data) return {};
      const get = (obj: any, path: string, def: any = "N/A") => path.split('.').reduce((a: any, p: string) => a && a[p], obj) ?? def;
      return {
        Codec: get(data, 'codec_name'),
        Duration: data.duration_seconds ? `${Math.floor(data.duration_seconds / 60)}m ${Math.round(data.duration_seconds % 60)}s` : "N/A",
        FPS: data.fps ? `${data.fps}` : "N/A",
        Frames: data.frame_count ? data.frame_count.toLocaleString() : "N/A",
        Resolution: get(data, 'resolution'),
      };
    } catch { return {}; }
  }

  async function openVideoFile() {
    try {
      const filePath = await open({ filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv"] }] });
      if (filePath) {
        videoPath = filePath as string;
        videoSrc = convertFileSrc(videoPath);
        videoResult = await invoke("get_video_info", { filename: videoPath });
        errorMessage = "";
      }
    } catch (error) { errorMessage = `Error loading video: ${error}`; videoSrc = ""; }
  }

  async function openAnnotationFile() {
    try {
      const selected = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] });
      if (selected) {
        annotationPath = selected as string;
        const data = await invoke('read_annotation_file', { path: annotationPath }) as any;
        labels = data.unique_labels;
        selectedLabels = [...labels];
        totalFrames = data.total_frames;
        totalObjects = data.total_objects;
        errorMessage = "";
      }
    } catch (error) { errorMessage = `Error reading annotation: ${error}`; }
  }

  function toggleLabel(label: string) {
    selectedLabels = selectedLabels.includes(label) ? selectedLabels.filter(l => l !== label) : [...selectedLabels, label];
  }
  function selectAllLabels() { selectedLabels = [...labels]; }
  function deselectAllLabels() { selectedLabels = []; }

  async function startAnnotation() {
    if (!videoPath || !annotationPath) { errorMessage = "Select both video and annotation files"; return; }
    if (selectedLabels.length === 0) { errorMessage = "Select at least one label"; return; }
    try {
      const selectedDir = await open({ directory: true, multiple: false });
      if (!selectedDir) { errorMessage = "No output directory selected"; return; }
      outputPath = selectedDir as string;
      isProcessing = true; errorMessage = ""; annotationResponse = null;
      const result = await invoke("start_video_annotation", {
        payload: JSON.stringify({ video_path: videoPath, annotation_path: annotationPath, output_directory: outputPath, label_selected: selectedLabels })
      });
      if (result) annotationResponse = result; else throw new Error("Failed to process video");
    } catch (error) { errorMessage = `Error: ${error}`; }
    finally { isProcessing = false; }
  }
</script>

<!-- Video Player -->
<Panel title="Video" icon="movie">
    <div class="bg-black aspect-video flex items-center justify-center">
        {#if videoSrc}
            <video src={videoSrc} controls class="w-full h-full object-contain"><track kind="captions" /></video>
        {:else}
            <div class="text-center text-slate-500">
                <span class="material-symbols-outlined text-4xl mb-2">movie</span>
                <p class="text-xs">No video selected</p>
            </div>
        {/if}
    </div>
</Panel>

<!-- File Selection -->
<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
    <Panel title="Input Source" icon="video_file">
        <div class="p-3 flex flex-col gap-2">
            <button onclick={openVideoFile} class="w-full px-3 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold transition-colors">Open Video File</button>
            <p class="text-meta truncate">{videoPath || "No file selected"}</p>
        </div>
    </Panel>
    <Panel title="Annotation Data" icon="description">
        <div class="p-3 flex flex-col gap-2">
            <button onclick={openAnnotationFile} class="w-full px-3 py-2 bg-purple-500 hover:bg-purple-600 text-white rounded text-xs font-bold transition-colors">Open JSON File</button>
            <p class="text-meta truncate">{annotationPath || "No file selected"}</p>
        </div>
    </Panel>
</div>

<!-- Video Info -->
{#if videoResult}
    <div class="grid grid-cols-2 lg:grid-cols-5 gap-3">
        {#each Object.entries(formatVideoInfo(videoResult)) as [key, value]}
            <StatCard label={key} icon="info" iconColor="text-slate-400" value={String(value)} />
        {/each}
    </div>
{/if}

<!-- Labels -->
{#if labels.length > 0}
    <Panel title="Labels" icon="label">
        {#snippet actions()}
            <div class="flex gap-1">
                <button onclick={selectAllLabels} class="px-2 py-0.5 text-[10px] font-bold text-slate-500 hover:text-[#137fec] border border-slate-200 dark:border-[#2a3441] rounded transition-colors">All</button>
                <button onclick={deselectAllLabels} class="px-2 py-0.5 text-[10px] font-bold text-slate-500 hover:text-red-500 border border-slate-200 dark:border-[#2a3441] rounded transition-colors">None</button>
            </div>
        {/snippet}
        <div class="p-3">
            <div class="flex flex-wrap gap-1.5">
                {#each labels as label}
                    <button onclick={() => toggleLabel(label)}
                        class="px-2 py-1 rounded text-xs font-bold transition-colors {selectedLabels.includes(label) ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] text-slate-600 dark:text-slate-300'}">
                        {label}
                    </button>
                {/each}
            </div>
            <div class="mt-2 text-center text-[10px] text-slate-500">Total objects: {totalObjects}</div>
        </div>
    </Panel>
{/if}

<!-- Action -->
<button onclick={startAnnotation} disabled={isProcessing} class="w-full py-2.5 bg-[#137fec] hover:bg-blue-600 text-white rounded font-bold text-sm disabled:opacity-50 flex items-center justify-center gap-2 transition-colors">
    {#if isProcessing}<span class="material-symbols-outlined animate-spin text-[18px]">sync</span> Processing...{:else}Start Annotation{/if}
</button>

<!-- Alerts -->
{#if errorMessage}
    <ErrorAlert message={errorMessage} />
{/if}
{#if annotationResponse}
    <div class="p-2 rounded bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-900/30 text-green-600 dark:text-green-400 text-xs flex items-center gap-2">
        <span class="material-symbols-outlined text-[16px]">check_circle</span>
        <div><span class="font-bold">Success!</span> Output: {annotationResponse.data.output_video}</div>
    </div>
{/if}
