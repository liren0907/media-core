<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { Panel, EmptyState, ErrorAlert } from '$lib/components/ui';
  import { selectDirectory } from '$lib/utils/file-dialog';
  import { getFileName, formatFileSize } from '$lib/utils/format';
  import { onDedupScanProgress } from '$lib/events';
  import DirectoryTree from './DirectoryTree.svelte';
  import type { DedupSource, DedupTreeNode, DedupScanProgressEvent, DedupScanResult } from '$lib/types';

  let sources = $state<DedupSource[]>([]);
  let selectedSourceId = $state<string | null>(null);
  let tree = $state<DedupTreeNode[]>([]);
  let error = $state('');
  let loading = $state(false);
  let scanningId = $state<string | null>(null);
  let scanProgress = $state<DedupScanProgressEvent | null>(null);
  let lastScanResult = $state<DedupScanResult | null>(null);

  function sourceIdStr(source: DedupSource): string {
    return source.id ?? '';
  }

  async function fetchSources() {
    try {
      sources = await invoke('get_scan_sources');
    } catch (e) {
      error = String(e);
    }
  }

  async function addSource() {
    const path = await selectDirectory();
    if (!path) return;

    try {
      error = '';
      const dirName = path.split('/').pop() || path;
      await invoke('add_scan_source', { path, label: dirName });
      await fetchSources();
    } catch (e) {
      error = String(e);
    }
  }

  async function scanSource(source: DedupSource) {
    const id = sourceIdStr(source);
    if (!id) return;

    try {
      error = '';
      scanningId = id;
      lastScanResult = null;
      const result: DedupScanResult = await invoke('start_scan', { sourceId: id });
      lastScanResult = result;
      await fetchSources();
      // Refresh directory tree if this source is currently selected
      if (selectedSourceId === id) {
        tree = await invoke('get_source_tree', { sourceId: id });
      }
    } catch (e) {
      error = String(e);
    } finally {
      scanningId = null;
      scanProgress = null;
    }
  }

  async function cancelScan() {
    if (!scanningId) return;
    try {
      await invoke('cancel_dedup', { sourceId: scanningId });
    } catch (e) {
      error = String(e);
    }
  }

  async function removeSource(source: DedupSource) {
    const id = sourceIdStr(source);
    if (!id) return;
    if (!confirm(`Remove "${source.label}" and all its data?`)) return;

    try {
      error = '';
      await invoke('delete_scan_source', { sourceId: id });
      if (selectedSourceId === id) {
        selectedSourceId = null;
        tree = [];
      }
      await fetchSources();
    } catch (e) {
      error = String(e);
    }
  }

  async function selectSource(source: DedupSource) {
    const id = sourceIdStr(source);
    selectedSourceId = id;
    try {
      tree = await invoke('get_source_tree', { sourceId: id });
    } catch (e) {
      tree = [];
    }
  }

  onMount(() => {
    fetchSources();

    let unlisten: UnlistenFn | null = null;
    onDedupScanProgress((e) => { scanProgress = e; }).then(fn => unlisten = fn);
    return () => { unlisten?.(); };
  });
</script>

{#if error}
  <ErrorAlert message={error} />
{/if}

<div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
  <!-- Sources List -->
  <Panel title="Sources" icon="folder">
    {#snippet actions()}
      <button onclick={addSource} class="flex items-center gap-1 text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">
        <span class="material-symbols-outlined text-[14px]">add</span> Add Directory
      </button>
    {/snippet}

    {#if sources.length === 0}
      <EmptyState icon="folder_off" message="No sources added yet" />
    {:else}
      <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
        {#each sources as source}
          {@const id = sourceIdStr(source)}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            onclick={() => selectSource(source)}
            class="flex items-center justify-between px-3 py-2 text-left hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors cursor-pointer {selectedSourceId === id ? 'bg-[#137fec]/5 border-l-2 border-[#137fec]' : ''}"
          >
            <div class="flex-1 min-w-0">
              <div class="text-xs font-bold text-slate-900 dark:text-white truncate">{source.label}</div>
              <div class="text-[10px] text-slate-500 font-mono truncate">{source.path}</div>
              <div class="flex items-center gap-2 mt-1">
                <span class="text-[10px] px-1.5 py-0.5 rounded {source.status === 'hashed' ? 'bg-green-500/10 text-green-600' : source.status === 'scanned' ? 'bg-blue-500/10 text-blue-600' : 'bg-slate-100 dark:bg-[#283039] text-slate-500'} font-bold">
                  {source.status}
                </span>
                {#if scanningId === id && scanProgress}
                  <span class="text-[10px] text-[#137fec] font-mono animate-pulse">{scanProgress.filesFound} files found...</span>
                  <button onclick={(e: MouseEvent) => { e.stopPropagation(); cancelScan(); }} class="text-[10px] text-red-500 hover:text-red-600 font-bold">Cancel</button>
                {:else if source.fileCount > 0}
                  <span class="text-[10px] text-slate-500">{source.fileCount} files</span>
                {/if}
              </div>
            </div>
            <div class="flex items-center gap-1 ml-2 shrink-0">
              <button
                onclick={(e: MouseEvent) => { e.stopPropagation(); scanSource(source); }}
                class="p-1 rounded text-slate-400 hover:text-[#137fec] hover:bg-[#137fec]/10 transition-colors"
                title={source.status === 'pending' ? 'Scan' : 'Rescan'}
                disabled={scanningId === id}
              >
                <span class="material-symbols-outlined text-[16px]">{scanningId === id ? 'hourglass_top' : source.status === 'pending' ? 'search' : 'sync'}</span>
              </button>
              <button
                onclick={(e: MouseEvent) => { e.stopPropagation(); removeSource(source); }}
                class="p-1 rounded text-slate-400 hover:text-red-500 hover:bg-red-500/10 transition-colors"
                title="Remove"
              >
                <span class="material-symbols-outlined text-[16px]">delete</span>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </Panel>

  <!-- Directory Tree -->
  <div class="lg:col-span-2">
    {#if lastScanResult}
      <div class="mb-3 flex items-center gap-3 px-3 py-2 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#111418]">
        <span class="material-symbols-outlined text-[16px] text-green-500">check_circle</span>
        <span class="text-xs text-slate-700 dark:text-slate-300">Scan complete:</span>
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-green-500/10 text-green-600">+{lastScanResult.added} added</span>
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-red-500/10 text-red-600">-{lastScanResult.removed} removed</span>
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600">{lastScanResult.updated} changed</span>
        <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-500">{lastScanResult.unchanged} unchanged</span>
        <button onclick={() => lastScanResult = null} class="ml-auto text-slate-400 hover:text-slate-600">
          <span class="material-symbols-outlined text-[14px]">close</span>
        </button>
      </div>
    {/if}

    <Panel title="Directory Tree" icon="account_tree">
      {#if !selectedSourceId}
        <EmptyState icon="account_tree" message="Select a source to browse its file tree" />
      {:else if tree.length === 0}
        <EmptyState icon="folder_open" message="No files found. Scan the source first." />
      {:else}
        <div class="max-h-[500px] overflow-y-auto">
          <DirectoryTree nodes={tree} />
        </div>
      {/if}
    </Panel>
  </div>
</div>
