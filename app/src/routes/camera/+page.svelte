<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { PageContent, Panel, StatusBadge } from '$lib/components/ui';
  import type { CameraCaptureResult } from '$lib/types';

  let availableCameras: string[] = $state([]);
  let selectedCameraIndex = $state(0);
  let isLoading = $state(false);
  let isCapturing = $state(false);
  let error = $state("");

  let lastCapture: CameraCaptureResult | null = $state(null);
  let previewImage: string | null = $state(null);
  let captureHistory: { timestamp: Date; path: string | null; data: string }[] = $state([]);
  let isPreviewActive = $state(false);
  let previewInterval: ReturnType<typeof setInterval> | null = $state(null);

  async function loadCameras() {
    isLoading = true;
    error = "";
    try {
      availableCameras = await invoke('get_available_cameras');
      if (availableCameras.length === 0) {
        error = "No cameras detected. Please connect a camera and refresh.";
      }
    } catch (e) {
      error = `Failed to detect cameras: ${e}`;
    }
    isLoading = false;
  }

  async function checkCameraAccess(): Promise<boolean> {
    try {
      return await invoke('check_camera_access', { index: selectedCameraIndex }) as boolean;
    } catch (e) {
      return false;
    }
  }

  async function captureSnapshot(saveToFile = false) {
    isCapturing = true;
    error = "";
    try {
      let outputPath: string | null = null;
      if (saveToFile) {
        outputPath = await save({
          filters: [{ name: "Image", extensions: ["jpg", "jpeg", "png"] }],
          defaultPath: `camera_${selectedCameraIndex}_${Date.now()}.jpg`,
        }) as string | null;
        if (!outputPath) { isCapturing = false; return; }
      }
      const result: CameraCaptureResult = await invoke('execute_camera_pipeline', {
        config: { cameraId: selectedCameraIndex, outputPath }
      });
      lastCapture = result;
      if (result.frameData) {
        previewImage = `data:image/jpeg;base64,${result.frameData}`;
        captureHistory = [
          { timestamp: new Date(), path: outputPath, data: result.frameData },
          ...captureHistory.slice(0, 9),
        ];
      }
    } catch (e) {
      error = `Capture failed: ${e}`;
    }
    isCapturing = false;
  }

  async function startLivePreview() {
    if (isPreviewActive) return;
    const accessible = await checkCameraAccess();
    if (!accessible) { error = `Camera ${selectedCameraIndex} is not accessible`; return; }
    isPreviewActive = true;
    error = "";
    previewInterval = setInterval(async () => {
      if (!isPreviewActive) return;
      try {
        const result: CameraCaptureResult = await invoke('execute_camera_pipeline', {
          config: { cameraId: selectedCameraIndex, outputPath: null }
        });
        if (result.frameData) previewImage = `data:image/jpeg;base64,${result.frameData}`;
      } catch (e) {
        console.error('Preview frame error:', e);
      }
    }, 200);
  }

  function stopLivePreview() {
    isPreviewActive = false;
    if (previewInterval) { clearInterval(previewInterval); previewInterval = null; }
  }

  function selectHistoryItem(item: { data: string }) {
    previewImage = `data:image/jpeg;base64,${item.data}`;
  }

  function clearHistory() { captureHistory = []; }

  $effect(() => {
    loadCameras();
    return () => stopLivePreview();
  });

  let cameraName = $derived(availableCameras[selectedCameraIndex] ?? `Camera ${selectedCameraIndex}`);

  const liveColors: Record<string, string> = {
    live: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
  };
</script>

<svelte:head>
  <title>Camera Capture</title>
</svelte:head>

