<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  let videoSrc = "";
  let videoResult = "";

  async function loadVideo() {
    try {
      const filePath = await open({
        filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv"] }],
      });

      if (filePath) {
        // The backend returns a stringified JSON, so we keep parsing it here.
        // If we updated the backend to return an object, we would remove the JSON.parse in formatVideoInfo.
        const result = await invoke("get_video_info", { filename: filePath });
        videoResult = result as string;
        videoSrc = convertFileSrc(filePath as string);
      }
    } catch (error) {
      console.error("Error loading video:", error);
    }
  }

  function formatVideoInfo(jsonStr: string): Record<string, string> {
    try {
      const data = JSON.parse(jsonStr);
      return {
        Codec: String(data.codec_name ?? "N/A"),
        Format: String(data.codec_str ?? "N/A").toUpperCase(),
        Duration: Number.isFinite(data.duration_seconds)
          ? `${Math.floor(data.duration_seconds / 60)}m ${Math.round(data.duration_seconds % 60)}s`
          : "N/A",
        "Frame Rate": Number.isFinite(data.fps) ? `${data.fps} FPS` : "N/A",
        "Total Frames": Number.isFinite(data.frame_count)
          ? data.frame_count.toLocaleString()
          : "N/A",
        Resolution: String(data.resolution ?? "N/A"),
      };
    } catch (e) {
      return {};
    }
  }
</script>

<svelte:head>
  <title>Video Viewer</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 max-w-5xl">
    <div class="text-center mb-8">
      <h1 class="text-3xl font-bold mb-2">Local Video Player</h1>
      <p class="text-base-content/70">Analyze and play local video files</p>
    </div>

    <!-- Main Player Card -->
    <div class="card bg-base-100 shadow-xl overflow-hidden mb-8">
      <div class="card-body p-0">
        <!-- Toolbar -->
        <div class="p-4 bg-base-200 border-b border-base-300 flex justify-between items-center">
             <h2 class="font-bold text-lg">Player</h2>
             <button on:click={loadVideo} class="btn btn-primary btn-sm">
                Open File
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
                </div>
             {/if}
        </div>
      </div>
    </div>

    <!-- Metadata Grid -->
    {#if videoResult}
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
            <h2 class="card-title text-sm opacity-70 mb-4">Media Information</h2>
            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4">
                {#each Object.entries(formatVideoInfo(videoResult)) as [key, value]}
                    <div class="flex flex-col p-3 bg-base-200 rounded-box text-center">
                        <span class="text-xs uppercase tracking-wider opacity-60 mb-1">{key}</span>
                        <span class="font-bold text-sm truncate" title={value}>{value}</span>
                    </div>
                {/each}
            </div>
        </div>
      </div>
    {/if}
</div>
