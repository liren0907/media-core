<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import Hls from "hls.js";
  import { appConfig, getDefaultRtspUrl, getHlsOutputDir } from "$lib/config.svelte";
  import { Panel, ToggleSwitch, FormField } from '$lib/components/ui';
  import { StreamStatsStrip, ActiveStreamsList } from '$lib/components/features/stream';
  import type { StreamStats } from '$lib/types';

  interface Props {
    streamStats: StreamStats | null;
  }

  let { streamStats }: Props = $props();

  let rtspConfig = $state({
    rtsp_url: "",
    rtsp_url_list: [""],
    output_directory: "",
    show_preview: false,
    saving_option: "single",
    saved_time_duration: 60,
    use_fps: false,
    fps: 30.0,
    hls: { enabled: false, output_directory: "", segment_duration: 6, playlist_size: 10 },
  });

  let isStreaming = $state(false);
  let status = $state("");
  let urlCount = $state(1);
  let hls: Hls | null = $state(null);
  let videoElement: HTMLVideoElement;

  function addUrlField() { rtspConfig.rtsp_url_list = [...rtspConfig.rtsp_url_list, ""]; urlCount++; }
  function removeUrlField(index: number) { rtspConfig.rtsp_url_list = rtspConfig.rtsp_url_list.filter((_, i) => i !== index); urlCount--; }

  async function selectOutputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) rtspConfig.output_directory = selected as string;
  }

  async function startCapture() {
    try {
      isStreaming = true;
      status = "Starting RTSP capture...";
      rtspConfig.rtsp_url_list = rtspConfig.rtsp_url_list.filter((url) => url.trim() !== "");
      const result = await invoke("start_rtsp_capture", { payload: rtspConfig });
      status = result as string;
    } catch (error) {
      status = `Error: ${error}`;
      isStreaming = false;
    }
  }

  async function playInApp() {
    status = "Starting HLS stream...";
    try {
      const playlistUrl = await invoke("start_hls_playback", { url: rtspConfig.rtsp_url });
      if (Hls.isSupported()) {
        if (hls) hls.destroy();
        hls = new Hls();
        hls.loadSource(playlistUrl as string);
        hls.attachMedia(videoElement);
        hls.on(Hls.Events.ERROR, function (event, data) {
          if (data.fatal) { status = `Fatal HLS Error: ${data.details}`; hls?.destroy(); }
          else status = `HLS Error: ${data.details}`;
        });
        videoElement.style.display = "block";
        status = "HLS stream is loading...";
      } else if (videoElement.canPlayType("application/vnd.apple.mpegurl")) {
        videoElement.src = playlistUrl as string;
        videoElement.style.display = "block";
        status = "HLS stream is playing.";
      }
    } catch (e) { status = `Error starting HLS stream: ${e}`; }
  }

  async function playInNewWindow() {
    status = "Requesting direct playback...";
    try {
      await invoke("start_direct_playback", { url: rtspConfig.rtsp_url });
      status = "Direct playback window opened.";
    } catch (e) { status = `Error: ${e}`; }
  }

  function getHealthStatus(): { text: string; status: string } {
    if (!streamStats) return { text: 'Unknown', status: 'idle' };
    const r = streamStats.totalCount > 0 ? streamStats.activeCount / streamStats.totalCount : 0;
    if (r >= 0.9) return { text: 'Healthy', status: 'active' };
    if (r >= 0.5) return { text: 'Degraded', status: 'connecting' };
    if (streamStats.activeCount > 0) return { text: 'Limited', status: 'connecting' };
    return { text: 'Idle', status: 'idle' };
  }

  $effect(() => {
    rtspConfig.rtsp_url = getDefaultRtspUrl();
    rtspConfig.hls.output_directory = getHlsOutputDir() || 'hls_output';
    return () => { isStreaming = false; if (hls) hls.destroy(); };
  });

  let healthStatus = $derived(getHealthStatus());
  let latencyPercent = $derived(streamStats ? Math.min((streamStats.avgLatencyMs / 200) * 100, 100) : 0);

  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded-lg px-3 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]';
