// Single source of truth for the gallery's own sidebar + the Overview page.
// "Pages" map to per-route catalogs (src/routes/gallery/pages/<id>/+page.svelte);
// "Tokens"/"Components" map to the base design-system showcases (src/routes/gallery/<id>/).

export interface GalleryLink {
  id: string;
  label: string;
  href: string;
  /** Sub-items shown when the entry is expanded in the sidebar — e.g. the
   *  components a page is composed of, anchoring into that page's catalog. */
  children?: GalleryLink[];
}

export interface GalleryGroup {
  title: string;
  items: GalleryLink[];
}

export const galleryGroups: GalleryGroup[] = [
  {
    title: 'Pages',
    items: [
      {
        id: 'dashboard',
        label: 'Dashboard',
        href: '/gallery/pages/dashboard',
        children: [
          { id: 'dashboard-tabbar', label: 'TabBar', href: '/gallery/pages/dashboard#tabbar' },
          { id: 'dashboard-statcard', label: 'StatCard', href: '/gallery/pages/dashboard#statcard' },
          { id: 'dashboard-sparklinebar', label: 'SparklineBar', href: '/gallery/pages/dashboard#sparklinebar' },
          { id: 'dashboard-progressbar', label: 'ProgressBar', href: '/gallery/pages/dashboard#progressbar' },
          { id: 'dashboard-panel', label: 'Panel', href: '/gallery/pages/dashboard#panel' },
          { id: 'dashboard-throughputchart', label: 'ThroughputChart', href: '/gallery/pages/dashboard#throughputchart' },
          { id: 'dashboard-statusbadge', label: 'StatusBadge', href: '/gallery/pages/dashboard#statusbadge' },
        ],
      },
      {
        id: 'streams',
        label: 'Streams',
        href: '/gallery/pages/streams',
        children: [
          { id: 'streams-tabbar', label: 'TabBar', href: '/gallery/pages/streams#tabbar' },
          { id: 'streams-panel', label: 'Panel', href: '/gallery/pages/streams#panel' },
          { id: 'streams-statcard', label: 'StatCard', href: '/gallery/pages/streams#statcard' },
          { id: 'streams-statusbadge', label: 'StatusBadge', href: '/gallery/pages/streams#statusbadge' },
          { id: 'streams-formfield', label: 'FormField', href: '/gallery/pages/streams#formfield' },
          { id: 'streams-toggleswitch', label: 'ToggleSwitch', href: '/gallery/pages/streams#toggleswitch' },
          { id: 'streams-sparklinebar', label: 'SparklineBar', href: '/gallery/pages/streams#sparklinebar' },
          { id: 'streams-videoplayer', label: 'VideoPlayer', href: '/gallery/pages/streams#videoplayer' },
        ],
      },
      {
        id: 'analysis',
        label: 'Analysis',
        href: '/gallery/pages/analysis',
        children: [
          { id: 'analysis-tabbar', label: 'TabBar', href: '/gallery/pages/analysis#tabbar' },
          { id: 'analysis-panel', label: 'Panel', href: '/gallery/pages/analysis#panel' },
          { id: 'analysis-runbutton', label: 'RunButton', href: '/gallery/pages/analysis#runbutton' },
          { id: 'analysis-formfield', label: 'FormField', href: '/gallery/pages/analysis#formfield' },
          { id: 'analysis-filepicker', label: 'FilePicker / DirPicker', href: '/gallery/pages/analysis#filepicker' },
          { id: 'analysis-statusbadge', label: 'StatusBadge', href: '/gallery/pages/analysis#statusbadge' },
          { id: 'analysis-erroralert', label: 'ErrorAlert', href: '/gallery/pages/analysis#erroralert' },
          { id: 'analysis-emptystate', label: 'EmptyState', href: '/gallery/pages/analysis#emptystate' },
        ],
      },
      {
        id: 'processing',
        label: 'Processing',
        href: '/gallery/pages/processing',
        children: [
          { id: 'processing-tabbar', label: 'TabBar', href: '/gallery/pages/processing#tabbar' },
          { id: 'processing-panel', label: 'Panel', href: '/gallery/pages/processing#panel' },
          { id: 'processing-statcard', label: 'StatCard', href: '/gallery/pages/processing#statcard' },
          { id: 'processing-runbutton', label: 'RunButton', href: '/gallery/pages/processing#runbutton' },
          { id: 'processing-progressbar', label: 'ProgressBar', href: '/gallery/pages/processing#progressbar' },
          { id: 'processing-statusbadge', label: 'StatusBadge', href: '/gallery/pages/processing#statusbadge' },
          { id: 'processing-toggleswitch', label: 'ToggleSwitch', href: '/gallery/pages/processing#toggleswitch' },
          { id: 'processing-formfield', label: 'FormField', href: '/gallery/pages/processing#formfield' },
        ],
      },
      {
        id: 'camera',
        label: 'Camera',
        href: '/gallery/pages/camera',
        children: [
          { id: 'camera-panel', label: 'Panel', href: '/gallery/pages/camera#panel' },
          { id: 'camera-statusbadge', label: 'StatusBadge', href: '/gallery/pages/camera#statusbadge' },
          { id: 'camera-erroralert', label: 'ErrorAlert', href: '/gallery/pages/camera#erroralert' },
        ],
      },
      {
        id: 'audio',
        label: 'Audio',
        href: '/gallery/pages/audio',
        children: [
          { id: 'audio-tabbar', label: 'TabBar', href: '/gallery/pages/audio#tabbar' },
          { id: 'audio-panel', label: 'Panel', href: '/gallery/pages/audio#panel' },
          { id: 'audio-levelmeter', label: 'LevelMeter', href: '/gallery/pages/audio#levelmeter' },
          { id: 'audio-statusbadge', label: 'StatusBadge', href: '/gallery/pages/audio#statusbadge' },
          { id: 'audio-runbutton', label: 'RunButton', href: '/gallery/pages/audio#runbutton' },
          { id: 'audio-progressbar', label: 'ProgressBar', href: '/gallery/pages/audio#progressbar' },
          { id: 'audio-filepicker', label: 'FilePicker', href: '/gallery/pages/audio#filepicker' },
          { id: 'audio-formfield', label: 'FormField', href: '/gallery/pages/audio#formfield' },
          { id: 'audio-statcard', label: 'StatCard', href: '/gallery/pages/audio#statcard' },
          { id: 'audio-erroralert', label: 'ErrorAlert', href: '/gallery/pages/audio#erroralert' },
          { id: 'audio-emptystate', label: 'EmptyState', href: '/gallery/pages/audio#emptystate' },
        ],
      },
      {
        id: 'benchmark',
        label: 'Benchmark',
        href: '/gallery/pages/benchmark',
        children: [
          { id: 'benchmark-statcard', label: 'StatCard', href: '/gallery/pages/benchmark#statcard' },
          { id: 'benchmark-panel', label: 'Panel', href: '/gallery/pages/benchmark#panel' },
          { id: 'benchmark-runbutton', label: 'RunButton', href: '/gallery/pages/benchmark#runbutton' },
          { id: 'benchmark-filepicker', label: 'FilePicker', href: '/gallery/pages/benchmark#filepicker' },
          { id: 'benchmark-formfield', label: 'FormField', href: '/gallery/pages/benchmark#formfield' },
          { id: 'benchmark-erroralert', label: 'ErrorAlert', href: '/gallery/pages/benchmark#erroralert' },
          { id: 'benchmark-emptystate', label: 'EmptyState', href: '/gallery/pages/benchmark#emptystate' },
        ],
      },
      {
        id: 'dedup',
        label: 'Dedup',
        href: '/gallery/pages/dedup',
        children: [
          { id: 'dedup-panel', label: 'Panel', href: '/gallery/pages/dedup#panel' },
          { id: 'dedup-toggleswitch', label: 'ToggleSwitch', href: '/gallery/pages/dedup#toggleswitch' },
          { id: 'dedup-runbutton', label: 'RunButton', href: '/gallery/pages/dedup#runbutton' },
          { id: 'dedup-progressbar', label: 'ProgressBar', href: '/gallery/pages/dedup#progressbar' },
          { id: 'dedup-emptystate', label: 'EmptyState', href: '/gallery/pages/dedup#emptystate' },
          { id: 'dedup-erroralert', label: 'ErrorAlert', href: '/gallery/pages/dedup#erroralert' },
        ],
      },
      {
        id: 'demo-360',
        label: '360° Demo',
        href: '/gallery/pages/demo-360',
        children: [
          { id: 'demo-360-tabbar', label: 'TabBar', href: '/gallery/pages/demo-360#tabbar' },
          { id: 'demo-360-panel', label: 'Panel', href: '/gallery/pages/demo-360#panel' },
          { id: 'demo-360-toggleswitch', label: 'ToggleSwitch', href: '/gallery/pages/demo-360#toggleswitch' },
        ],
      },
      {
        id: 'settings',
        label: 'Settings',
        href: '/gallery/pages/settings',
        children: [
          { id: 'settings-panel', label: 'Panel', href: '/gallery/pages/settings#panel' },
          { id: 'settings-formfield', label: 'FormField', href: '/gallery/pages/settings#formfield' },
          { id: 'settings-toggleswitch', label: 'ToggleSwitch', href: '/gallery/pages/settings#toggleswitch' },
          { id: 'settings-statusbadge', label: 'StatusBadge', href: '/gallery/pages/settings#statusbadge' },
        ],
      },
    ],
  },
  {
    title: 'Tokens',
    items: [
      { id: 'typography', label: 'Typography', href: '/gallery/typography' },
      { id: 'colors', label: 'Colors', href: '/gallery/colors' },
      { id: 'fonts', label: 'Fonts', href: '/gallery/fonts' },
      { id: 'icons', label: 'Icons', href: '/gallery/icons' },
    ],
  },
  {
    title: 'Components',
    items: [
      { id: 'card', label: 'Card', href: '/gallery/card' },
      { id: 'panel', label: 'Panel', href: '/gallery/panel' },
      { id: 'statcard', label: 'StatCard', href: '/gallery/statcard' },
      { id: 'statusbadge', label: 'StatusBadge', href: '/gallery/statusbadge' },
      { id: 'progressbar', label: 'ProgressBar', href: '/gallery/progressbar' },
      { id: 'erroralert', label: 'ErrorAlert', href: '/gallery/erroralert' },
      { id: 'emptystate', label: 'EmptyState', href: '/gallery/emptystate' },
      { id: 'toggleswitch', label: 'ToggleSwitch', href: '/gallery/toggleswitch' },
      { id: 'runbutton', label: 'RunButton', href: '/gallery/runbutton' },
      { id: 'formfield', label: 'FormField', href: '/gallery/formfield' },
      { id: 'tabbar', label: 'TabBar', href: '/gallery/tabbar' },
      { id: 'breadcrumb', label: 'Breadcrumb', href: '/gallery/breadcrumb' },
      { id: 'pagecontent', label: 'PageContent', href: '/gallery/pagecontent' },
    ],
  },
  {
    title: 'shadcn-svelte',
    items: [
      { id: 'shadcn-button', label: 'Button', href: '/gallery/shadcn/button' },
      { id: 'shadcn-dialog', label: 'Dialog', href: '/gallery/shadcn/dialog' },
    ],
  },
];

/** Flat, ordered list of every gallery section. */
export const gallerySections: GalleryLink[] = galleryGroups.flatMap((g) => g.items);

/** Resolve the human label for a pathname like "/gallery/card" or "/gallery/pages/dashboard". */
export function galleryLabel(pathname: string): string | undefined {
  return gallerySections.find((s) => s.href === pathname)?.label;
}
