<script lang="ts">
  import { Panel, StatusBadge } from '$lib/components/ui';

  interface EventLogEntry {
    time: string;
    source: string;
    message: string;
  }

  interface Props {
    eventLog: EventLogEntry[];
    eventSourceColors: Record<string, string>;
    onclear: () => void;
  }

  let { eventLog, eventSourceColors, onclear }: Props = $props();
</script>

<Panel title="Event Log" icon="list_alt">
    {#snippet actions()}
        <button onclick={onclear} class="text-stat-label hover:text-slate-600 dark:hover:text-slate-200">Clear</button>
    {/snippet}
    <div class="overflow-x-auto max-h-[280px] overflow-y-auto">
        <table class="w-full text-left text-xs font-mono">
            <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441] sticky top-0 bg-white dark:bg-[#161e27]">
                <tr>
                    <th class="px-3 py-1.5 font-medium w-20">TIME</th>
                    <th class="px-3 py-1.5 font-medium w-24">SOURCE</th>
                    <th class="px-3 py-1.5 font-medium">MESSAGE</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
                {#if eventLog.length > 0}
                    {#each eventLog as entry}
                        <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
                            <td class="px-3 py-1.5 text-slate-500">{entry.time}</td>
                            <td class="px-3 py-1.5">
                                <StatusBadge status={entry.source} colorMap={eventSourceColors} />
                            </td>
                            <td class="px-3 py-1.5 text-slate-700 dark:text-slate-300 truncate max-w-[400px]">{entry.message}</td>
                        </tr>
                    {/each}
                {:else}
                    <tr>
                        <td colspan="3" class="px-3 py-4 text-center text-slate-500">
                            Waiting for events...
                        </td>
                    </tr>
                {/if}
            </tbody>
        </table>
    </div>
</Panel>
