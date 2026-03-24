<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import Hls from "hls.js";
  import { appConfig, getDefaultRtspUrl, getHlsOutputDir } from "$lib/config.svelte";

  // Stream stats from backend
  interface StreamStatus {
    id: string;
    name: string;
    status: string;
    streamType: string;
    codec: string | null;
    resolution: string | null;
    fps: number | null;
    bitrateKbps: number | null;
    durationSeconds: number | null;
    latencyMs: number | null;
  }

  interface StreamStats {
    activeCount: number;
    totalCount: number;
    avgLatencyMs: number;
    totalBitrateKbps: number;
    streams: StreamStatus[];
  }

  let streamStats: StreamStats | null = $state(null);

  // Initialize with config values
  let rtspConfig = $state({
    rtsp_url: "",
    rtsp_url_list: [""],
    output_directory: "", // User must select or default to empty
    show_preview: false,
    saving_option: "single",
    saved_time_duration: 60,
    use_fps: false,
    fps: 30.0,
    hls: {
      enabled: false,
      output_directory: "", // Will be set from config
      segment_duration: 6,
      playlist_size: 10,
    },
  });

  let isStreaming = $state(false);
  let status = $state("");
  let urlCount = $state(1);
  let hls: Hls | null = $state(null);
  let videoElement: HTMLVideoElement;

  async function fetchStreamStats() {
    try {
      streamStats = await invoke('get_stream_stats');
    } catch (e) {
      console.error('Failed to fetch stream stats:', e);
    }
  }

  function addUrlField() {
    rtspConfig.rtsp_url_list = [...rtspConfig.rtsp_url_list, ""];
    urlCount++;
  }

  function removeUrlField(index: number) {
    rtspConfig.rtsp_url_list = rtspConfig.rtsp_url_list.filter(
      (_, i) => i !== index,
    );
    urlCount--;
  }

  async function selectOutputDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected) {
      rtspConfig.output_directory = selected as string;
    }
  }

  async function startCapture() {
    try {
      isStreaming = true;
      status = "Starting RTSP capture...";

      rtspConfig.rtsp_url_list = rtspConfig.rtsp_url_list.filter(
        (url) => url.trim() !== "",
      );

      const result = await invoke("start_rtsp_capture", {
        payload: rtspConfig,
      });
      status = result as string;
      
      // Refresh stats after starting capture
      await fetchStreamStats();
    } catch (error) {
      status = `Error: ${error}`;
      isStreaming = false;
    }
  }

  async function playInApp() {
    status = "Starting HLS stream...";
    try {
      const playlistUrl = await invoke("start_hls_playback", {
        url: rtspConfig.rtsp_url,
      });

      console.log("Playlist URL from backend:", playlistUrl);

      if (Hls.isSupported()) {
        if (hls) {
          hls.destroy();
        }
        hls = new Hls();

        const finalUrl = playlistUrl as string;
        console.log("HLS.js is loading source from URL:", finalUrl);
        hls.loadSource(finalUrl);

        hls.attachMedia(videoElement);

        hls.on(Hls.Events.ERROR, function (event, data) {
          console.error("HLS.js Error:", data);
          if (data.fatal) {
            status = `Fatal HLS Error: ${data.details}`;
            hls?.destroy();
          } else {
            status = `HLS Error: ${data.details}`;
          }
        });

        videoElement.style.display = "block";
        status = "HLS stream is loading...";
        
        // Refresh stats
        await fetchStreamStats();
      } else if (videoElement.canPlayType("application/vnd.apple.mpegurl")) {
        videoElement.src = playlistUrl as string;
        videoElement.style.display = "block";
        status = "HLS stream is playing.";
      }
    } catch (e) {
      status = `Error starting HLS stream: ${e}`;
    }
  }

  async function playInNewWindow() {
    status = "Requesting direct playback in new window...";
    try {
      await invoke("start_direct_playback", { url: rtspConfig.rtsp_url });
      status = "Direct playback window should have opened.";
    } catch (e) {
      status = `Error requesting direct playback: ${e}`;
    }
  }

  function getHealthStatus(): { text: string; class: string } {
    if (!streamStats) return { text: 'Unknown', class: 'text-slate-500' };
    
    const activeRatio = streamStats.totalCount > 0 
      ? streamStats.activeCount / streamStats.totalCount 
      : 0;
    
    if (activeRatio >= 0.9) return { text: 'Healthy', class: 'text-[#0bda5b]' };
    if (activeRatio >= 0.5) return { text: 'Degraded', class: 'text-orange-500' };
    if (streamStats.activeCount > 0) return { text: 'Limited', class: 'text-yellow-500' };
    return { text: 'Idle', class: 'text-slate-500' };
  }

  function getLatencyBarWidth(): number {
    if (!streamStats || streamStats.avgLatencyMs <= 0) return 0;
    // Cap at 200ms for visual purposes
    return Math.min((streamStats.avgLatencyMs / 200) * 100, 100);
  }

  $effect(() => {
    rtspConfig.rtsp_url = getDefaultRtspUrl();
    rtspConfig.hls.output_directory = getHlsOutputDir() || 'hls_output';

    fetchStreamStats();
    const statsInterval = setInterval(fetchStreamStats, 3000);

    return () => {
      isStreaming = false;
      clearInterval(statsInterval);
      if (hls) hls.destroy();
    };
  });

  let healthStatus = $derived(getHealthStatus());
  let latencyBarWidth = $derived(getLatencyBarWidth());
