<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { Panel, EmptyState, ErrorAlert, RunButton, ToggleSwitch } from '$lib/components/ui';
  import { onDedupCompareProgress } from '$lib/events';
  import ProgressPanel from './ProgressPanel.svelte';
  import DuplicateGroupCard from './DuplicateGroupCard.svelte';
  import type { DedupSource, DedupGroupExpanded, DedupCompareProgressEvent } from '$lib/types';

  let sources = $state<DedupSource[]>([]);
  let selectedSourceId = $state('');
  let groups = $state<DedupGroupExpanded[]>([]);
  let error = $state('');
  let isProcessing = $state(false);
  let progress = $state<DedupCompareProgressEvent | null>(null);
  let filter = $state<'all' | 'exact' | 'similar'>('all');
  let includeSelf = $state(true);
  let threshold = $state(10);

  function sourceIdStr(source: DedupSource): string {
    return source.id ?? '';
  }

  let filteredGroups = $derived(
    groups.filter(g => filter === 'all' || g.matchType === filter)
  );

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

  async function cancelCompare() {
    if (!selectedSourceId) return;
    try {
      await invoke('cancel_dedup', { sourceId: selectedSourceId });
    } catch (e) {
      error = String(e);
    }
  }

  async function runCompare() {
    if (!selectedSourceId) return;

    try {
      error = '';
      isProcessing = true;
      progress = null;
      groups = [];

      const result: DedupGroupExpanded[] = await invoke('compare_source', { sourceId: selectedSourceId, includeSelf, threshold });
      groups = result;
    } catch (e) {
      error = String(e);
    } finally {
      isProcessing = false;
      progress = null;
    }
  }

  async function loadExistingGroups() {
    if (!selectedSourceId) return;
    try {
      groups = await invoke('get_duplicate_groups', { sourceId: selectedSourceId });
    } catch (_) {
      groups = [];
    }
  }

  onMount(() => {
    fetchSources();

    let unlisten: UnlistenFn | null = null;
    onDedupCompareProgress((e) => { progress = e; }).then(fn => unlisten = fn);
    return () => { unlisten?.(); };
  });

  $effect(() => {
    if (selectedSourceId) {
      loadExistingGroups();
    }
  });

  let progressPercent = $derived(
    progress ? (progress.total > 0 ? (progress.current / progress.total) * 100 : 0) : 0
  );
</script>

{#if error}
  <ErrorAlert message={error} />
{/if}

<!-- Controls bar -->
<Panel title="Compare" icon="compare">
  {#snippet actions()}
    <div class="flex items-center gap-3">
      <select bind:value={selectedSourceId} onchange={() => loadExistingGroups()} class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-2 py-1 text-[10px] text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec]">
        {#each sources as source}
          <option value={sourceIdStr(source)}>{source.label}</option>
        {/each}
      </select>

      <div class="flex gap-1">
        {#each ['all', 'exact', 'similar'] as f}
          <button
            onclick={() => filter = f as 'all' | 'exact' | 'similar'}
            class="px-2 py-0.5 rounded text-[10px] font-bold transition-colors {filter === f ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#283039] text-slate-500 hover:text-slate-700 dark:hover:text-white'}"
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        {/each}
      </div>

      <ToggleSwitch bind:checked={includeSelf} label="Within source" />

      <div class="flex items-center gap-1.5">
        <label class="text-[10px] text-slate-500 whitespace-nowrap">Threshold</label>
        <input type="range" min="0" max="64" bind:value={threshold} class="w-16 h-1 accent-[#137fec]" />
        <span class="text-[10px] font-mono text-slate-500 w-8 text-right">{threshold}</span>
      </div>

      <RunButton
        loading={isProcessing}
        disabled={!selectedSourceId}
        label="Run Compare"
        onclick={runCompare}
      />
    </div>
  {/snippet}

  {#if isProcessing && progress}
    <div class="p-2">
      <ProgressPanel
        title="Comparing"
        percent={progressPercent}
        message="{progress.current} / {progress.total} files checked"
      />
      <div class="flex justify-end mt-1">
        <button onclick={cancelCompare} class="px-3 py-1 text-xs font-bold text-red-500 hover:text-red-600 hover:bg-red-500/10 rounded transition-colors">Cancel</button>
      </div>
    </div>
  {:else if filteredGroups.length > 0}
    <div class="flex flex-col gap-2 p-1">
      <p class="text-[10px] text-slate-500">{filteredGroups.length} duplicate group(s) found</p>
      {#each filteredGroups as group (group.id)}
        <DuplicateGroupCard {group} />
      {/each}
    </div>
  {:else if groups.length === 0}
    <EmptyState icon="compare" message="Select a source and run comparison to find duplicates" />
  {:else}
    <EmptyState icon="filter_list" message="No matches for current filter" />
  {/if}
</Panel>
