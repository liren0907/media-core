<script lang="ts">
  import * as THREE from 'three';
  import { untrack } from 'svelte';

  interface Props {
    source: HTMLImageElement | HTMLVideoElement | null;
    fov?: number;
    autoRotate?: boolean;
    dragSensitivity?: number;
    resetToken?: number;
    onView?: (state: { lon: number; lat: number; fov: number }) => void;
  }

  let {
    source,
    fov = $bindable(75),
    autoRotate = false,
    dragSensitivity = 0.6,
    resetToken = 0,
    onView,
  }: Props = $props();

  let container: HTMLDivElement;
  let canvas: HTMLCanvasElement;

  let renderer: THREE.WebGLRenderer | null = null;
  let scene: THREE.Scene | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let mesh: THREE.Mesh | null = null;
  let texture: THREE.Texture | null = null;
  let rafId: number | null = null;
  let resizeObserver: ResizeObserver | null = null;

  let lon = 0;
  let lat = 0;
  let isDragging = $state(false);
  let pointerStartX = 0;
  let pointerStartY = 0;
  let lonStart = 0;
  let latStart = 0;

  const FOV_MIN = 30;
  const FOV_MAX = 90;
  const LAT_CLAMP = 85;
  const AUTO_ROTATE_SPEED = 0.05;

  $effect(() => {
    if (!canvas || !container) return;

    untrack(() => {
      renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
      renderer.setPixelRatio(window.devicePixelRatio);

      scene = new THREE.Scene();
      camera = new THREE.PerspectiveCamera(fov, 1, 0.1, 1100);
      camera.position.set(0, 0, 0);

      const geometry = new THREE.SphereGeometry(500, 60, 40);
      geometry.scale(-1, 1, 1);
      const material = new THREE.MeshBasicMaterial({ color: 0x111111 });
      mesh = new THREE.Mesh(geometry, material);
      scene.add(mesh);

      resizeObserver = new ResizeObserver(handleResize);
      resizeObserver.observe(container);
      handleResize();

      animate();
    });

    return () => {
      if (rafId !== null) cancelAnimationFrame(rafId);
      resizeObserver?.disconnect();
      disposeTexture();
      mesh?.geometry.dispose();
      (mesh?.material as THREE.Material)?.dispose();
      renderer?.dispose();
      renderer = null;
      scene = null;
      camera = null;
      mesh = null;
    };
  });

  $effect(() => {
    if (!mesh) return;
    applySource(source);
  });

  $effect(() => {
    if (!camera) return;
    const clamped = Math.max(FOV_MIN, Math.min(FOV_MAX, fov));
    camera.fov = clamped;
    camera.updateProjectionMatrix();
  });

  let firstResetRun = true;
  $effect(() => {
    void resetToken;
    if (firstResetRun) {
      firstResetRun = false;
      return;
    }
    untrack(() => {
      lon = 0;
      lat = 0;
      fov = 75;
    });
  });

  function applySource(src: HTMLImageElement | HTMLVideoElement | null) {
    if (!mesh) return;
    disposeTexture();

    if (!src) {
      (mesh.material as THREE.MeshBasicMaterial).map = null;
      (mesh.material as THREE.MeshBasicMaterial).color.set(0x111111);
      (mesh.material as THREE.MeshBasicMaterial).needsUpdate = true;
      return;
    }

    if (src instanceof HTMLVideoElement) {
      texture = new THREE.VideoTexture(src);
    } else {
      texture = new THREE.Texture(src);
      texture.needsUpdate = true;
    }
    texture.colorSpace = THREE.SRGBColorSpace;
    texture.minFilter = THREE.LinearFilter;
    texture.magFilter = THREE.LinearFilter;
    texture.generateMipmaps = false;

    const mat = mesh.material as THREE.MeshBasicMaterial;
    mat.map = texture;
    mat.color.set(0xffffff);
    mat.needsUpdate = true;
  }

  function disposeTexture() {
    if (texture) {
      texture.dispose();
      texture = null;
    }
  }

  function handleResize() {
    if (!renderer || !camera || !container) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w === 0 || h === 0) return;
    renderer.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  function animate() {
    rafId = requestAnimationFrame(animate);
    if (!renderer || !scene || !camera) return;

    if (autoRotate && !isDragging) {
      lon += AUTO_ROTATE_SPEED;
    }

    lat = Math.max(-LAT_CLAMP, Math.min(LAT_CLAMP, lat));
    const phi = THREE.MathUtils.degToRad(90 - lat);
    const theta = THREE.MathUtils.degToRad(lon);
    const tx = 500 * Math.sin(phi) * Math.cos(theta);
    const ty = 500 * Math.cos(phi);
    const tz = 500 * Math.sin(phi) * Math.sin(theta);
    camera.lookAt(tx, ty, tz);

    renderer.render(scene, camera);
    onView?.({ lon, lat, fov: camera.fov });
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    isDragging = true;
    pointerStartX = e.clientX;
    pointerStartY = e.clientY;
    lonStart = lon;
    latStart = lat;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!isDragging || !camera || !container) return;
    const dx = e.clientX - pointerStartX;
    const dy = e.clientY - pointerStartY;
    const h = container.clientHeight || 1;
    const degPerPx = (camera.fov / h) * dragSensitivity;
    lon = lonStart - dx * degPerPx;
    lat = latStart + dy * degPerPx;
  }

  function onPointerUp(e: PointerEvent) {
    isDragging = false;
    try {
      (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {}
  }

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const next = fov + e.deltaY * 0.05;
    fov = Math.max(FOV_MIN, Math.min(FOV_MAX, next));
  }
</script>

<div
  bind:this={container}
  class="relative w-full h-full bg-black rounded overflow-hidden"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
  role="application"
  aria-label="360 degree panoramic viewer"
>
  <canvas bind:this={canvas} class="block w-full h-full touch-none select-none {isDragging ? 'cursor-grabbing' : 'cursor-grab'}"></canvas>
  {#if !source}
    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
      <div class="text-center text-muted">
        <span class="material-symbols-outlined text-4xl mb-2">panorama_photosphere</span>
        <p class="text-xs">Load a 360° image or video to begin</p>
      </div>
    </div>
  {/if}
</div>
