<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { Panel, EmptyState } from '$lib/components/ui';
  import { formatFileSize } from '$lib/utils/format';
  import FilePreview from './FilePreview.svelte';
  import type { DedupGroupExpanded, DedupMediaFile, TrashResult } from '$lib/types';

  interface Props {
    groups: DedupGroupExpanded[];
    onTrashDone?: () => void;
  }

  let { groups = $bindable(), onTrashDone }: Props = $props();

  let filter = $state<'all' | 'exact' | 'similar'>('all');
  let expandedFile = $state<string | null>(null);
  let selectedTargetIds = $state<Set<string>>(new Set());
  let isTrashRunning = $state(false);
  let trashMessage = $state('');

  let filteredGroups = $derived(
    groups.filter(g => filter === 'all' || g.matchType === filter)
  );

  // Check if groups use the new source/target pair format
  let hasPairLayout = $derived(groups.some(g => g.sourceMember && g.targetMember));

  // All selectable target IDs from current filtered view
  let allTargetIds = $derived(
    filteredGroups
      .filter(g => g.targetMember?.id)
      .map(g => g.targetMember!.id!)
  );

  let allSelected = $derived(
    allTargetIds.length > 0 && allTargetIds.every(id => selectedTargetIds.has(id))
  );

  // Selected files info
  let selectedFiles = $derived(
    filteredGroups
      .filter(g => g.targetMember?.id && selectedTargetIds.has(g.targetMember!.id!))
      .map(g => g.targetMember!)
  );
  let selectedTotalSize = $derived(
    selectedFiles.reduce((sum, f) => sum + f.fileSize, 0)
  );

  function togglePreview(filePath: string) {
    expandedFile = expandedFile === filePath ? null : filePath;
  }

  function toggleSelect(fileId: string) {
    const next = new Set(selectedTargetIds);
    if (next.has(fileId)) {
      next.delete(fileId);
    } else {
      next.add(fileId);
    }
    selectedTargetIds = next;
  }

  function toggleSelectAll() {
    if (allSelected) {
      selectedTargetIds = new Set();
    } else {
      selectedTargetIds = new Set(allTargetIds);
    }
  }

  async function trashSelected() {
    if (selectedTargetIds.size === 0) return;
    const ok = await ask(
      `Move ${selectedTargetIds.size} file(s) to Trash? This can be undone from the system Trash.`,
      {
        title: 'Move to Trash',
        kind: 'warning',
        okLabel: 'Move to Trash',
        cancelLabel: 'Cancel',
      }
    );
    if (!ok) return;

    try {
      isTrashRunning = true;
      trashMessage = '';
      const fileIds = Array.from(selectedTargetIds);
      const result: TrashResult = await invoke('trash_files', { fileIds });

      // Remove trashed groups from the list
      const trashedIds = new Set(fileIds);
      groups = groups.filter(g => {
        if (g.targetMember?.id && trashedIds.has(g.targetMember.id)) return false;
        return true;
      });

      selectedTargetIds = new Set();

      if (result.failed > 0) {
        trashMessage = `Moved ${result.trashed} file(s) to Trash. ${result.failed} failed.`;
      } else {
        trashMessage = `Moved ${result.trashed} file(s) to Trash.`;
      }

      onTrashDone?.();
    } catch (e) {
      trashMessage = `Error: ${String(e)}`;
    } finally {
      isTrashRunning = false;
    }
  }

  let similarityPercent = (score: number) => (score * 100).toFixed(0);
</script>