</script>

<div class="p-8 max-w-[1600px] w-full mx-auto space-y-8">
    <!-- Page Heading & Actions -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div class="flex flex-col gap-2 max-w-2xl">
            <h2 class="text-slate-900 dark:text-white text-3xl font-bold font-display tracking-tight">Stream Ingestion</h2>
            <p class="text-slate-500 dark:text-[#9dabb9] text-base">Manage active RTSP streams, monitor latency, and configure real-time HLS playlist generation.</p>
        </div>
        <div class="flex gap-3">
            <button class="flex items-center gap-2 h-10 px-4 rounded-lg bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] text-slate-600 dark:text-[#9dabb9] hover:text-slate-900 dark:hover:text-white font-medium transition-all text-sm font-display">
                <span class="material-symbols-outlined" style="font-size: 20px;">tune</span>
                Sync Settings
            </button>
            <button class="flex items-center gap-2 h-10 px-4 rounded-lg bg-[#137fec] hover:bg-blue-600 text-white font-medium shadow-lg shadow-[#137fec]/20 transition-all text-sm font-display">
                <span class="material-symbols-outlined" style="font-size: 20px;">add</span>
                Add Stream
            </button>
        </div>
    </div>

    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="flex flex-col p-5 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm relative overflow-hidden group">
            <div class="absolute right-0 top-0 p-5 opacity-10 group-hover:opacity-20 transition-opacity">
                <span class="material-symbols-outlined text-slate-900 dark:text-white text-6xl">cast_connected</span>
            </div>
            <p class="text-slate-500 dark:text-[#9dabb9] text-sm font-medium font-display uppercase tracking-wider">Active Streams</p>
            <div class="flex items-baseline gap-2 mt-2">
                <p class="text-slate-900 dark:text-white text-3xl font-bold font-display">{streamStats?.activeCount ?? 0}</p>
                <span class="text-slate-400 dark:text-[#55606d] text-lg font-medium">/ {streamStats?.totalCount ?? 0}</span>
            </div>
            <div class="flex items-center gap-1.5 mt-2 text-sm font-medium {streamStats && streamStats.activeCount > 0 ? 'text-[#0bda5b]' : 'text-slate-400'}">
                {#if streamStats && streamStats.activeCount > 0}
                    <span class="material-symbols-outlined text-sm">trending_up</span>
                    <span>{streamStats.activeCount} stream{streamStats.activeCount !== 1 ? 's' : ''} active</span>
                {:else}
                    <span class="material-symbols-outlined text-sm">radio_button_unchecked</span>
                    <span>No active streams</span>
                {/if}
            </div>
        </div>
        <div class="flex flex-col p-5 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm relative overflow-hidden group">
            <div class="absolute right-0 top-0 p-5 opacity-10 group-hover:opacity-20 transition-opacity">
                <span class="material-symbols-outlined text-slate-900 dark:text-white text-6xl">speed</span>
            </div>
            <p class="text-slate-500 dark:text-[#9dabb9] text-sm font-medium font-display uppercase tracking-wider">Avg Latency</p>
            <div class="flex items-baseline gap-2 mt-2">
                <p class="text-slate-900 dark:text-white text-3xl font-bold font-display">
                    {streamStats?.avgLatencyMs?.toFixed(0) ?? '--'}<span class="text-lg text-slate-400 dark:text-[#9dabb9] ml-1">ms</span>
                </p>
            </div>
            <div class="w-full bg-slate-100 dark:bg-[#283039] h-1.5 rounded-full mt-3 overflow-hidden">
                <div 
                    class="h-full rounded-full transition-all duration-500 {streamStats && streamStats.avgLatencyMs > 100 ? 'bg-orange-500' : 'bg-[#137fec]'}" 
                    style="width: {latencyBarWidth}%"
                ></div>
            </div>
        </div>
        <div class="flex flex-col p-5 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm relative overflow-hidden group">
            <div class="absolute right-0 top-0 p-5 opacity-10 group-hover:opacity-20 transition-opacity">
                <span class="material-symbols-outlined text-slate-900 dark:text-white text-6xl">playlist_play</span>
            </div>
            <p class="text-slate-500 dark:text-[#9dabb9] text-sm font-medium font-display uppercase tracking-wider">HLS Generation</p>
            <div class="flex items-baseline gap-2 mt-2">
                <p class="{healthStatus.class} text-3xl font-bold font-display">{healthStatus.text}</p>
            </div>
            <p class="text-slate-500 dark:text-[#9dabb9] text-sm mt-2">
                {#if streamStats && streamStats.totalBitrateKbps > 0}
                    {(streamStats.totalBitrateKbps / 1000).toFixed(1)} Mbps total throughput
                {:else}
                    Ready for streaming
                {/if}
            </p>
        </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Configuration Form (Existing Logic) -->
        <div class="lg:col-span-2 flex flex-col rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] overflow-hidden shadow-sm">
            <div class="px-6 py-4 border-b border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d]">
                <h3 class="text-base font-bold text-slate-900 dark:text-white font-display">Configure New Stream</h3>
            </div>
            
            <div class="p-6">
                <!-- Video Player Preview -->
                <div class="mb-6 rounded-lg overflow-hidden bg-black flex justify-center items-center min-h-[200px] border border-slate-200 dark:border-[#283039]" class:hidden={videoElement?.style.display === 'none'}>
                    <video
                        bind:this={videoElement}
                        controls
                        class="w-full max-h-[400px]"
                        style="display: none;"
                    >
                        <track kind="captions" />
                    </video>
                </div>

                <form onsubmit={(e) => { e.preventDefault(); startCapture(); }} class="space-y-6">
                    <!-- Main URL Section -->
                    <div class="space-y-2">
                        <label for="mainRtspUrl" class="text-sm font-medium text-slate-700 dark:text-slate-300">Main RTSP URL</label>
                        <div class="flex gap-2">
                            <input
                                id="mainRtspUrl"
                                type="text"
                                bind:value={rtspConfig.rtsp_url}
                                class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-4 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]"
                                placeholder="rtsp://example.com/stream"
                            />
                            <button type="button" onclick={playInApp} class="px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded-lg font-medium text-sm transition-colors">Play</button>
                            <button type="button" onclick={playInNewWindow} class="px-4 py-2 bg-slate-100 dark:bg-[#283039] hover:bg-slate-200 dark:hover:bg-[#3b4754] text-slate-700 dark:text-white rounded-lg font-medium text-sm transition-colors">Pop-out</button>
                        </div>
                    </div>

                    <!-- Additional URLs -->
                    <div class="space-y-2">
                        <div class="text-sm font-medium text-slate-700 dark:text-slate-300">Additional URLs ({rtspConfig.rtsp_url_list.length})</div>
                        {#each rtspConfig.rtsp_url_list as url, i}
                            <div class="flex gap-2">
                                <input
                                    id={`additionalRtspUrl-${i}`}
                                    type="text"
                                    bind:value={rtspConfig.rtsp_url_list[i]}
                                    class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-4 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]"
                                    placeholder="rtsp://..."
                                    aria-label={`Additional RTSP URL ${i + 1}`}
                                />
                                <button type="button" onclick={() => removeUrlField(i)} class="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors">
                                    <span class="material-symbols-outlined">delete</span>
                                </button>
                            </div>
                        {/each}
                        <button type="button" onclick={addUrlField} class="w-full py-2 border-2 border-dashed border-slate-200 dark:border-[#283039] rounded-lg text-slate-500 hover:text-[#137fec] hover:border-[#137fec] transition-colors text-sm font-medium">
                            + Add Another Stream URL
                        </button>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="space-y-2">
                            <label for="outputDirectory" class="text-sm font-medium text-slate-700 dark:text-slate-300">Output Directory</label>
                            <div class="flex gap-2">
                                <input id="outputDirectory" type="text" readonly bind:value={rtspConfig.output_directory} class="flex-1 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-4 py-2 text-sm text-slate-900 dark:text-white" placeholder="Select output directory..." />
                                <button type="button" onclick={selectOutputDirectory} class="px-4 py-2 bg-slate-100 dark:bg-[#283039] hover:bg-slate-200 dark:hover:bg-[#3b4754] text-slate-700 dark:text-white rounded-lg font-medium text-sm transition-colors">Browse</button>
                            </div>
                        </div>
                        <div class="space-y-2">
                            <label for="segmentDuration" class="text-sm font-medium text-slate-700 dark:text-slate-300">Segment Duration (sec)</label>
                            <input id="segmentDuration" type="number" bind:value={rtspConfig.saved_time_duration} min="1" class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-4 py-2 text-sm text-slate-900 dark:text-white" />
                        </div>
                    </div>

                    <!-- Toggles -->
                    <div class="flex gap-6">
                        <label class="flex items-center gap-3 cursor-pointer">
                            <div class="relative inline-flex items-center">
                                <input type="checkbox" bind:checked={rtspConfig.show_preview} class="sr-only peer">
                                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#137fec]"></div>
                            </div>
                            <span class="text-sm font-medium text-slate-700 dark:text-slate-300">Show Preview</span>
                        </label>
                        <label class="flex items-center gap-3 cursor-pointer">
                            <div class="relative inline-flex items-center">
                                <input type="checkbox" bind:checked={rtspConfig.use_fps} class="sr-only peer">
                                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#137fec]"></div>
                            </div>
                            <span class="text-sm font-medium text-slate-700 dark:text-slate-300">Custom FPS</span>
                        </label>
                    </div>

                    <!-- HLS Config -->
                    <div class="p-4 rounded-lg border border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d]">
                        <label class="flex items-center gap-3 cursor-pointer mb-4">
                            <div class="relative inline-flex items-center">
                                <input type="checkbox" bind:checked={rtspConfig.hls.enabled} class="sr-only peer">
                                <div class="w-11 h-6 bg-slate-200 peer-focus:outline-none rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#0bda5b]"></div>
                            </div>
                            <span class="text-sm font-bold text-slate-900 dark:text-white">Enable HLS Streaming</span>
                        </label>
                        
                        {#if rtspConfig.hls.enabled}
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                                <div class="space-y-2">
                                    <label for="hlsOutputDir" class="text-xs font-medium text-slate-500">HLS Output Dir</label>
                                    <input id="hlsOutputDir" type="text" bind:value={rtspConfig.hls.output_directory} class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded px-3 py-1.5 text-sm" />
                                </div>
                                <div class="space-y-2">
                                    <label for="hlsPlaylistSize" class="text-xs font-medium text-slate-500">Playlist Size</label>
                                    <input id="hlsPlaylistSize" type="number" bind:value={rtspConfig.hls.playlist_size} min="3" max="50" class="w-full bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded px-3 py-1.5 text-sm" />
                                </div>
                            </div>
                        {/if}
                    </div>

                    <div class="flex justify-end pt-4">
                        <button type="submit" disabled={isStreaming} class="px-6 py-2.5 bg-[#137fec] hover:bg-blue-600 text-white rounded-lg font-bold shadow-lg shadow-[#137fec]/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2">
                            {#if isStreaming}
                                <span class="material-symbols-outlined animate-spin text-sm">sync</span>
                                Processing...
                            {:else}
                                Start Capture Process
                            {/if}
                        </button>
                    </div>
                </form>
            </div>
        </div>

        <!-- Status & Logs Sidebar -->
        <div class="flex flex-col gap-4">
            <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm">
                <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display mb-4 uppercase tracking-wider">System Status</h3>
                {#if status}
                    <div class="p-3 rounded-lg text-sm border {status.includes('Error') ? 'bg-red-50 dark:bg-red-900/10 border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400' : 'bg-blue-50 dark:bg-blue-900/10 border-blue-200 dark:border-blue-900/30 text-blue-600 dark:text-blue-400'}">
                        {status}
                    </div>
                {:else}
                    <p class="text-sm text-slate-500 dark:text-[#9dabb9]">System idle. Ready to start capture.</p>
                {/if}
            </div>

            <!-- Active Streams List -->
            {#if streamStats && streamStats.streams.length > 0}
                <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm">
                    <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display mb-4 uppercase tracking-wider">Active Streams</h3>
                    <div class="space-y-2">
                        {#each streamStats.streams as stream}
                            <div class="flex items-center justify-between p-3 rounded-lg bg-slate-50 dark:bg-[#1a242d] border border-slate-100 dark:border-[#283039]">
                                <div class="flex items-center gap-3">
                                    <div class="size-2 rounded-full {stream.status === 'active' ? 'bg-green-500' : 'bg-slate-400'}"></div>
                                    <div>
                                        <p class="text-sm font-medium text-slate-900 dark:text-white">{stream.name}</p>
                                        <p class="text-xs text-slate-500">{stream.codec ?? 'Unknown'} • {stream.resolution ?? 'N/A'}</p>
                                    </div>
                                </div>
                                {#if stream.latencyMs}
                                    <span class="text-xs font-mono text-slate-500">{stream.latencyMs}ms</span>
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}

            <div class="rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm flex-1">
                <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display mb-4 uppercase tracking-wider">Help & Docs</h3>
                <div class="space-y-4 text-sm text-slate-600 dark:text-[#9dabb9]">
                    <p>Use this panel to ingest RTSP streams into the Media Core engine.</p>
                    <ul class="list-disc pl-4 space-y-2">
                        <li>Supports H.264/H.265 streams</li>
                        <li>Real-time HLS playlist generation</li>
                        <li>Automatic segment rotation</li>
                        <li>Low-latency preview</li>
                    </ul>
                </div>
            </div>
        </div>
    </div>
</div>
