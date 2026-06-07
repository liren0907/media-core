<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import ControlsOverlay from '$lib/components/features/demo360/ControlsOverlay.svelte';
  import Panoramic360Viewer from '$lib/components/features/demo360/Panoramic360Viewer.svelte';
  import PhotoDemo from '$lib/components/features/demo360/PhotoDemo.svelte';
  import VideoDemo from '$lib/components/features/demo360/VideoDemo.svelte';

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

<GallerySection
  id="demo-360"
  title="360° Demo"
  description="Feature components composed by the /demo-360 route. ControlsOverlay is driven by bindable mock state; the viewer renders its placeholder with a null source."
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
