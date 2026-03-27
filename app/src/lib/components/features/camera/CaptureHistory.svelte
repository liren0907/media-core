<script lang="ts">
  import { Panel } from '$lib/components/ui';

  interface CaptureEntry {
    timestamp: Date;
    path: string | null;
    data: string;
  }

  interface Props {
    captureHistory: CaptureEntry[];
    onselect?: (item: CaptureEntry) => void;
    onclear?: () => void;
  }

  let { captureHistory, onselect, onclear }: Props = $props();
</script>

<Panel title="Recent Captures" icon="photo_library">
    {#snippet actions()}
        {#if captureHistory.length > 0 && onclear}
            <button onclick={onclear} class="text-[10px] font-bold uppercase tracking-wider text-slate-400 hover:text-red-500">Clear</button>
        {/if}
    {/snippet}
    <div class="p-3">
        {#if captureHistory.length > 0}
            <div class="grid grid-cols-3 gap-1.5">
                {#each captureHistory as item, i}
                    <button onclick={() => onselect?.(item)} class="aspect-square rounded overflow-hidden border-2 border-transparent hover:border-[#137fec] transition-colors relative group">
                        <img src="data:image/jpeg;base64,{item.data}" alt="Capture {i + 1}" class="w-full h-full object-cover" />
                        <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                            <span class="material-symbols-outlined text-white text-[14px]">visibility</span>
                        </div>
                        {#if item.path}
                            <div class="absolute bottom-0.5 right-0.5">
                                <span class="material-symbols-outlined text-white text-[12px] drop-shadow">save</span>
                            </div>
                        {/if}
                    </button>
                {/each}
            </div>
        {:else}
            <div class="text-center py-4 text-slate-500">
                <span class="material-symbols-outlined text-2xl mb-1">photo_library</span>
                <p class="text-[10px]">No captures yet</p>
            </div>
        {/if}
    </div>
</Panel>
