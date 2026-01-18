<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  // State
  let videoSrc = "";
  let videoPath = "";
  let annotationPath = "";
  let outputPath = "";
  let labels: string[] = [];
  let selectedLabels: string[] = [];
  let totalFrames = 0;
  let totalObjects = 0;
  let isProcessing = false;
  let errorMessage = "";
  let annotationResponse = null;
  let videoResult = "";

  function formatVideoInfo(jsonStr: string) {
    try {
      console.log("Raw video info:", jsonStr);
      const data = typeof jsonStr === 'string' ? JSON.parse(jsonStr) : jsonStr;
      
      if (!data) return {};

      const getValue = (obj: any, path: string, defaultValue: any = "N/A") => {
        return path.split('.').reduce((acc, part) => acc && acc[part], obj) ?? defaultValue;
      };

      return {
        Codec: getValue(data, 'codec_name'),
        Format: getValue(data, 'codec_str')?.toUpperCase(),
        Duration: data.duration_seconds 
          ? `${Math.floor(data.duration_seconds / 60)}m ${Math.round(data.duration_seconds % 60)}s`
          : "N/A",
        "Frame Rate": data.fps ? `${data.fps} FPS` : "N/A",
        "Total Frames": data.frame_count ? data.frame_count.toLocaleString() : "N/A",
        Resolution: getValue(data, 'resolution')
      };
    } catch (e) {
      console.error("Error formatting video info:", e);
      return {};
    }
  }

  async function openVideoFile() {
    try {
      const filePath = await open({
        filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv"] }]
      });

      if (filePath) {
        videoPath = filePath as string;
        videoSrc = convertFileSrc(videoPath);
        const videoInfo = await invoke("get_video_info", { filename: videoPath });
        videoResult = videoInfo;
        errorMessage = "";
      }
    } catch (error) {
      errorMessage = `Error loading video: ${error}`;
      videoSrc = "";
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
        
        labels = data.unique_labels;
        selectedLabels = [...labels];
        totalFrames = data.total_frames;
        totalObjects = data.total_objects;
        errorMessage = "";
      }
    } catch (error) {
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
    selectedLabels = [...labels];
  }

  function deselectAllLabels() {
    selectedLabels = [];
  }

  async function startAnnotation() {
    if (!videoPath || !annotationPath) {
      errorMessage = "Please select both video and annotation files first";
      return;
    }

    if (selectedLabels.length === 0) {
      errorMessage = "Please select at least one label";
      return;
    }

    try {
      const selectedDir = await open({
        directory: true,
        multiple: false,
      });

      if (!selectedDir) {
        errorMessage = "No output directory selected";
        return;
      }

      outputPath = selectedDir as string;
      isProcessing = true;
      errorMessage = "";
      annotationResponse = null;

      const payload = {
        video_path: videoPath,
        annotation_path: annotationPath,
        output_directory: outputPath,
        label_selected: selectedLabels
      };

      const result = await invoke("start_video_annotation", { 
        payload: JSON.stringify(payload)
      });
      
      if (result) {
        annotationResponse = result;
      } else {
        throw new Error("Failed to process video");
      }
    } catch (error) {
      errorMessage = `Error during annotation: ${error}`;
    } finally {
      isProcessing = false;
    }
  }
</script>

<svelte:head>
	<title>Video Annotator</title>
	<meta name="description" content="Annotate videos with AI-generated data" />
</svelte:head>

<div class="container mx-auto p-4 max-w-5xl">
    <div class="text-center mb-8">
      <h1 class="text-3xl font-bold mb-2">Video Annotator</h1>
      <p class="text-base-content/70">Annotate videos with AI-generated data</p>
    </div>

    <!-- Video Player -->
    <div class="card bg-base-100 shadow-xl mb-6">
      <div class="card-body p-4">
        <h2 class="card-title text-sm opacity-70 mb-2">Original Video</h2>
        {#if videoSrc}
          <div class="rounded-xl overflow-hidden bg-base-300">
            <video src={videoSrc} controls class="w-full aspect-video">
              <track kind="captions">
            </video>
          </div>
        {:else}
          <div class="w-full aspect-video bg-base-300 rounded-xl flex items-center justify-center text-base-content/50">
            No video selected
          </div>
        {/if}
      </div>
    </div>

    <!-- Controls -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
      <!-- Video Selection -->
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
            <h2 class="card-title text-sm">Input Source</h2>
            <button class="btn btn-primary w-full" on:click={openVideoFile}>
                Open Video File
            </button>
            <div class="text-xs truncate opacity-70 mt-2">
                {videoPath || "No file selected"}
            </div>
        </div>
      </div>

      <!-- Annotation Selection -->
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
            <h2 class="card-title text-sm">Annotation Data</h2>
            <button class="btn btn-secondary w-full" on:click={openAnnotationFile}>
                Open JSON File
            </button>
             <div class="text-xs truncate opacity-70 mt-2">
                {annotationPath || "No file selected"}
            </div>
        </div>
      </div>
    </div>

    <!-- Video Info -->
    {#if videoResult}
      <div class="card bg-base-100 shadow-xl mb-6">
        <div class="card-body">
          <h2 class="card-title mb-4">Video Information</h2>
          <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
            {#each Object.entries(formatVideoInfo(videoResult)) as [key, value]}
              <div class="stats bg-base-200 shadow">
                <div class="stat p-2 place-items-center">
                    <div class="stat-title text-xs">{key}</div>
                    <div class="stat-value text-lg">{value}</div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Labels -->
    {#if labels.length > 0}
      <div class="card bg-base-100 shadow-xl mb-6">
        <div class="card-body">
          <div class="flex justify-between items-center mb-4">
            <h2 class="card-title">Select Labels</h2>
            <div class="join">
              <button class="btn btn-xs btn-outline join-item" on:click={selectAllLabels}>All</button>
              <button class="btn btn-xs btn-outline join-item" on:click={deselectAllLabels}>None</button>
            </div>
          </div>
          
          <div class="flex flex-wrap gap-2">
            {#each labels as label}
              <button
                class="btn btn-sm {selectedLabels.includes(label) ? 'btn-primary' : 'btn-outline'}"
                on:click={() => toggleLabel(label)}
              >
                {label}
              </button>
            {/each}
          </div>
          <div class="divider my-2"></div>
          <p class="text-center text-sm opacity-70">Total objects: {totalObjects}</p>
        </div>
      </div>
    {/if}

    <!-- Action -->
    <div class="flex justify-center mb-8">
      <button
        class="btn btn-lg btn-primary w-full max-w-md"
        on:click={startAnnotation}
        disabled={isProcessing}
      >
        {#if isProcessing}
          <span class="loading loading-spinner"></span>
          Processing...
        {:else}
          Start Annotation Process
        {/if}
      </button>
    </div>

    <!-- Alerts -->
    {#if errorMessage}
      <div class="alert alert-error shadow-lg mb-6">
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
        <span>{errorMessage}</span>
      </div>
    {/if}

    {#if annotationResponse}
      <div class="alert alert-success shadow-lg mb-6">
        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
        <div>
          <h3 class="font-bold">Success!</h3>
          <div class="text-xs">Output saved to: {annotationResponse.data.output_video}</div>
        </div>
      </div>
    {/if}
</div>
