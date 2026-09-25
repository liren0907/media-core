<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatusBadge, ErrorAlert, EmptyState, RunButton, FormField, TabBar } from '$lib/components/ui';
  import { FilePicker, DirPicker } from '$lib/components/form';
  import AnnotateTab from '$lib/components/features/analysis/AnnotateTab.svelte';
  import CompareTab from '$lib/components/features/analysis/CompareTab.svelte';
  import InferenceTab from '$lib/components/features/analysis/InferenceTab.svelte';
  import MotionTab from '$lib/components/features/analysis/MotionTab.svelte';
  import SimilarityTab from '$lib/components/features/analysis/SimilarityTab.svelte';

  // What the /analysis route is actually made of, per the source.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'TabBar', href: '#tabbar', note: 'Motion / Similarity / Compare / Inference / Annotate' },
    {
      name: 'MotionTab',
      note: 'features/analysis',
      children: [
        { name: 'Panel', href: '#panel', note: '“Configuration” with RunButton in actions' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'FilePicker', href: '#filepicker', note: 'components/form' },
        { name: 'FormField', href: '#formfield', note: 'algorithm select' },
        { name: 'StatusBadge', href: '#statusbadge' },
        { name: 'ErrorAlert', href: '#erroralert' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
    {
      name: 'SimilarityTab',
      note: 'features/analysis',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'DirPicker', href: '#filepicker', note: 'components/form · input + output dirs' },
        { name: 'FormField', href: '#formfield' },
        { name: 'ErrorAlert', href: '#erroralert' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
    {
      name: 'CompareTab',
      note: 'features/analysis',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'FilePicker', href: '#filepicker' },
        { name: 'StatusBadge', href: '#statusbadge', note: 'custom duplicate/different colorMap' },
        { name: 'ErrorAlert', href: '#erroralert' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
    {
      name: 'InferenceTab',
      note: 'features/analysis',
      children: [
        {
          name: 'InferenceConfig',
          note: 'features/inferencer',
          children: [
            { name: 'Panel', href: '#panel' },
            { name: 'StatCard', href: '/gallery/statcard' },
            { name: 'StatusBadge', href: '#statusbadge' },
            { name: 'ProgressBar', href: '/gallery/progressbar' },
            { name: 'FormField', href: '#formfield' },
            { name: 'ErrorAlert', href: '#erroralert' },
          ],
        },
        { name: 'DetectionTimeline', note: 'features/inferencer', children: [{ name: 'Panel', href: '#panel' }] },
      ],
    },
    {
      name: 'AnnotateTab',
      note: 'features/analysis',
      children: [
        { name: 'Panel', href: '#panel' },
        { name: 'StatCard', href: '/gallery/statcard', note: 'metadata key/value cards' },
        { name: 'ErrorAlert', href: '#erroralert' },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoTab = $state('motion');
  const demoTabs = [
    { id: 'motion', label: 'Motion', icon: 'motion_photos_on' },
    { id: 'similarity', label: 'Similarity', icon: 'group_work' },
    { id: 'compare', label: 'Compare', icon: 'compare' },
    { id: 'inference', label: 'Inference', icon: 'bar_chart' },
    { id: 'annotate', label: 'Annotate', icon: 'draw' },
  ];
  let demoAlgorithm = $state('frame_diff');
  let demoVideoPath = $state('');
  let demoInputDir = $state('');
  const compareColors: Record<string, string> = {
    duplicate: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
    different: 'bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-500/20',
  };
  const inputClass = 'bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded-lg px-3 py-2 text-sm text-slate-900 dark:text-white focus:outline-none focus:border-[#137fec] focus:ring-1 focus:ring-[#137fec]';

  const tabBarCode = `const tabs = [
  { id: 'motion', label: 'Motion', icon: 'motion_photos_on' },
  { id: 'similarity', label: 'Similarity', icon: 'group_work' },
  { id: 'compare', label: 'Compare', icon: 'compare' },
  { id: 'inference', label: 'Inference', icon: 'bar_chart' },
  { id: 'annotate', label: 'Annotate', icon: 'draw' },
];

<TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />`;

  const panelCode = `<Panel title="Configuration" icon="tune">
  {#snippet actions()}
    <RunButton loading={isProcessing} onclick={runMotionDetection} />
  {/snippet}
  <!-- pickers + form fields -->
</Panel>`;

  const runButtonCode = `<RunButton loading={isProcessing} onclick={runMotionDetection} />`;

  const formFieldCode = `<FormField label="Algorithm">
  <select bind:value={motionAlgorithm} class="{inputClass} w-full">
    <option value="frame_diff">Frame Difference</option>
    <option value="mog2">MOG2</option>
    <option value="knn">KNN</option>
    <option value="optical_flow">Optical Flow</option>
  </select>
</FormField>`;

  const pickerCode = `<FilePicker bind:value={motionVideoPath} label="Video File" filters={[{ name: "Video", extensions: ["mp4", "avi", "mkv", "mov"] }]} placeholder="Select video..." />
<DirPicker bind:value={similarityInputDir} label="Input Directory" />`;

  const statusBadgeCode = `<StatusBadge status={result.imageComparison.isDuplicate ? 'duplicate' : 'different'} colorMap={{
  duplicate: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
  different: 'bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-500/20',
}} />`;

  const errorAlertCode = `{#if error}
  <ErrorAlert message={error} />
{/if}`;

  const emptyStateCode = `<EmptyState icon="motion_photos_off" message="No motion detected" />`;
</script>

<svelte:head>
  <title>Analysis · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /analysis route is assembled — five self-contained tabs sharing the same Configuration-Panel + RunButton pattern. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="Five tabs — Motion, Similarity, Compare, Inference, Annotate."
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
    description="Every tab opens with a “Configuration” Panel that hosts a RunButton in its actions snippet — the page's signature pattern."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full">
        <Panel title="Configuration" icon="tune">
          {#snippet actions()}
            <RunButton loading={false} onclick={() => {}} />
          {/snippet}
          <div class="p-3 text-caption">Panel body — pickers and form fields live here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="runbutton"
    title="RunButton"
    description="Kicks off each analysis; flips to its loading spinner while the backend works."
    link={{ href: '/gallery/runbutton', label: 'Component page' }}
  >
    <Showcase code={runButtonCode}>
      <RunButton loading={false} onclick={() => {}} />
      <RunButton loading={true} onclick={() => {}} />
      <RunButton loading={false} disabled onclick={() => {}} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="MotionTab's algorithm selector — a labelled select."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-xs">
        <FormField label="Algorithm">
          <select bind:value={demoAlgorithm} class="{inputClass} w-full">
            <option value="frame_diff">Frame Difference</option>
            <option value="mog2">MOG2</option>
            <option value="knn">KNN</option>
            <option value="optical_flow">Optical Flow</option>
          </select>
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="filepicker"
    title="FilePicker / DirPicker"
    description="components/form — native-dialog pickers used by every tab to choose videos and directories. Browsing opens the Tauri dialog (in-app only)."
  >
    <Showcase code={pickerCode}>
      <div class="w-full max-w-md flex flex-col gap-3">
        <FilePicker bind:value={demoVideoPath} label="Video File" filters={[{ name: 'Video', extensions: ['mp4', 'avi', 'mkv', 'mov'] }]} placeholder="Select video..." />
        <DirPicker bind:value={demoInputDir} label="Input Directory" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="CompareTab feeds it a custom colorMap to tint duplicate (green) vs different (orange) verdicts."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="duplicate" colorMap={compareColors} />
      <StatusBadge status="different" colorMap={compareColors} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="erroralert"
    title="ErrorAlert"
    description="Every tab surfaces backend failures the same way — conditionally rendered above the results."
    link={{ href: '/gallery/erroralert', label: 'Component page' }}
  >
    <Showcase code={errorAlertCode}>
      <div class="w-full">
        <ErrorAlert message="Failed to run motion detection: video file not found" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="emptystate"
    title="EmptyState"
    description="Shown until a run produces results — each tab brings its own icon and message."
    link={{ href: '/gallery/emptystate', label: 'Component page' }}
  >
    <Showcase code={emptyStateCode}>
      <div class="w-full">
        <EmptyState icon="motion_photos_off" message="No motion detected" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The five tabs rendered whole. All are self-contained (no props); they show their config / empty state until run against the backend."
  >
    <LiveComponent name="AnnotateTab" note="no props"><AnnotateTab /></LiveComponent>
    <LiveComponent name="CompareTab" note="no props"><CompareTab /></LiveComponent>
    <LiveComponent name="InferenceTab" note="no props"><InferenceTab /></LiveComponent>
    <LiveComponent name="MotionTab" note="no props"><MotionTab /></LiveComponent>
    <LiveComponent name="SimilarityTab" note="no props"><SimilarityTab /></LiveComponent>
  </GallerySection>
</div>
