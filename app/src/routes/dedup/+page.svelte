<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { PageContent, Panel, EmptyState, ErrorAlert } from '$lib/components/ui';
  import { selectDirectory, selectFiles } from '$lib/utils/file-dialog';
  import { onDedupScanProgress } from '$lib/events';
  import DirectoryTree from '$lib/components/features/dedup/DirectoryTree.svelte';
  import ActionBar from '$lib/components/features/dedup/ActionBar.svelte';
  import ComparisonResults from '$lib/components/features/dedup/ComparisonResults.svelte';
  import type {
    DedupSource, DedupTreeNode, DedupMediaFile,
    DedupScanProgressEvent, DedupScanResult,
    DedupGroupExpanded, DedupStats
  } from '$lib/types';

  // Sources state
  let sources = $state<DedupSource[]>([]);
  let selectedSourceId = $state<string | null>(null);
  let error = $state('');

  // Tree state
  let tree = $state<DedupTreeNode[]>([]);
  let filesMap = $state<Record<string, DedupMediaFile>>({});
  let previewFile = $state<string | null>(null);

  // Scan state
  let scanningId = $state<string | null>(null);
  let scanProgress = $state<DedupScanProgressEvent | null>(null);
  let lastScanResult = $state<DedupScanResult | null>(null);

  // Compare state
  let groups = $state<DedupGroupExpanded[]>([]);

  // Stats
  let stats = $state<DedupStats | null>(null);

  function sourceIdStr(source: DedupSource): string {
    return source.id ?? '';
  }

  let selectedSource = $derived(
    sources.find(s => sourceIdStr(s) === selectedSourceId) ?? null
  );

  // Role-based derived state
  let sourceEntry = $derived(sources.find(s => s.role === 'source') ?? null);
  let targetEntries = $derived(sources.filter(s => s.role === 'target'));

  // ── Data loading ──

  async function fetchSources() {
    try {
      sources = await invoke('get_scan_sources');
    } catch (e) {
      error = String(e);
    }
  }

  async function fetchStats() {
    try {
      stats = await invoke('get_dedup_stats');
    } catch (_) {}
  }

  async function loadSourceData(sourceId: string) {
    try {
      tree = await invoke('get_source_tree', { sourceId });
      const files: DedupMediaFile[] = await invoke('get_files_by_source', { sourceId });
      filesMap = Object.fromEntries(files.map(f => [f.filePath, f]));
      // Load comparison results from the source-role entry
      if (sourceEntry?.id) {
        groups = await invoke('get_duplicate_groups', { sourceId: sourceEntry.id });
      } else {
        groups = [];
      }
    } catch (e) {
      tree = [];
      filesMap = {};
      groups = [];
    }
  }

  // ── Actions ──

  async function addAndScanSource(role: 'source' | 'target' = 'target') {
    const path = await selectDirectory();
    if (!path) return;

    try {
      error = '';

      // Check if this path already exists
      const existing = sources.find(s => s.path === path);
      if (existing) {
        const id = sourceIdStr(existing);
        // If role differs, update it
        if (existing.role !== role) {
          await invoke('set_source_role', { sourceId: id, role });
          await fetchSources();
        }
        selectedSourceId = id;
        await scanSource(id);
        return;
      }

      const dirName = path.split('/').pop() || path;
      const source: DedupSource = await invoke('add_scan_source', { path, label: dirName, role });
      await fetchSources();

      const id = sourceIdStr(source);
      selectedSourceId = id;
      await scanSource(id);
    } catch (e) {
      error = String(e);
    }
  }

  async function setRole(source: DedupSource, role: 'source' | 'target') {
    const id = sourceIdStr(source);
    if (!id) return;
    try {
      error = '';
      await invoke('set_source_role', { sourceId: id, role });
      groups = [];
      await fetchSources();
      await fetchStats();
    } catch (e) {
      error = String(e);
    }
  }

  async function scanSource(id: string) {
    try {
      error = '';
      scanningId = id;
      lastScanResult = null;
      const result: DedupScanResult = await invoke('start_scan', { sourceId: id });
      lastScanResult = result;
      await fetchSources();
      await fetchStats();
      if (selectedSourceId === id) {
        await loadSourceData(id);
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
    const ok = await ask(
      `Remove "${source.label}" and all its data?`,
      {
        title: 'Remove Scan Source',
        kind: 'error',
        okLabel: 'Remove',
        cancelLabel: 'Cancel',
      }
    );
    if (!ok) return;

    try {
      error = '';
      await invoke('delete_scan_source', { sourceId: id });
      if (selectedSourceId === id) {
        selectedSourceId = null;
        tree = [];
        filesMap = {};
        groups = [];
      }
      await fetchSources();
      await fetchStats();
    } catch (e) {
      error = String(e);
    }
  }

  function selectSource(source: DedupSource) {
    const id = sourceIdStr(source);
    selectedSourceId = id;
    previewFile = null;
    lastScanResult = null;
    loadSourceData(id);
  }

  function handleFingerprintDone() {
    if (selectedSourceId) {
      loadSourceData(selectedSourceId);
      fetchStats();
    }
  }

  function handleCompareDone(result: DedupGroupExpanded[]) {
    groups = result;
    fetchStats();
  }

  function handleTrashDone() {
    fetchSources();
    fetchStats();
  }

  async function addFilesAsTarget() {
    const paths = await selectFiles([
      { name: 'Media Files', extensions: ['jpg','jpeg','png','gif','bmp','webp','tiff','tif','heic','heif','avif','mp4','mkv','avi','mov','wmv','flv','webm','m4v','mpg','mpeg','3gp','ts','mts'] }
    ]);
    if (!paths || paths.length === 0) return;

    try {
      error = '';
      const source: DedupSource = await invoke('add_files_as_target', { filePaths: paths });
      await fetchSources();
      await fetchStats();
      const id = sourceIdStr(source);
      selectedSourceId = id;
      await loadSourceData(id);
    } catch (e) {
      error = String(e);
    }
  }

  function togglePreview(path: string) {
    previewFile = previewFile === path ? null : path;
  }

  // ── Lifecycle ──

  onMount(() => {
    fetchSources();
    fetchStats();

    let unlisten: UnlistenFn | null = null;
    onDedupScanProgress((e) => { scanProgress = e; }).then(fn => unlisten = fn);
    return () => { unlisten?.(); };
  });

  // File counts for selected source
  let imageCount = $derived(Object.values(filesMap).filter(f => f.fileType === 'image').length);
  let videoCount = $derived(Object.values(filesMap).filter(f => f.fileType === 'video').length);
  let hashedCount = $derived(Object.values(filesMap).filter(f => f.contentHash || f.phash || f.dhash).length);
</script>

<svelte:head>
  <title>Deduplication</title>
</svelte:head>

<PageContent>
  {#if error}
    <ErrorAlert message={error} />
  {/if}

  <div class="grid grid-cols-[280px_1fr] gap-3 h-full">
    <!-- Left column: Source + Targets + Stats -->
    <div class="flex flex-col gap-3">
      <!-- Source (master) section -->
      <Panel title="Source" icon="star">
        {#snippet actions()}
          {#if !sourceEntry}
            <button onclick={() => addAndScanSource('source')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <span class="material-symbols-outlined text-[14px]">add</span> Set
            </button>
          {/if}
        {/snippet}

        {#if !sourceEntry}
          <EmptyState icon="star" message="Set a master directory as Source" />
        {:else}
          {@const id = sourceIdStr(sourceEntry)}
          <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
          <div
            onclick={() => selectSource(sourceEntry)}
            class="px-3 py-2 border-l-2 border-[#137fec] bg-[#137fec]/5 hover:bg-[#137fec]/10 transition-colors cursor-pointer"
          >
            <div class="flex items-center gap-1.5">
              <span class="material-symbols-outlined text-[14px] text-[#137fec]">star</span>
              <span class="text-card-title truncate">{sourceEntry.label}</span>
            </div>
            <div class="flex items-center gap-2 mt-0.5 ml-5">
              <span class="text-[10px] px-1.5 py-0.5 rounded {sourceEntry.status === 'hashed' ? 'bg-green-500/10 text-green-600' : sourceEntry.status === 'scanned' ? 'bg-blue-500/10 text-blue-600' : 'bg-slate-100 dark:bg-[#283039] text-slate-500'} font-bold">
                {sourceEntry.status}
              </span>
              {#if scanningId === id && scanProgress}
                <span class="text-[10px] text-[#137fec] font-mono animate-pulse">{scanProgress.filesFound} files...</span>
                <button onclick={(e: MouseEvent) => { e.stopPropagation(); cancelScan(); }} class="text-[10px] text-red-500 font-bold">Stop</button>
              {:else if sourceEntry.fileCount > 0}
                <span class="text-[10px] text-slate-500">{sourceEntry.fileCount} files</span>
              {/if}
            </div>
            <div class="flex items-center gap-0.5 mt-1 ml-4">
              <button
                onclick={(e: MouseEvent) => { e.stopPropagation(); scanSource(id); }}
                class="p-1 rounded text-slate-400 hover:text-[#137fec] hover:bg-[#137fec]/10 transition-colors"
                title={sourceEntry.status === 'pending' ? 'Scan' : 'Rescan'}
                disabled={scanningId === id}
              >
                <span class="material-symbols-outlined text-[14px]">{scanningId === id ? 'hourglass_top' : sourceEntry.status === 'pending' ? 'search' : 'sync'}</span>
              </button>
              <button
                onclick={(e: MouseEvent) => { e.stopPropagation(); setRole(sourceEntry, 'target'); }}
                class="p-1 rounded text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-colors"
                title="Demote to Target"
              >
                <span class="material-symbols-outlined text-[14px]">arrow_downward</span>
              </button>
              <button
                onclick={(e: MouseEvent) => { e.stopPropagation(); removeSource(sourceEntry); }}
                class="p-1 rounded text-slate-400 hover:text-red-500 hover:bg-red-500/10 transition-colors"
                title="Remove"
              >
                <span class="material-symbols-outlined text-[14px]">delete</span>
              </button>
            </div>
          </div>
        {/if}
      </Panel>

      <!-- Targets section -->
      <Panel title="Targets" icon="filter_center_focus">
        {#snippet actions()}
          <div class="flex items-center gap-2">
            <button onclick={() => addAndScanSource('target')} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <span class="material-symbols-outlined text-[14px]">create_new_folder</span> Folder
            </button>
            <button onclick={addFilesAsTarget} class="flex items-center gap-1 text-stat-label text-status-info hover:text-blue-400">
              <span class="material-symbols-outlined text-[14px]">note_add</span> Files
            </button>
          </div>
        {/snippet}

        {#if targetEntries.length === 0}
          <EmptyState icon="folder_off" message="No targets added" />
        {:else}
          <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
            {#each targetEntries as source}
              {@const id = sourceIdStr(source)}
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
              <div
                onclick={() => selectSource(source)}
                class="flex items-center justify-between px-3 py-2 hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors cursor-pointer {selectedSourceId === id ? 'bg-[#137fec]/5 border-l-2 border-[#137fec]' : ''}"
              >
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-1.5">
                    <span class="material-symbols-outlined text-[14px] text-slate-400">filter_center_focus</span>
                    <span class="text-card-title truncate">{source.label}</span>
                  </div>
                  <div class="flex items-center gap-2 mt-0.5 ml-5">
                    <span class="text-[10px] px-1.5 py-0.5 rounded {source.status === 'hashed' ? 'bg-green-500/10 text-green-600' : source.status === 'scanned' ? 'bg-blue-500/10 text-blue-600' : 'bg-slate-100 dark:bg-[#283039] text-slate-500'} font-bold">
                      {source.status}
                    </span>
                    {#if scanningId === id && scanProgress}
                      <span class="text-[10px] text-[#137fec] font-mono animate-pulse">{scanProgress.filesFound} files...</span>
                      <button onclick={(e: MouseEvent) => { e.stopPropagation(); cancelScan(); }} class="text-[10px] text-red-500 font-bold">Stop</button>
                    {:else if source.fileCount > 0}
                      <span class="text-[10px] text-slate-500">{source.fileCount} files</span>
                    {/if}
                  </div>
                </div>
                <div class="flex items-center gap-0.5 ml-1 shrink-0">
                  <button
                    onclick={(e: MouseEvent) => { e.stopPropagation(); scanSource(id); }}
                    class="p-1 rounded text-slate-400 hover:text-[#137fec] hover:bg-[#137fec]/10 transition-colors"
                    title={source.status === 'pending' ? 'Scan' : 'Rescan'}
                    disabled={scanningId === id}
                  >
                    <span class="material-symbols-outlined text-[14px]">{scanningId === id ? 'hourglass_top' : source.status === 'pending' ? 'search' : 'sync'}</span>
                  </button>
                  <button
                    onclick={(e: MouseEvent) => { e.stopPropagation(); setRole(source, 'source'); }}
                    class="p-1 rounded text-slate-400 hover:text-amber-500 hover:bg-amber-500/10 transition-colors"
                    title="Promote to Source"
                  >
                    <span class="material-symbols-outlined text-[14px]">arrow_upward</span>
                  </button>
                  <button
                    onclick={(e: MouseEvent) => { e.stopPropagation(); removeSource(source); }}
                    class="p-1 rounded text-slate-400 hover:text-red-500 hover:bg-red-500/10 transition-colors"
                    title="Remove"
                  >
                    <span class="material-symbols-outlined text-[14px]">delete</span>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </Panel>

      <!-- Stats summary -->
      {#if stats}
        <div class="px-3 py-2 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#111418]">
          <div class="text-stat-label mb-1.5">Overview</div>
          <div class="grid grid-cols-2 gap-1.5 text-[11px]">
            <div class="flex justify-between">
              <span class="text-slate-500">Source</span>
              <span class="font-bold text-slate-700 dark:text-white">{sourceEntry ? 1 : 0}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-slate-500">Targets</span>
              <span class="font-bold text-slate-700 dark:text-white">{targetEntries.length}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-slate-500">Files</span>
              <span class="font-bold text-slate-700 dark:text-white">{stats.totalFiles}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-slate-500">Dupes</span>
              <span class="font-bold text-orange-500">{stats.totalDuplicateGroups}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Right column: Tree + Actions + Results -->
    <div class="flex flex-col gap-3">
      {#if !selectedSourceId}
        <Panel title="Getting Started" icon="info">
          <EmptyState icon="folder_open" message="Click '+ Add' to open a directory and start scanning" />
        </Panel>
      {:else}
        <!-- Scan result summary -->
        {#if lastScanResult}
          <div class="flex items-center gap-3 px-3 py-2 rounded-lg border border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#111418]">
            <span class="material-symbols-outlined text-[16px] text-green-500">check_circle</span>
            <span class="text-body">Scan complete:</span>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-green-500/10 text-green-600">+{lastScanResult.added}</span>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-red-500/10 text-red-600">-{lastScanResult.removed}</span>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600">{lastScanResult.updated} changed</span>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-500">{lastScanResult.unchanged} same</span>
            <button onclick={() => lastScanResult = null} class="ml-auto text-slate-400 hover:text-slate-600">
              <span class="material-symbols-outlined text-[14px]">close</span>
            </button>
          </div>
        {/if}

        <!-- Directory Tree -->
        <Panel title="Files" icon="account_tree">
          {#snippet actions()}
            <div class="flex items-center gap-2 text-[10px] text-slate-500">
              {#if Object.keys(filesMap).length > 0}
                <span>{imageCount} images</span>
                <span>{videoCount} videos</span>
                <span class="text-slate-300 dark:text-slate-600">|</span>
                <span>{hashedCount}/{Object.keys(filesMap).length} hashed</span>
              {/if}
            </div>
          {/snippet}

          {#if tree.length === 0}
            <EmptyState icon="folder_open" message={scanningId ? 'Scanning...' : 'No files found. Click scan to start.'} />
          {:else}
            <div class="max-h-[400px] overflow-y-auto">
              <DirectoryTree nodes={tree} {filesMap} {previewFile} onPreviewToggle={togglePreview} />
            </div>
          {/if}
        </Panel>

        <!-- Action Bar -->
        <ActionBar
          sourceId={selectedSourceId}
          hasSourceRole={!!sourceEntry}
          onFingerprintDone={handleFingerprintDone}
          onCompareDone={handleCompareDone}
        />

      {/if}
    </div>
  </div>

  <!-- Comparison Results: full width, outside the grid -->
  {#if groups.length > 0}
    <ComparisonResults bind:groups onTrashDone={handleTrashDone} />
  {/if}
</PageContent>
