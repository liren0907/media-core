<script lang="ts">
  import { Panel, FormField } from '$lib/components/ui';
  import { DirPicker } from '$lib/components/form';
  import { appConfig } from '$lib/config.svelte';

  interface Props {
    onsave?: () => void;
  }

  let { onsave }: Props = $props();

  let hlsServerUrl = $state('');
  let hlsServerPort = $state(1521);
  let defaultRtspUrl = $state('');
  let hlsOutputDir = $state('');

  const inputClass = 'bg-white dark:bg-[#1a222c] border border-slate-200 dark:border-[#2a3441] text-slate-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[#137fec]';

  $effect(() => {
    const config = appConfig.current;
    hlsServerUrl = config.streams.hlsServerUrl;
    hlsServerPort = config.streams.hlsServerPort;
    defaultRtspUrl = config.streams.defaultRtspUrl;
    hlsOutputDir = config.streams.hlsOutputDir;
  });

  function save() {
    appConfig.updateStreams({ hlsServerUrl, hlsServerPort, defaultRtspUrl, hlsOutputDir });
    onsave?.();
  }
</script>

<Panel title="Stream Settings" icon="stream">
  {#snippet actions()}
    <button onclick={save} class="text-stat-label text-status-info hover:text-blue-400">Save</button>
  {/snippet}
  <div class="p-4">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <FormField label="HLS Server URL" id="hlsServerUrl">
        <input id="hlsServerUrl" type="text" bind:value={hlsServerUrl} placeholder="http://127.0.0.1" class={inputClass} />
      </FormField>
      <FormField label="HLS Server Port" id="hlsServerPort">
        <input id="hlsServerPort" type="number" bind:value={hlsServerPort} placeholder="1521" class={inputClass} />
      </FormField>
      <div class="md:col-span-2">
        <FormField label="Default RTSP URL" id="defaultRtspUrl">
          <input id="defaultRtspUrl" type="text" bind:value={defaultRtspUrl} placeholder="rtsp://localhost:8554/mystream" class={inputClass} />
        </FormField>
      </div>
      <div class="md:col-span-2">
        <DirPicker bind:value={hlsOutputDir} label="HLS Output Directory" placeholder="hls_output" />
      </div>
    </div>
  </div>
</Panel>
