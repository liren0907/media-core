<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appConfig, getPlaylistUrl, getHlsOutputDir } from "$lib/config.svelte";
  import { PageContent, Panel, StatCard, StatusBadge, ProgressBar } from '$lib/components/ui';

  let videoElement: HTMLVideoElement;
  let hlsStatus = $state({ status: "inactive", playlist_exists: false, segment_count: 0, playlist_path: "" });
  let playlistUrl = $state("");
  let hlsOutputDir = $state("");
  let isLoading = $state(false);
  let error = $state("");

  let statsInterval: ReturnType<typeof setInterval>;
  let Hls: any = $state(null);
  let hls: any = $state(null);

  interface StreamQuality { resolution: string; bitrate: number; codec: string; fps: number; level: number; levelCount: number; }
  interface BufferHealth { length: number; position: number; end: number; status: "healthy" | "warning" | "critical"; }
  interface LatencyMetrics { latency: number; drift: number; targetLatency: number; maxLatency: number; }

  let streamQuality: StreamQuality | null = $state(null);
  let bufferHealth: BufferHealth | null = $state(null);
  let latencyMetrics: LatencyMetrics | null = $state(null);
  let droppedFrames = $state(0);
  let totalFrames = $state(0);
  let bandwidth = $state(0);
  let isPlaying = $state(false);

  $effect(() => {
    playlistUrl = getPlaylistUrl('stream_0');
    hlsOutputDir = getHlsOutputDir();
    loadHlsJS();
    const refreshInterval = setInterval(checkHlsStatus, 2000);
    checkHlsStatus();
    return () => { clearInterval(refreshInterval); if (statsInterval) clearInterval(statsInterval); if (hls) hls.destroy(); };
  });

  $effect(() => { const _ = appConfig.current; playlistUrl = getPlaylistUrl('stream_0'); hlsOutputDir = getHlsOutputDir(); });

  async function loadHlsJS() {
    try {
      const script = document.createElement('script');
      script.src = 'https://cdn.jsdelivr.net/npm/hls.js@latest';
      script.onload = () => { Hls = (window as any).Hls; };
      document.head.appendChild(script);
    } catch (err) { error = "Failed to load HLS.js"; }
  }

  async function checkHlsStatus() {
    try {
      const outputDir = hlsOutputDir ? `${hlsOutputDir}/stream_0` : 'hls_output/stream_0';
      const result = await invoke("get_hls_status", { outputDir });
      hlsStatus = JSON.parse(result as string);
    } catch (err) { console.error("Failed to check HLS status:", err); }
  }

  function updateStreamStats() {
    if (!hls || !videoElement) return;
    const buffered = videoElement.buffered;
    if (buffered.length > 0) {
      const ct = videoElement.currentTime, be = buffered.end(buffered.length - 1), bl = be - ct;
      let s: "healthy" | "warning" | "critical" = bl < 1 ? "critical" : bl < 3 ? "warning" : "healthy";
      bufferHealth = { length: bl, position: ct, end: be, status: s };
    }
    if (hls.levels && hls.currentLevel >= 0) {
      const l = hls.levels[hls.currentLevel];
      if (l) streamQuality = { resolution: `${l.width || 0}x${l.height || 0}`, bitrate: l.bitrate || 0, codec: l.codecSet || l.videoCodec || "unknown", fps: l.frameRate || 0, level: hls.currentLevel, levelCount: hls.levels.length };
    }
    if (hls.latency !== undefined) latencyMetrics = { latency: hls.latency, drift: hls.drift || 0, targetLatency: hls.targetLatency || 0, maxLatency: hls.maxLatency || 0 };
    if (hls.bandwidthEstimate) bandwidth = hls.bandwidthEstimate;
    if ((videoElement as any).getVideoPlaybackQuality) { const q = (videoElement as any).getVideoPlaybackQuality(); droppedFrames = q.droppedVideoFrames; totalFrames = q.totalVideoFrames; }
    isPlaying = !videoElement.paused && !videoElement.ended;
  }

  async function loadPlaylist() {
    if (!videoElement || !Hls) { error = "Player not ready"; return; }
    isLoading = true; error = "";
    try {
      if (Hls.isSupported()) {
        if (hls) hls.destroy();
        if (statsInterval) clearInterval(statsInterval);
        hls = new Hls({ debug: false, enableWorker: true, lowLatencyMode: true, backBufferLength: 90, maxBufferLength: 30, maxMaxBufferLength: 60 });
        hls.loadSource(playlistUrl);
        hls.attachMedia(videoElement);
        hls.on(Hls.Events.MANIFEST_PARSED, () => { videoElement.play().catch(console.error); statsInterval = setInterval(updateStreamStats, 500); });
        hls.on(Hls.Events.LEVEL_SWITCHED, () => updateStreamStats());
        hls.on(Hls.Events.FRAG_LOADED, () => updateStreamStats());
        hls.on(Hls.Events.ERROR, (_: any, data: any) => { if (data.fatal) { error = `Fatal: ${data.details}`; hls.destroy(); if (statsInterval) clearInterval(statsInterval); } });
      } else if (videoElement.canPlayType('application/vnd.apple.mpegurl')) {
        videoElement.src = playlistUrl; videoElement.play(); statsInterval = setInterval(updateStreamStats, 500);
      } else { error = "HLS not supported"; }
    } catch (err) { error = `Failed: ${err}`; }
    finally { isLoading = false; }
  }

  async function validatePlaylist() {
    try { await invoke("validate_config", { configPath: "rtsp_hls_config.json" }); }
    catch (err) { error = `Validation failed: ${err}`; }
  }

  function formatBitrate(bps: number): string { return bps >= 1e6 ? `${(bps / 1e6).toFixed(1)} Mbps` : bps >= 1e3 ? `${(bps / 1e3).toFixed(0)} Kbps` : `${bps} bps`; }
  function formatLatency(ms: number): string { return ms < 1000 ? `${ms.toFixed(0)}ms` : `${(ms / 1000).toFixed(1)}s`; }
  function setQualityLevel(level: number) { if (hls) hls.currentLevel = level; }

  let dropRate = $derived(totalFrames > 0 ? ((droppedFrames / totalFrames) * 100).toFixed(2) : "0.00");

  const bufferColors: Record<string, string> = {
    healthy: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
    warning: 'bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 border-yellow-500/20',
    critical: 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20',
  };
