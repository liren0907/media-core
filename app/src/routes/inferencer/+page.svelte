<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';

  let videoPath = '';
  let annotationPath = '';
  let outputDirectory = '';
  let videoPlayer: HTMLVideoElement;
  let availableLabels: string[] = [];
  let selectedLabels: string[] = [];
  let isProcessing = false;
  let processedVideoPath = '';
  let errorMessage = '';

  // UI State for tabs
  let activeTab = 'configuration';

  async function openVideoFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Video', extensions: ['mp4'] }]
      });

      if (selected) {
        videoPath = selected as string;
        outputDirectory = videoPath.substring(0, videoPath.lastIndexOf('/'));
        if (videoPlayer) {
          videoPlayer.src = convertFileSrc(videoPath);
        }
        errorMessage = '';
      }
    } catch (error) {
      console.error('Error opening video file:', error);
      errorMessage = `Error opening video: ${error}`;
    }
  }

  async function openAnnotationFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });

      if (selected) {
        annotationPath = selected as string;
        const result = await invoke('read_annotation_file', { path: annotationPath });
        const data = result as any;
        availableLabels = data.unique_labels;
        selectedLabels = [...availableLabels];
        errorMessage = '';
      }
    } catch (error) {
      console.error('Error opening annotation file:', error);
      errorMessage = `Error reading annotation file: ${error}`;
    }
  }

  function toggleLabel(label: string) {
    if (selectedLabels.includes(label)) {
      selectedLabels = selectedLabels.filter(l => l !== label);
    } else {
      selectedLabels = [...selectedLabels, label];
    }
  }

  function selectAllLabels() {
    selectedLabels = [...availableLabels];
  }

  function deselectAllLabels() {
    selectedLabels = [];
  }

  async function processVideo() {
    if (!videoPath || !annotationPath || selectedLabels.length === 0) {
      errorMessage = 'Please select video, annotation file, and at least one label';
      return;
    }

    isProcessing = true;
    errorMessage = '';
    
    try {
      const payload = {
        video_path: videoPath,
        annotation_path: annotationPath,
        output_directory: outputDirectory,
        label_selected: selectedLabels
      };

      // Pass object directly, do not JSON.stringify
      const result = await invoke('start_video_annotation', { payload });
      const response = result as any;

      if (response.status === 'success') {
        processedVideoPath = response.data.output_video;
      } else {
        throw new Error(response.message || 'Processing failed');
      }
    } catch (error) {
      console.error('Error processing video:', error);
      errorMessage = `Error processing video: ${error}`;
    } finally {
      isProcessing = false;
    }
  }
</script>

