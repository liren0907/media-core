<script lang="ts">
  import { FormField } from '$lib/components/ui';
  import { selectFiles } from '$lib/utils/file-dialog';
  import { getFileName } from '$lib/utils/format';

  interface Props {
    files: string[];
    label: string;
    filters: { name: string; extensions: string[] }[];
  }

  let { files = $bindable(), label, filters }: Props = $props();

  async function browse() {
    const result = await selectFiles(filters);
    if (result) {
      files = [...files, ...result];
    }
  }

  function removeFile(index: number) {
    files = files.filter((_, i) => i !== index);
  }

  function clearFiles() {
    files = [];
  }
</script>

<FormField {label}>
  <button onclick={browse} class="w-full py-2 border-2 border-dashed border-slate-200 dark:border-[#2a3441] rounded text-caption hover:text-[#137fec] hover:border-[#137fec] transition-colors">
    + Select Files
  </button>
  {#if files.length > 0}
    <div class="flex flex-col gap-1 max-h-[200px] overflow-y-auto mt-1">
      {#each files as file, i}
        <div class="flex items-center justify-between px-2 py-1 rounded bg-slate-50 dark:bg-[#1f2937]/50 border border-slate-100 dark:border-[#2a3441] text-[10px] font-mono">
          <span class="truncate max-w-[180px]" title={file}>{getFileName(file)}</span>
          <button onclick={() => removeFile(i)} class="text-slate-400 hover:text-red-500 transition-colors">
            <span class="material-symbols-outlined text-[14px]">close</span>
          </button>
        </div>
      {/each}
    </div>
    <button onclick={clearFiles} class="text-[10px] text-slate-400 hover:text-red-500 self-end">Clear all</button>
  {/if}
</FormField>