</script>

<svelte:head>
  <title>HLS Viewer</title>
</svelte:head>

<PageContent>
    <!-- Status Stats -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Stream" icon="cell_tower" iconColor="text-[#137fec]" value={hlsStatus.status.toUpperCase()}>
            {#snippet extra()}<StatusBadge status={hlsStatus.status === 'active' ? 'active' : 'idle'} />{/snippet}
        </StatCard>
        <StatCard label="Playlist" icon="playlist_play" iconColor="text-green-500" value={hlsStatus.playlist_exists ? 'Ready' : 'Waiting'} />
        <StatCard label="Segments" icon="segment" iconColor="text-purple-500" value="{String(hlsStatus.segment_count)}" sub="TS chunks" />
        <StatCard label="Playback" icon="play_circle" iconColor="text-orange-500" value={isPlaying ? 'Playing' : 'Stopped'}>
            {#snippet extra()}<StatusBadge status={isPlaying ? 'active' : 'idle'} />{/snippet}
        </StatCard>
    </div>

    <!-- Quality metrics (visible when playing) -->
    {#if streamQuality || bufferHealth || latencyMetrics}
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <StatCard label="Resolution" icon="aspect_ratio" iconColor="text-[#137fec]" value={streamQuality?.resolution || '—'} sub={streamQuality?.fps ? `${streamQuality.fps.toFixed(0)} fps` : ''} />
            <StatCard label="Bitrate" icon="speed" iconColor="text-orange-500" value={streamQuality ? formatBitrate(streamQuality.bitrate) : '—'} sub="Est: {formatBitrate(bandwidth)}" />
            <StatCard label="Buffer" icon="hourglass_empty" iconColor="text-green-500" value={bufferHealth ? bufferHealth.length.toFixed(1) + 's' : '—'}>
                {#snippet extra()}{#if bufferHealth}<StatusBadge status={bufferHealth.status} colorMap={bufferColors} />{/if}{/snippet}
            </StatCard>
            <StatCard label="Latency" icon="timer" iconColor="text-purple-500" value={latencyMetrics ? formatLatency(latencyMetrics.latency * 1000) : '—'} sub={latencyMetrics ? `Target: ${formatLatency(latencyMetrics.targetLatency * 1000)}` : ''} />
        </div>

        <div class="grid grid-cols-3 gap-3">
            <StatCard label="Codec" icon="code" iconColor="text-slate-400" value={streamQuality?.codec || '—'} />
            <StatCard label="Quality" icon="tune" iconColor="text-[#137fec]" value={streamQuality ? `${streamQuality.level + 1}/${streamQuality.levelCount}` : '—'}>
                {#snippet extra()}
                    {#if streamQuality && streamQuality.levelCount > 1}
                        <select onchange={(e) => setQualityLevel(Number((e.target as HTMLSelectElement).value))} class="mt-1 bg-transparent border border-slate-200 dark:border-[#2a3441] rounded text-[10px] px-1 py-0.5">
                            <option value="-1">Auto</option>
                            {#each Array(streamQuality.levelCount) as _, i}
                                <option value={i}>Level {i + 1}</option>
                            {/each}
                        </select>
                    {/if}
                {/snippet}
            </StatCard>
            <StatCard label="Dropped" icon="warning" iconColor={Number(dropRate) > 1 ? 'text-orange-500' : 'text-slate-400'} value="{String(droppedFrames)}" sub="{dropRate}%" />
        </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Player -->
        <div class="lg:col-span-2">
            <Panel title="Player" icon="play_circle">
                <div class="bg-black aspect-video relative">
                    <video bind:this={videoElement} class="w-full h-full object-contain" controls muted playsinline><track kind="captions" /></video>
                    {#if isLoading}
                        <div class="absolute inset-0 flex items-center justify-center bg-black/50">
                            <span class="material-symbols-outlined text-3xl text-white animate-spin">sync</span>
                        </div>
                    {/if}
                </div>
            </Panel>
        </div>

        <!-- Controls -->
        <Panel title="Controls" icon="tune">
            <div class="p-3 flex flex-col gap-3">
                <div class="flex flex-col gap-1">
                    <label for="playlistUrl" class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Playlist URL</label>
                    <input id="playlistUrl" type="text" bind:value={playlistUrl} class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]" />
                </div>

                <button onclick={loadPlaylist} disabled={isLoading || !hlsStatus.playlist_exists} class="w-full py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold disabled:opacity-50 flex items-center justify-center gap-1.5 transition-colors">
                    {#if isLoading}<span class="material-symbols-outlined animate-spin text-[16px]">sync</span>{:else}<span class="material-symbols-outlined text-[16px]">play_arrow</span>{/if}
                    Load Stream
                </button>

                <button onclick={validatePlaylist} class="w-full py-2 bg-white dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-white rounded text-xs font-bold transition-colors hover:bg-slate-50 dark:hover:bg-[#283039]">
                    Validate Config
                </button>

                {#if error}
                    <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-xs flex items-center gap-1.5">
                        <span class="material-symbols-outlined text-[14px]">error</span> {error}
                    </div>
                {/if}

                <div class="p-2 rounded bg-blue-50 dark:bg-blue-900/10 border border-blue-200 dark:border-blue-900/30 text-blue-600 dark:text-blue-400 text-[10px]">
                    <span class="font-bold">Note:</span> Start HLS capture in Stream page first.
                </div>
            </div>
        </Panel>
    </div>
</PageContent>
