<script lang="ts">
  import { Panel } from '$lib/components/ui';
  import { DirPicker } from '$lib/components/form';
  import { appConfig } from '$lib/config.svelte';

  interface Props {
    onsave?: () => void;
  }

  let { onsave }: Props = $props();

  let defaultVideoDir = $state('');
  let defaultImageDir = $state('');
  let defaultOutputDir = $state('');
  let annotationDir = $state('');

  $effect(() => {
    const config = appConfig.current;
    defaultVideoDir = config.paths.defaultVideoDir;
    defaultImageDir = config.paths.defaultImageDir;
    defaultOutputDir = config.paths.defaultOutputDir;
    annotationDir = config.paths.annotationDir;
  });

  function save() {
    appConfig.updatePaths({ defaultVideoDir, defaultImageDir, defaultOutputDir, annotationDir });
    onsave?.();
  }
</script>

<Panel title="Default Paths" icon="folder">
  {#snippet actions()}
    <button onclick={save} class="text-[10px] font-bold uppercase tracking-wider text-[#137fec] hover:text-blue-400">Save</button>
  {/snippet}
  <div class="p-4 flex flex-col gap-3">
    <DirPicker bind:value={defaultVideoDir} label="Video Directory" placeholder="Select default video directory..." />
    <DirPicker bind:value={defaultImageDir} label="Image Directory" placeholder="Select default image directory..." />
    <DirPicker bind:value={defaultOutputDir} label="Output Directory" placeholder="Select default output directory..." />
    <DirPicker bind:value={annotationDir} label="Annotation Directory" placeholder="Select default annotation directory..." />
  </div>
</Panel>
