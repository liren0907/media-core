<script lang="ts">
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { PageContent, Panel, StatCard } from '$lib/components/ui';
  import { VideoPlayer } from '$lib/components/media';
  import type { MediaMetadata } from '$lib/types';
  import { formatDuration, formatFileSize, formatBitrate } from '$lib/utils/format';

  let videoSrc = $state("");
  let currentFilePath = $state("");
  let isLoading = $state(false);
  let error = $state("");

  let metadata = $state<MediaMetadata | null>(null);

  async function loadVideo() {
    try {
      error = "";
      const filePath = await open({ filters: [{ name: "Video", extensions: ["mp4", "avi", "mkv", "mov", "webm"] }] });
      if (filePath) {
        currentFilePath = filePath as string;
        videoSrc = convertFileSrc(currentFilePath);
        isLoading = true;
        try {
          metadata = await invoke("execute_metadata_pipeline", { filePath: currentFilePath, includeThumbnail: true });
        } catch (e) {
          error = `Failed to load metadata: ${e}`;
          metadata = null;
        }
        isLoading = false;
      }
    } catch (e) { error = `Error loading video: ${e}`; }
  }

</script>

<svelte:head>
  <title>Video Viewer</title>
</svelte:head>

<PageContent>
    <!-- Player -->
    <Panel title="Player" icon="play_circle">
        {#snippet actions()}
            <div class="flex items-center gap-2">
                {#if metadata}
                    <span class="text-[10px] font-mono text-slate-500 truncate max-w-[200px]">{metadata.filename}</span>
                {/if}
                <button onclick={loadVideo} disabled={isLoading} class="flex items-center gap-1 px-2 py-1 bg-[#137fec] hover:bg-blue-600 text-white rounded text-[10px] font-bold transition-colors disabled:opacity-50">
                    {#if isLoading}<span class="material-symbols-outlined animate-spin text-[14px]">sync</span>{/if}
                    Open File
                </button>
            </div>
        {/snippet}
        <VideoPlayer src={videoSrc} placeholderIcon="movie" placeholderText="Select a video file to play" />
    </Panel>

    {#if error}
        <div class="p-2 rounded bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/30 text-red-600 dark:text-red-400 text-xs">{error}</div>
    {/if}

    {#if metadata}
        <!-- Primary Stats -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
            <StatCard label="Duration" icon="schedule" iconColor="text-[#137fec]" value={formatDuration(metadata.duration)} />
            <StatCard label="Resolution" icon="aspect_ratio" iconColor="text-purple-500" value={metadata.width && metadata.height ? `${metadata.width}x${metadata.height}` : 'N/A'} />
            <StatCard label="Frame Rate" icon="speed" iconColor="text-orange-500" value={metadata.fps ? `${metadata.fps.toFixed(1)} FPS` : 'N/A'} />
            <StatCard label="Frames" icon="filter_frames" iconColor="text-green-500" value={metadata.frameCount?.toLocaleString() ?? 'N/A'} />
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
            <!-- Technical Details -->
            <div class="lg:col-span-2">
                <Panel title="Technical Details" icon="info">
                    <div class="p-3">
                        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
                            {#each [
                                { label: "Video Codec", value: metadata.codec ?? "N/A" },
                                { label: "Audio Codec", value: metadata.audioCodec ?? "N/A" },
                                { label: "Bitrate", value: formatBitrate(metadata.bitrate) },
                                { label: "File Size", value: formatFileSize(metadata.fileSize) },
                                { label: "MIME Type", value: metadata.mimeType ?? "N/A" },
                                { label: "Sample Rate", value: metadata.audioSampleRate ? `${metadata.audioSampleRate} Hz` : "N/A" },
                            ] as item}
                                <div class="flex justify-between items-center p-2 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441]">
                                    <span class="text-[10px] uppercase tracking-wider text-slate-500">{item.label}</span>
                                    <span class="font-mono text-xs text-slate-900 dark:text-white">{item.value}</span>
                                </div>
                            {/each}
                        </div>
                    </div>
                </Panel>
            </div>

            <!-- Thumbnail & File Info -->
            <Panel title="Preview" icon="image">
                <div class="p-3">
                    {#if metadata.thumbnail}
                        <div class="aspect-video bg-black rounded overflow-hidden mb-3">
                            <img src="data:image/jpeg;base64,{metadata.thumbnail}" alt="Thumbnail" class="w-full h-full object-cover" />
                        </div>
                    {:else}
                        <div class="aspect-video bg-slate-100 dark:bg-[#1f2937] rounded flex items-center justify-center mb-3">
                            <span class="text-xs text-slate-500">No thumbnail</span>
                        </div>
                    {/if}
                    <div class="flex flex-col gap-1.5 text-xs font-mono">
                        <div class="flex justify-between"><span class="text-slate-500">File</span><span class="truncate max-w-[140px]" title={metadata.filename}>{metadata.filename}</span></div>
                        {#if metadata.createdAt}
                            <div class="flex justify-between"><span class="text-slate-500">Created</span><span>{new Date(metadata.createdAt).toLocaleDateString()}</span></div>
                        {/if}
                        {#if metadata.modifiedAt}
                            <div class="flex justify-between"><span class="text-slate-500">Modified</span><span>{new Date(metadata.modifiedAt).toLocaleDateString()}</span></div>
                        {/if}
                    </div>
                </div>
            </Panel>
        </div>
    {/if}
</PageContent>
