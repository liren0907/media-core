<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import { ToggleSwitch, RunButton, Panel } from '$lib/components/ui';
  import { onDedupHashProgress, onDedupCompareProgress } from '$lib/events';
  import ProgressPanel from './ProgressPanel.svelte';
  import type { DedupHashProgressEvent, DedupCompareProgressEvent, DedupGroupExpanded } from '$lib/types';

  interface Props {
    sourceId: string;
    onFingerprintDone?: () => void;
    onCompareDone?: (groups: DedupGroupExpanded[]) => void;
  }

  let { sourceId, onFingerprintDone, onCompareDone }: Props = $props();

  // Algorithm config
  let useBlake3 = $state(true);
  let usePHash = $state(true);
  let useDHash = $state(false);

  // Compare config
  let includeSelf = $state(true);
  let threshold = $state(10);

  // Processing state
  let isFingerprintRunning = $state(false);
  let isCompareRunning = $state(false);
  let hashProgress = $state<DedupHashProgressEvent | null>(null);
  let compareProgress = $state<DedupCompareProgressEvent | null>(null);

  let hashPercent = $derived(
    hashProgress ? (hashProgress.total > 0 ? (hashProgress.current / hashProgress.total) * 100 : 0) : 0
  );
  let comparePercent = $derived(
    compareProgress ? (compareProgress.total > 0 ? (compareProgress.current / compareProgress.total) * 100 : 0) : 0
  );

  let isProcessing = $derived(isFingerprintRunning || isCompareRunning);

  async function runFingerprint() {
    if (!sourceId) return;

    const algorithms: string[] = [];
    if (useBlake3) algorithms.push('blake3');
    if (usePHash) algorithms.push('pHash');
    if (useDHash) algorithms.push('dHash');

    if (algorithms.length === 0) return;

    try {
      isFingerprintRunning = true;
      hashProgress = null;
      await invoke('start_fingerprinting', {
        config: { sourceId, algorithms }
      });
      onFingerprintDone?.();
    } catch (e) {
      console.error('Fingerprint error:', e);
    } finally {
      isFingerprintRunning = false;
      hashProgress = null;
    }
  }

  async function runCompare() {
    if (!sourceId) return;

    try {
      isCompareRunning = true;
      compareProgress = null;
      const algorithms: string[] = [];
      if (useBlake3) algorithms.push('blake3');
      if (usePHash) algorithms.push('pHash');
      if (useDHash) algorithms.push('dHash');

      const result: DedupGroupExpanded[] = await invoke('compare_source', {
        sourceId, includeSelf, threshold, algorithms
      });
      onCompareDone?.(result);
    } catch (e) {
      console.error('Compare error:', e);
    } finally {
      isCompareRunning = false;
      compareProgress = null;
    }
  }

  async function cancel() {
    if (!sourceId) return;
    await invoke('cancel_dedup', { sourceId });
  }

  onMount(() => {
    let unlistenHash: UnlistenFn | null = null;
    let unlistenCompare: UnlistenFn | null = null;

    onDedupHashProgress((e) => { hashProgress = e; }).then(fn => unlistenHash = fn);
    onDedupCompareProgress((e) => { compareProgress = e; }).then(fn => unlistenCompare = fn);

    return () => {
      unlistenHash?.();
      unlistenCompare?.();
    };
  });
</script>

<Panel title="Actions" icon="play_circle">
  {#if isFingerprintRunning && hashProgress}
    <div class="p-2">
      <ProgressPanel
        title="Computing Fingerprints"
        percent={hashPercent}
        message="{hashProgress.current} / {hashProgress.total} files"
        detail={hashProgress.currentFile}
      />
      <div class="flex justify-end mt-1">
        <button onclick={cancel} class="px-3 py-1 text-xs font-bold text-red-500 hover:text-red-600 hover:bg-red-500/10 rounded transition-colors">Cancel</button>
      </div>
    </div>
  {:else if isCompareRunning && compareProgress}
    <div class="p-2">
      <ProgressPanel
        title="Comparing"
        percent={comparePercent}
        message="{compareProgress.current} / {compareProgress.total} files checked"
      />
      <div class="flex justify-end mt-1">
        <button onclick={cancel} class="px-3 py-1 text-xs font-bold text-red-500 hover:text-red-600 hover:bg-red-500/10 rounded transition-colors">Cancel</button>
      </div>
    </div>
  {:else}
    <div class="flex flex-wrap items-center gap-3 p-2">
      <!-- Algorithm toggles -->
      <div class="flex items-center gap-2 border-r border-slate-200 dark:border-[#2a3441] pr-3">
        <ToggleSwitch bind:checked={useBlake3} label="BLAKE3" />
        <ToggleSwitch bind:checked={usePHash} label="pHash" />
        <ToggleSwitch bind:checked={useDHash} label="dHash" />
      </div>

      <!-- Compare config -->
      <div class="flex items-center gap-2 border-r border-slate-200 dark:border-[#2a3441] pr-3">
        <ToggleSwitch bind:checked={includeSelf} label="Within source" />
        <div class="flex items-center gap-1">
          <label class="text-[10px] text-slate-500 whitespace-nowrap">Threshold</label>
          <input type="range" min="0" max="64" bind:value={threshold} class="w-14 h-1 accent-[#137fec]" />
          <span class="text-[10px] font-mono text-slate-500 w-6 text-right">{threshold}</span>
        </div>
      </div>

      <!-- Action buttons -->
      <div class="flex items-center gap-2">
        <RunButton
          loading={isFingerprintRunning}
          disabled={isProcessing || !sourceId}
          label="Fingerprint"
          onclick={runFingerprint}
        />
        <RunButton
          loading={isCompareRunning}
          disabled={isProcessing || !sourceId}
          label="Compare"
          onclick={runCompare}
        />
      </div>
    </div>
  {/if}
</Panel>
