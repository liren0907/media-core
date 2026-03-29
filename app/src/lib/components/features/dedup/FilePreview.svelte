<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { formatFileSize } from '$lib/utils/format';

  interface Props {
    filePath: string;
    fileType: string;
    fileName: string;
    fileSize?: number;
  }

  let { filePath, fileType, fileName, fileSize }: Props = $props();

  let imgLoaded = $state(false);
  let assetSrc = $derived(convertFileSrc(filePath));
</script>

<div class="border border-slate-200 dark:border-[#2a3441] rounded-lg overflow-hidden bg-slate-50 dark:bg-[#0d1117] my-1">
  <div class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-[#111418] border-b border-slate-100 dark:border-[#2a3441]">
    <span class="material-symbols-outlined text-[14px] text-slate-400">
      {fileType === 'image' ? 'image' : 'movie'}
    </span>
    <span class="text-[11px] font-bold text-slate-700 dark:text-slate-300 truncate">{fileName}</span>
    {#if fileSize}
      <span class="text-[10px] text-slate-400 ml-auto shrink-0">{formatFileSize(fileSize)}</span>
    {/if}
  </div>

  <div class="flex items-center justify-center p-2 max-h-[300px]">
    {#if fileType === 'image'}
      {#if !imgLoaded}
        <div class="flex items-center justify-center h-32 text-slate-400">
          <span class="material-symbols-outlined text-[24px] animate-pulse">hourglass_top</span>
        </div>
      {/if}
      <img
        src={assetSrc}
        alt={fileName}
        class="max-h-[280px] max-w-full object-contain rounded {imgLoaded ? '' : 'hidden'}"
        onload={() => imgLoaded = true}
      />
    {:else if fileType === 'video'}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video
        src={assetSrc}
        controls
        class="max-h-[280px] max-w-full rounded"
        preload="metadata"
      ></video>
    {:else}
      <div class="flex items-center justify-center h-20 text-slate-400">
        <span class="material-symbols-outlined text-[32px]">description</span>
      </div>
    {/if}
  </div>
</div>
