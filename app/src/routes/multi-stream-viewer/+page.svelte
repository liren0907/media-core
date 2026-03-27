<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import Hls from "hls.js";
  import { appConfig, getDefaultRtspUrl } from "$lib/config.svelte";
  import { PageContent, Panel, StatCard, StatusBadge, EmptyState } from '$lib/components/ui';
  import type { StreamStats } from '$lib/types';

  let streamStats = $state<StreamStats | null>(null);
  let urlsInput = $state("");
  let streams: { url: string; hls: Hls | null; videoElement: HTMLVideoElement; status: string }[] = $state([]);
  let status = $state("Enter RTSP URLs and click 'Start Streams' to begin.");
  let isLoading = $state(false);

  type LayoutMode = 'grid' | '1+3' | '2x2' | 'focus';
  let layoutMode: LayoutMode = $state('grid');
  let focusedStreamIndex = $state(0);

  async function fetchStreamStats() {
    try { streamStats = await invoke('get_stream_stats'); } catch (e) { console.error('Failed to fetch stream stats:', e); }
  }

  async function startAllStreams() {
    const urls = urlsInput.split("\n").map(u => u.trim()).filter(u => u);
    if (urls.length === 0) { status = "Please enter at least one RTSP URL."; return; }
    isLoading = true;
    status = `Starting ${urls.length} stream(s)...`;
    streams.forEach(s => s.hls?.destroy());
    streams = urls.map(url => ({ url, hls: null, videoElement: null as any, status: 'connecting' }));
    try {
      const playlistPaths = await invoke("start_multiple_hls_playback", { urls });
      await new Promise(r => setTimeout(r, 1000));
      (playlistPaths as string[]).forEach((path, i) => {
        const stream = streams[i];
        const videoSrc = convertFileSrc(path);
        if (Hls.isSupported()) {
          const hls = new Hls();
          hls.loadSource(videoSrc);
          hls.attachMedia(stream.videoElement);
          hls.on(Hls.Events.MANIFEST_PARSED, () => { streams[i].status = 'active'; streams = [...streams]; });
          hls.on(Hls.Events.ERROR, (_, data) => { if (data.fatal) { streams[i].status = 'error'; streams = [...streams]; } });
          stream.hls = hls;
        } else if (stream.videoElement.canPlayType("application/vnd.apple.mpegurl")) {
          stream.videoElement.src = videoSrc;
          stream.status = 'active';
        }
      });
      status = "All streams active.";
      await fetchStreamStats();
    } catch (e) {
      status = `Error starting streams: ${e}`;
      streams = streams.map(s => ({ ...s, status: 'error' }));
    } finally { isLoading = false; }
  }

  function setFocusedStream(index: number) {
    focusedStreamIndex = index;
    if (layoutMode !== 'focus') layoutMode = 'focus';
  }

  function getStatusColor(s: string): string {
    if (s === 'active') return 'bg-green-500';
    if (s === 'connecting') return 'bg-yellow-500 animate-pulse';
    if (s === 'error') return 'bg-red-500';
    return 'bg-slate-500';
  }

  $effect(() => {
    const defaultUrl = getDefaultRtspUrl();
    urlsInput = `${defaultUrl}\n${defaultUrl.replace('/mystream', '/mystream2')}`;
    fetchStreamStats();
    const statsInterval = setInterval(fetchStreamStats, 5000);
    return () => { streams.forEach(s => s.hls?.destroy()); clearInterval(statsInterval); };
  });

  let activeStreams = $derived(streams.filter(s => s.status === 'active').length);
  let gridCols = $derived(streams.length <= 4 ? 'grid-cols-1 sm:grid-cols-2' : 'grid-cols-1 sm:grid-cols-2 xl:grid-cols-3');
</script>

<svelte:head>
  <title>Multi-Stream Viewer</title>
</svelte:head>

