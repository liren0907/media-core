<script lang="ts">
  import { formatFileSize } from '$lib/utils/format';
  import FilePreview from './FilePreview.svelte';
  import type { DedupTreeNode, DedupMediaFile } from '$lib/types';

  interface Props {
    nodes: DedupTreeNode[];
    depth?: number;
    filesMap?: Record<string, DedupMediaFile>;
    previewFile?: string | null;
    onPreviewToggle?: (path: string) => void;
  }

  let { nodes, depth = 0, filesMap = {}, previewFile = null, onPreviewToggle }: Props = $props();
  let expanded = $state<Record<string, boolean>>({});

  function toggle(path: string) {
    expanded[path] = !expanded[path];
  }

  function getFileType(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase() ?? '';
    const imageExts = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'tiff', 'tif', 'heic', 'heif', 'avif'];
    const videoExts = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', 'mpg', 'mpeg', '3gp', 'ts', 'mts'];
    if (imageExts.includes(ext)) return 'image';
    if (videoExts.includes(ext)) return 'video';
    return 'unknown';
  }
</script>

<div class="text-xs font-mono">
  {#each nodes as node}
    {#if node.isDir}
      <button
        onclick={() => toggle(node.path)}
        class="flex items-center gap-1 w-full px-2 py-0.5 hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors text-left"
        style="padding-left: {depth * 16 + 8}px"
      >
        <span class="material-symbols-outlined text-[14px] text-slate-400 transition-transform {expanded[node.path] ? 'rotate-90' : ''}">chevron_right</span>
        <span class="material-symbols-outlined text-[14px] text-amber-500">{expanded[node.path] ? 'folder_open' : 'folder'}</span>
        <span class="text-slate-900 dark:text-white">{node.name}</span>
        {#if node.children.length > 0}
          <span class="text-[10px] text-slate-400 ml-1">({node.children.length})</span>
        {/if}
      </button>
      {#if expanded[node.path]}
        <svelte:self nodes={node.children} depth={depth + 1} {filesMap} {previewFile} {onPreviewToggle} />
      {/if}
    {:else}
      {@const fileInfo = filesMap[node.path]}
      {@const fileType = fileInfo?.fileType ?? getFileType(node.path)}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
      <div>
        <div
          onclick={() => onPreviewToggle?.(node.path)}
          class="flex items-center gap-1 px-2 py-0.5 hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors cursor-pointer group"
          style="padding-left: {depth * 16 + 28}px"
        >
          <span class="material-symbols-outlined text-[14px] {fileType === 'image' ? 'text-blue-400' : fileType === 'video' ? 'text-purple-400' : 'text-slate-400'}">
            {fileType === 'image' ? 'image' : fileType === 'video' ? 'movie' : 'description'}
          </span>
          <span class="text-slate-700 dark:text-slate-300 truncate flex-1">{node.name}</span>

          <!-- Hash status badges -->
          {#if fileInfo}
            <div class="flex items-center gap-0.5 shrink-0">
              {#if fileInfo.contentHash}
                <span class="text-[8px] px-1 py-px rounded bg-green-500/10 text-green-600 font-bold">B3</span>
              {/if}
              {#if fileInfo.phash}
                <span class="text-[8px] px-1 py-px rounded bg-blue-500/10 text-blue-600 font-bold">pH</span>
              {/if}
              {#if fileInfo.dhash}
                <span class="text-[8px] px-1 py-px rounded bg-purple-500/10 text-purple-600 font-bold">dH</span>
              {/if}
            </div>
          {/if}

          {#if node.fileSize}
            <span class="text-[10px] text-slate-400 shrink-0">{formatFileSize(node.fileSize)}</span>
          {/if}

          <span class="material-symbols-outlined text-[14px] text-slate-300 group-hover:text-[#137fec] transition-colors shrink-0">
            {previewFile === node.path ? 'expand_less' : 'visibility'}
          </span>
        </div>

        {#if previewFile === node.path}
          <div style="padding-left: {depth * 16 + 28}px; padding-right: 8px">
            <FilePreview
              filePath={node.path}
              {fileType}
              fileName={node.name}
              fileSize={node.fileSize ?? undefined}
            />
          </div>
        {/if}
      </div>
    {/if}
  {/each}
</div>
