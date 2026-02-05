<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import Hls from "hls.js";
  import { appConfig, getDefaultRtspUrl } from "$lib/config";

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

  let streamStats: StreamStats | null = null;
  let statsInterval: ReturnType<typeof setInterval>;

  let urlsInput = "";
  let streams: { url: string; hls: Hls | null; videoElement: HTMLVideoElement; status: string }[] = [];
  let status = "Enter RTSP URLs and click 'Start Streams' to begin.";
  let isLoading = false;

  // Layout options
  type LayoutMode = 'grid' | '1+3' | '2x2' | 'focus';
  let layoutMode: LayoutMode = 'grid';
  let focusedStreamIndex = 0;

  async function fetchStreamStats() {
    try {
      streamStats = await invoke('get_stream_stats');
    } catch (e) {
      console.error('Failed to fetch stream stats:', e);
    }
  }

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
    streams = urls.map(url => ({ url, hls: null, videoElement: null as any, status: 'connecting' }));

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
                
                hls.on(Hls.Events.MANIFEST_PARSED, () => {
                    streams[i].status = 'active';
                    streams = [...streams];
                });
                
                hls.on(Hls.Events.ERROR, (_, data) => {
                    if (data.fatal) {
                        streams[i].status = 'error';
                        streams = [...streams];
                    }
                });
                
                stream.hls = hls;
            } else if (stream.videoElement.canPlayType("application/vnd.apple.mpegurl")) {
                stream.videoElement.src = videoSrc;
                stream.status = 'active';
            }
        });

        status = "All streams active.";
        
        // Refresh stats
        await fetchStreamStats();
    } catch (e) {
        status = `Error starting streams: ${e}`;
        console.error("Failed to start multiple streams:", e);
        streams = streams.map(s => ({ ...s, status: 'error' }));
    } finally {
        isLoading = false;
    }
  }

  function setFocusedStream(index: number) {
    focusedStreamIndex = index;
    if (layoutMode !== 'focus') {
      layoutMode = 'focus';
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'active': return 'bg-green-500';
      case 'connecting': return 'bg-yellow-500 animate-pulse';
      case 'error': return 'bg-red-500';
      default: return 'bg-slate-500';
    }
  }

  function getStatusBadgeClass(status: string): string {
    switch (status) {
      case 'active': return 'badge-success';
      case 'connecting': return 'badge-warning';
      case 'error': return 'badge-error';
      default: return 'badge-ghost';
    }
  }

  onMount(() => {
    // Initialize with default RTSP URL from config
    const defaultUrl = getDefaultRtspUrl();
    urlsInput = `${defaultUrl}\n${defaultUrl.replace('/mystream', '/mystream2')}`;
    
    fetchStreamStats();
    statsInterval = setInterval(fetchStreamStats, 5000);
  });

  onDestroy(() => {
    streams.forEach(s => s.hls?.destroy());
    if (statsInterval) clearInterval(statsInterval);
  });

  $: activeStreams = streams.filter(s => s.status === 'active').length;
  $: gridCols = streams.length <= 2 ? 'grid-cols-1 sm:grid-cols-2' : 
                streams.length <= 4 ? 'grid-cols-1 sm:grid-cols-2' : 
                'grid-cols-1 sm:grid-cols-2 xl:grid-cols-3';
</script>

<svelte:head>
	<title>Multi-Stream Viewer</title>
</svelte:head>

