<script lang="ts">
  import { icons } from './icon-registry';

  interface Props {
    /** Icon name (Material-Symbol-style key, mapped to a Lucide component in icon-registry). */
    name: string;
    /** Extra classes. Size is controlled via font-size (the icon is `1em`), so a
        `text-[16px]` / `text-3xl` class sizes it; text color sets its stroke color. */
    class?: string;
  }

  let { name, class: cls = '' }: Props = $props();
  const Cmp = $derived(icons[name]);
</script>

{#if Cmp}
  <Cmp class={`inline-block align-middle w-[1em] h-[1em] ${cls}`} aria-hidden="true" />
{/if}
