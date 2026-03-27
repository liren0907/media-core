<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Panel, StatusBadge, ToggleSwitch, FormField } from '$lib/components/ui';
  import type { HardwareAccelConfig, HardwareCapabilities } from '$lib/types';

  let hwConfig = $state<HardwareAccelConfig>({ enabled: false, mode: 'auto', fallbackToCpu: true, preferBackends: [] });
  let hwCapabilities = $state<HardwareCapabilities | null>(null);

  const inputClass = 'bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#137fec]';

  async function loadHardwareInfo() {
    try {
      const [config, caps] = await Promise.all([
        invoke('get_hardware_accel_config') as Promise<HardwareAccelConfig>,
        invoke('detect_hardware_capabilities') as Promise<HardwareCapabilities>,
      ]);
      hwConfig = config;
      hwCapabilities = caps;
    } catch (e) {
      console.error('Failed to load hardware info:', e);
    }
  }

  $effect(() => { loadHardwareInfo(); });
</script>

<Panel title="Hardware Acceleration" icon="bolt">
  <div class="p-4 flex flex-col gap-3">
    {#if hwCapabilities}
      <div class="flex items-center gap-2 text-xs text-slate-500 bg-slate-50 dark:bg-[#1a222c] p-2 rounded border border-slate-100 dark:border-[#2a3441]">
        <span class="material-symbols-outlined text-[16px]">info</span>
        <span>Platform: <span class="font-bold text-slate-900 dark:text-white">{hwCapabilities.platform}</span></span>
        {#if hwCapabilities.isAppleSilicon}
          <StatusBadge status="Apple Silicon" colorMap={{ 'Apple Silicon': 'bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20' }} />
        {/if}
      </div>
    {/if}

    <ToggleSwitch bind:checked={hwConfig.enabled} label="Enable Hardware Acceleration" />

    {#if hwConfig.enabled}
      <FormField label="Mode" id="hwMode">
        <select id="hwMode" bind:value={hwConfig.mode} class="{inputClass} w-full">
          {#if hwCapabilities}
            {#each hwCapabilities.availableModes as mode}
              <option value={mode}>{mode === 'auto' ? 'Auto Detect' : mode === 'apple_silicon' ? 'Apple Silicon (VideoToolbox)' : mode === 'cuda' ? 'NVIDIA CUDA' : mode === 'disabled' ? 'Disabled (CPU)' : mode}</option>
            {/each}
          {:else}
            <option value="auto">Auto Detect</option>
            <option value="disabled">Disabled</option>
          {/if}
        </select>
      </FormField>

      <ToggleSwitch bind:checked={hwConfig.fallbackToCpu} label="Fallback to CPU if acceleration fails" />
    {/if}
  </div>
</Panel>