<div class="p-8 max-w-[1800px] w-full mx-auto space-y-6">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div class="flex flex-col gap-2">
            <h2 class="text-slate-900 dark:text-white text-3xl font-bold font-display tracking-tight">Multi-Stream Viewer</h2>
            <p class="text-slate-500 dark:text-[#9dabb9] text-base">Monitor multiple live RTSP/HLS streams simultaneously</p>
        </div>
        <div class="flex gap-3">
            <!-- Layout Buttons -->
            <div class="flex bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] rounded-lg p-1">
                <button 
                    on:click={() => layoutMode = 'grid'}
                    class="p-2 rounded transition-colors {layoutMode === 'grid' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}"
                    title="Grid View"
                >
                    <span class="material-symbols-outlined text-sm">grid_view</span>
                </button>
                <button 
                    on:click={() => layoutMode = '2x2'}
                    class="p-2 rounded transition-colors {layoutMode === '2x2' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}"
                    title="2x2 View"
                >
                    <span class="material-symbols-outlined text-sm">view_module</span>
                </button>
                <button 
                    on:click={() => layoutMode = 'focus'}
                    class="p-2 rounded transition-colors {layoutMode === 'focus' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}"
                    title="Focus View"
                >
                    <span class="material-symbols-outlined text-sm">center_focus_strong</span>
                </button>
            </div>
        </div>
    </div>

    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div class="p-4 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm">
            <p class="text-slate-500 dark:text-[#9dabb9] text-xs font-medium uppercase tracking-wider">Active</p>
            <p class="text-slate-900 dark:text-white text-2xl font-bold mt-1">{activeStreams} / {streams.length}</p>
        </div>
        <div class="p-4 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm">
            <p class="text-slate-500 dark:text-[#9dabb9] text-xs font-medium uppercase tracking-wider">Avg Latency</p>
            <p class="text-slate-900 dark:text-white text-2xl font-bold mt-1">{streamStats?.avgLatencyMs?.toFixed(0) ?? '--'} <span class="text-sm font-normal text-slate-500">ms</span></p>
        </div>
        <div class="p-4 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm">
            <p class="text-slate-500 dark:text-[#9dabb9] text-xs font-medium uppercase tracking-wider">Total Bitrate</p>
            <p class="text-slate-900 dark:text-white text-2xl font-bold mt-1">{streamStats?.totalBitrateKbps ? (streamStats.totalBitrateKbps / 1000).toFixed(1) : '--'} <span class="text-sm font-normal text-slate-500">Mbps</span></p>
        </div>
        <div class="p-4 rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm">
            <p class="text-slate-500 dark:text-[#9dabb9] text-xs font-medium uppercase tracking-wider">Backend Streams</p>
            <p class="text-slate-900 dark:text-white text-2xl font-bold mt-1">{streamStats?.totalCount ?? 0}</p>
        </div>
    </div>

    <!-- Configuration Card -->
    <div class="rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] shadow-sm overflow-hidden">
        <div class="p-4 border-b border-slate-200 dark:border-[#283039] bg-slate-50 dark:bg-[#1a242d] flex justify-between items-center">
            <h3 class="text-sm font-bold text-slate-900 dark:text-white font-display">Stream Configuration</h3>
            <button 
                class="flex items-center gap-2 px-4 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded-lg font-medium text-sm transition-colors disabled:opacity-50" 
                on:click={startAllStreams} 
                disabled={isLoading}
            >
                {#if isLoading}
                    <span class="material-symbols-outlined animate-spin text-sm">sync</span>
                {:else}
                    <span class="material-symbols-outlined text-sm">play_arrow</span>
                {/if}
                Start Streams
            </button>
        </div>
        <div class="p-4">
            <label for="rtspUrlsInput" class="text-xs font-medium text-slate-500 dark:text-slate-400 mb-2 block">
                RTSP/HLS URLs (one per line)
            </label>
            <textarea
                id="rtspUrlsInput"
                bind:value={urlsInput}
                class="w-full h-24 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#283039] rounded-lg px-4 py-3 font-mono text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]"
                placeholder="rtsp://192.168.1.100/stream1&#10;rtsp://192.168.1.101/stream2"
            ></textarea>
            
            {#if status && status !== "Enter RTSP URLs and click 'Start Streams' to begin."}
                <div class="mt-3 p-3 rounded-lg text-sm {status.includes('Error') ? 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 border border-red-200 dark:border-red-900/30' : 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-900/30'}">
                    {status}
                </div>
            {/if}
        </div>
    </div>

    <!-- Video Grid -->
    {#if streams.length > 0}
        {#if layoutMode === 'focus'}
            <!-- Focus Layout: One large + thumbnails -->
            <div class="grid grid-cols-4 gap-4">
                <!-- Main focused stream -->
                <div class="col-span-3 rounded-xl bg-black border border-slate-200 dark:border-[#283039] overflow-hidden shadow-lg">
                    <div class="aspect-video relative">
                        <video 
                            bind:this={streams[focusedStreamIndex].videoElement} 
                            controls 
                            muted
                            autoplay
                            class="w-full h-full object-contain"
                        >
                            <track kind="captions" />
                        </video>
                        <div class="absolute top-3 left-3 flex items-center gap-2">
                            <span class="size-2 rounded-full {getStatusColor(streams[focusedStreamIndex].status)}"></span>
                            <span class="bg-black/60 text-white text-xs px-2 py-1 rounded font-mono">CAM {focusedStreamIndex + 1}</span>
                        </div>
                    </div>
                    <div class="p-3 bg-slate-900 border-t border-slate-800">
                        <p class="text-xs text-slate-400 truncate font-mono">{streams[focusedStreamIndex].url}</p>
                    </div>
                </div>
                
                <!-- Thumbnails -->
                <div class="col-span-1 flex flex-col gap-2 overflow-y-auto max-h-[600px]">
                    {#each streams as stream, i}
                        <button 
                            on:click={() => setFocusedStream(i)}
                            class="rounded-lg overflow-hidden border-2 transition-colors {i === focusedStreamIndex ? 'border-[#137fec]' : 'border-transparent hover:border-slate-500'}"
                        >
                            <div class="aspect-video bg-black relative">
                                {#if i !== focusedStreamIndex}
                                    <video 
                                        bind:this={stream.videoElement} 
                                        muted
                                        autoplay
                                        class="w-full h-full object-contain pointer-events-none"
                                    >
                                        <track kind="captions" />
                                    </video>
                                {/if}
                                <div class="absolute top-1 left-1 flex items-center gap-1">
                                    <span class="size-1.5 rounded-full {getStatusColor(stream.status)}"></span>
                                    <span class="text-[10px] text-white/80 font-mono">CAM {i + 1}</span>
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            </div>
        {:else}
            <!-- Grid/2x2 Layout -->
            <div class="grid {layoutMode === '2x2' ? 'grid-cols-2' : gridCols} gap-4">
                {#each streams as stream, i (stream.url)}
                    <div 
                        class="rounded-xl bg-black border border-slate-200 dark:border-[#283039] overflow-hidden shadow-lg group cursor-pointer"
                        on:click={() => setFocusedStream(i)}
                        on:keydown={(e) => e.key === 'Enter' && setFocusedStream(i)}
                        role="button"
                        tabindex="0"
                    >
                        <div class="aspect-video relative">
                            <video 
                                bind:this={stream.videoElement} 
                                controls 
                                muted
                                autoplay
                                class="w-full h-full object-contain"
                            >
                                <track kind="captions" />
                            </video>
                            <div class="absolute top-2 left-2 flex items-center gap-2">
                                <span class="size-2 rounded-full {getStatusColor(stream.status)}"></span>
                                <span class="bg-black/60 text-white text-xs px-2 py-0.5 rounded font-mono">CAM {i + 1}</span>
                            </div>
                            <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                <span class="bg-black/60 text-white text-xs px-2 py-1 rounded flex items-center gap-1">
                                    <span class="material-symbols-outlined text-sm">fullscreen</span>
                                    Focus
                                </span>
                            </div>
                        </div>
                        <div class="p-3 bg-slate-900/50 border-t border-slate-800/50">
                            <div class="flex items-center justify-between">
                                <p class="text-xs text-slate-400 truncate font-mono max-w-[70%]" title={stream.url}>{stream.url}</p>
                                <span class="badge badge-sm {getStatusBadgeClass(stream.status)} capitalize">{stream.status}</span>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    {:else}
        <!-- Empty State -->
        <div class="rounded-xl bg-white dark:bg-[#182129] border border-slate-200 dark:border-[#283039] p-12 text-center">
            <span class="material-symbols-outlined text-6xl text-slate-300 dark:text-slate-600 mb-4">videocam_off</span>
            <h3 class="text-lg font-bold text-slate-700 dark:text-slate-300 mb-2">No Streams Active</h3>
            <p class="text-sm text-slate-500 dark:text-slate-400">Enter RTSP URLs above and click "Start Streams" to begin monitoring</p>
        </div>
    {/if}
</div>
