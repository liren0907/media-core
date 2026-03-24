<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";

  // Camera state
  let availableCameras: string[] = $state([]);
  let selectedCameraIndex = $state(0);
  let isLoading = $state(false);
  let isCapturing = $state(false);
  let error = $state("");

  // Capture result
  interface CameraCaptureResult {
    cameraId: number;
    outputPath: string | null;
    frameData: string | null;
    success: boolean;
  }

  let lastCapture: CameraCaptureResult | null = $state(null);
  let previewImage: string | null = $state(null);
  let captureHistory: { timestamp: Date; path: string | null; data: string }[] = $state([]);

  // Live preview state
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
      const accessible = await invoke('check_camera_access', { index: selectedCameraIndex });
      return accessible as boolean;
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

        if (!outputPath) {
          isCapturing = false;
          return; // User cancelled
        }
      }

      const result: CameraCaptureResult = await invoke('execute_camera_pipeline', {
        config: {
          cameraId: selectedCameraIndex,
          outputPath: outputPath,
        }
      });

      lastCapture = result;
      
      if (result.frameData) {
        previewImage = `data:image/jpeg;base64,${result.frameData}`;
        
        // Add to history
        captureHistory = [
          { timestamp: new Date(), path: outputPath, data: result.frameData },
          ...captureHistory.slice(0, 9), // Keep last 10
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
    if (!accessible) {
      error = `Camera ${selectedCameraIndex} is not accessible`;
      return;
    }

    isPreviewActive = true;
    error = "";

    // Capture frames at ~5 FPS for preview
    previewInterval = setInterval(async () => {
      if (!isPreviewActive) return;
      
      try {
        const result: CameraCaptureResult = await invoke('execute_camera_pipeline', {
          config: {
            cameraId: selectedCameraIndex,
            outputPath: null,
          }
        });

        if (result.frameData) {
          previewImage = `data:image/jpeg;base64,${result.frameData}`;
        }
      } catch (e) {
        console.error('Preview frame error:', e);
      }
    }, 200); // 5 FPS
  }

  function stopLivePreview() {
    isPreviewActive = false;
    if (previewInterval) {
      clearInterval(previewInterval);
      previewInterval = null;
    }
  }

  function selectHistoryItem(item: { data: string }) {
    previewImage = `data:image/jpeg;base64,${item.data}`;
  }

  function clearHistory() {
    captureHistory = [];
  }

  $effect(() => {
    loadCameras();
    return () => stopLivePreview();
  });

  let cameraName = $derived(availableCameras[selectedCameraIndex] ?? `Camera ${selectedCameraIndex}`);
</script>

<svelte:head>
  <title>Camera Capture</title>
</svelte:head>

<div class="p-8 max-w-[1400px] w-full mx-auto space-y-8">
    <!-- Page Header -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div class="flex flex-col gap-2 max-w-2xl">
            <h2 class="text-slate-900 dark:text-white text-3xl font-bold font-display tracking-tight">Camera Capture</h2>
            <p class="text-slate-500 dark:text-[#9dabb9] text-base">Capture snapshots and preview live feeds from connected cameras.</p>
        </div>
        <div class="flex gap-3">
            <button 
                onclick={loadCameras}
                disabled={isLoading}
                class="flex items-center gap-2 h-10 px-4 rounded-lg bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] text-slate-600 dark:text-[#9dabb9] hover:text-slate-900 dark:hover:text-white font-medium transition-all text-sm font-display disabled:opacity-50"
            >
                <span class="material-symbols-outlined {isLoading ? 'animate-spin' : ''}" style="font-size: 20px;">refresh</span>
                Refresh Cameras
            </button>
        </div>
    </div>

    <!-- Error Display -->
    {#if error}
        <div class="p-4 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-sm">
            {error}
        </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Main Preview Area -->
        <div class="lg:col-span-2 flex flex-col rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d] flex justify-between items-center">
                <div class="flex items-center gap-3">
                    <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Preview</h3>
                    {#if isPreviewActive}
                        <span class="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-green-500/10 text-green-600 dark:text-green-400 text-xs font-bold">
                            <span class="size-1.5 rounded-full bg-green-500 animate-pulse"></span>
                            LIVE
                        </span>
                    {/if}
                </div>
                <div class="flex gap-2">
                    {#if !isPreviewActive}
                        <button 
                            onclick={startLivePreview}
                            disabled={availableCameras.length === 0}
                            class="px-3 py-1.5 bg-green-500 hover:bg-green-600 text-white rounded-lg font-medium text-xs transition-colors disabled:opacity-50"
                        >
                            Start Preview
                        </button>
                    {:else}
                        <button 
                            onclick={stopLivePreview}
                            class="px-3 py-1.5 bg-red-500 hover:bg-red-600 text-white rounded-lg font-medium text-xs transition-colors"
                        >
                            Stop Preview
                        </button>
                    {/if}
                </div>
            </div>
            
            <!-- Preview Display -->
            <div class="aspect-video bg-black flex items-center justify-center relative">
                {#if previewImage}
                    <img src={previewImage} alt="Camera preview" class="w-full h-full object-contain" />
                {:else}
                    <div class="text-center text-slate-500">
                        <span class="material-symbols-outlined text-6xl mb-4">videocam</span>
                        <p>No preview available</p>
                        <p class="text-sm mt-2">Select a camera and click "Start Preview" or capture a snapshot</p>
                    </div>
                {/if}
                
                {#if isCapturing}
                    <div class="absolute inset-0 bg-white/20 flex items-center justify-center">
                        <span class="material-symbols-outlined text-4xl text-white animate-pulse">photo_camera</span>
                    </div>
                {/if}
            </div>

            <!-- Capture Controls -->
            <div class="p-4 border-t border-slate-200 dark:border-[#283039] flex justify-center gap-4">
                <button 
                    onclick={() => captureSnapshot(false)}
                    disabled={isCapturing || availableCameras.length === 0}
                    class="flex items-center gap-2 px-6 py-3 bg-[#137fec] hover:bg-blue-600 text-white rounded-lg font-bold shadow-lg shadow-[#137fec]/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    <span class="material-symbols-outlined">photo_camera</span>
                    Capture
                </button>
                <button 
                    onclick={() => captureSnapshot(true)}
                    disabled={isCapturing || availableCameras.length === 0}
                    class="flex items-center gap-2 px-6 py-3 bg-white dark:bg-[#283039] border border-slate-200 dark:border-[#3b4754] text-slate-700 dark:text-white rounded-lg font-bold transition-all disabled:opacity-50 disabled:cursor-not-allowed hover:bg-slate-50 dark:hover:bg-[#3b4754]"
                >
                    <span class="material-symbols-outlined">save</span>
                    Capture & Save
                </button>
            </div>
        </div>

        <!-- Settings & History Sidebar -->
        <div class="flex flex-col gap-4">
            <!-- Camera Selection -->
            <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm">
                <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display mb-4 uppercase tracking-wider">Camera Selection</h3>
                
                {#if availableCameras.length > 0}
                    <div class="space-y-2">
                        {#each availableCameras as camera, index}
                            <button
                                onclick={() => { selectedCameraIndex = index; stopLivePreview(); }}
                                class="w-full flex items-center gap-3 p-3 rounded-lg border transition-all {selectedCameraIndex === index ? 'border-[#137fec] bg-[#137fec]/5' : 'border-slate-200 dark:border-[#283039] hover:border-slate-300 dark:hover:border-[#3b4754]'}"
                            >
                                <div class="size-10 rounded-lg bg-slate-100 dark:bg-[#283039] flex items-center justify-center">
                                    <span class="material-symbols-outlined text-slate-600 dark:text-slate-400">videocam</span>
                                </div>
                                <div class="flex-1 text-left">
                                    <p class="text-sm font-medium text-slate-900 dark:text-white">{camera}</p>
                                    <p class="text-xs text-slate-500">Index: {index}</p>
                                </div>
                                {#if selectedCameraIndex === index}
                                    <span class="material-symbols-outlined text-[#137fec]">check_circle</span>
                                {/if}
                            </button>
                        {/each}
                    </div>
                {:else}
                    <div class="text-center py-6 text-slate-500">
                        <span class="material-symbols-outlined text-3xl mb-2">videocam_off</span>
                        <p class="text-sm">No cameras detected</p>
                    </div>
                {/if}
            </div>

            <!-- Capture History -->
            <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm flex-1">
                <div class="flex justify-between items-center mb-4">
                    <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display uppercase tracking-wider">Recent Captures</h3>
                    {#if captureHistory.length > 0}
                        <button onclick={clearHistory} class="text-xs text-slate-500 hover:text-red-500 transition-colors">
                            Clear
                        </button>
                    {/if}
                </div>
                
                {#if captureHistory.length > 0}
                    <div class="grid grid-cols-3 gap-2">
                        {#each captureHistory as item, i}
                            <button
                                onclick={() => selectHistoryItem(item)}
                                class="aspect-square rounded-lg overflow-hidden border-2 border-transparent hover:border-[#137fec] transition-colors relative group"
                            >
                                <img 
                                    src="data:image/jpeg;base64,{item.data}" 
                                    alt="Capture {i + 1}" 
                                    class="w-full h-full object-cover"
                                />
                                <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                    <span class="material-symbols-outlined text-white text-sm">visibility</span>
                                </div>
                                {#if item.path}
                                    <div class="absolute bottom-0 right-0 p-1">
                                        <span class="material-symbols-outlined text-white text-xs drop-shadow">save</span>
                                    </div>
                                {/if}
                            </button>
                        {/each}
                    </div>
                {:else}
                    <div class="text-center py-6 text-slate-500">
                        <span class="material-symbols-outlined text-3xl mb-2">photo_library</span>
                        <p class="text-sm">No captures yet</p>
                    </div>
                {/if}
            </div>

            <!-- Last Capture Info -->
            {#if lastCapture && lastCapture.success}
                <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm">
                    <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display mb-3 uppercase tracking-wider">Last Capture</h3>
                    <div class="space-y-2 text-sm">
                        <div class="flex justify-between">
                            <span class="text-slate-500">Camera</span>
                            <span class="font-mono">{cameraName}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-slate-500">Status</span>
                            <span class="text-green-500 font-medium">Success</span>
                        </div>
                        {#if lastCapture.outputPath}
                            <div class="flex justify-between">
                                <span class="text-slate-500">Saved to</span>
                                <span class="font-mono text-xs truncate max-w-[150px]" title={lastCapture.outputPath}>
                                    {lastCapture.outputPath.split('/').pop()}
                                </span>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>
