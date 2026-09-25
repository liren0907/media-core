<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatusBadge, ErrorAlert, Icon } from '$lib/components/ui';
  import CaptureHistory from '$lib/components/features/camera/CaptureHistory.svelte';

  // What the /camera route is actually made of, per the source. Unlike most
  // pages it has no tabs — the route composes its panels directly.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'ErrorAlert', href: '#erroralert', note: 'top of page, conditional' },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Preview” · main column',
      children: [
        { name: 'StatusBadge', href: '#statusbadge', note: '“live” with custom colorMap, in actions' },
        { name: 'Icon', href: '/gallery/icons', note: 'placeholder + capture buttons' },
      ],
    },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Camera” · device list, sidebar',
      children: [{ name: 'Icon', href: '/gallery/icons', note: 'per-device videocam + check_circle' }],
    },
    {
      name: 'CaptureHistory',
      note: 'features/camera · sidebar',
      children: [{ name: 'Panel', href: '#panel' }],
    },
    {
      name: 'Panel',
      href: '#panel',
      note: '“Last Capture” · conditional',
      children: [{ name: 'StatusBadge', href: '#statusbadge' }],
    },
  ];

  const liveColors: Record<string, string> = {
    live: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
  };

  const panelCode = `<Panel title="Preview" icon="videocam">
  {#snippet actions()}
    {#if isPreviewActive}
      <StatusBadge status="live" colorMap={liveColors} />
    {/if}
    <button onclick={startLivePreview} class="…">Start</button>
  {/snippet}
  <!-- aspect-video preview surface -->
</Panel>`;

  const statusBadgeCode = `const liveColors: Record<string, string> = {
  live: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
};

<StatusBadge status="live" colorMap={liveColors} />`;

  const errorAlertCode = `{#if error}
  <ErrorAlert message={error} />
{/if}`;
</script>

<svelte:head>
  <title>Camera · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /camera route is assembled — a 2/3 + 1/3 grid of Panels with no tabs. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="The “Preview” Panel pins the live badge and Start/Stop controls in its actions snippet; the body is a black aspect-video surface with an icon placeholder."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-lg">
        <Panel title="Preview" icon="videocam">
          {#snippet actions()}
            <div class="flex items-center gap-2">
              <StatusBadge status="live" colorMap={liveColors} />
              <button class="px-2 py-1 bg-red-500 hover:bg-red-600 text-white rounded text-[10px] font-bold transition-colors">Stop</button>
            </div>
          {/snippet}
          <div class="aspect-video bg-black flex items-center justify-center">
            <div class="text-center text-slate-500">
              <Icon name="videocam" class="text-5xl mb-2" />
              <p class="text-xs">No preview — start preview or capture a snapshot</p>
            </div>
          </div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="The preview's “live” state isn't a built-in status — the route passes a custom colorMap for it."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="live" colorMap={liveColors} />
      <StatusBadge status="active" />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="erroralert"
    title="ErrorAlert"
    description="Camera/permission failures surface once at the top of the page."
    link={{ href: '/gallery/erroralert', label: 'Component page' }}
  >
    <Showcase code={errorAlertCode}>
      <div class="w-full">
        <ErrorAlert message="Failed to open camera: device busy or permission denied" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="CaptureHistory rendered whole — presentational, driven here by mock capture entries (thumbnails won't decode from placeholder data)."
  >
    <LiveComponent name="CaptureHistory" note="prop: captureHistory[]">
      <CaptureHistory
        captureHistory={[
          { timestamp: new Date('2026-06-06T10:00:00'), path: '/captures/img_001.jpg', data: '/9j/4AAQSkZJRg==' },
          { timestamp: new Date('2026-06-06T10:01:30'), path: null, data: '/9j/4AAQSkZJRg==' },
        ]}
        onselect={() => {}}
        onclear={() => {}}
      />
    </LiveComponent>
  </GallerySection>
</div>
