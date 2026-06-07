<script lang="ts">
  import { GallerySection, LiveComponent } from '$lib/components/features/gallery';
  import AnnotateTab from '$lib/components/features/analysis/AnnotateTab.svelte';
  import CompareTab from '$lib/components/features/analysis/CompareTab.svelte';
  import InferenceTab from '$lib/components/features/analysis/InferenceTab.svelte';
  import MotionTab from '$lib/components/features/analysis/MotionTab.svelte';
  import SimilarityTab from '$lib/components/features/analysis/SimilarityTab.svelte';
</script>

<svelte:head>
  <title>Analysis · Gallery</title>
</svelte:head>

<GallerySection
  id="analysis"
  title="Analysis"
  description="Feature components composed by the /analysis route. All are self-contained tabs (no props); they show their config / empty state until run against the backend."
>
  <LiveComponent name="AnnotateTab" note="no props"><AnnotateTab /></LiveComponent>
  <LiveComponent name="CompareTab" note="no props"><CompareTab /></LiveComponent>
  <LiveComponent name="InferenceTab" note="no props"><InferenceTab /></LiveComponent>
  <LiveComponent name="MotionTab" note="no props"><MotionTab /></LiveComponent>
  <LiveComponent name="SimilarityTab" note="no props"><SimilarityTab /></LiveComponent>
</GallerySection>
