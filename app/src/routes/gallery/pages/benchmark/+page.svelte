<script lang="ts">
  import { GallerySection, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatCard, RunButton, FormField, ErrorAlert, EmptyState } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';

  // What the /benchmark route is actually made of, per the source. It has no
  // dedicated feature components — the route is assembled straight from primitives.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'StatCard', href: '#statcard', note: '× 4 — Video / Operations / Fastest / Total Time' },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Configuration” · left column',
      children: [
        { name: 'RunButton', href: '#runbutton', note: 'in actions snippet' },
        { name: 'FilePicker', href: '#filepicker', note: 'components/form' },
        { name: 'FormField', href: '#formfield', note: 'runs-per-operation range + operations checkboxes' },
        { name: 'ErrorAlert', href: '#erroralert' },
      ],
    },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Results” · right column (2-col span)',
      children: [
        { name: 'bar chart', note: 'hand-rolled width-percentage divs' },
        { name: 'results table', note: 'hand-rolled — AVG / MIN / MAX / STD DEV / RUNS' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoVideoPath = $state('');
  let demoRuns = $state(5);

  const statCardCode = `<StatCard label="Video" icon="movie" iconColor="text-[#137fec]" value={videoPath ? getFileName(videoPath) : 'None'} />`;

  const panelCode = `<Panel title="Configuration" icon="tune">
  {#snippet actions()}
    <RunButton loading={isRunning} disabled={!videoPath} onclick={runBenchmark} />
  {/snippet}
  <!-- picker + form fields -->
</Panel>`;

  const runButtonCode = `<RunButton loading={isRunning} disabled={!videoPath} onclick={runBenchmark} />`;

  const filePickerCode = `<FilePicker bind:value={videoPath} label="Video File" filters={[{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov'] }]} />`;

  const formFieldCode = `<FormField label="Runs per operation = {runs}">
  <input type="range" bind:value={runs} min="1" max="20" step="1" class="w-full" />
</FormField>`;

  const errorAlertCode = `<ErrorAlert message={error} />`;

  const emptyStateCode = `<EmptyState icon="sync" message="Running benchmarks..." />`;
</script>

<svelte:head>
  <title>Benchmark · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /benchmark route is assembled. No dedicated feature components — the page is built directly from base primitives, so there is no assembled-result render here."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="statcard"
    title="StatCard"
    description="The summary strip — selected video, operation count, fastest op, and total time."
    link={{ href: '/gallery/statcard', label: 'Component page' }}
  >
    <Showcase code={statCardCode}>
      <div class="w-full grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Video" icon="movie" iconColor="text-[#137fec]" value="sample.mp4" />
        <StatCard label="Operations" icon="checklist" iconColor="text-orange-500" value="3" />
        <StatCard label="Fastest" icon="bolt" iconColor="text-green-500" value="metadata" sub="2.1ms" />
        <StatCard label="Total Time" icon="schedule" iconColor="text-purple-500" value="1.84s" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="“Configuration” (left) hosts the RunButton in its actions snippet; “Results” (right, 2-col span) holds the hand-rolled chart and table."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-md">
        <Panel title="Configuration" icon="tune">
          {#snippet actions()}
            <RunButton loading={false} disabled onclick={() => {}} />
          {/snippet}
          <div class="p-3 text-caption">Panel body — picker, range, and operation checkboxes live here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="runbutton"
    title="RunButton"
    description="Disabled until a video is picked; spins while the benchmark loops run."
    link={{ href: '/gallery/runbutton', label: 'Component page' }}
  >
    <Showcase code={runButtonCode}>
      <RunButton loading={false} onclick={() => {}} />
      <RunButton loading={true} onclick={() => {}} />
      <RunButton loading={false} disabled onclick={() => {}} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="filepicker"
    title="FilePicker"
    description="components/form — picks the benchmark's input video via the native dialog (in-app only)."
  >
    <Showcase code={filePickerCode}>
      <div class="w-full max-w-md">
        <FilePicker bind:value={demoVideoPath} label="Video File" filters={[{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov'] }]} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="The label doubles as live feedback — it interpolates the bound range value."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-xs">
        <FormField label="Runs per operation = {demoRuns}">
          <input type="range" bind:value={demoRuns} min="1" max="20" step="1" class="w-full" />
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="erroralert"
    title="ErrorAlert"
    description="Benchmark failures render inside the Configuration panel."
    link={{ href: '/gallery/erroralert', label: 'Component page' }}
  >
    <Showcase code={errorAlertCode}>
      <div class="w-full">
        <ErrorAlert message="Benchmark failed: could not open video context" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="emptystate"
    title="EmptyState"
    description="The Results panel shows it while running and before the first run."
    link={{ href: '/gallery/emptystate', label: 'Component page' }}
  >
    <Showcase code={emptyStateCode}>
      <div class="w-full">
        <EmptyState icon="sync" message="Running benchmarks..." />
      </div>
    </Showcase>
  </GallerySection>
</div>
