<script lang="ts">
  interface Props {
    status: string;
    colorMap?: Record<string, string>;
  }

  const defaultColors: Record<string, string> = {
    active: 'bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20',
    idle: 'bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20',
    error: 'bg-red-500/10 text-red-600 dark:text-red-400 border-red-500/20',
    connecting: 'bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 border-yellow-500/20',
  };

  let { status, colorMap }: Props = $props();

  let colors = $derived((colorMap ?? defaultColors)[status.toLowerCase()] ?? 'bg-slate-500/10 text-muted border-slate-500/20');
</script>

<span class="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded text-badge border uppercase tracking-wide {colors}">
  {status}
</span>
