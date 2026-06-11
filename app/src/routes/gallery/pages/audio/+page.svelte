<script lang="ts">
  import { GallerySection, LiveComponent, Showcase, CompositionTree } from '$lib/components/features/gallery';
  import type { CompositionNode } from '$lib/components/features/gallery/CompositionTree.svelte';
  import { Panel, StatusBadge, ErrorAlert, EmptyState, RunButton, FormField, TabBar, ProgressBar, StatCard } from '$lib/components/ui';
  import { FilePicker } from '$lib/components/form';
  import { inputClass } from '$lib/utils/styles';
  import LevelMeter from '$lib/components/features/audio/LevelMeter.svelte';
  import MetadataTab from '$lib/components/features/audio/MetadataTab.svelte';
  import RecorderTab from '$lib/components/features/audio/RecorderTab.svelte';
  import TranscribeTab from '$lib/components/features/audio/TranscribeTab.svelte';

  // What the /audio route is actually made of, per the source.
  const composition: CompositionNode[] = [
    { name: 'PageContent', href: '/gallery/pagecontent', note: 'page root wrapper' },
    { name: 'TabBar', href: '#tabbar', note: 'Recorder / Transcribe / Metadata' },
    {
      name: 'RecorderTab',
      note: 'features/audio',
      children: [
        { name: 'Panel', href: '#panel', note: '“Recorder” + “Recordings”, side by side' },
        { name: 'StatusBadge', href: '#statusbadge', note: 'recording/paused/standby custom colorMap, in actions' },
        {
          name: 'LevelMeter',
          href: '#levelmeter',
          note: 'features/audio · live mic level',
          children: [
            { name: 'ProgressBar', href: '#progressbar' },
            { name: 'SparklineBar', note: 'components/data · rolling level history' },
          ],
        },
        { name: 'Icon', href: '/gallery/icons', note: 'record / pause / stop / play / rename / delete buttons' },
        { name: 'ErrorAlert', href: '#erroralert', note: 'mic-permission and save failures' },
        { name: 'EmptyState', href: '#emptystate', note: 'empty recordings list' },
      ],
    },
    {
      name: 'TranscribeTab',
      note: 'features/audio',
      children: [
        { name: 'Panel', href: '#panel', note: '“Source” / “Model” / “Transcribe”' },
        { name: 'FilePicker', href: '#filepicker', note: 'components/form' },
        { name: 'FormField', href: '#formfield', note: 'model + language selects, recording picker' },
        { name: 'StatusBadge', href: '#statusbadge', note: 'model ready / not downloaded' },
        { name: 'RunButton', href: '#runbutton', note: 'Download + Transcribe' },
        { name: 'ProgressBar', href: '#progressbar', note: 'model download + transcription progress' },
        { name: 'ErrorAlert', href: '#erroralert' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
    {
      name: 'MetadataTab',
      note: 'features/audio',
      children: [
        { name: 'Panel', href: '#panel', note: '“Analyze Audio” + conditional “SRT Statistics”' },
        { name: 'FilePicker', href: '#filepicker', note: 'audio + optional SRT' },
        { name: 'RunButton', href: '#runbutton' },
        { name: 'StatCard', href: '#statcard', note: 'Duration / Sample Rate / Bitrate / Channels' },
        { name: 'ErrorAlert', href: '#erroralert' },
        { name: 'EmptyState', href: '#emptystate' },
      ],
    },
  ];

  // Static demo state so every section renders fully without a backend.
  let demoTab = $state('recorder');
  const demoTabs = [
    { id: 'recorder', label: 'Recorder', icon: 'mic' },
    { id: 'transcribe', label: 'Transcribe', icon: 'graphic_eq' },
    { id: 'metadata', label: 'Metadata', icon: 'info' },
  ];
  let demoAudioPath = $state('');
  let demoSrtPath = $state('');
  let demoModel = $state('base');
  let demoLanguage = $state('auto');

  const recorderColors: Record<string, string> = {
    recording: 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20',
    paused: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
    standby: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
  };
  const modelColors: Record<string, string> = {
    ready: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
    'not downloaded': 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
  };

  // A frozen "someone is talking" waveform for the LevelMeter demo.
  const demoHistory = Array.from({ length: 48 }, (_, i) =>
    Math.max(4, Math.round(55 + 35 * Math.sin(i / 2.6) * Math.cos(i / 7)))
  );
  const demoLevel = demoHistory[demoHistory.length - 1];

  const tabBarCode = `const tabs = [
  { id: 'recorder', label: 'Recorder', icon: 'mic' },
  { id: 'transcribe', label: 'Transcribe', icon: 'graphic_eq' },
  { id: 'metadata', label: 'Metadata', icon: 'info' },
];

<TabBar {tabs} {activeTab} onchange={(id) => activeTab = id} />`;

  const panelCode = `<Panel title="Recorder" icon="mic">
  {#snippet actions()}
    <StatusBadge status={recorderStatus} colorMap={recorderColors} />
  {/snippet}
  <!-- timer + LevelMeter + transport buttons -->
</Panel>`;

  const levelMeterCode = `<!-- level: instantaneous 0–100 · history: 48-slot rolling buffer -->
<LevelMeter {level} {history} />`;

  const statusBadgeCode = `const recorderColors: Record<string, string> = {
  recording: 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20',
  paused: 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20',
  standby: 'bg-slate-500/10 text-slate-600 dark:text-slate-400 border-slate-500/20',
};

<StatusBadge status={recorderStatus} colorMap={recorderColors} />`;

  const runButtonCode = `<RunButton loading={downloading} label="Download" onclick={downloadModel} />
<RunButton loading={running} disabled={!audioPath || !currentModel?.downloaded} label="Transcribe" onclick={transcribe} />`;

  const progressBarCode = `<!-- model download (asr:model-progress) and transcription (asr:progress) -->
<ProgressBar percent={downloadPercent} />`;

  const pickerCode = `<FilePicker bind:value={audioPath} label="Audio File" filters={[{ name: 'Audio', extensions: ['wav', 'mp3', 'flac', 'm4a', 'aac', 'mp4', 'ogg'] }]} />
<FilePicker bind:value={srtPath} label="SRT Subtitles (optional)" filters={[{ name: 'Subtitles', extensions: ['srt'] }]} />`;

  const formFieldCode = `<FormField label="Whisper Model" id="asr-model">
  <select id="asr-model" bind:value={selectedModel} class="{inputClass} w-full">
    {#each models as m (m.name)}
      <option value={m.name}>{m.name} ({m.approxMb} MB){m.downloaded ? ' ✓' : ''}</option>
    {/each}
  </select>
</FormField>`;

  const statCardCode = `<StatCard label="Duration" icon="schedule" iconColor="text-[#137fec]" value="3m 42s" />
<StatCard label="Sample Rate" icon="graphic_eq" iconColor="text-green-500" value="44.1 kHz" />`;

  const errorAlertCode = `{#if error}
  <ErrorAlert message={error} />
{/if}`;

  const emptyStateCode = `<EmptyState icon="mic" message="No recordings yet — hit Start Recording" />`;
</script>

<svelte:head>
  <title>Audio · Gallery</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <GallerySection
    id="composition"
    title="Composition"
    description="How the /audio route is assembled — three self-contained tabs: a Web-Audio recorder, a whisper.cpp transcriber, and a metadata inspector. Blue chips jump to that component's section below (or to its gallery page)."
  >
    <CompositionTree nodes={composition} />
  </GallerySection>

  <GallerySection
    id="tabbar"
    title="TabBar"
    description="Three tabs — Recorder, Transcribe, Metadata."
    link={{ href: '/gallery/tabbar', label: 'Component page' }}
  >
    <Showcase code={tabBarCode}>
      <div class="w-full">
        <TabBar tabs={demoTabs} activeTab={demoTab} onchange={(id) => (demoTab = id)} />
        <p class="text-meta mt-2">Active tab: {demoTab}</p>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="panel"
    title="Panel"
    description="The Recorder panel pins the live status badge in its actions snippet; Transcribe splits Source / Model / Transcribe into three panels."
    link={{ href: '/gallery/panel', label: 'Component page' }}
  >
    <Showcase code={panelCode}>
      <div class="w-full max-w-lg">
        <Panel title="Recorder" icon="mic">
          {#snippet actions()}
            <StatusBadge status="standby" colorMap={recorderColors} />
          {/snippet}
          <div class="p-4 flex flex-col gap-4">
            <div class="text-center">
              <span class="font-mono text-4xl font-bold tabular-nums text-slate-900 dark:text-white">00:00</span>
            </div>
            <LevelMeter level={0} history={Array(48).fill(0)} />
          </div>
        </Panel>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="levelmeter"
    title="LevelMeter"
    description="features/audio — the recorder's input meter: an instantaneous ProgressBar plus a 48-slot SparklineBar history, fed by an AnalyserNode at ~15 fps."
  >
    <Showcase code={levelMeterCode}>
      <div class="w-full max-w-md">
        <LevelMeter level={demoLevel} history={demoHistory} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statusbadge"
    title="StatusBadge"
    description="Two custom colorMaps: the recorder's recording / paused / standby states, and the Transcribe tab's model readiness."
    link={{ href: '/gallery/statusbadge', label: 'Component page' }}
  >
    <Showcase code={statusBadgeCode}>
      <StatusBadge status="recording" colorMap={recorderColors} />
      <StatusBadge status="paused" colorMap={recorderColors} />
      <StatusBadge status="standby" colorMap={recorderColors} />
      <StatusBadge status="ready" colorMap={modelColors} />
      <StatusBadge status="not downloaded" colorMap={modelColors} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="runbutton"
    title="RunButton"
    description="Download fetches the selected whisper model; Transcribe stays disabled until both an audio file and a downloaded model are picked."
    link={{ href: '/gallery/runbutton', label: 'Component page' }}
  >
    <Showcase code={runButtonCode}>
      <RunButton loading={false} label="Download" onclick={() => {}} />
      <RunButton loading={true} label="Transcribe" onclick={() => {}} />
      <RunButton loading={false} disabled label="Transcribe" onclick={() => {}} />
    </Showcase>
  </GallerySection>

  <GallerySection
    id="progressbar"
    title="ProgressBar"
    description="Driven by Tauri events — asr:model-progress during the chunked model download, asr:progress while whisper works through the audio."
    link={{ href: '/gallery/progressbar', label: 'Component page' }}
  >
    <Showcase code={progressBarCode}>
      <div class="w-full max-w-md flex flex-col gap-3">
        <div class="flex items-center justify-between">
          <span class="text-meta">Downloading base — 67 / ~148 MB</span>
          <span class="text-meta tabular-nums">45%</span>
        </div>
        <ProgressBar percent={45} />
        <div class="flex items-center justify-between">
          <span class="text-meta">Transcribing — live segments stream in below</span>
          <span class="text-meta tabular-nums">82%</span>
        </div>
        <ProgressBar percent={82} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="filepicker"
    title="FilePicker"
    description="components/form — Transcribe filters to symphonia-decodable formats (no Opus); Metadata adds an optional SRT picker. Browsing opens the Tauri dialog (in-app only)."
  >
    <Showcase code={pickerCode}>
      <div class="w-full max-w-md flex flex-col gap-3">
        <FilePicker bind:value={demoAudioPath} label="Audio File" filters={[{ name: 'Audio', extensions: ['wav', 'mp3', 'flac', 'm4a', 'aac', 'mp4', 'ogg'] }]} />
        <FilePicker bind:value={demoSrtPath} label="SRT Subtitles (optional)" filters={[{ name: 'Subtitles', extensions: ['srt'] }]} />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="formfield"
    title="FormField"
    description="The Transcribe tab's model picker (with size and a ✓ for downloaded models) and language select."
    link={{ href: '/gallery/formfield', label: 'Component page' }}
  >
    <Showcase code={formFieldCode}>
      <div class="w-full max-w-md grid grid-cols-1 sm:grid-cols-2 gap-4">
        <FormField label="Whisper Model" id="gallery-asr-model">
          <select id="gallery-asr-model" bind:value={demoModel} class="{inputClass} w-full">
            <option value="tiny">tiny (78 MB) ✓</option>
            <option value="base">base (148 MB)</option>
            <option value="small">small (488 MB)</option>
            <option value="large-v3-turbo">large-v3-turbo (1620 MB)</option>
          </select>
        </FormField>
        <FormField label="Language" id="gallery-asr-language">
          <select id="gallery-asr-language" bind:value={demoLanguage} class="{inputClass} w-full">
            <option value="auto">Auto-detect</option>
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </FormField>
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="statcard"
    title="StatCard"
    description="The Metadata tab's result row — duration, sample rate, bitrate, channels."
    link={{ href: '/gallery/statcard', label: 'Component page' }}
  >
    <Showcase code={statCardCode}>
      <div class="w-full grid grid-cols-2 lg:grid-cols-4 gap-3">
        <StatCard label="Duration" icon="schedule" iconColor="text-[#137fec]" value="3m 42s" />
        <StatCard label="Sample Rate" icon="graphic_eq" iconColor="text-green-500" value="44.1 kHz" />
        <StatCard label="Bitrate" icon="speed" iconColor="text-orange-500" value="128 kbps" />
        <StatCard label="Channels" icon="volume_up" iconColor="text-purple-500" value="Stereo" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="erroralert"
    title="ErrorAlert"
    description="Mic-permission failures, decode errors, and download problems all surface the same way."
    link={{ href: '/gallery/erroralert', label: 'Component page' }}
  >
    <Showcase code={errorAlertCode}>
      <div class="w-full">
        <ErrorAlert message="Microphone access failed: NotAllowedError: Permission denied" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="emptystate"
    title="EmptyState"
    description="Each tab brings its own icon and call to action."
    link={{ href: '/gallery/emptystate', label: 'Component page' }}
  >
    <Showcase code={emptyStateCode}>
      <div class="w-full flex flex-col gap-3">
        <EmptyState icon="mic" message="No recordings yet — hit Start Recording" />
        <EmptyState icon="graphic_eq" message="Pick an audio file and a downloaded model, then hit Transcribe" />
      </div>
    </Showcase>
  </GallerySection>

  <GallerySection
    id="live"
    title="Assembled result"
    description="The three tabs rendered whole. All are self-contained (no props); recorder capture, model downloads, and transcription need the Tauri backend."
  >
    <LiveComponent name="RecorderTab" note="no props"><RecorderTab /></LiveComponent>
    <LiveComponent name="TranscribeTab" note="no props"><TranscribeTab /></LiveComponent>
    <LiveComponent name="MetadataTab" note="no props"><MetadataTab /></LiveComponent>
  </GallerySection>
</div>
