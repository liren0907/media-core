<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { Panel, EmptyState, ErrorAlert, ToggleSwitch, RunButton } from '$lib/components/ui';
  import { onDedupHashProgress } from '$lib/events';
  import ProgressPanel from './ProgressPanel.svelte';
  import type { DedupSource, DedupHashProgressEvent } from '$lib/types';

  let sources = $state<DedupSource[]>([]);
  let selectedSourceId = $state('');
  let error = $state('');
  let isProcessing = $state(false);
  let progress = $state<DedupHashProgressEvent | null>(null);
  let lastResult = $state<number | null>(null);

  let useBlake3 = $state(true);
  let usePHash = $state(true);
  let useDHash = $state(false);

  function sourceIdStr(source: DedupSource): string {
    return source.id ?? '';
  }

  async function fetchSources() {
    try {
      sources = await invoke('get_scan_sources');
      if (sources.length > 0 && !selectedSourceId) {
        selectedSourceId = sourceIdStr(sources[0]);
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function cancelFingerprint() {
    if (!selectedSourceId) return;
    try {
      await invoke('cancel_dedup', { sourceId: selectedSourceId });
    } catch (e) {
      error = String(e);
    }
  }

  async function runFingerprint() {
    if (!selectedSourceId) return;

    const algorithms: string[] = [];
    if (useBlake3) algorithms.push('blake3');
    if (usePHash) algorithms.push('pHash');
    if (useDHash) algorithms.push('dHash');

    if (algorithms.length === 0) {
      error = 'Select at least one algorithm';
      return;
    }

    try {
      error = '';
      isProcessing = true;
      progress = null;
      lastResult = null;

      const count: number = await invoke('start_fingerprinting', {
        config: { sourceId: selectedSourceId, algorithms }
      });
      lastResult = count;
    } catch (e) {
      error = String(e);
    } finally {
      isProcessing = false;
      progress = null;
    }
  }

  onMount(() => {
    fetchSources();

    let unlisten: UnlistenFn | null = null;
    onDedupHashProgress((e) => { progress = e; }).then(fn => unlisten = fn);
    return () => { unlisten?.(); };
  });

  let progressPercent = $derived(
    progress ? (progress.total > 0 ? (progress.current / progress.total) * 100 : 0) : 0
  );
</script>

{#if error}
  <ErrorAlert message={error} />
{/if}

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
  <!-- Config -->
  <Panel title="Configuration" icon="settings">
    <div class="flex flex-col gap-4 p-1">
      <!-- Source selection -->
      <div class="flex flex-col gap-1">
        <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Source</label>
        <select bind:value={selectedSourceId} class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-3 py-2 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]">
          {#each sources as source}
            <option value={sourceIdStr(source)}>{source.label} ({source.fileCount} files)</option>
          {/each}
        </select>
      </div>

      <!-- Algorithm toggles -->
      <div class="flex flex-col gap-2">
        <label class="text-[10px] font-medium uppercase tracking-wider text-slate-500">Algorithms</label>
        <ToggleSwitch bind:checked={useBlake3} label="BLAKE3 (exact match)" />
        <ToggleSwitch bind:checked={usePHash} label="pHash (perceptual)" />
        <ToggleSwitch bind:checked={useDHash} label="dHash (difference)" />
      </div>

      <RunButton
        loading={isProcessing}
        disabled={!selectedSourceId}
        label="Run Fingerprint"
        onclick={runFingerprint}
      />
    </div>
  </Panel>

  <!-- Progress / Results -->
  <div class="lg:col-span-2">
    {#if isProcessing && progress}
      <ProgressPanel
        title="Computing Fingerprints"
        percent={progressPercent}
        message="{progress.current} / {progress.total} files"
        detail={progress.currentFile}
      />
      <div class="flex justify-end px-2 pb-2">
        <button onclick={cancelFingerprint} class="px-3 py-1 text-xs font-bold text-red-500 hover:text-red-600 hover:bg-red-500/10 rounded transition-colors">Cancel</button>
      </div>
    {:else if lastResult !== null}
      <Panel title="Complete" icon="check_circle">
        <div class="p-4 text-center">
          <span class="material-symbols-outlined text-[48px] text-green-500">check_circle</span>
          <p class="text-sm font-bold mt-2 text-slate-900 dark:text-white">Fingerprinting Complete</p>
          <p class="text-xs text-slate-500 mt-1">{lastResult} files processed</p>
        </div>
      </Panel>
    {:else}
      <Panel title="Results" icon="fingerprint">
        <EmptyState icon="fingerprint" message="Select a source and algorithms, then click Run" />
      </Panel>
    {/if}
  </div>
</div>