{#snippet fileCard(file: DedupMediaFile)}
  {@const filePath = file.filePath}
  <div class="flex flex-col">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      onclick={() => togglePreview(filePath)}
      class="flex items-center gap-2 p-2 rounded border border-slate-100 dark:border-[#2a3441] hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 cursor-pointer transition-colors"
    >
      <div class="flex items-center justify-center size-10 rounded bg-slate-100 dark:bg-[#283039] shrink-0 overflow-hidden">
        {#if file.fileType === 'image'}
          <img src={convertFileSrc(file.filePath)} class="size-10 rounded object-cover" alt="" />
        {:else}
          <span class="material-symbols-outlined text-[18px] text-slate-400">
            {file.fileType === 'video' ? 'movie' : 'description'}
          </span>
        {/if}
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[11px] font-bold text-slate-900 dark:text-white truncate">{file.fileName}</div>
        <div class="text-meta truncate">{file.filePath}</div>
        <div class="text-[10px] text-slate-400 mt-0.5">{formatFileSize(file.fileSize)}</div>
      </div>
      <span class="material-symbols-outlined text-[14px] text-slate-400 shrink-0">
        {expandedFile === filePath ? 'expand_less' : 'visibility'}
      </span>
    </div>
    {#if expandedFile === filePath}
      <FilePreview
        filePath={file.filePath}
        fileType={file.fileType}
        fileName={file.fileName}
        fileSize={file.fileSize}
      />
    {/if}
  </div>
{/snippet}

{#snippet targetFileCard(file: DedupMediaFile)}
  {@const filePath = file.filePath}
  {@const fileId = file.id ?? ''}
  {@const isChecked = selectedTargetIds.has(fileId)}
  <div class="flex flex-col">
    <div class="flex items-center gap-2 p-2 rounded border transition-colors {isChecked ? 'border-red-300 dark:border-red-500/40 bg-red-50 dark:bg-red-500/5' : 'border-slate-100 dark:border-[#2a3441] hover:bg-slate-50 dark:hover:bg-[#1f2937]/50'}">
      <!-- Checkbox -->
      <input
        type="checkbox"
        checked={isChecked}
        onclick={(e: MouseEvent) => { e.stopPropagation(); toggleSelect(fileId); }}
        class="size-3.5 accent-red-500 shrink-0 cursor-pointer"
      />
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div onclick={() => togglePreview(filePath)} class="flex items-center gap-2 flex-1 min-w-0 cursor-pointer">
        <div class="flex items-center justify-center size-10 rounded bg-slate-100 dark:bg-[#283039] shrink-0 overflow-hidden">
          {#if file.fileType === 'image'}
            <img src={convertFileSrc(file.filePath)} class="size-10 rounded object-cover" alt="" />
          {:else}
            <span class="material-symbols-outlined text-[18px] text-slate-400">
              {file.fileType === 'video' ? 'movie' : 'description'}
            </span>
          {/if}
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[11px] font-bold text-slate-900 dark:text-white truncate">{file.fileName}</div>
          <div class="text-meta truncate">{file.filePath}</div>
          <div class="text-[10px] text-slate-400 mt-0.5">{formatFileSize(file.fileSize)}</div>
        </div>
        <span class="material-symbols-outlined text-[14px] text-slate-400 shrink-0">
          {expandedFile === filePath ? 'expand_less' : 'visibility'}
        </span>
      </div>
    </div>
    {#if expandedFile === filePath}
      <FilePreview
        filePath={file.filePath}
        fileType={file.fileType}
        fileName={file.fileName}
        fileSize={file.fileSize}
      />
    {/if}
  </div>
{/snippet}

<Panel title="Comparison Results" icon="compare">
  {#snippet actions()}
    <div class="flex items-center gap-2">
      <span class="text-[10px] text-slate-500">{filteredGroups.length} group(s)</span>
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
    </div>
  {/snippet}

  {#if trashMessage}
    <div class="flex items-center gap-2 px-3 py-1.5 text-[11px] {trashMessage.startsWith('Error') ? 'text-red-600 bg-red-50 dark:bg-red-500/10' : 'text-green-600 bg-green-50 dark:bg-green-500/10'}">
      <span class="material-symbols-outlined text-[14px]">{trashMessage.startsWith('Error') ? 'error' : 'check_circle'}</span>
      {trashMessage}
      <button onclick={() => trashMessage = ''} class="ml-auto text-slate-400 hover:text-slate-600">
        <span class="material-symbols-outlined text-[14px]">close</span>
      </button>
    </div>
  {/if}

  {#if filteredGroups.length === 0}
    <EmptyState icon="check_circle" message={groups.length === 0 ? 'No duplicates found' : 'No matches for current filter'} />
  {:else if hasPairLayout}
    <!-- Source vs Target pair layout -->
    <div class="flex flex-col">
      <!-- Column headers -->
      <div class="grid grid-cols-2 gap-3 px-3 py-2 border-b border-slate-200 dark:border-[#2a3441]">
        <div class="flex items-center gap-1.5">
          <span class="material-symbols-outlined text-[14px] text-[#137fec]">star</span>
          <span class="text-stat-label text-status-info">Source</span>
          <span class="text-[10px] text-slate-400">(keep)</span>
        </div>
        <div class="flex items-center gap-1.5">
          <input
            type="checkbox"
            checked={allSelected}
            onclick={toggleSelectAll}
            class="size-3 accent-red-500 cursor-pointer"
            title="Select all targets"
          />
          <span class="material-symbols-outlined text-[14px] text-amber-500">filter_center_focus</span>
          <span class="text-stat-label text-amber-500">Target</span>
          <span class="text-[10px] text-slate-400">(duplicate)</span>
        </div>
      </div>

      <!-- Pair rows -->
      <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
        {#each filteredGroups as group (group.id)}
          <div class="px-3 py-2">
            <!-- Match badge -->
            <div class="flex items-center gap-2 mb-2">
              <span class="material-symbols-outlined text-[14px] {group.matchType === 'exact' ? 'text-green-500' : 'text-amber-500'}">
                {group.matchType === 'exact' ? 'check_circle' : 'change_circle'}
              </span>
              <span class="text-[10px] font-bold {group.matchType === 'exact' ? 'text-green-600' : 'text-amber-600'}">
                {group.matchType === 'exact' ? 'Exact Match' : `${similarityPercent(group.similarityScore)}% Similar`}
              </span>
              <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-500 font-mono uppercase">
                {group.algorithm}
              </span>
            </div>

            <!-- Source ↔ Target side-by-side -->
            <div class="grid grid-cols-2 gap-3">
              {#if group.sourceMember}
                {@render fileCard(group.sourceMember)}
              {:else}
                <div class="p-2 rounded border border-dashed border-slate-200 dark:border-[#2a3441] text-[10px] text-slate-400 text-center">N/A</div>
              {/if}
              {#if group.targetMember}
                {@render targetFileCard(group.targetMember)}
              {:else}
                <div class="p-2 rounded border border-dashed border-slate-200 dark:border-[#2a3441] text-[10px] text-slate-400 text-center">N/A</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Bottom action bar -->
    {#if selectedTargetIds.size > 0}
      <div class="sticky bottom-0 flex items-center justify-between gap-3 px-4 py-2.5 border-t border-slate-200 dark:border-[#2a3441] bg-white dark:bg-[#111418]">
        <div class="flex items-center gap-2 text-[11px]">
          <span class="material-symbols-outlined text-[16px] text-red-500">delete</span>
          <span class="font-bold text-slate-700 dark:text-white">{selectedTargetIds.size} file(s) selected</span>
          <span class="text-slate-400">({formatFileSize(selectedTotalSize)})</span>
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={() => { selectedTargetIds = new Set(); }}
            class="px-3 py-1 text-xs font-bold text-slate-500 hover:text-slate-700 dark:hover:text-white hover:bg-slate-100 dark:hover:bg-[#283039] rounded transition-colors"
          >
            Clear
          </button>
          <button
            onclick={trashSelected}
            disabled={isTrashRunning}
            class="flex items-center gap-1.5 px-3 py-1 text-xs font-bold text-white bg-red-500 hover:bg-red-600 disabled:opacity-50 rounded transition-colors"
          >
            {#if isTrashRunning}
              <span class="material-symbols-outlined text-[14px] animate-spin">progress_activity</span>
              Moving...
            {:else}
              <span class="material-symbols-outlined text-[14px]">delete</span>
              Move to Trash
            {/if}
          </button>
        </div>
      </div>
    {/if}
  {:else}
    <!-- Fallback: legacy flat member layout -->
    <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
      {#each filteredGroups as group (group.id)}
        <div class="px-3 py-2">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="material-symbols-outlined text-[14px] {group.matchType === 'exact' ? 'text-green-500' : 'text-amber-500'}">
              {group.matchType === 'exact' ? 'check_circle' : 'change_circle'}
            </span>
            <span class="text-[10px] font-bold {group.matchType === 'exact' ? 'text-green-600' : 'text-amber-600'}">
              {group.matchType === 'exact' ? 'Exact Match' : `${similarityPercent(group.similarityScore)}% Similar`}
            </span>
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-500 font-mono uppercase">
              {group.algorithm}
            </span>
            <span class="text-[10px] text-slate-400">{group.members.length} files</span>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
            {#each group.members as file}
              {@render fileCard(file)}
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</Panel>
