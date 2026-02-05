<script lang="ts">
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

<div class="p-8 max-w-[1400px] mx-auto space-y-8">
  <div class="flex items-end justify-between gap-6 flex-wrap">
    <div class="flex flex-col gap-2">
      <h1 class="text-3xl font-bold text-slate-900 dark:text-white font-display">Launcher</h1>
      <p class="text-slate-500 dark:text-slate-400">
        Quick access to every workspace and tool.
      </p>
    </div>
  </div>

  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
    {#each pages as page}
      <a
        href={page.href}
        class="group rounded-xl border border-slate-200 dark:border-[#283039] bg-white dark:bg-[#182129] p-5 shadow-sm hover:shadow-lg hover:border-[#137fec]/40 transition-all flex flex-col gap-4"
      >
        <div class="flex items-center justify-between">
          <div class={`size-10 rounded-lg ${page.accent} flex items-center justify-center`}>
            <span class="material-symbols-outlined text-[22px]">{page.icon}</span>
          </div>
          <span class="text-[10px] font-bold uppercase tracking-widest text-slate-400 group-hover:text-slate-600 dark:group-hover:text-slate-300">
            Open
          </span>
        </div>
        <div class="flex flex-col gap-1">
          <h3 class="text-lg font-bold font-display text-slate-900 dark:text-white">{page.name}</h3>
          <p class="text-sm text-slate-500 dark:text-slate-400">{page.description}</p>
        </div>
        <div class="flex items-center justify-between text-xs font-mono text-slate-400">
          <span>{page.href}</span>
          <span class="material-symbols-outlined text-[18px] text-slate-400 group-hover:text-[#137fec]">
            arrow_forward
          </span>
        </div>
      </a>
    {/each}
  </div>
</div>
