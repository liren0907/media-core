<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Icon } from '$lib/components/ui';

  interface Props {
    src?: string;
    placeholderIcon?: string;
    placeholderText?: string;
    controls?: boolean;
    videoElement?: HTMLVideoElement;
    children?: Snippet;
  }

  let { src, placeholderIcon = 'videocam', placeholderText = 'No video loaded', controls = true, videoElement = $bindable(), children }: Props = $props();
</script>

<div class="bg-black aspect-video rounded overflow-hidden flex items-center justify-center relative">
  {#if src}
    <video bind:this={videoElement} {src} {controls} class="w-full h-full object-contain">
      <track kind="captions" />
    </video>
  {:else}
    <div class="text-center text-slate-500">
      <Icon name={placeholderIcon} class="text-3xl mb-1" />
      <p class="text-xs">{placeholderText}</p>
    </div>
  {/if}
  {#if children}{@render children()}{/if}
</div>
