/**
 * Tauri Event Utilities
 * 
 * This module provides helpers for subscribing to real-time events
 * from the Rust backend.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// EVENT TYPES
// ============================================================================

export interface PipelineProgressEvent {
  pipelineId: string;
  stepName: string;
  stepIndex: number;
  totalSteps: number;
  progressPercent: number;
  message: string;
  isComplete: boolean;
  hasError: boolean;
  errorMessage?: string;
}

export interface PipelineCompleteEvent {
  pipelineId: string;
  total: number;
  succeeded: number;
  failed: number;
}

export interface StreamStatusEvent {
  streamId: string;
  status: 'starting' | 'active' | 'paused' | 'error' | 'stopped';
  message?: string;
  timestamp: number;
}

export interface SystemMetricsEvent {
  cpuPercent: number;
  memoryPercent: number;
  diskPercent: number;
  activeStreams: number;
  timestamp: number;
}

// ============================================================================
// EVENT NAMES
// ============================================================================

export const EVENTS = {
  PIPELINE_PROGRESS: 'pipeline:progress',
  PIPELINE_COMPLETE: 'pipeline:complete',
  STREAM_STATUS: 'stream:status',
  SYSTEM_METRICS: 'system:metrics',
  RTSP_STATUS: 'rtsp-status',
} as const;

// ============================================================================
// EVENT LISTENERS
// ============================================================================

/**
 * Listen for pipeline progress events
 */
export async function onPipelineProgress(
  callback: (event: PipelineProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<PipelineProgressEvent>(EVENTS.PIPELINE_PROGRESS, (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for pipeline completion events
 */
export async function onPipelineComplete(
  callback: (event: PipelineCompleteEvent) => void
): Promise<UnlistenFn> {
  return listen<PipelineCompleteEvent>(EVENTS.PIPELINE_COMPLETE, (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for stream status changes
 */
export async function onStreamStatus(
  callback: (event: StreamStatusEvent) => void
): Promise<UnlistenFn> {
  return listen<StreamStatusEvent>(EVENTS.STREAM_STATUS, (event) => {
    callback(event.payload);
  });
}

/**
 * Listen for system metrics updates
 */
export async function onSystemMetrics(
  callback: (event: SystemMetricsEvent) => void
): Promise<UnlistenFn> {
  return listen<SystemMetricsEvent>(EVENTS.SYSTEM_METRICS, (event) => {
    callback(event.payload);
  });
}

// ============================================================================
// METRICS STREAM CONTROL
// ============================================================================

/**
 * Start receiving system metrics at regular intervals
 * @param intervalMs - Update interval in milliseconds (minimum 500ms)
 */
export async function startMetricsStream(intervalMs: number = 2000): Promise<void> {
  await invoke('start_metrics_stream', { intervalMs });
}

/**
 * Stop receiving system metrics updates
 */
export async function stopMetricsStream(): Promise<void> {
  await invoke('stop_metrics_stream');
}

/**
 * Check if metrics streaming is currently active
 */
export async function isMetricsStreaming(): Promise<boolean> {
  return invoke<boolean>('is_metrics_streaming');
}

// ============================================================================
// PIPELINE CONTROL
// ============================================================================

/**
 * Start tracking a pipeline execution
 * @returns The pipeline ID for tracking
 */
export async function startPipelineTracking(pipelineId: string): Promise<string> {
  return invoke<string>('start_pipeline_tracking', { pipelineId });
}

/**
 * Cancel a running pipeline
 * @returns true if the pipeline was found and cancelled
 */
export async function cancelPipeline(pipelineId: string): Promise<boolean> {
  return invoke<boolean>('cancel_pipeline', { pipelineId });
}

/**
 * Check if a pipeline has been cancelled
 */
export async function isPipelineCancelled(pipelineId: string): Promise<boolean> {
  return invoke<boolean>('is_pipeline_cancelled', { pipelineId });
}

/**
 * Clean up pipeline tracking after completion
 */
export async function finishPipelineTracking(pipelineId: string): Promise<void> {
  await invoke('finish_pipeline_tracking', { pipelineId });
}

/**
 * Get list of currently active pipeline IDs
 */
export async function getActivePipelines(): Promise<string[]> {
  return invoke<string[]>('get_active_pipelines');
}

// ============================================================================
// BATCH PROCESSING
// ============================================================================

export interface BatchProcessConfig {
  filePaths: string[];
  operation: 'metadata' | 'motion' | 'hls' | 'extract';
  parallel?: boolean;
  options?: Record<string, unknown>;
}

export interface BatchItemResult {
  filePath: string;
  success: boolean;
  error?: string;
  result?: unknown;
}

export interface BatchProcessResult {
  total: number;
  succeeded: number;
  failed: number;
  results: BatchItemResult[];
}

/**
 * Execute batch processing on multiple files
 * Emits pipeline:progress events during execution
 */
export async function executeBatchPipeline(
  config: BatchProcessConfig
): Promise<BatchProcessResult> {
  return invoke<BatchProcessResult>('execute_batch_pipeline', { config });
}

// ============================================================================
// CONVENIENCE HELPERS
// ============================================================================

/**
 * Create a progress tracker for a pipeline
 * Returns an object with methods to subscribe and manage the pipeline
 */
export function createPipelineTracker(pipelineId: string) {
  let unlisten: UnlistenFn | null = null;
  let progressCallback: ((event: PipelineProgressEvent) => void) | null = null;
  let completeCallback: ((event: PipelineCompleteEvent) => void) | null = null;

  return {
    pipelineId,
    
    async start() {
      await startPipelineTracking(pipelineId);
      
      if (progressCallback) {
        unlisten = await onPipelineProgress((event) => {
          if (event.pipelineId === pipelineId && progressCallback) {
            progressCallback(event);
          }
        });
      }
    },
    
    onProgress(callback: (event: PipelineProgressEvent) => void) {
      progressCallback = callback;
      return this;
    },
    
    onComplete(callback: (event: PipelineCompleteEvent) => void) {
      completeCallback = callback;
      return this;
    },
    
    async cancel() {
      return cancelPipeline(pipelineId);
    },
    
    async cleanup() {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
      await finishPipelineTracking(pipelineId);
    },
  };
}
