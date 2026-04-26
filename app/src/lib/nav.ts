export interface NavItem {
    href: string;
    icon: string;
    /** Short label used in the sidebar */
    label: string;
    /** Descriptive title used in the header breadcrumb */
    title: string;
    match: (pathname: string) => boolean;
}

export const navItems: NavItem[] = [
    { href: '/',          icon: 'dashboard',             label: 'Dashboard', title: 'Dashboard',       match: (p) => p === '/' },
    { href: '/streams',   icon: 'videocam',              label: 'Streams',   title: 'Streams',         match: (p) => p.startsWith('/streams') },
    { href: '/analysis',  icon: 'query_stats',           label: 'Analysis',  title: 'Media Analysis',  match: (p) => p.startsWith('/analysis') },
    { href: '/processing',icon: 'conversion_path',       label: 'Processing',title: 'Processing',      match: (p) => p.startsWith('/processing') },
    { href: '/camera',    icon: 'photo_camera',          label: 'Camera',    title: 'Camera Capture',  match: (p) => p.startsWith('/camera') },
    { href: '/benchmark', icon: 'speed',                 label: 'Benchmark', title: 'Benchmark',       match: (p) => p.startsWith('/benchmark') },
    { href: '/dedup',     icon: 'fingerprint',           label: 'Dedup',     title: 'Deduplication',   match: (p) => p.startsWith('/dedup') },
    { href: '/demo-360',  icon: 'panorama_photosphere',  label: '360° Demo', title: '360° Demo',       match: (p) => p.startsWith('/demo-360') },
    { href: '/settings',  icon: 'settings',              label: 'Settings',  title: 'Settings',        match: (p) => p.startsWith('/settings') },
];

export interface BreadcrumbSegment {
    label: string;
    href: string;
}

export function buildBreadcrumb(pathname: string): BreadcrumbSegment[] {
    if (pathname === '/') {
        return [{ label: 'Dashboard', href: '/' }];
    }

    const topLevel = navItems.find((item) => item.href !== '/' && item.match(pathname));
    if (!topLevel) {
        return [{ label: 'Media Core', href: pathname }];
    }

    const segments: BreadcrumbSegment[] = [{ label: topLevel.title, href: topLevel.href }];

    const remaining = pathname.slice(topLevel.href.length).replace(/^\/+/, '');
    if (remaining) {
        const subParts = remaining.split('/').filter(Boolean);
        let currentPath = topLevel.href;
        for (const part of subParts) {
            currentPath += '/' + part;
            segments.push({ label: decodeURIComponent(part), href: currentPath });
        }
    }

    return segments;
}
