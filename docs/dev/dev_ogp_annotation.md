# Annotation Module Documentation

## Overview

The annotation module (`core/annotation.rs`) provides the core data structures for handling video annotation data. It manages frame metadata and object detection results.

## Data Structures

### `Annotation`

The main container for all annotation data for a single video.

```rust
pub struct Annotation {
    pub metadata_count: i32,
    pub timestamp: String,
    pub frame_metadata: Vec<FrameMetadata>,
}
```

#### Fields
- `metadata_count`: The total number of objects across all frames. This is updated after filtering.
- `timestamp`: A creation timestamp, typically in `YYYYMMDD_HHMMSS` format.
- `frame_metadata`: A vector containing the detailed annotation data for each frame.

#### Methods

##### `get_total_labels(&self) -> (Vec<String>, i32)`
Scans through all frames and objects to find unique labels.

-   **Returns**: A tuple of unique labels and total object count.

### `FrameMetadata`

Contains all the annotation information for a specific frame in the video.

```rust
pub struct FrameMetadata {
    pub frame_id: i32,
    pub frame_message: FrameMessage,
}
```

#### Fields
- `frame_id`: The zero-based index of the frame.
- `frame_message`: Contains the object detections for this frame.

### `FrameMessage`

A container for all the detected objects within a single frame.

```rust
pub struct FrameMessage {
    pub status: String,
    pub object_message: Vec<ObjectMessage>,
}
```

#### Fields
- `status`: The status of the frame (e.g., "keyframe", "interpolated").
- `object_message`: A vector of all objects detected in this frame.

### `ObjectMessage`

A detailed structure holding all information for a single detected object.

```rust
pub struct ObjectMessage {
    pub label: String,
    pub bbox: Vec<i32>,
    pub polygon: Vec<Vec<i32>>,
    pub keypoints: Vec<Vec<i32>>,
    pub bbox_confidence_score: f32,
    pub polygon_confidence_score: f32,
    pub keypoint_confidence_score: f32,
    pub branch: String,
    pub children: Option<Vec<ObjectMessage>>,
}
```

#### Fields
- `label`: The object's class label (e.g., "person", "car").
- `bbox`: The bounding box coordinates `[x1, y1, x2, y2]`.
- `polygon`: A vector of points for segmentation masks.
- `keypoints`: A vector of keypoint coordinates for pose estimation.
- `bbox_confidence_score`: Confidence score for the bounding box detection.
- `polygon_confidence_score`: Confidence score for the polygon segmentation.
-   `keypoint_confidence_score`: Overall confidence score for the keypoints.
-   `branch`: The object's category branch (e.g., "person_equipment").
-   `children`: An optional vector of nested `ObjectMessage` structs, representing child objects.

## Usage Example

### Filtering Labels

The `Annotation` struct provides a method to filter objects by label.

```rust
// Assume 'annotation' is an existing Annotation struct instance
let (labels, total) = annotation.get_total_labels();
println!("Unique labels: {:?}", labels);
println!("Total objects: {}", total);

// Filter to only include specific labels
annotation.filter_by_labels(&["person".to_string(), "car".to_string()]);
```

## Supporting Data Structures

## Best Practices

1.  **Data Validation**: Before processing, ensure that array lengths and coordinate values are valid (e.g., a `bbox` vector should always have 4 elements).
2.  **Memory Management**: When processing large annotation files, consider passing references (`&ObjectMessage`) where possible to avoid unnecessary data cloning.
3.  **Error Handling**: The application should gracefully handle missing optional fields (like `children`) and potential parsing errors from the source JSON. 