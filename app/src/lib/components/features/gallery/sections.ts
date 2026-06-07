// Single source of truth for the gallery's own sidebar + the Overview page.
// "Pages" map to per-route catalogs (src/routes/gallery/pages/<id>/+page.svelte);
// "Tokens"/"Components" map to the base design-system showcases (src/routes/gallery/<id>/).

export interface GalleryLink {
  id: string;
  label: string;
  href: string;
}

export interface GalleryGroup {
  title: string;
  items: GalleryLink[];
}

export const galleryGroups: GalleryGroup[] = [
  {
    title: 'Pages',
    items: [
      { id: 'dashboard', label: 'Dashboard', href: '/gallery/pages/dashboard' },
      { id: 'streams', label: 'Streams', href: '/gallery/pages/streams' },
      { id: 'analysis', label: 'Analysis', href: '/gallery/pages/analysis' },
      { id: 'processing', label: 'Processing', href: '/gallery/pages/processing' },
      { id: 'camera', label: 'Camera', href: '/gallery/pages/camera' },
      { id: 'benchmark', label: 'Benchmark', href: '/gallery/pages/benchmark' },
      { id: 'dedup', label: 'Dedup', href: '/gallery/pages/dedup' },
      { id: 'demo-360', label: '360° Demo', href: '/gallery/pages/demo-360' },
      { id: 'settings', label: 'Settings', href: '/gallery/pages/settings' },
    ],
  },
  {
    title: 'Tokens',
    items: [
      { id: 'typography', label: 'Typography', href: '/gallery/typography' },
      { id: 'colors', label: 'Colors', href: '/gallery/colors' },
      { id: 'fonts', label: 'Fonts', href: '/gallery/fonts' },
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
];

/** Flat, ordered list of every gallery section. */
export const gallerySections: GalleryLink[] = galleryGroups.flatMap((g) => g.items);

/** Resolve the human label for a pathname like "/gallery/card" or "/gallery/pages/dashboard". */
export function galleryLabel(pathname: string): string | undefined {
  return gallerySections.find((s) => s.href === pathname)?.label;
}
