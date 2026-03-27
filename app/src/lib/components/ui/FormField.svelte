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
  <label for={id} class="text-[10px] font-medium uppercase tracking-wider text-slate-500">{label}</label>
  {@render children()}
</div>
