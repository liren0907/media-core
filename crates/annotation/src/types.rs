use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub metadata_count: i32,
    pub timestamp: String,
    pub frame_metadata: Vec<FrameMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub frame_id: i32,
    pub frame_message: FrameMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMessage {
    pub status: String,
    pub object_message: Vec<ObjectMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMessage {
    pub label: String,
    pub bbox: Vec<i32>,
    pub polygon: Vec<Vec<i32>>,
    pub keypoints: Vec<Vec<i32>>,
    pub bbox_confidence_score: f32,
    pub polygon_confidence_score: f32,
    pub keypoint_confidence_score: f32,
    pub branch: String,
    #[serde(default)]
    pub children: Option<Vec<ObjectMessage>>,
}

impl Annotation {
    pub fn get_total_labels(&self) -> (Vec<String>, i32) {
        let mut unique_labels = std::collections::HashSet::new();
        let mut total_objects = 0;

        for frame in &self.frame_metadata {
            for obj in &frame.frame_message.object_message {
                unique_labels.insert(obj.label.clone());
                total_objects += 1;

                // Count children objects too
                if let Some(children) = &obj.children {
                    for child in children {
                        unique_labels.insert(child.label.clone());
                        total_objects += 1;
                    }
                }
            }
        }

        (unique_labels.into_iter().collect(), total_objects)
    }

    pub fn filter_by_labels(&mut self, selected_labels: &[String]) {
        for frame in &mut self.frame_metadata {
            // Collect all objects to keep and new objects to add
            let mut new_objects: Vec<ObjectMessage> = Vec::new();

            // First pass: collect children that should be promoted
            for obj in &frame.frame_message.object_message {
                if let Some(ref children) = obj.children {
                    // Create new objects from selected children
                    let selected_children: Vec<ObjectMessage> = children
                        .iter()
                        .filter(|child| selected_labels.contains(&child.label))
                        .cloned()
                        .map(|mut child| {
                            // Remove children field from the converted object
                            child.children = None;
                            child
                        })
                        .collect();

                    new_objects.extend(selected_children);
                }
            }

            // Second pass: filter main objects
            frame
                .frame_message
                .object_message
                .retain(|obj| selected_labels.contains(&obj.label));

            // Add the new objects (promoted children)
            frame.frame_message.object_message.extend(new_objects);
        }

        // Update metadata count
        let (_, total_objects) = self.get_total_labels();
        self.metadata_count = total_objects;
    }
}

