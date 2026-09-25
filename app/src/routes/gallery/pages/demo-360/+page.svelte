<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, ToggleSwitch, TabBar } from '$lib/components/ui';
  import ControlsOverlay from '$lib/components/features/demo360/ControlsOverlay.svelte';
  import Panoramic360Viewer from '$lib/components/features/demo360/Panoramic360Viewer.svelte';
  import PhotoDemo from '$lib/components/features/demo360/PhotoDemo.svelte';
  import VideoDemo from '$lib/components/features/demo360/VideoDemo.svelte';

  // What the /demo-360 route is actually made of, per the source.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'Icon', href: '/gallery/icons', note: 'panorama_photosphere page header' },
    { name: 'TabBar', href: '#tabbar', note: 'Photo / Video tabs' },
    {
      name: 'PhotoDemo',
      note: 'features/demo360 · Photo tab',
      children: [
        { name: 'Panel', href: '#panel', note: '“Source Image”' },
        { name: 'Panoramic360Viewer', note: 'features/demo360 · three.js canvas' },
        {
          name: 'ControlsOverlay',
          note: 'features/demo360',
          children: [{ name: 'ToggleSwitch', href: '#toggleswitch', note: 'Auto-rotate' }],
        },
      ],
    },
    {
      name: 'VideoDemo',
      note: 'features/demo360 · Video tab',
      children: [
        { name: 'Panel', href: '#panel', note: '“Source Video”' },
        { name: 'Panoramic360Viewer', note: 'features/demo360' },
        {
          name: 'ControlsOverlay',
          note: 'features/demo360',
          children: [{ name: 'ToggleSwitch', href: '#toggleswitch' }],
        },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoTab = $state('photo');
  const demoTabs = [
    { id: 'photo', label: 'Photo', icon: 'image' },
    { id: 'video', label: 'Video', icon: 'movie' },
  ];
  let demoAutoRotate = $state(false);

  const tabBarCode = `const tabs = [
  { id: 'photo', label: 'Photo', icon: 'image' },
  { id: 'video', label: 'Video', icon: 'movie' },
];

<TabBar {tabs} {activeTab} onchange={(id) => (activeTab = id)} />`;

  const panelCode = `<Panel title="Source Image" icon="image">
  <div class="p-3 flex flex-col gap-2">
    <!-- source picker + equirectangular preview -->
  </div>
</Panel>`;

  const toggleCode = `<ToggleSwitch bind:checked={autoRotate} label="Auto-rotate" />`;

  // ControlsOverlay mock state (it's a controlled component)
  let fov = $state(75);
  let autoRotate = $state(false);
  let dragSensitivity = $state(0.6);
  const view = { lon: 12.5, lat: -8.0, fov: 75 };

  // Panoramic360Viewer: null source → renders its placeholder (no decoded media needed)
  let viewerFov = $state(75);
  const source: HTMLImageElement | HTMLVideoElement | null = null;
</script>

<svelte:head>
  <title>360° Demo · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /demo-360 route is assembled — two tabs, each pairing the three.js viewer with a controls overlay. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="Two tabs — Photo and Video — each mounting its own demo wrapper."
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
    description="Each demo wraps its source picker in a Panel; the viewer itself sits outside on a black stage."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-md">
        <Panel title="Source Image" icon="image">
          <div class="p-3 text-caption">Panel body — source picker and equirectangular preview live here.</div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="toggleswitch"
    title="ToggleSwitch"
    description="ControlsOverlay's auto-rotate switch, alongside its FOV / sensitivity sliders."
    link={{ href: '/gallery/toggleswitch', label: 'Component page' }}
  >
    <Showcase code={toggleCode}>
      <ToggleSwitch bind:checked={demoAutoRotate} label="Auto-rotate" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The feature components rendered whole. ControlsOverlay is driven by bindable mock state; the viewer renders its placeholder with a null source."
  >
    <LiveComponent name="ControlsOverlay" note="bindable: fov, autoRotate, dragSensitivity">
      <ControlsOverlay bind:fov bind:autoRotate bind:dragSensitivity {view} onReset={() => {}} />
    </LiveComponent>
    <LiveComponent name="Panoramic360Viewer" note="prop: source (null → placeholder)">
      <div class="w-full h-[360px]">
        <Panoramic360Viewer {source} bind:fov={viewerFov} autoRotate={false} dragSensitivity={0.6} onView={() => {}} />
      </div>
    </LiveComponent>
    <LiveComponent name="PhotoDemo" note="no props"><PhotoDemo /></LiveComponent>
    <LiveComponent name="VideoDemo" note="no props"><VideoDemo /></LiveComponent>
  </GallerySection>
</div>