<div class="flex h-full w-full bg-[#101922] overflow-hidden">
    <!-- Left Sidebar: Configuration -->
    <aside class="w-[360px] flex flex-col border-r border-[#2d3b4f] bg-[#151c24] overflow-y-auto shrink-0">
        <!-- Section: File Inputs -->
        <div class="p-5 border-b border-[#2d3b4f]">
            <h3 class="text-slate-400 text-xs font-bold uppercase tracking-wider mb-4">Input Sources</h3>
            
            <div class="flex flex-col gap-4">
                <div>
                    <label for="sourceVideoPath" class="text-xs font-medium text-slate-500 mb-1 block">Source Video</label>
                    <div class="flex gap-2">
                        <input id="sourceVideoPath" type="text" value={videoPath ? '...'+videoPath.slice(-20) : ''} readonly class="flex-1 bg-[#1e2936] border border-[#2d3b4f] rounded text-xs text-white px-2 py-1.5 focus:outline-none" placeholder="No file selected">
                        <button on:click={openVideoFile} class="bg-[#137fec] hover:bg-[#0f6bd0] text-white px-3 py-1.5 rounded text-xs font-bold transition-colors">
                            BROWSE
                        </button>
                    </div>
                </div>

                <div>
                    <label for="annotationDataPath" class="text-xs font-medium text-slate-500 mb-1 block">Annotation Data</label>
                    <div class="flex gap-2">
                        <input id="annotationDataPath" type="text" value={annotationPath ? '...'+annotationPath.slice(-20) : ''} readonly class="flex-1 bg-[#1e2936] border border-[#2d3b4f] rounded text-xs text-white px-2 py-1.5 focus:outline-none" placeholder="No file selected">
                        <button on:click={openAnnotationFile} class="bg-[#1e2936] hover:bg-[#2d3b4f] border border-[#2d3b4f] text-slate-300 px-3 py-1.5 rounded text-xs font-bold transition-colors">
                            LOAD
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Section: Labels -->
        <div class="p-5 border-b border-[#2d3b4f] flex flex-col gap-4 flex-1">
            <div class="flex items-center justify-between">
                <h3 class="text-slate-400 text-xs font-bold uppercase tracking-wider">Object Classes</h3>
                <div class="flex gap-2">
                    <button on:click={selectAllLabels} class="text-[10px] text-[#137fec] hover:text-white font-bold uppercase">All</button>
                    <button on:click={deselectAllLabels} class="text-[10px] text-slate-500 hover:text-white font-bold uppercase">None</button>
                </div>
            </div>

            {#if availableLabels.length > 0}
                <div class="flex flex-col gap-2 overflow-y-auto max-h-[300px] pr-2">
                    {#each availableLabels as label}
                        <label class="flex items-center gap-3 p-2 rounded bg-[#1e2936] border border-[#2d3b4f] hover:border-[#137fec]/50 cursor-pointer transition-colors group">
                            <input 
                                type="checkbox" 
                                checked={selectedLabels.includes(label)} 
                                on:change={() => toggleLabel(label)}
                                class="rounded bg-[#101922] border-slate-600 text-[#137fec] focus:ring-offset-0 focus:ring-0 w-4 h-4" 
                            />
                            <span class="text-sm text-slate-300 group-hover:text-white font-mono">{label}</span>
                        </label>
                    {/each}
                </div>
            {:else}
                <div class="p-4 rounded border border-dashed border-[#2d3b4f] text-center">
                    <p class="text-xs text-slate-500">Load annotation file to see classes</p>
                </div>
            {/if}
        </div>

        <!-- Action Area -->
        <div class="p-5 mt-auto">
            {#if errorMessage}
                <div class="mb-4 p-3 rounded bg-red-900/20 border border-red-900/50 text-red-400 text-xs">
                    {errorMessage}
                </div>
            {/if}

            <button 
                on:click={processVideo}
                disabled={isProcessing || !videoPath || !annotationPath}
                class="w-full py-3 rounded bg-[#137fec] hover:bg-[#0f6bd0] text-white font-bold text-sm shadow-[0_0_20px_rgba(19,127,236,0.2)] disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 transition-all"
            >
                {#if isProcessing}
                    <span class="material-symbols-outlined animate-spin text-sm">sync</span>
                    PROCESSING...
                {:else}
                    <span class="material-symbols-outlined text-sm">play_arrow</span>
                    RUN INFERENCE
                {/if}
            </button>
        </div>
    </aside>

    <!-- Main Workspace -->
    <main class="flex-1 flex flex-col bg-[#101922] p-6 overflow-hidden gap-6">
        <!-- Video Stage -->
        <div class="flex flex-col gap-2 flex-1 min-h-0">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <span class="material-symbols-outlined text-[#137fec] animate-pulse">analytics</span>
                    <h2 class="text-white font-medium tracking-tight">Analytics Preview</h2>
                    <span class="bg-[#1e2936] border border-[#2d3b4f] text-slate-400 text-[10px] px-1.5 py-0.5 rounded font-mono">1080p</span>
                </div>
                <div class="flex gap-4 text-xs font-mono text-slate-400">
                    {#if processedVideoPath}
                        <span class="flex items-center gap-1.5"><span class="size-2 rounded-full bg-green-500"></span> Result Ready</span>
                    {:else}
                        <span class="flex items-center gap-1.5"><span class="size-2 rounded-full bg-yellow-500"></span> Awaiting Input</span>
                    {/if}
                </div>
            </div>

            <!-- Video Container -->
            <div class="relative flex-1 bg-black rounded-lg border border-[#2d3b4f] overflow-hidden shadow-2xl flex items-center justify-center">
                {#if processedVideoPath}
                    <video src={convertFileSrc(processedVideoPath)} controls class="w-full h-full object-contain">
                        <track kind="captions" />
                    </video>
                {:else if videoPath}
                    <video bind:this={videoPlayer} controls class="w-full h-full object-contain opacity-80">
                        <track kind="captions" />
                    </video>
                    <!-- Grid Overlay -->
                    <div class="absolute inset-0 pointer-events-none opacity-10" style="background-image: linear-gradient(to right, #2d3b4f 1px, transparent 1px), linear-gradient(to bottom, #2d3b4f 1px, transparent 1px); background-size: 40px 40px;"></div>
                {:else}
                    <div class="flex flex-col items-center gap-4 text-slate-600">
                        <span class="material-symbols-outlined text-6xl">perm_media</span>
                        <p class="text-sm font-medium uppercase tracking-wider">Select Video Source</p>
                    </div>
                {/if}
            </div>
        </div>

        <!-- Bottom Data Panel -->
        <div class="h-48 grid grid-cols-3 gap-6 shrink-0">
            <!-- Chart 1 -->
            <div class="col-span-2 flex flex-col rounded-lg bg-[#1e2936] border border-[#2d3b4f] p-4 shadow-lg">
                <div class="flex justify-between items-center mb-2">
                    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-wider">Detection Timeline</h3>
                    <div class="flex gap-2">
                        <span class="size-2 rounded-full bg-[#137fec]"></span>
                        <span class="text-[10px] text-slate-500 uppercase">Confidence</span>
                    </div>
                </div>
                <div class="flex-1 flex items-end gap-1 overflow-hidden relative">
                    <!-- Placeholder Visual -->
                    <div class="w-full h-full flex items-end gap-[2px] opacity-50">
                        {#each Array(40) as _}
                            <div class="bg-[#137fec]/30 flex-1 rounded-t-sm" style="height: {Math.random() * 100}%"></div>
                        {/each}
                    </div>
                </div>
            </div>

            <!-- Stats -->
            <div class="col-span-1 flex flex-col rounded-lg bg-[#1e2936] border border-[#2d3b4f] p-4 shadow-lg">
                <div class="flex justify-between items-center mb-2">
                    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-wider">Session Stats</h3>
                </div>
                <div class="flex flex-col justify-center gap-4 flex-1">
                    <div class="flex items-center justify-between">
                        <span class="text-sm text-slate-300">Selected Classes</span>
                        <span class="text-lg font-bold text-white font-mono">{selectedLabels.length}</span>
                    </div>
                    <div class="w-full h-1.5 bg-slate-700 rounded-full overflow-hidden">
                        <div class="h-full bg-blue-500" style="width: {availableLabels.length > 0 ? (selectedLabels.length / availableLabels.length) * 100 : 0}%"></div>
                    </div>
                    
                    <div class="flex items-center justify-between mt-2">
                        <span class="text-sm text-slate-300">Processing Status</span>
                        <span class="text-xs font-bold {isProcessing ? 'text-yellow-500' : 'text-green-500'} font-mono uppercase">{isProcessing ? 'BUSY' : 'IDLE'}</span>
                    </div>
                </div>
            </div>
        </div>
    </main>
</div>
