<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appConfig, getPlaylistUrl, getHlsOutputDir } from "$lib/config";

  let videoElement: HTMLVideoElement;
  let hlsStatus = {
    status: "inactive",
    playlist_exists: false,
    segment_count: 0,
    playlist_path: "",
  };
  let playlistUrl = "";
  let hlsOutputDir = "";
  let isLoading = false;
  let error = "";
  let refreshInterval: ReturnType<typeof setInterval>;
  let statsInterval: ReturnType<typeof setInterval>;

  let Hls: any = null;
  let hls: any = null;

  // Stream quality metrics from HLS.js
  interface StreamQuality {
    resolution: string;
    bitrate: number;
    codec: string;
    fps: number;
    level: number;
    levelCount: number;
  }
  
  interface BufferHealth {
    length: number;
    position: number;
    end: number;
    status: "healthy" | "warning" | "critical";
  }
  
  interface LatencyMetrics {
    latency: number;
    drift: number;
    targetLatency: number;
    maxLatency: number;
  }

  let streamQuality: StreamQuality | null = null;
  let bufferHealth: BufferHealth | null = null;
  let latencyMetrics: LatencyMetrics | null = null;
  let droppedFrames = 0;
  let totalFrames = 0;
  let bandwidth = 0;
  let isPlaying = false;

  onMount(async () => {
    // Initialize from config
    playlistUrl = getPlaylistUrl('stream_0');
    hlsOutputDir = getHlsOutputDir();
    
    await loadHlsJS();
    refreshInterval = setInterval(checkHlsStatus, 2000);
    await checkHlsStatus();
  });
  
  // Update when config changes
  $: if ($appConfig) {
    playlistUrl = getPlaylistUrl('stream_0');
    hlsOutputDir = getHlsOutputDir();
  }

  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
    if (statsInterval) clearInterval(statsInterval);
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
      // Use configured output directory or fallback
      const outputDir = hlsOutputDir ? `${hlsOutputDir}/stream_0` : 'hls_output/stream_0';
      const result = await invoke("get_hls_status", { outputDir });
      hlsStatus = JSON.parse(result as string);
    } catch (err) {
      console.error("Failed to check HLS status:", err);
    }
  }

  function updateStreamStats() {
    if (!hls || !videoElement) return;

    // Buffer health
    const buffered = videoElement.buffered;
    if (buffered.length > 0) {
      const currentTime = videoElement.currentTime;
      const bufferEnd = buffered.end(buffered.length - 1);
      const bufferLength = bufferEnd - currentTime;
      
      let status: "healthy" | "warning" | "critical" = "healthy";
      if (bufferLength < 1) status = "critical";
      else if (bufferLength < 3) status = "warning";
      
      bufferHealth = {
        length: bufferLength,
        position: currentTime,
        end: bufferEnd,
        status
      };
    }

    // Current level quality
    if (hls.levels && hls.currentLevel >= 0) {
      const level = hls.levels[hls.currentLevel];
      if (level) {
        streamQuality = {
          resolution: `${level.width || 0}x${level.height || 0}`,
          bitrate: level.bitrate || 0,
          codec: level.codecSet || level.videoCodec || "unknown",
          fps: level.frameRate || 0,
          level: hls.currentLevel,
          levelCount: hls.levels.length
        };
      }
    }

    // Latency for live streams
    if (hls.latency !== undefined) {
      latencyMetrics = {
        latency: hls.latency,
        drift: hls.drift || 0,
        targetLatency: hls.targetLatency || 0,
        maxLatency: hls.maxLatency || 0
      };
    }

    // Bandwidth
    if (hls.bandwidthEstimate) {
      bandwidth = hls.bandwidthEstimate;
    }

    // Frame stats from video element
    if ((videoElement as any).getVideoPlaybackQuality) {
      const quality = (videoElement as any).getVideoPlaybackQuality();
      droppedFrames = quality.droppedVideoFrames;
      totalFrames = quality.totalVideoFrames;
    }

    isPlaying = !videoElement.paused && !videoElement.ended;
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
        if (statsInterval) clearInterval(statsInterval);
        
        hls = new Hls({ 
          debug: false, 
          enableWorker: true, 
          lowLatencyMode: true,
          backBufferLength: 90,
          maxBufferLength: 30,
          maxMaxBufferLength: 60
        });
        hls.loadSource(playlistUrl);
        hls.attachMedia(videoElement);

        hls.on(Hls.Events.MANIFEST_PARSED, (_: any, data: any) => {
          console.log("Manifest parsed, levels:", data.levels);
          videoElement.play().catch(console.error);
          // Start stats collection
          statsInterval = setInterval(updateStreamStats, 500);
        });

        hls.on(Hls.Events.LEVEL_SWITCHED, (_: any, data: any) => {
          console.log("Level switched to:", data.level);
          updateStreamStats();
        });

        hls.on(Hls.Events.FRAG_LOADED, () => {
          updateStreamStats();
        });

        hls.on(Hls.Events.ERROR, (_: any, data: any) => {
          if (data.fatal) {
             error = `Fatal HLS Error: ${data.type} - ${data.details}`;
             hls.destroy();
             if (statsInterval) clearInterval(statsInterval);
          }
        });
      } else if (videoElement.canPlayType('application/vnd.apple.mpegurl')) {
        videoElement.src = playlistUrl;
        videoElement.play();
        statsInterval = setInterval(updateStreamStats, 500);
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

  function formatBitrate(bps: number): string {
    if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} Mbps`;
    if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} Kbps`;
    return `${bps} bps`;
  }

  function formatLatency(ms: number): string {
    return ms < 1000 ? `${ms.toFixed(0)}ms` : `${(ms / 1000).toFixed(1)}s`;
  }

  function getBufferColor(status: string): string {
    switch (status) {
      case "healthy": return "text-success";
      case "warning": return "text-warning";
      case "critical": return "text-error";
      default: return "text-base-content";
    }
  }

  // Quality level switching
  function setQualityLevel(level: number) {
    if (hls) {
      hls.currentLevel = level;
    }
  }

  $: dropRate = totalFrames > 0 ? ((droppedFrames / totalFrames) * 100).toFixed(2) : "0.00";
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
  <div class="stats shadow w-full mb-6 bg-base-100">
    
    <div class="stat place-items-center">
      <div class="stat-title">Stream Status</div>
      <div class="stat-value text-lg {hlsStatus.status === 'active' ? 'text-success' : 'text-error'}">
        {hlsStatus.status.toUpperCase()}
      </div>
      <div class="stat-desc">Backend Process</div>
    </div>
    
    <div class="stat place-items-center">
      <div class="stat-title">Playlist</div>
      <div class="stat-value text-lg {hlsStatus.playlist_exists ? 'text-success' : 'text-warning'}">
        {hlsStatus.playlist_exists ? 'Ready' : 'Waiting'}
      </div>
      <div class="stat-desc">m3u8 file</div>
    </div>
    
    <div class="stat place-items-center">
      <div class="stat-title">Segments</div>
      <div class="stat-value text-lg text-secondary">{hlsStatus.segment_count}</div>
      <div class="stat-desc">TS chunks</div>
    </div>

    <div class="stat place-items-center">
      <div class="stat-title">Playback</div>
      <div class="stat-value text-lg {isPlaying ? 'text-success' : 'text-base-content/50'}">
        {isPlaying ? 'Playing' : 'Stopped'}
      </div>
      <div class="stat-desc">{isPlaying ? 'Live' : 'Idle'}</div>
    </div>
    
  </div>

  <!-- Quality & Latency Stats (visible when playing) -->
  {#if streamQuality || bufferHealth || latencyMetrics}
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
    <!-- Resolution -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Resolution</div>
      <div class="text-lg font-bold">{streamQuality?.resolution || '—'}</div>
      {#if streamQuality?.fps}
        <div class="text-xs text-base-content/50">{streamQuality.fps.toFixed(0)} fps</div>
      {/if}
    </div>
    
    <!-- Bitrate -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Bitrate</div>
      <div class="text-lg font-bold">{streamQuality ? formatBitrate(streamQuality.bitrate) : '—'}</div>
      <div class="text-xs text-base-content/50">Est: {formatBitrate(bandwidth)}</div>
    </div>
    
    <!-- Buffer -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Buffer</div>
      <div class="text-lg font-bold {bufferHealth ? getBufferColor(bufferHealth.status) : ''}">
        {bufferHealth ? bufferHealth.length.toFixed(1) + 's' : '—'}
      </div>
      <div class="text-xs text-base-content/50">
        {bufferHealth?.status === 'healthy' ? 'Healthy' : bufferHealth?.status === 'warning' ? 'Low' : bufferHealth?.status === 'critical' ? 'Critical' : '—'}
      </div>
    </div>
    
    <!-- Latency -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Latency</div>
      <div class="text-lg font-bold">
        {latencyMetrics ? formatLatency(latencyMetrics.latency * 1000) : '—'}
      </div>
      <div class="text-xs text-base-content/50">
        Target: {latencyMetrics ? formatLatency(latencyMetrics.targetLatency * 1000) : '—'}
      </div>
    </div>
  </div>

  <!-- Additional metrics row -->
  <div class="grid grid-cols-3 gap-3 mb-6">
    <!-- Codec -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Codec</div>
      <div class="font-medium">{streamQuality?.codec || '—'}</div>
    </div>
    
    <!-- Quality Level -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Quality Level</div>
      <div class="flex items-center gap-2">
        <span class="font-medium">{streamQuality ? `${streamQuality.level + 1}/${streamQuality.levelCount}` : '—'}</span>
        {#if streamQuality && streamQuality.levelCount > 1}
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-xs btn-ghost">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </label>
            <ul tabindex="0" class="dropdown-content z-[1] menu p-1 shadow bg-base-200 rounded-box w-32">
              <li><button on:click={() => setQualityLevel(-1)} class="text-xs">Auto</button></li>
              {#each Array(streamQuality.levelCount) as _, i}
                <li><button on:click={() => setQualityLevel(i)} class="text-xs">Level {i + 1}</button></li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    </div>
    
    <!-- Dropped Frames -->
    <div class="bg-base-100 rounded-lg p-3 shadow">
      <div class="text-xs text-base-content/60 mb-1">Dropped Frames</div>
      <div class="font-medium {Number(dropRate) > 1 ? 'text-warning' : Number(dropRate) > 5 ? 'text-error' : ''}">
        {droppedFrames} <span class="text-xs text-base-content/50">({dropRate}%)</span>
      </div>
    </div>
  </div>
  {/if}

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
          <label for="playlistUrl" class="label">
            <span class="label-text">Playlist URL</span>
          </label>
          <input id="playlistUrl" type="text" bind:value={playlistUrl} class="input input-bordered w-full" />
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