<PageContent>
    {#if error}
        <div class="p-2 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-xs">
            {error}
        </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Main Preview Area -->
        <div class="lg:col-span-2">
            <Panel title="Preview" icon="videocam">
                {#snippet actions()}
                    <div class="flex items-center gap-2">
                        {#if isPreviewActive}
                            <StatusBadge status="live" colorMap={liveColors} />
                        {/if}
                        {#if !isPreviewActive}
                            <button onclick={startLivePreview} disabled={availableCameras.length === 0} class="px-2 py-1 bg-green-500 hover:bg-green-600 text-white rounded text-[10px] font-bold transition-colors disabled:opacity-50">Start</button>
                        {:else}
                            <button onclick={stopLivePreview} class="px-2 py-1 bg-red-500 hover:bg-red-600 text-white rounded text-[10px] font-bold transition-colors">Stop</button>
                        {/if}
                    </div>
                {/snippet}
                <div class="aspect-video bg-black flex items-center justify-center relative">
                    {#if previewImage}
                        <img src={previewImage} alt="Camera preview" class="w-full h-full object-contain" />
                    {:else}
                        <div class="text-center text-slate-500">
                            <span class="material-symbols-outlined text-5xl mb-2">videocam</span>
                            <p class="text-xs">No preview — start preview or capture a snapshot</p>
                        </div>
                    {/if}
                    {#if isCapturing}
                        <div class="absolute inset-0 bg-white/20 flex items-center justify-center">
                            <span class="material-symbols-outlined text-3xl text-white animate-pulse">photo_camera</span>
                        </div>
                    {/if}
                </div>
                <div class="p-3 border-t border-slate-200 dark:border-[#2a3441] flex justify-center gap-3">
                    <button onclick={() => captureSnapshot(false)} disabled={isCapturing || availableCameras.length === 0} class="flex items-center gap-1.5 px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold transition-colors disabled:opacity-50">
                        <span class="material-symbols-outlined text-[18px]">photo_camera</span> Capture
                    </button>
                    <button onclick={() => captureSnapshot(true)} disabled={isCapturing || availableCameras.length === 0} class="flex items-center gap-1.5 px-4 py-2 bg-white dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-white rounded text-xs font-bold transition-colors disabled:opacity-50 hover:bg-slate-50 dark:hover:bg-[#283039]">
                        <span class="material-symbols-outlined text-[18px]">save</span> Capture & Save
                    </button>
                </div>
            </Panel>
        </div>

        <!-- Sidebar -->
        <div class="flex flex-col gap-3">
            <!-- Camera Selection -->
            <Panel title="Camera" icon="photo_camera">
                {#snippet actions()}
                    <button onclick={loadCameras} disabled={isLoading} class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400 disabled:opacity-50">
                        {isLoading ? 'Scanning...' : 'Refresh'}
                    </button>
                {/snippet}
                <div class="p-3">
                    {#if availableCameras.length > 0}
                        <div class="flex flex-col gap-1.5">
                            {#each availableCameras as camera, index}
                                <button onclick={() => { selectedCameraIndex = index; stopLivePreview(); }} class="w-full flex items-center gap-2 p-2 rounded-lg border transition-all text-left {selectedCameraIndex === index ? 'border-[#137fec] bg-[#137fec]/5' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-[#3b4754]'}">
                                    <span class="material-symbols-outlined text-[18px] text-slate-500">videocam</span>
                                    <div class="flex-1 min-w-0">
                                        <p class="text-xs font-medium text-slate-900 dark:text-white truncate">{camera}</p>
                                        <p class="text-[10px] text-slate-500 font-mono">Index: {index}</p>
                                    </div>
                                    {#if selectedCameraIndex === index}
                                        <span class="material-symbols-outlined text-[16px] text-[#137fec]">check_circle</span>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {:else}
                        <div class="text-center py-4 text-slate-500">
                            <span class="material-symbols-outlined text-2xl mb-1">videocam_off</span>
                            <p class="text-[10px]">No cameras detected</p>
                        </div>
                    {/if}
                </div>
            </Panel>

            <!-- Capture History -->
            <Panel title="Recent Captures" icon="photo_library">
                {#snippet actions()}
                    {#if captureHistory.length > 0}
                        <button onclick={clearHistory} class="text-[10px] font-bold uppercase tracking-wider text-slate-400 hover:text-red-500">Clear</button>
                    {/if}
                {/snippet}
                <div class="p-3">
                    {#if captureHistory.length > 0}
                        <div class="grid grid-cols-3 gap-1.5">
                            {#each captureHistory as item, i}
                                <button onclick={() => selectHistoryItem(item)} class="aspect-square rounded overflow-hidden border-2 border-transparent hover:border-[#137fec] transition-colors relative group">
                                    <img src="data:image/jpeg;base64,{item.data}" alt="Capture {i + 1}" class="w-full h-full object-cover" />
                                    <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                        <span class="material-symbols-outlined text-white text-[14px]">visibility</span>
                                    </div>
                                    {#if item.path}
                                        <div class="absolute bottom-0.5 right-0.5">
                                            <span class="material-symbols-outlined text-white text-[12px] drop-shadow">save</span>
                                        </div>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    {:else}
                        <div class="text-center py-4 text-slate-500">
                            <span class="material-symbols-outlined text-2xl mb-1">photo_library</span>
                            <p class="text-[10px]">No captures yet</p>
                        </div>
                    {/if}
                </div>
            </Panel>

            <!-- Last Capture Info -->
            {#if lastCapture && lastCapture.success}
                <Panel title="Last Capture" icon="info">
                    <div class="p-3 text-xs font-mono flex flex-col gap-1.5">
                        <div class="flex justify-between">
                            <span class="text-slate-500">Camera</span>
                            <span class="text-slate-900 dark:text-white">{cameraName}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-slate-500">Status</span>
                            <StatusBadge status="active" />
                        </div>
                        {#if lastCapture.outputPath}
                            <div class="flex justify-between">
                                <span class="text-slate-500">File</span>
                                <span class="truncate max-w-[120px]" title={lastCapture.outputPath}>{lastCapture.outputPath.split('/').pop()}</span>
                            </div>
                        {/if}
                    </div>
                </Panel>
            {/if}
        </div>
    </div>
</PageContent>
