/**
 * Application Configuration Store
 * 
 * Centralized store for configurable paths, URLs, and settings.
 * Values are persisted to localStorage and can be modified in Settings.
 */

import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

export interface StreamConfig {
  hlsServerUrl: string;      // Base URL for HLS server (e.g., http://127.0.0.1:1521)
  hlsServerPort: number;     // HLS server port
  defaultRtspUrl: string;    // Default RTSP URL for testing
  hlsOutputDir: string;      // Directory for HLS output (relative or absolute)
}

export interface PathConfig {
  defaultVideoDir: string;   // Default directory for video files
  defaultImageDir: string;   // Default directory for image files
  defaultOutputDir: string;  // Default directory for output/exports
  annotationDir: string;     // Directory for annotation files
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

function createConfigStore() {
  // Load from localStorage if available
  const stored = browser ? localStorage.getItem(STORAGE_KEY) : null;
  let initial: AppConfig;
  
  try {
    initial = stored ? { ...DEFAULT_CONFIG, ...JSON.parse(stored) } : DEFAULT_CONFIG;
  } catch {
    initial = DEFAULT_CONFIG;
  }

  const { subscribe, set, update } = writable<AppConfig>(initial);

  return {
    subscribe,
    
    /**
     * Update stream configuration
     */
    updateStreams(streams: Partial<StreamConfig>) {
      update(config => {
        const newConfig = {
          ...config,
          streams: { ...config.streams, ...streams }
        };
        if (browser) {
          localStorage.setItem(STORAGE_KEY, JSON.stringify(newConfig));
        }
        return newConfig;
      });
    },
    
    /**
     * Update path configuration
     */
    updatePaths(paths: Partial<PathConfig>) {
      update(config => {
        const newConfig = {
          ...config,
          paths: { ...config.paths, ...paths }
        };
        if (browser) {
          localStorage.setItem(STORAGE_KEY, JSON.stringify(newConfig));
        }
        return newConfig;
      });
    },
    
    /**
     * Reset to default configuration
     */
    reset() {
      set(DEFAULT_CONFIG);
      if (browser) {
        localStorage.removeItem(STORAGE_KEY);
      }
    },
    
    /**
     * Get current config value synchronously
     */
    get(): AppConfig {
      return get({ subscribe });
    },
  };
}

export const appConfig = createConfigStore();

// ============================================================================
// DERIVED STORES FOR CONVENIENCE
// ============================================================================

/**
 * Full HLS server base URL (e.g., http://127.0.0.1:1521)
 */
export const hlsBaseUrl = derived(appConfig, $config => 
  `${$config.streams.hlsServerUrl}:${$config.streams.hlsServerPort}`
);

/**
 * Build a playlist URL for a specific stream
 */
export function getPlaylistUrl(streamName: string = 'stream_0'): string {
  const config = appConfig.get();
  return `${config.streams.hlsServerUrl}:${config.streams.hlsServerPort}/${streamName}/playlist.m3u8`;
}

/**
 * Get HLS output directory path
 */
export function getHlsOutputDir(): string {
  return appConfig.get().streams.hlsOutputDir;
}

/**
 * Get default RTSP URL
 */
export function getDefaultRtspUrl(): string {
  return appConfig.get().streams.defaultRtspUrl;
}
