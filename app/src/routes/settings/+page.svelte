<script lang="ts">
  import { PageContent, Panel, StatusBadge } from '$lib/components/ui';
  import { ThemeSelector, StreamSettingsForm, PathSettingsForm, HardwareSettings } from '$lib/components/features/settings';
  import { appConfig } from '$lib/config.svelte';

  const languages = [
    { code: 'en', name: 'English' },
    { code: 'zh-TW', name: '繁體中文' },
    { code: 'ja', name: '日本語' }
  ];

  let selectedLanguage = $state('en');
  let saveStatus = $state('');

  const inputClass = 'bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#137fec]';

  const comingSoonColors: Record<string, string> = {
    'coming soon': 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
  };

  function showSaveStatus() {
    saveStatus = 'Settings saved!';
    setTimeout(() => { saveStatus = ''; }, 2000);
  }

  function resetToDefaults() {
    if (confirm('Reset all settings to defaults?')) {
      appConfig.reset();
      showSaveStatus();
    }
  }
</script>

<svelte:head>
  <title>Settings | Media Core</title>
</svelte:head>

<PageContent>
  <ThemeSelector />
  <StreamSettingsForm onsave={showSaveStatus} />
  <PathSettingsForm onsave={showSaveStatus} />
  <HardwareSettings />

  <Panel title="Language" icon="translate">
    {#snippet actions()}
      <StatusBadge status="coming soon" colorMap={comingSoonColors} />
    {/snippet}
    <div class="p-4 opacity-60">
      <div class="max-w-md">
        <div class="relative">
          <select class="{inputClass} w-full appearance-none pr-10 disabled:opacity-60 disabled:cursor-not-allowed" bind:value={selectedLanguage} disabled>
            {#each languages as lang}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center px-3 pointer-events-none text-slate-500">
            <span class="material-symbols-outlined text-[18px]">expand_more</span>
          </div>
        </div>
        <p class="mt-2 text-[10px] text-slate-400">Language switching is under development.</p>
      </div>
    </div>
  </Panel>

  <div class="flex items-center justify-between">
    <button onclick={resetToDefaults} class="px-3 py-1.5 text-xs font-medium bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition-colors">
      Reset All to Defaults
    </button>
    {#if saveStatus}
      <div class="flex items-center gap-1.5 text-xs text-green-600 dark:text-green-400">
        <span class="material-symbols-outlined text-[16px]">check_circle</span>
        <span>{saveStatus}</span>
      </div>
    {/if}
  </div>
</PageContent>
