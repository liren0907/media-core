<script>
	import { page } from '$app/stores';
</script>

<header>
	<nav class="glass-card-static border-b border-white/50 sticky top-0 z-50 bg-white/40 rounded-none">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex justify-between h-20">
				<div class="flex-shrink-0 flex items-center">
					<span class="text-2xl font-bold">
						<!-- Warm Gradient: Orange -> Pink -> Purple -->
						<span class="bg-gradient-to-r from-orange-500 via-pink-500 to-purple-500 bg-clip-text text-transparent">Stream Viewer</span>
					</span>
				</div>
				<div class="hidden sm:flex sm:items-center sm:space-x-4">
					<a
						href="/"
						class="nav-link {$page.url.pathname === '/' ? 'active' : ''}"
					>
						Home
					</a>
					<a
						href="/viewVideo"
						class="nav-link {$page.url.pathname.startsWith('/viewVideo') ? 'active' : ''}"
					>
						Video Viewer
					</a>
					<a
						href="/stream"
						class="nav-link {$page.url.pathname.startsWith('/stream') ? 'active' : ''}"
					>
						Stream
					</a>
					<a
						href="/hlsViewer"
						class="nav-link {$page.url.pathname.startsWith('/hlsViewer') ? 'active' : ''}"
					>
						HLS Viewer
					</a>
				</div>
			</div>
		</div>

		<!-- Mobile menu -->
		<div class="sm:hidden px-4 pb-4">
			<div class="flex flex-wrap gap-2 justify-center">
				<a
					href="/"
					class="nav-link-mobile {$page.url.pathname === '/' ? 'active' : ''}"
				>
					Home
				</a>
				<a
					href="/viewVideo"
					class="nav-link-mobile {$page.url.pathname.startsWith('/viewVideo') ? 'active' : ''}"
				>
					Video Viewer
				</a>
				<a
					href="/stream"
					class="nav-link-mobile {$page.url.pathname.startsWith('/stream') ? 'active' : ''}"
				>
					Stream
				</a>
				<a
					href="/hlsViewer"
					class="nav-link-mobile {$page.url.pathname.startsWith('/hlsViewer') ? 'active' : ''}"
				>
					HLS Viewer
				</a>
			</div>
		</div>
	</nav>
</header>