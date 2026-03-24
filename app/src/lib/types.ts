// ============================================================================
// Shared TypeScript types for Tauri backend responses
// ============================================================================

// -- System --

export interface SystemMetrics {
  cpu: { usagePercent: number; coreCount: number; frequencyMhz: number | null };
  memory: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number };
  disk: { totalGb: number; usedGb: number; availableGb: number; usagePercent: number; readSpeedMbps: number | null; writeSpeedMbps: number | null };
  uptimeSeconds: number;
}

// -- Streams --

export interface StreamStatus {
  id: string;
  name: string;
  status: string;
  streamType: string;
  codec: string | null;
  resolution: string | null;
  fps: number | null;
  bitrateKbps: number | null;
  durationSeconds: number | null;
  latencyMs: number | null;
}

export interface StreamStats {
  activeCount: number;
  totalCount: number;
  avgLatencyMs: number;
  totalBitrateKbps: number;
  streams: StreamStatus[];
}

// -- Throughput --

export interface ThroughputPoint {
  timestamp: number;
  networkMbps: number;
  fps: number;
  cpuPercent: number;
}

export interface ThroughputHistory {
  points: ThroughputPoint[];
  periodSeconds: number;
}

// -- Media --

export interface MediaMetadata {
  filename: string;
  filePath: string;
  fileSize: number;
  mimeType: string | null;
  duration: number | null;
  width: number | null;
  height: number | null;
  fps: number | null;
  frameCount: number | null;
  bitrate: number | null;
  codec: string | null;
  audioCodec: string | null;
  audioSampleRate: number | null;
  audioChannels: number | null;
  createdAt: string | null;
  modifiedAt: string | null;
  thumbnail: string | null;
}

// -- Camera --

export interface CameraCaptureResult {
  cameraId: number;
  outputPath: string | null;
  frameData: string | null;
  success: boolean;
}

// -- Analysis --

export interface MotionEvent {
  startFrame: number;
  endFrame: number;
  eventType: string;
}

export interface SimilarityGroup {
  groupName: string;
  members: string[];
}

export interface ImageComparisonResult {
  image1: string;
  image2: string;
  similarityScore: number;
  isDuplicate: boolean;
}

export interface AnalysisResult {
  motionEvents: MotionEvent[];
  similarityGroups: SimilarityGroup[];
  imageComparison: ImageComparisonResult | null;
}

// -- Annotation --

export interface AnnotationData {
  unique_labels: string[];
  frame_count?: number;
  detections_per_frame?: number[];
  label_counts?: Record<string, number>;
  total_detections?: number;
  avg_confidence?: number;
}
