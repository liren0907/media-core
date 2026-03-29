<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { Panel, EmptyState } from '$lib/components/ui';
  import { formatFileSize } from '$lib/utils/format';
  import FilePreview from './FilePreview.svelte';
  import type { DedupGroupExpanded, DedupMediaFile } from '$lib/types';

  interface Props {
    groups: DedupGroupExpanded[];
  }

  let { groups }: Props = $props();

  let filter = $state<'all' | 'exact' | 'similar'>('all');
  let expandedFile = $state<string | null>(null);

  let filteredGroups = $derived(
    groups.filter(g => filter === 'all' || g.matchType === filter)
  );

  function togglePreview(filePath: string) {
    expandedFile = expandedFile === filePath ? null : filePath;
  }

  let similarityPercent = (score: number) => (score * 100).toFixed(0);
</script>

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

  {#if filteredGroups.length === 0}
    <EmptyState icon="check_circle" message={groups.length === 0 ? 'No duplicates found' : 'No matches for current filter'} />
  {:else}
    <div class="flex flex-col divide-y divide-slate-100 dark:divide-[#2a3441]">
      {#each filteredGroups as group (group.id)}
        <div class="px-3 py-2">
          <!-- Group header -->
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

          <!-- Member files -->
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
            {#each group.members as file}
              {@const filePath = file.filePath}
              <div class="flex flex-col">
                <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                <div
                  onclick={() => togglePreview(filePath)}
                  class="flex items-center gap-2 p-1.5 rounded border border-slate-100 dark:border-[#2a3441] hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 cursor-pointer transition-colors"
                >
                  <div class="flex items-center justify-center size-8 rounded bg-slate-100 dark:bg-[#283039] shrink-0 overflow-hidden">
                    {#if file.fileType === 'image'}
                      <img src={convertFileSrc(file.filePath)} class="size-8 rounded object-cover" alt="" />
                    {:else}
                      <span class="material-symbols-outlined text-[16px] text-slate-400">
                        {file.fileType === 'video' ? 'movie' : 'description'}
                      </span>
                    {/if}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="text-[11px] font-bold text-slate-900 dark:text-white truncate">{file.fileName}</div>
                    <div class="text-[10px] text-slate-400 font-mono truncate">{file.filePath}</div>
                  </div>
                  <div class="flex items-center gap-1 shrink-0">
                    <span class="text-[10px] text-slate-400">{formatFileSize(file.fileSize)}</span>
                    <span class="material-symbols-outlined text-[14px] text-slate-400">
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
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</Panel>
