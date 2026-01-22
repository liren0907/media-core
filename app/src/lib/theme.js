import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// Initial value from localStorage or default to 'auto'
const storedTheme = browser ? localStorage.getItem('theme') : 'auto';
const initialTheme = storedTheme || 'auto';

export const theme = writable(initialTheme);

// Derived store to get the actual effective theme (light/dark)
// This is useful for UI that needs to know the *current* visual state, not just the setting
export const effectiveTheme = writable('light');

if (browser) {
    // Function to apply the theme to the document
    const applyTheme = (value) => {
        let isDark = false;
        
        if (value === 'auto') {
            isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        } else {
            isDark = value === 'dark';
        }
        
        if (isDark) {
            document.documentElement.classList.add('dark');
            effectiveTheme.set('dark');
        } else {
            document.documentElement.classList.remove('dark');
            effectiveTheme.set('light');
        }
    };

    // Subscribe to changes
    theme.subscribe(value => {
        localStorage.setItem('theme', value);
        applyTheme(value);
    });

    // Listen for system preference changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        // Only react if we are in auto mode
        // We can't easily access the store value inside this callback without get(), 
        // so we check localStorage which is synced.
        if (localStorage.getItem('theme') === 'auto') {
             applyTheme('auto');
        }
    });
}
