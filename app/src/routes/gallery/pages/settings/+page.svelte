<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, FormField, ToggleSwitch, StatusBadge } from '$lib/components/ui';
  import HardwareSettings from '$lib/components/features/settings/HardwareSettings.svelte';
  import PathSettingsForm from '$lib/components/features/settings/PathSettingsForm.svelte';
  import StreamSettingsForm from '$lib/components/features/settings/StreamSettingsForm.svelte';
  import ThemeSelector from '$lib/components/features/settings/ThemeSelector.svelte';

  // What the /settings route is actually made of, per the source — a stack of
  // Panel-based forms, no tabs.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    {
      name: 'ThemeSelector',
      note: 'features/settings · writes global theme',
      children: [{ name: 'Panel', href: '#panel', note: '“Theme”' }],
    },
    {
      name: 'StreamSettingsForm',
      note: 'features/settings',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'FormField', href: '#formfield', note: 'HLS server URL / ports' },
      ],
    },
    {
      name: 'PathSettingsForm',
      note: 'features/settings',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'DirPicker', note: 'components/form' },
      ],
    },
    {
      name: 'HardwareSettings',
      note: 'features/settings · self-fetches capabilities',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatusBadge', href: '#statusbadge' },
        { name: 'ToggleSwitch', href: '#toggleswitch', note: 'Enable Hardware Acceleration' },
        { name: 'FormField', href: '#formfield' },
      ],
    },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Language” · directly in the route',
      children: [{ name: 'StatusBadge', href: '#statusbadge', note: '“coming soon” custom colorMap' }],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoUrl = $state('http://127.0.0.1');
  let demoHwAccel = $state(true);
  const comingSoonColors: Record<string, string> = {
    'coming soon': 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
  };
  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded-lg px-3 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]';

  const panelCode = `<Panel title="Language" icon="translate">
  {#snippet actions()}
    <StatusBadge status="coming soon" colorMap={comingSoonColors} />
  {/snippet}
  <!-- disabled select -->
</Panel>`;

  const formFieldCode = `<FormField label="HLS Server URL" id="hlsServerUrl">
  <input id="hlsServerUrl" type="text" bind:value={hlsServerUrl} placeholder="http://127.0.0.1" class={inputClass} />
</FormField>`;

  const toggleCode = `<ToggleSwitch bind:checked={hwConfig.enabled} label="Enable Hardware Acceleration" />`;

  const statusBadgeCode = `const comingSoonColors: Record<string, string> = {
  'coming soon': 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
};

<StatusBadge status="coming soon" colorMap={comingSoonColors} />`;
</script>

<svelte:head>
  <title>Settings · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /settings route is assembled — a vertical stack of Panel-based forms, no tabs. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="Every settings group is one Panel; the route's own “Language” Panel pins a coming-soon badge in its actions snippet."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-md">
        <Panel title="Language" icon="translate">
          {#snippet actions()}
            <StatusBadge status="coming soon" colorMap={comingSoonColors} />
          {/snippet}
          <div class="p-4 opacity-60 text-caption">Panel body — the disabled language select lives here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="StreamSettingsForm's labelled inputs — server URL and ports, saved to appConfig."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-md">
        <FormField label="HLS Server URL" id="demo-hls-url">
          <input id="demo-hls-url" type="text" bind:value={demoUrl} placeholder="http://127.0.0.1" class="{inputClass} w-full" />
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="toggleswitch"
    title="ToggleSwitch"
    description="HardwareSettings' master switch for hardware acceleration."
    link={{ href: '/gallery/toggleswitch', label: 'Component page' }}
  >
    <Showcase code={toggleCode}>
      <ToggleSwitch bind:checked={demoHwAccel} label="Enable Hardware Acceleration" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="“coming soon” isn't a built-in status — the route passes a custom amber colorMap."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="coming soon" colorMap={comingSoonColors} />
      <StatusBadge status="active" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The feature components rendered whole. Heads-up: ThemeSelector and the *SettingsForm components read/write global app state — interacting with them here affects real settings."
  >
    <LiveComponent name="ThemeSelector" note="no props · writes global theme"><ThemeSelector /></LiveComponent>
    <LiveComponent name="HardwareSettings" note="no props · self-fetches"><HardwareSettings /></LiveComponent>
    <LiveComponent name="PathSettingsForm" note="onsave? · writes appConfig on save"><PathSettingsForm onsave={() => {}} /></LiveComponent>
    <LiveComponent name="StreamSettingsForm" note="onsave? · writes appConfig on save"><StreamSettingsForm onsave={() => {}} /></LiveComponent>
  </GallerySection>
</div>
