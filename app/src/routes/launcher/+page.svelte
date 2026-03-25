<script lang="ts">
  import { PageContent, Card } from '$lib/components/ui';

  const pages = [
    {
      name: "Dashboard",
      description: "System overview, metrics, and operational status.",
      href: "/",
      icon: "dashboard",
      accent: "bg-[#137fec]/10 text-[#137fec]",
    },
    {
      name: "Monitor",
      description: "Live command-style monitoring and stream overview.",
      href: "/monitor",
      icon: "monitor",
      accent: "bg-blue-500/10 text-blue-500",
    },
    {
      name: "Streams",
      description: "RTSP ingestion, HLS playback, and stream controls.",
      href: "/stream",
      icon: "videocam",
      accent: "bg-emerald-500/10 text-emerald-500",
    },
    {
      name: "Analytics",
      description: "Inference and analytics console with detection tools.",
      href: "/inferencer",
      icon: "bar_chart",
      accent: "bg-indigo-500/10 text-indigo-500",
    },
    {
      name: "Annotator",
      description: "Video annotation workflow and label selection.",
      href: "/annotator",
      icon: "edit_square",
      accent: "bg-purple-500/10 text-purple-500",
    },
    {
      name: "Analysis",
      description: "Motion detection, similarity grouping, and image comparison.",
      href: "/analysis",
      icon: "query_stats",
      accent: "bg-pink-500/10 text-pink-500",
    },
    {
      name: "Pipeline",
      description: "Batch processing with progress tracking and cancellation.",
      href: "/pipeline",
      icon: "conversion_path",
      accent: "bg-teal-500/10 text-teal-500",
    },
    {
      name: "Extractor",
      description: "Extract frames from video with multiple sampling strategies.",
      href: "/frame-extractor",
      icon: "photo_library",
      accent: "bg-lime-500/10 text-lime-500",
    },
    {
      name: "Benchmark",
      description: "Performance profiling for video processing operations.",
      href: "/benchmark",
      icon: "speed",
      accent: "bg-red-500/10 text-red-500",
    },
    {
      name: "Player",
      description: "HLS viewer and playback diagnostics.",
      href: "/hlsViewer",
      icon: "play_circle",
      accent: "bg-rose-500/10 text-rose-500",
    },
    {
      name: "Local Player",
      description: "Local video playback and metadata inspection.",
      href: "/viewVideo",
      icon: "movie",
      accent: "bg-amber-500/10 text-amber-500",
    },
    {
      name: "Multi Stream",
      description: "Parallel stream preview grid and control panel.",
      href: "/multi-stream-viewer",
      icon: "grid_view",
      accent: "bg-cyan-500/10 text-cyan-500",
    },
    {
      name: "Settings",
      description: "Theme preferences and global app configuration.",
      href: "/settings",
      icon: "settings",
      accent: "bg-slate-500/10 text-slate-500",
    },
  ];
</script>

<svelte:head>
  <title>Launcher | Media Core</title>
</svelte:head>

<PageContent>
  <div class="flex items-end justify-between gap-6 flex-wrap">
    <div class="flex flex-col gap-1">
      <h1 class="text-xl font-bold text-slate-900 dark:text-white font-display">Launcher</h1>
      <p class="text-xs text-slate-500 dark:text-slate-400">Quick access to every workspace and tool.</p>
    </div>
  </div>

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
    {#each pages as page}
      <a href={page.href} class="group">
        <Card>
          <div class="flex items-center justify-between">
            <div class={`size-9 rounded-lg ${page.accent} flex items-center justify-center`}>
              <span class="material-symbols-outlined text-[20px]">{page.icon}</span>
            </div>
            <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400 group-hover:text-[#137fec]">Open</span>
          </div>
          <div class="flex flex-col gap-0.5 mt-3">
            <h3 class="text-sm font-bold font-display text-slate-900 dark:text-white">{page.name}</h3>
            <p class="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">{page.description}</p>
          </div>
          <div class="flex items-center justify-between text-[10px] font-mono text-slate-400 mt-2">
            <span>{page.href}</span>
            <span class="material-symbols-outlined text-[16px] text-slate-400 group-hover:text-[#137fec]">arrow_forward</span>
          </div>
        </Card>
      </a>
    {/each}
  </div>
</PageContent>
