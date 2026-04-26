import { getContext, setContext, onDestroy, type Snippet } from 'svelte';

const HEADER_ACTIONS_KEY = Symbol('header-actions');

class HeaderActionsStore {
    current = $state<Snippet | null>(null);
}

export function provideHeaderActions(): HeaderActionsStore {
    const store = new HeaderActionsStore();
    setContext(HEADER_ACTIONS_KEY, store);
    return store;
}

/**
 * Register a snippet to render in the header's right-hand action area.
 * Must be called during a page component's setup (synchronously in <script>).
 * The snippet is automatically cleared when the calling component unmounts.
 */
export function setHeaderActions(snippet: Snippet): void {
    const store = getContext<HeaderActionsStore | undefined>(HEADER_ACTIONS_KEY);
    if (!store) return;
    store.current = snippet;
    onDestroy(() => {
        if (store.current === snippet) store.current = null;
    });
}
