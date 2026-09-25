<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label: string;
    id?: string;
    children: Snippet;
  }

  let { label, id, children }: Props = $props();
</script>

<div class="flex flex-col gap-1">
  <label for={id} class="text-stat-label">{label}</label>
  {@render children()}
</div>