<PageContent>
    <!-- Stats Strip -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Active" icon="cell_tower" iconColor="text-[#137fec]" value="{activeStreams} / {streams.length}" />
        <StatCard label="Avg Latency" icon="speed" iconColor="text-orange-500" value="{streamStats?.avgLatencyMs?.toFixed(0) ?? '--'}ms" />
        <StatCard label="Bitrate" icon="data_usage" iconColor="text-purple-500" value="{streamStats?.totalBitrateKbps ? (streamStats.totalBitrateKbps / 1000).toFixed(1) : '--'} Mbps" />
        <StatCard label="Backend" icon="dns" iconColor="text-green-500" value="{streamStats?.totalCount ?? 0} streams" />
    </div>

    <!-- Config + Layout controls -->
    <Panel title="Stream Configuration" icon="input">
        {#snippet actions()}
            <div class="flex items-center gap-2">
                <div class="flex bg-slate-100 dark:bg-[#1f2937] rounded p-0.5">
                    <button onclick={() => layoutMode = 'grid'} class="p-1 rounded transition-colors {layoutMode === 'grid' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}">
                        <span class="material-symbols-outlined text-[14px]">grid_view</span>
                    </button>
                    <button onclick={() => layoutMode = '2x2'} class="p-1 rounded transition-colors {layoutMode === '2x2' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}">
                        <span class="material-symbols-outlined text-[14px]">view_module</span>
                    </button>
                    <button onclick={() => layoutMode = 'focus'} class="p-1 rounded transition-colors {layoutMode === 'focus' ? 'bg-[#137fec] text-white' : 'text-slate-500 hover:text-slate-700 dark:hover:text-white'}">
                        <span class="material-symbols-outlined text-[14px]">center_focus_strong</span>
                    </button>
                </div>
                <button class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors disabled:opacity-50" onclick={startAllStreams} disabled={isLoading}>
                    {#if isLoading}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{/if}
                    Start
                </button>
            </div>
        {/snippet}
        <div class="p-3">
            <label for="rtspUrlsInput" class="text-[10px] font-medium uppercase tracking-wider text-slate-500 mb-1.5 block">RTSP/HLS URLs (one per line)</label>
            <textarea id="rtspUrlsInput" bind:value={urlsInput} class="w-full h-20 bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 font-mono text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]" placeholder="rtsp://192.168.1.100/stream1&#10;rtsp://192.168.1.101/stream2"></textarea>
            {#if status && status !== "Enter RTSP URLs and click 'Start Streams' to begin."}
                <div class="mt-2 p-2 rounded text-xs {status.includes('Error') ? 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 border border-red-200 dark:border-red-900/30' : 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-900/30'}">
                    {status}
                </div>
            {/if}
        </div>
    </Panel>

    <!-- Video Grid -->
    {#if streams.length > 0}
        {#if layoutMode === 'focus'}
            <div class="grid grid-cols-4 gap-3">
                <div class="col-span-3 rounded-lg bg-black border border-slate-200 dark:border-[#2a3441] overflow-hidden">
                    <div class="aspect-video relative">
                        <video bind:this={streams[focusedStreamIndex].videoElement} controls muted autoplay class="w-full h-full object-contain"><track kind="captions" /></video>
                        <div class="absolute top-2 left-2 flex items-center gap-1.5">
                            <span class="size-2 rounded-full {getStatusColor(streams[focusedStreamIndex].status)}"></span>
                            <span class="bg-black/60 text-white text-[10px] px-1.5 py-0.5 rounded font-mono">CAM {focusedStreamIndex + 1}</span>
                        </div>
                    </div>
                    <div class="p-2 bg-slate-900 border-t border-slate-800">
                        <p class="text-[10px] text-slate-400 truncate font-mono">{streams[focusedStreamIndex].url}</p>
                    </div>
                </div>
                <div class="col-span-1 flex flex-col gap-1.5 overflow-y-auto max-h-[500px]">
                    {#each streams as stream, i}
                        <button onclick={() => setFocusedStream(i)} class="rounded overflow-hidden border-2 transition-colors {i === focusedStreamIndex ? 'border-[#137fec]' : 'border-transparent hover:border-slate-500'}">
                            <div class="aspect-video bg-black relative">
                                {#if i !== focusedStreamIndex}
                                    <video bind:this={stream.videoElement} muted autoplay class="w-full h-full object-contain pointer-events-none"><track kind="captions" /></video>
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
            <div class="grid {layoutMode === '2x2' ? 'grid-cols-2' : gridCols} gap-3">
                {#each streams as stream, i (stream.url)}
                    <div class="rounded-lg bg-black border border-slate-200 dark:border-[#2a3441] overflow-hidden group cursor-pointer" onclick={() => setFocusedStream(i)} onkeydown={(e) => e.key === 'Enter' && setFocusedStream(i)} role="button" tabindex="0">
                        <div class="aspect-video relative">
                            <video bind:this={stream.videoElement} controls muted autoplay class="w-full h-full object-contain"><track kind="captions" /></video>
                            <div class="absolute top-2 left-2 flex items-center gap-1.5">
                                <span class="size-2 rounded-full {getStatusColor(stream.status)}"></span>
                                <span class="bg-black/60 text-white text-[10px] px-1.5 py-0.5 rounded font-mono">CAM {i + 1}</span>
                            </div>
                            <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                <span class="bg-black/60 text-white text-[10px] px-1.5 py-0.5 rounded flex items-center gap-0.5">
                                    <span class="material-symbols-outlined text-[12px]">fullscreen</span> Focus
                                </span>
                            </div>
                        </div>
                        <div class="p-2 bg-slate-900/50 border-t border-slate-800/50 flex items-center justify-between">
                            <p class="text-[10px] text-slate-400 truncate font-mono max-w-[70%]" title={stream.url}>{stream.url}</p>
                            <StatusBadge status={stream.status} />
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    {:else}
        <Panel title="Streams" icon="videocam">
            <EmptyState icon="videocam_off" message="No streams active. Enter URLs and click Start." />
        </Panel>
    {/if}
</PageContent>
