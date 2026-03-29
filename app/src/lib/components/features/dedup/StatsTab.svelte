<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { StatCard, Panel, EmptyState } from '$lib/components/ui';
  import { formatFileSize } from '$lib/utils/format';
  import type { DedupStats, DedupSource } from '$lib/types';

  let stats = $state<DedupStats | null>(null);
  let sources = $state<DedupSource[]>([]);
  let error = $state('');

  async function fetchData() {
    try {
      stats = await invoke('get_dedup_stats');
      sources = await invoke('get_scan_sources');
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => {
    fetchData();
  });
</script>

<!-- Stat Cards -->
<div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
  <StatCard
    label="Total Files"
    icon="description"
    iconColor="text-[#137fec]"
    value={stats ? stats.totalFiles.toLocaleString() : '0'}
  />
  <StatCard
    label="Sources"
    icon="folder"
    iconColor="text-amber-500"
    value={stats ? String(stats.totalSources) : '0'}
  />
  <StatCard
    label="Fingerprinted"
    icon="fingerprint"
    iconColor="text-purple-500"
    value={stats ? stats.totalHashed.toLocaleString() : '0'}
  />
  <StatCard
    label="Duplicate Groups"
    icon="content_copy"
    iconColor="text-orange-500"
    value={stats ? String(stats.totalDuplicateGroups) : '0'}
  />
</div>

<!-- Recent Sources -->
<Panel title="Scan Sources" icon="history">
  {#if sources.length === 0}
    <EmptyState icon="folder_off" message="No scan sources yet" />
  {:else}
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs font-mono">
        <thead class="text-slate-500 border-b border-slate-100 dark:border-[#2a3441]">
          <tr>
            <th class="px-3 py-1.5 font-medium">Label</th>
            <th class="px-3 py-1.5 font-medium">Path</th>
            <th class="px-3 py-1.5 font-medium">Status</th>
            <th class="px-3 py-1.5 font-medium text-right">Files</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100 dark:divide-[#2a3441]">
          {#each sources as source}
            <tr class="hover:bg-slate-50 dark:hover:bg-[#1f2937]/50 transition-colors">
              <td class="px-3 py-1.5 text-slate-900 dark:text-white font-bold">{source.label}</td>
              <td class="px-3 py-1.5 text-slate-600 dark:text-slate-400 truncate max-w-xs">{source.path}</td>
              <td class="px-3 py-1.5">
                <span class="text-[10px] px-1.5 py-0.5 rounded font-bold {source.status === 'hashed' ? 'bg-green-500/10 text-green-600' : source.status === 'scanned' ? 'bg-blue-500/10 text-blue-600' : 'bg-slate-100 dark:bg-[#283039] text-slate-500'}">
                  {source.status}
                </span>
              </td>
              <td class="px-3 py-1.5 text-right text-slate-600 dark:text-slate-400">{source.fileCount}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</Panel>