</script>

    <!-- Stats Strip -->
    <StreamStatsStrip {streamStats} {latencyPercent} healthStatus={healthStatus.text} />

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <!-- Configuration Form -->
        <div class="lg:col-span-2">
            <Panel title="Configure Stream" icon="settings">
                {#snippet actions()}
                    <button type="button" disabled={isStreaming} onclick={startCapture} class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors disabled:opacity-50">
                        {#if isStreaming}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{/if}
                        Start Capture
                    </button>
                {/snippet}
                <div class="p-4">
                    <!-- Video Preview -->
                    <div class="mb-4 rounded-lg overflow-hidden bg-black flex justify-center items-center min-h-[160px] border border-slate-200 dark:border-[#2a3441]" class:hidden={videoElement?.style.display === 'none'}>
                        <video bind:this={videoElement} controls class="w-full max-h-[300px]" style="display: none;"><track kind="captions" /></video>
                    </div>

                    <form onsubmit={(e) => { e.preventDefault(); startCapture(); }} class="flex flex-col gap-4">
                        <!-- Main URL -->
                        <FormField label="Main RTSP URL" id="mainRtspUrl">
                            <div class="flex gap-2">
                                <input id="mainRtspUrl" type="text" bind:value={rtspConfig.rtsp_url} class="{inputClass} flex-1" placeholder="rtsp://example.com/stream" />
                                <button type="button" onclick={playInApp} class="px-3 py-2 bg-[#137fec] hover:bg-blue-600 text-white rounded text-xs font-bold transition-colors">Play</button>
                                <button type="button" onclick={playInNewWindow} class="px-3 py-2 bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] text-slate-700 dark:text-white rounded text-xs font-bold transition-colors">Pop-out</button>
                            </div>
                        </FormField>

                        <!-- Additional URLs -->
                        <div class="flex flex-col gap-1.5">
                            <span class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Additional URLs ({rtspConfig.rtsp_url_list.length})</span>
                            {#each rtspConfig.rtsp_url_list as url, i}
                                <div class="flex gap-2">
                                    <input id={`additionalRtspUrl-${i}`} type="text" bind:value={rtspConfig.rtsp_url_list[i]} class="{inputClass} flex-1" placeholder="rtsp://..." aria-label={`Additional RTSP URL ${i + 1}`} />
                                    <button type="button" onclick={() => removeUrlField(i)} class="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors">
                                        <span class="material-symbols-outlined text-[18px]">delete</span>
                                    </button>
                                </div>
                            {/each}
                            <button type="button" onclick={addUrlField} class="w-full py-1.5 border-2 border-dashed border-slate-200 dark:border-[#2a3441] rounded text-slate-500 hover:text-[#137fec] hover:border-[#137fec] transition-colors text-xs font-medium">
                                + Add URL
                            </button>
                        </div>

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                            <FormField label="Output Directory" id="outputDirectory">
                                <div class="flex gap-2">
                                    <input id="outputDirectory" type="text" readonly bind:value={rtspConfig.output_directory} class="{inputClass} flex-1" placeholder="Select..." />
                                    <button type="button" onclick={selectOutputDirectory} class="px-3 py-2 bg-slate-100 dark:bg-[#1f2937] border border-slate-200 dark:border-[#2a3441] rounded text-xs transition-colors"><span class="material-symbols-outlined text-[18px]">folder_open</span></button>
                                </div>
                            </FormField>
                            <FormField label="Segment Duration (sec)" id="segmentDuration">
                                <input id="segmentDuration" type="number" bind:value={rtspConfig.saved_time_duration} min="1" class="{inputClass} w-full" />
                            </FormField>
                        </div>

                        <!-- Toggles -->
                        <div class="flex gap-4">
                            <ToggleSwitch bind:checked={rtspConfig.show_preview} label="Preview" />
                            <ToggleSwitch bind:checked={rtspConfig.use_fps} label="Custom FPS" />
                        </div>

                        <!-- HLS Config -->
                        <div class="p-3 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-slate-50 dark:bg-[#1f2937]/50">
                            <label class="flex items-center gap-2 cursor-pointer mb-3">
                                <div class="relative inline-flex items-center">
                                    <input type="checkbox" bind:checked={rtspConfig.hls.enabled} class="sr-only peer">
                                    <div class="w-9 h-5 bg-slate-200 rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-green-500"></div>
                                </div>
                                <span class="text-xs font-bold text-slate-900 dark:text-white">Enable HLS Streaming</span>
                            </label>
                            {#if rtspConfig.hls.enabled}
                                <div class="grid grid-cols-2 gap-3">
                                    <FormField label="HLS Output Dir" id="hlsOutputDir">
                                        <input id="hlsOutputDir" type="text" bind:value={rtspConfig.hls.output_directory} class="{inputClass} w-full" />
                                    </FormField>
                                    <FormField label="Playlist Size" id="hlsPlaylistSize">
                                        <input id="hlsPlaylistSize" type="number" bind:value={rtspConfig.hls.playlist_size} min="3" max="50" class="{inputClass} w-full" />
                                    </FormField>
                                </div>
                            {/if}
                        </div>
                    </form>
                </div>
            </Panel>
        </div>

        <!-- Sidebar -->
        <div class="flex flex-col gap-3">
            <Panel title="Status" icon="info">
                <div class="p-3">
                    {#if status}
                        <div class="p-2 rounded text-xs border {status.includes('Error') ? 'bg-red-50 dark:bg-red-900/10 border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400' : 'bg-blue-50 dark:bg-blue-900/10 border-blue-200 dark:border-blue-900/30 text-blue-600 dark:text-blue-400'}">
                            {status}
                        </div>
                    {:else}
                        <p class="text-xs text-slate-500">System idle. Ready to start capture.</p>
                    {/if}
                </div>
            </Panel>

            <ActiveStreamsList {streamStats} />

            <Panel title="Help" icon="help">
                <div class="p-3 text-[10px] text-slate-500 flex flex-col gap-2">
                    <p>Use this panel to ingest RTSP streams.</p>
                    <ul class="list-disc pl-3 flex flex-col gap-1">
                        <li>H.264/H.265 streams</li>
                        <li>Real-time HLS generation</li>
                        <li>Automatic segment rotation</li>
                        <li>Low-latency preview</li>
                    </ul>
                </div>
            </Panel>
        </div>
    </div>
