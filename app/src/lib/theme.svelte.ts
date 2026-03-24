import { browser } from '$app/environment';

const storedTheme = browser ? localStorage.getItem('theme') : 'auto';

let themeSetting = $state(storedTheme || 'auto');

let effective = $state('light');

export const theme = {
	get value() { return themeSetting; },
	set(val: string) { themeSetting = val; }
};

export const effectiveTheme = {
	get value() { return effective; }
};

if (browser) {
	const applyTheme = (value: string) => {
		let isDark = false;

		if (value === 'auto') {
			isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
		} else {
			isDark = value === 'dark';
		}

		if (isDark) {
			document.documentElement.classList.add('dark');
			effective = 'dark';
		} else {
			document.documentElement.classList.remove('dark');
			effective = 'light';
		}
	};

	$effect.root(() => {
		$effect(() => {
			localStorage.setItem('theme', themeSetting);
			applyTheme(themeSetting);
		});
	});

	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
		if (localStorage.getItem('theme') === 'auto') {
			applyTheme('auto');
		}
	});
}
