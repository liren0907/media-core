<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Panel, RunButton, StatCard, EmptyState, ErrorAlert } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import type { AudioMetadata } from '$lib/types';

  let audioPath = $state('');
  let srtPath = $state('');
  let loading = $state(false);
  let error = $state('');
  let result = $state<AudioMetadata | null>(null);

  const audioFilters = [
    { name: 'Audio', extensions: ['wav', 'mp3', 'flac', 'm4a', 'aac', 'ogg', 'opus', 'aiff'] },
  ];
  const srtFilters = [{ name: 'Subtitles', extensions: ['srt'] }];

  function formatDuration(totalSeconds: number): string {
    const h = Math.floor(totalSeconds / 3600);
    const m = Math.floor((totalSeconds % 3600) / 60);
    const sec = (totalSeconds % 60).toFixed(1).replace(/\.0$/, '');
    if (h > 0) return `${h}h ${m}m ${sec}s`;
    if (m > 0) return `${m}m ${sec}s`;
    return `${sec}s`;
  }

  function channelLabel(channels: number): string {
    if (channels === 1) return 'Mono';
    if (channels === 2) return 'Stereo';
    return channels > 0 ? `${channels} ch` : 'Unknown';
  }

  async function analyze() {
    loading = true;
    error = '';
    try {
      result = await invoke<AudioMetadata>('get_audio_metadata', {
        audioPath,
        srtPath: srtPath || null,
      });
    } catch (e) {
      error = String(e);
      result = null;
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex flex-col gap-4">
  <Panel title="Analyze Audio" icon="info">
    {#snippet actions()}
      <RunButton {loading} disabled={!audioPath} label="Analyze" onclick={analyze} />
    {/snippet}
    <div class="p-4 flex flex-col gap-4">
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <FilePicker bind:value={audioPath} label="Audio File" filters={audioFilters} />
        <FilePicker bind:value={srtPath} label="SRT Subtitles (optional)" filters={srtFilters} />
      </div>
      {#if error}
        <ErrorAlert message={error} />
      {/if}
    </div>
  </Panel>

  {#if result}
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
      <StatCard label="Duration" icon="schedule" iconColor="text-[#137fec]" value={formatDuration(result.durationSeconds)} />
      <StatCard label="Sample Rate" icon="graphic_eq" iconColor="text-green-500" value={result.sampleRate ? `${(result.sampleRate / 1000).toFixed(1)} kHz` : 'Unknown'} />
      <StatCard label="Bitrate" icon="speed" iconColor="text-orange-500" value={result.bitrate ? `${result.bitrate} kbps` : 'Unknown'} />
      <StatCard label="Channels" icon="volume_up" iconColor="text-purple-500" value={channelLabel(result.channels)} />
    </div>

    {#if result.srtSegments != null}
      <Panel title="SRT Statistics" icon="description">
        <div class="p-4 grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div>
            <p class="text-stat-label">Segments</p>
            <p class="text-stat-value">{result.srtSegments}</p>
          </div>
          <div>
            <p class="text-stat-label">Speech Duration</p>
            <p class="text-stat-value">{formatDuration(result.srtSpeechDuration ?? 0)}</p>
          </div>
          <div>
            <p class="text-stat-label">Avg Segment</p>
            <p class="text-stat-value">{(result.srtAvgSegmentDuration ?? 0).toFixed(2)}s</p>
          </div>
        </div>
      </Panel>
    {/if}
  {:else if !error}
    <EmptyState icon="audio_file" message="Pick an audio file (and optional SRT) to inspect its properties" />
  {/if}
</div>
