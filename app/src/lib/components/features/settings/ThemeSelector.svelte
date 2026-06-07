<script lang="ts">
  import { Panel, Icon } from '$lib/components/ui';
  import { theme, effectiveTheme } from '$lib/theme.svelte';
</script>

<Panel title="Theme" icon="palette">
  <div class="p-4">
    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <button
        onclick={() => theme.set('light')}
        class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'light' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
      >
        <div class="w-full aspect-video bg-[#f6f7f8] rounded-lg border border-slate-200 shadow-xs flex items-center justify-center overflow-hidden">
          <div class="w-3/4 h-3/4 bg-white rounded shadow-xs flex flex-col gap-2 p-2">
            <div class="w-1/3 h-2 bg-slate-200 rounded"></div>
            <div class="w-full h-16 bg-slate-100 rounded"></div>
          </div>
        </div>
        <div class="flex items-center gap-1.5 text-xs">
          <Icon name="light_mode" class="text-[16px] text-slate-500" />
          <span class="font-medium text-slate-900 dark:text-white">Light</span>
        </div>
        {#if theme.value === 'light'}
          <div class="absolute top-2 right-2 text-[#137fec]"><Icon name="check_circle" class="text-[18px]" /></div>
        {/if}
      </button>

      <button
        onclick={() => theme.set('dark')}
        class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'dark' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
      >
        <div class="w-full aspect-video bg-[#101922] rounded-lg border border-slate-700 shadow-xs flex items-center justify-center overflow-hidden">
          <div class="w-3/4 h-3/4 bg-[#1a222c] rounded shadow-xs flex flex-col gap-2 p-2">
            <div class="w-1/3 h-2 bg-slate-700 rounded"></div>
            <div class="w-full h-16 bg-slate-800 rounded"></div>
          </div>
        </div>
        <div class="flex items-center gap-1.5 text-xs">
          <Icon name="dark_mode" class="text-[16px] text-slate-500" />
          <span class="font-medium text-slate-900 dark:text-white">Dark</span>
        </div>
        {#if theme.value === 'dark'}
          <div class="absolute top-2 right-2 text-[#137fec]"><Icon name="check_circle" class="text-[18px]" /></div>
        {/if}
      </button>

      <button
        onclick={() => theme.set('auto')}
        class="relative flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all {theme.value === 'auto' ? 'border-[#137fec] bg-blue-50/50 dark:bg-blue-900/20' : 'border-slate-200 dark:border-[#2a3441] hover:border-slate-300 dark:hover:border-slate-600'}"
      >
        <div class="w-full aspect-video bg-gradient-to-br from-[#f6f7f8] to-[#101922] rounded-lg border border-slate-200 dark:border-[#2a3441] shadow-xs flex items-center justify-center">
          <Icon name="brightness_auto" class="text-3xl text-slate-400" />
        </div>
        <div class="flex items-center gap-1.5 text-xs">
          <Icon name="settings_brightness" class="text-[16px] text-slate-500" />
          <span class="font-medium text-slate-900 dark:text-white">Auto</span>
        </div>
        {#if theme.value === 'auto'}
          <div class="absolute top-2 right-2 text-[#137fec]"><Icon name="check_circle" class="text-[18px]" /></div>
        {/if}
      </button>
    </div>

    <div class="mt-3 flex items-center gap-2 text-caption bg-slate-50 dark:bg-[#1a222c] p-2 rounded-lg border border-slate-100 dark:border-[#2a3441]">
      <Icon name="info" class="text-[16px]" />
      <p>
        Current: <span class="font-bold text-slate-900 dark:text-white uppercase">{effectiveTheme.value}</span>
        {#if theme.value === 'auto'} (system){/if}
      </p>
    </div>
  </div>
</Panel>
