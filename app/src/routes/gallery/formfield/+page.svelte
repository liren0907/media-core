<script lang="ts">
  import { FormField } from '$lib/components/ui';
  import { GallerySection, Showcase } from '$lib/components/features/gallery';
</script>

<svelte:head>
  <title>FormField · Gallery</title>
</svelte:head>

<GallerySection id="formfield" title="FormField" description="Labeled wrapper for form inputs. Props: label, id?, children.">
  <Showcase code={'<FormField label="Source label" id="name">\n  <input … />\n</FormField>'}>
    <div class="w-full max-w-xs flex flex-col gap-3">
      <FormField label="Source label" id="demo-name">
        <input id="demo-name" type="text" placeholder="My photos" class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-2 py-1 text-body focus:outline-none focus:border-[#137fec]" />
      </FormField>
      <FormField label="Algorithm">
        <select class="bg-white dark:bg-[#111418] border border-slate-200 dark:border-[#2a3441] rounded px-2 py-1 text-body focus:outline-none focus:border-[#137fec]">
          <option>BLAKE3</option>
          <option>pHash</option>
          <option>dHash</option>
        </select>
      </FormField>
    </div>
  </Showcase>
</GallerySection>
