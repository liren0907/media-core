/**
 * Application Configuration Store
 *
 * Centralized store for configurable paths, URLs, and settings.
 * Values are persisted to localStorage and can be modified in Settings.
 */

import { browser } from '$app/environment';

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

export interface StreamConfig {
  hlsServerUrl: string;
  hlsServerPort: number;
  defaultRtspUrl: string;
  hlsOutputDir: string;
}

export interface PathConfig {
  defaultVideoDir: string;
  defaultImageDir: string;
  defaultOutputDir: string;
  annotationDir: string;
}

export interface AppConfig {
  streams: StreamConfig;
  paths: PathConfig;
}

// ============================================================================
// DEFAULT CONFIGURATION
// ============================================================================

const DEFAULT_CONFIG: AppConfig = {
  streams: {
    hlsServerUrl: 'http://127.0.0.1',
    hlsServerPort: 1521,
    defaultRtspUrl: 'rtsp://localhost:8554/mystream',
    hlsOutputDir: 'hls_output',
  },
  paths: {
    defaultVideoDir: '',
    defaultImageDir: '',
    defaultOutputDir: '',
    annotationDir: '',
  },
};

const STORAGE_KEY = 'media-app-config';

// ============================================================================
// CONFIGURATION STORE
// ============================================================================

function loadInitialConfig(): AppConfig {
  const stored = browser ? localStorage.getItem(STORAGE_KEY) : null;
  try {
    return stored ? { ...DEFAULT_CONFIG, ...JSON.parse(stored) } : DEFAULT_CONFIG;
  } catch {
    return DEFAULT_CONFIG;
  }
}

function persist(config: AppConfig) {
  if (browser) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  }
}

let config = $state<AppConfig>(loadInitialConfig());

export const appConfig = {
  get current(): AppConfig { return config; },

  updateStreams(streams: Partial<StreamConfig>) {
    config = {
      ...config,
      streams: { ...config.streams, ...streams }
    };
    persist(config);
  },

  updatePaths(paths: Partial<PathConfig>) {
    config = {
      ...config,
      paths: { ...config.paths, ...paths }
    };
    persist(config);
  },

  reset() {
    config = DEFAULT_CONFIG;
    if (browser) {
      localStorage.removeItem(STORAGE_KEY);
    }
  },
};

// ============================================================================
// DERIVED VALUES
// ============================================================================

export function getHlsBaseUrl(): string {
  return `${config.streams.hlsServerUrl}:${config.streams.hlsServerPort}`;
}

/**
 * Build a playlist URL for a specific stream
 */
export function getPlaylistUrl(streamName: string = 'stream_0'): string {
  return `${config.streams.hlsServerUrl}:${config.streams.hlsServerPort}/${streamName}/playlist.m3u8`;
}

/**
 * Get HLS output directory path
 */
export function getHlsOutputDir(): string {
  return config.streams.hlsOutputDir;
}

/**
 * Get default RTSP URL
 */
export function getDefaultRtspUrl(): string {
  return config.streams.defaultRtspUrl;
}
