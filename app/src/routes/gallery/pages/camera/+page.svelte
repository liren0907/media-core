<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import CaptureHistory from '$lib/components/features/camera/CaptureHistory.svelte';
</script>

<svelte:head>
  <title>Camera · Gallery</title>
</svelte:head>

<GallerySection
  id="camera"
  title="Camera"
  description="Feature components composed by the /camera route. CaptureHistory is presentational — driven here by mock capture entries (thumbnails won't decode from placeholder data)."
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
