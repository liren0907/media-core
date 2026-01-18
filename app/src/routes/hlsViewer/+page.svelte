<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let videoElement: HTMLVideoElement;
  let hlsStatus = {
    status: "inactive",
    playlist_exists: false,
    segment_count: 0,
    playlist_path: "",
  };
  let playlistUrl = "http://127.0.0.1:1521/stream_0/playlist.m3u8";
  let isLoading = false;
  let error = "";
  let refreshInterval: any;

  let Hls: any = null;
  let hls: any = null;

  onMount(async () => {
    await loadHlsJS();
    refreshInterval = setInterval(checkHlsStatus, 2000);
    await checkHlsStatus();
  });

  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
    if (hls) hls.destroy();
  });

  async function loadHlsJS() {
    try {
      const script = document.createElement('script');
      script.src = 'https://cdn.jsdelivr.net/npm/hls.js@latest';
      script.onload = () => { Hls = (window as any).Hls; };
      document.head.appendChild(script);
    } catch (err) {
      console.error("Failed to load HLS.js:", err);
      error = "Failed to load HLS.js library";
    }
  }

  async function checkHlsStatus() {
    try {
      const result = await invoke("get_hls_status", { outputDir: "../hls_media_dev/stream_0" });
      hlsStatus = JSON.parse(result as string);
    } catch (err) {
      console.error("Failed to check HLS status:", err);
    }
  }

  async function loadPlaylist() {
    if (!videoElement || !Hls) {
      error = "Video player or HLS.js not ready";
      return;
    }

    isLoading = true;
    error = "";

    try {
      if (Hls.isSupported()) {
        if (hls) hls.destroy();
        
        hls = new Hls({ debug: true, enableWorker: true, lowLatencyMode: true });
        hls.loadSource(playlistUrl);
        hls.attachMedia(videoElement);

        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          videoElement.play().catch(console.error);
        });

        hls.on(Hls.Events.ERROR, (_: any, data: any) => {
          if (data.fatal) {
             error = `Fatal HLS Error: ${data.type}`;
             hls.destroy();
          }
        });
      } else if (videoElement.canPlayType('application/vnd.apple.mpegurl')) {
        videoElement.src = playlistUrl;
        videoElement.play();
      } else {
        error = "HLS is not supported in this browser";
      }
    } catch (err) {
      error = `Failed to load playlist: ${err}`;
    } finally {
      isLoading = false;
    }
  }

  async function validatePlaylist() {
    try {
      const result = await invoke("validate_config", { configPath: "rtsp_hls_config.json" });
      console.log("Validation result:", result);
    } catch (err) {
      error = `Configuration validation failed: ${err}`;
    }
  }
</script>

<svelte:head>
  <title>HLS Viewer</title>
</svelte:head>

<div class="container mx-auto p-4 max-w-6xl">
  <div class="text-center mb-8">
    <h1 class="text-3xl font-bold mb-2">HLS Stream Viewer</h1>
    <p class="text-base-content/70">Watch live RTSP streams converted to HLS format</p>
  </div>

  <!-- Status Stats -->
  <div class="stats shadow w-full mb-8 bg-base-100">
    
    <div class="stat place-items-center">
      <div class="stat-title">Stream Status</div>
      <div class="stat-value {hlsStatus.status === 'active' ? 'text-success' : 'text-error'}">
        {hlsStatus.status.toUpperCase()}
      </div>
      <div class="stat-desc">Backend Process</div>
    </div>
    
    <div class="stat place-items-center">
      <div class="stat-title">Playlist File</div>
      <div class="stat-value {hlsStatus.playlist_exists ? 'text-success' : 'text-warning'}">
        {hlsStatus.playlist_exists ? 'Ready' : 'Waiting'}
      </div>
      <div class="stat-desc">m3u8 availability</div>
    </div>
    
    <div class="stat place-items-center">
      <div class="stat-title">Segments</div>
      <div class="stat-value text-secondary">{hlsStatus.segment_count}</div>
      <div class="stat-desc">Generated chunks</div>
    </div>
    
    <div class="stat place-items-center">
      <div class="stat-title">Actions</div>
      <div class="stat-actions">
        <button class="btn btn-sm btn-ghost" on:click={checkHlsStatus}>Refresh</button>
      </div>
    </div>
    
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Player Card -->
    <div class="col-span-1 lg:col-span-2 card bg-base-100 shadow-xl">
      <div class="card-body p-0 overflow-hidden relative bg-black rounded-box">
        <video
          bind:this={videoElement}
          class="w-full aspect-video"
          controls
          muted
          playsinline
        >
          <track kind="captions">
        </video>
        
        {#if isLoading}
          <div class="absolute inset-0 flex items-center justify-center bg-black/50 text-white">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Controls Card -->
    <div class="card bg-base-100 shadow-xl h-fit">
      <div class="card-body">
        <h2 class="card-title text-sm opacity-70">Playback Controls</h2>
        
        <div class="form-control w-full">
          <label class="label">
            <span class="label-text">Playlist URL</span>
          </label>
          <input type="text" bind:value={playlistUrl} class="input input-bordered w-full" />
        </div>

        <div class="divider my-2"></div>

        <div class="flex flex-col gap-2">
            <button
                class="btn btn-primary w-full"
                on:click={loadPlaylist}
                disabled={isLoading || !hlsStatus.playlist_exists}
            >
                {#if isLoading}
                    <span class="loading loading-spinner"></span>
                {:else}
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                    Load Stream
                {/if}
            </button>
            
            <button class="btn btn-outline w-full" on:click={validatePlaylist}>
                Validate Config
            </button>
        </div>

        {#if error}
            <div class="alert alert-error mt-4 text-sm shadow-inner">
                <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                <span>{error}</span>
            </div>
        {/if}

        <div class="alert alert-info mt-4 text-xs">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            <div>
                <div class="font-bold">Instructions</div>
                <div>Start HLS capture in the Stream Manager first.</div>
            </div>
        </div>
      </div>
    </div>
  </div>
</div>
