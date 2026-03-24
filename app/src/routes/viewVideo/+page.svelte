<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  let videoSrc = $state("");
  let currentFilePath = $state("");
  let isLoading = $state(false);
  let error = $state("");

  // Metadata from pipeline
  interface MediaMetadata {
    filename: string;
    filePath: string;
    fileSize: number;
    mimeType: string | null;
    duration: number | null;
    width: number | null;
    height: number | null;
    fps: number | null;
    frameCount: number | null;
    bitrate: number | null;
    codec: string | null;
    audioCodec: string | null;
    audioSampleRate: number | null;
    audioChannels: number | null;
    createdAt: string | null;
    modifiedAt: string | null;
    thumbnail: string | null;
  }

  let metadata = $state<MediaMetadata | null>(null);

  async function loadVideo() {
    try {
      error = "";
      const filePath = await open({
        filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv", "mov", "webm"] }],
      });

      if (filePath) {
        currentFilePath = filePath as string;
        videoSrc = convertFileSrc(currentFilePath);
        
        // Fetch metadata using the pipeline
        isLoading = true;
        try {
          metadata = await invoke("execute_metadata_pipeline", {
            filePath: currentFilePath,
            includeThumbnail: true,
          });
        } catch (e) {
          console.error("Failed to fetch metadata:", e);
          error = `Failed to load metadata: ${e}`;
          metadata = null;
        }
        isLoading = false;
      }
    } catch (e) {
      console.error("Error loading video:", e);
      error = `Error loading video: ${e}`;
    }
  }

  function formatDuration(seconds: number | null): string {
    if (seconds === null || !Number.isFinite(seconds)) return "N/A";
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.round(seconds % 60);
    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    }
    return `${minutes}m ${secs}s`;
  }

  function formatFileSize(bytes: number | null): string {
    if (bytes === null || !Number.isFinite(bytes)) return "N/A";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function formatBitrate(bps: number | null): string {
    if (bps === null || !Number.isFinite(bps)) return "N/A";
    if (bps < 1000) return `${bps} bps`;
    if (bps < 1000000) return `${(bps / 1000).toFixed(0)} Kbps`;
    return `${(bps / 1000000).toFixed(1)} Mbps`;
  }

  let primaryInfo = $derived(metadata ? [
    { label: "Duration", value: formatDuration(metadata.duration) },
    { label: "Resolution", value: metadata.width && metadata.height ? `${metadata.width}x${metadata.height}` : "N/A" },
    { label: "Frame Rate", value: metadata.fps ? `${metadata.fps.toFixed(2)} FPS` : "N/A" },
    { label: "Total Frames", value: metadata.frameCount?.toLocaleString() ?? "N/A" },
  ] : []);

  let technicalInfo = $derived(metadata ? [
    { label: "Video Codec", value: metadata.codec ?? "N/A" },
    { label: "Audio Codec", value: metadata.audioCodec ?? "N/A" },
    { label: "Bitrate", value: formatBitrate(metadata.bitrate) },
    { label: "File Size", value: formatFileSize(metadata.fileSize) },
    { label: "MIME Type", value: metadata.mimeType ?? "N/A" },
    { label: "Audio Sample Rate", value: metadata.audioSampleRate ? `${metadata.audioSampleRate} Hz` : "N/A" },
  ] : []);
</script>

<svelte:head>
  <title>Video Viewer</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 max-w-6xl">
    <div class="text-center mb-8">
      <h1 class="text-3xl font-bold mb-2">Local Video Player</h1>
      <p class="text-base-content/70">Analyze and play local video files with full metadata extraction</p>
    </div>

    <!-- Main Player Card -->
    <div class="card bg-base-100 shadow-xl overflow-hidden mb-8">
      <div class="card-body p-0">
        <!-- Toolbar -->
        <div class="p-4 bg-base-200 border-b border-base-300 flex justify-between items-center">
             <div class="flex items-center gap-3">
                <h2 class="font-bold text-lg">Player</h2>
                {#if metadata}
                    <span class="badge badge-ghost text-xs">{metadata.filename}</span>
                {/if}
             </div>
             <button onclick={loadVideo} class="btn btn-primary btn-sm" disabled={isLoading}>
                {#if isLoading}
                    <span class="loading loading-spinner loading-xs"></span>
                    Loading...
                {:else}
                    Open File
                {/if}
             </button>
        </div>

        <!-- Video Area -->
        <div class="bg-black aspect-video flex items-center justify-center relative">
             {#if videoSrc}
                <video src={videoSrc} controls class="w-full h-full object-contain">
                    <track kind="captions" />
                </video>
             {:else}
                <div class="text-center text-neutral-content/50">
                    <div class="text-6xl mb-4">▶</div>
                    <p>No video loaded</p>
                    <p class="text-sm mt-2">Click "Open File" to select a video</p>
                </div>
             {/if}
        </div>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="alert alert-error mb-8">
        <span>{error}</span>
      </div>
    {/if}

    <!-- Metadata Display -->
    {#if metadata}
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Primary Info Card -->
        <div class="lg:col-span-2 card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title text-sm opacity-70 mb-4">Video Information</h2>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-4">
                {#each primaryInfo as item}
                    <div class="flex flex-col p-4 bg-base-200 rounded-box text-center">
                        <span class="text-xs uppercase tracking-wider opacity-60 mb-2">{item.label}</span>
                        <span class="font-bold text-lg truncate" title={item.value}>{item.value}</span>
                    </div>
                {/each}
            </div>
            
            <div class="divider my-4"></div>
            
            <h3 class="text-sm opacity-70 mb-4">Technical Details</h3>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {#each technicalInfo as item}
                    <div class="flex justify-between items-center p-3 bg-base-200 rounded-lg">
                        <span class="text-xs uppercase tracking-wider opacity-60">{item.label}</span>
                        <span class="font-mono text-sm">{item.value}</span>
                    </div>
                {/each}
            </div>
          </div>
        </div>

        <!-- Thumbnail & File Info Card -->
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title text-sm opacity-70 mb-4">Preview</h2>
            
            {#if metadata.thumbnail}
              <div class="aspect-video bg-base-200 rounded-lg overflow-hidden mb-4">
                <img src="data:image/jpeg;base64,{metadata.thumbnail}" alt="Video thumbnail" class="w-full h-full object-cover" />
              </div>
            {:else}
              <div class="aspect-video bg-base-200 rounded-lg flex items-center justify-center mb-4">
                <span class="opacity-50">No thumbnail</span>
              </div>
            {/if}

            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="opacity-60">Filename</span>
                <span class="font-mono truncate max-w-[180px]" title={metadata.filename}>{metadata.filename}</span>
              </div>
              {#if metadata.createdAt}
                <div class="flex justify-between">
                  <span class="opacity-60">Created</span>
                  <span class="font-mono">{new Date(metadata.createdAt).toLocaleDateString()}</span>
                </div>
              {/if}
              {#if metadata.modifiedAt}
                <div class="flex justify-between">
                  <span class="opacity-60">Modified</span>
                  <span class="font-mono">{new Date(metadata.modifiedAt).toLocaleDateString()}</span>
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}
</div>
