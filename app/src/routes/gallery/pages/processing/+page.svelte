<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatCard, StatusBadge, ProgressBar, ToggleSwitch, RunButton, FormField, TabBar } from '$lib/components/ui';
  import BatchTab from '$lib/components/features/processing/BatchTab.svelte';
  import ExtractTab from '$lib/components/features/processing/ExtractTab.svelte';

  // What the /processing route is actually made of, per the source.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'TabBar', href: '#tabbar', note: 'Batch / Extract tabs' },
    {
      name: 'BatchTab',
      note: 'features/processing · polls pipeline progress + events',
      children: [
        { name: 'Panel', href: '#panel', note: '“Batch Configuration” with RunButton in actions' },
        { name: 'StatCard', href: '#statcard', note: 'files selected' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'ProgressBar', href: '#progressbar', note: 'per-pipeline progress' },
        { name: 'StatusBadge', href: '#statusbadge', note: 'error / active per progress row' },
        { name: 'ToggleSwitch', href: '#toggleswitch', note: 'Parallel Processing' },
        { name: 'FormField', href: '#formfield' },
        { name: 'ErrorAlert', href: '/gallery/erroralert' },
        { name: 'EmptyState', href: '/gallery/emptystate' },
      ],
    },
    {
      name: 'ExtractTab',
      note: 'features/processing',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatCard', href: '#statcard' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'ProgressBar', href: '#progressbar' },
        { name: 'StatusBadge', href: '#statusbadge' },
        { name: 'FormField', href: '#formfield', note: 'Preview / Save mode button group' },
        { name: 'ErrorAlert', href: '/gallery/erroralert' },
        { name: 'EmptyState', href: '/gallery/emptystate' },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoTab = $state('batch');
  const demoTabs = [
    { id: 'batch', label: 'Batch', icon: 'conversion_path' },
    { id: 'extract', label: 'Extract', icon: 'photo_library' },
  ];
  let demoParallel = $state(true);
  let demoMode = $state('preview');

  const tabBarCode = `const tabs = [
  { id: 'batch', label: 'Batch', icon: 'conversion_path' },
  { id: 'extract', label: 'Extract', icon: 'photo_library' },
];

<TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />`;

  const panelCode = `<Panel title="Batch Configuration" icon="settings">
  {#snippet actions()}
    <RunButton loading={isRunning} disabled={selectedFiles.length === 0} onclick={runBatch} />
  {/snippet}
  <!-- form body -->
</Panel>`;

  const statCardCode = `<StatCard label="Files" icon="folder_open" iconColor="text-[#137fec]" value="{String(selectedFiles.length)}" sub="selected" />`;

  const runButtonCode = `<RunButton loading={isRunning} disabled={selectedFiles.length === 0} onclick={runBatch} />`;

  const progressBarCode = `<StatusBadge status={currentProgress.hasError ? 'error' : 'active'} />
<ProgressBar percent={currentProgress.progressPercent} />`;

  const statusBadgeCode = `<StatusBadge status={currentProgress.hasError ? 'error' : 'active'} />`;

  const toggleCode = `<ToggleSwitch bind:checked={parallel} label="Parallel Processing" />`;

  const formFieldCode = `<FormField label="Mode">
  <div class="flex gap-1">
    <button onclick={() => mode = 'preview'} class="… {mode === 'preview' ? 'bg-[#137fec] text-white' : '…'}">Preview</button>
    <button onclick={() => mode = 'save'} class="…">Save</button>
  </div>
</FormField>`;
</script>

<svelte:head>
  <title>Processing · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /processing route is assembled — two self-contained tabs. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="Two tabs — Batch (pipeline runs) and Extract (frame extraction)."
    link={{ href: '/gallery/tabbar', label: 'Component page' }}
  >
    <Showcase code={tabBarCode}>
      <div class="w-full">
        <TabBar tabs={demoTabs} activeTab={demoTab} onchange={(id) => (demoTab = id)} />
        <p class="text-meta mt-2">Active tab: {demoTab}</p>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="BatchTab's “Batch Configuration” hosts the RunButton in its actions snippet — disabled until files are selected."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full">
        <Panel title="Batch Configuration" icon="settings">
          {#snippet actions()}
            <RunButton loading={false} disabled onclick={() => {}} />
          {/snippet}
          <div class="p-3 text-caption">Panel body — file selection and pipeline options live here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statcard"
    title="StatCard"
    description="The Batch tab's summary strip — selected file count and run totals."
    link={{ href: '/gallery/statcard', label: 'Component page' }}
  >
    <Showcase code={statCardCode}>
      <div class="w-full grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Files" icon="folder_open" iconColor="text-[#137fec]" value="12" sub="selected" />
        <StatCard label="Completed" icon="check_circle" iconColor="text-green-500" value="8" />
        <StatCard label="Errors" icon="error" iconColor="text-red-500" value="1" />
        <StatCard label="Elapsed" icon="schedule" iconColor="text-purple-500" value="02:41" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="runbutton"
    title="RunButton"
    description="Starts the batch run / extraction; disabled with no input, spinning while a pipeline is live."
    link={{ href: '/gallery/runbutton', label: 'Component page' }}
  >
    <Showcase code={runButtonCode}>
      <RunButton loading={false} onclick={() => {}} />
      <RunButton loading={true} onclick={() => {}} />
      <RunButton loading={false} disabled onclick={() => {}} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="progressbar"
    title="ProgressBar"
    description="Live pipeline progress — paired with a StatusBadge per running file."
    link={{ href: '/gallery/progressbar', label: 'Component page' }}
  >
    <Showcase code={progressBarCode}>
      <div class="w-full max-w-md">
        <div class="flex items-center justify-between mb-1.5">
          <span class="text-meta font-mono">video_0042.mp4</span>
          <StatusBadge status="active" />
        </div>
        <ProgressBar percent={64} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="Flips between active and error based on the pipeline event's hasError flag."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="active" />
      <StatusBadge status="error" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="toggleswitch"
    title="ToggleSwitch"
    description="Batch option — run the pipeline files in parallel."
    link={{ href: '/gallery/toggleswitch', label: 'Component page' }}
  >
    <Showcase code={toggleCode}>
      <ToggleSwitch bind:checked={demoParallel} label="Parallel Processing" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="ExtractTab wraps a Preview / Save button group instead of an input — FormField only supplies the label frame."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-xs">
        <FormField label="Mode">
          <div class="flex gap-1">
            <button onclick={() => (demoMode = 'preview')} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {demoMode === 'preview' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Preview</button>
            <button onclick={() => (demoMode = 'save')} class="flex-1 py-1.5 rounded text-xs font-bold transition-colors {demoMode === 'save' ? 'bg-[#137fec] text-white' : 'bg-slate-100 dark:bg-[#1f2937] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#2a3441]'}">Save</button>
          </div>
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="Both tabs rendered whole. Self-contained (no props); BatchTab polls pipeline progress and subscribes to events on mount."
  >
    <LiveComponent name="BatchTab" note="no props · polls + events"><BatchTab /></LiveComponent>
    <LiveComponent name="ExtractTab" note="no props"><ExtractTab /></LiveComponent>
  </GallerySection>
</div>
