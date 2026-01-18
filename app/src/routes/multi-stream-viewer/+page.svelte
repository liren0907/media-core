<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import Hls from "hls.js";

  let urlsInput = `rtsp://localhost:8554/mystream\nrtsp://localhost:8554/mystream2`;
  let streams: { url: string; hls: Hls | null; videoElement: HTMLVideoElement }[] = [];
  let status = "Enter RTSP URLs and click 'Start Streams' to begin.";
  let isLoading = false;

  async function startAllStreams() {
    const urls = urlsInput.split("\n").map(u => u.trim()).filter(u => u);
    if (urls.length === 0) {
      status = "Please enter at least one RTSP URL.";
      return;
    }

    isLoading = true;
    status = `Starting ${urls.length} stream(s)...`;
    
    // Reset existing streams
    streams.forEach(s => s.hls?.destroy());
    streams = urls.map(url => ({ url, hls: null, videoElement: null as any }));

    try {
        const playlistPaths = await invoke("start_multiple_hls_playback", { urls });
        
        // Give a small delay for backend to init files
        await new Promise(r => setTimeout(r, 1000));

        (playlistPaths as string[]).forEach((path, i) => {
            const stream = streams[i];
            const videoSrc = convertFileSrc(path);

            if (Hls.isSupported()) {
                const hls = new Hls();
                hls.loadSource(videoSrc);
                hls.attachMedia(stream.videoElement);
                stream.hls = hls;
            } else if (stream.videoElement.canPlayType("application/vnd.apple.mpegurl")) {
                stream.videoElement.src = videoSrc;
            }
        });

        status = "All streams active.";
    } catch (e) {
        status = `Error starting streams: ${e}`;
        console.error("Failed to start multiple streams:", e);
    } finally {
        isLoading = false;
    }
  }

  onDestroy(() => {
    streams.forEach(s => s.hls?.destroy());
  });
</script>

<svelte:head>
	<title>Multi-Stream Viewer</title>
</svelte:head>

<div class="container mx-auto p-4 max-w-7xl">
    <div class="text-center mb-8">
      <h1 class="text-3xl font-bold mb-2">Multi-Stream Viewer</h1>
      <p class="text-base-content/70">Monitor multiple live RTSP streams simultaneously</p>
    </div>

    <!-- Configuration Card -->
    <div class="card bg-base-100 shadow-xl mb-8">
      <div class="card-body">
        <div class="form-control">
            <label class="label">
                <span class="label-text font-bold">RTSP Stream URLs (one per line)</span>
            </label>
            <textarea
                bind:value={urlsInput}
                class="textarea textarea-bordered h-32 font-mono text-sm"
                placeholder="rtsp://192.168.1.100/stream1&#10;rtsp://192.168.1.101/stream2"
            ></textarea>
        </div>
        
        <div class="card-actions justify-end mt-4">
            <button class="btn btn-primary" on:click={startAllStreams} disabled={isLoading}>
                {#if isLoading}
                    <span class="loading loading-spinner"></span>
                {/if}
                Start Streams
            </button>
        </div>
        
        {#if status}
             <div class="alert {status.includes('Error') ? 'alert-error' : 'alert-info'} mt-4 text-sm">
                <span>{status}</span>
             </div>
        {/if}
      </div>
    </div>

    <!-- Video Grid -->
    {#if streams.length > 0}
      <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
        {#each streams as stream, i (stream.url)}
          <div class="card bg-base-100 shadow-lg overflow-hidden group">
            <figure class="aspect-video bg-black relative">
              <video 
                bind:this={stream.videoElement} 
                controls 
                muted
                class="w-full h-full object-contain"
              ></video>
              <div class="absolute top-2 left-2 badge badge-sm badge-neutral opacity-70 group-hover:opacity-100 transition-opacity">
                 CAM {i + 1}
              </div>
            </figure>
            <div class="card-body p-3">
              <div class="text-xs text-base-content/60 truncate font-mono" title={stream.url}>
                {stream.url}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
</div>
