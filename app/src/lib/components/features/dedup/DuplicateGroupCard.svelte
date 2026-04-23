<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { Panel } from '$lib/components/ui';
  import { formatFileSize } from '$lib/utils/format';
  import type { DedupGroupExpanded } from '$lib/types';

  interface Props {
    group: DedupGroupExpanded;
  }

  let { group }: Props = $props();

  let similarityPercent = $derived((group.similarityScore * 100).toFixed(0));
</script>

<Panel title="Match Group" icon="content_copy">
  {#snippet actions()}
    <div class="flex items-center gap-2">
      <span class="text-[10px] px-1.5 py-0.5 rounded font-bold {group.matchType === 'exact' ? 'bg-green-500/10 text-green-600' : 'bg-orange-500/10 text-orange-600'}">
        {group.matchType === 'exact' ? 'Exact' : `${similarityPercent}% Similar`}
      </span>
      <span class="text-[10px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#283039] text-slate-500 font-bold uppercase">
        {group.algorithm}
      </span>
    </div>
  {/snippet}

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
    {#each group.members as file}
      <div class="flex items-start gap-2 p-2 rounded-lg border border-slate-100 dark:border-[#2a3441] hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
        <div class="flex items-center justify-center size-10 rounded bg-slate-100 dark:bg-[#283039] shrink-0 overflow-hidden">
          {#if file.fileType === 'image'}
            <img src={convertFileSrc(file.filePath)} class="size-10 rounded object-cover" alt="" />
          {:else}
            <span class="material-symbols-outlined text-[20px] text-slate-400">
              {file.fileType === 'video' ? 'movie' : 'description'}
            </span>
          {/if}
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-card-title truncate">{file.fileName}</div>
          <div class="text-meta truncate" title={file.filePath}>{file.filePath}</div>
          <div class="flex items-center gap-2 mt-1">
            <span class="text-[10px] text-slate-400">{formatFileSize(file.fileSize)}</span>
            <span class="text-[10px] text-slate-400 capitalize">{file.fileType}</span>
          </div>
        </div>
      </div>
    {/each}
  </div>

  {#if group.members.length === 0}
    <p class="text-caption text-center py-2">No member details available</p>
  {/if}
</Panel>
