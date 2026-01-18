use lazy_static::lazy_static;
use opencv::{imgproc, prelude::*, videoio, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::types::Annotation;

// Create a lazy static HashMap to store label-to-color mappings
lazy_static! {
    static ref COLOR_MAP: Mutex<HashMap<String, opencv::core::Scalar>> = Mutex::new(HashMap::new());
}

// Function to generate distinct colors
fn get_color_for_label(label: &str) -> Result<opencv::core::Scalar> {
    let mut color_map = COLOR_MAP.lock().unwrap();

    if let Some(&color) = color_map.get(label) {
        return Ok(color);
    }

    // Generate a new color based on the golden ratio conjugate
    // This creates a better distribution of colors
    let golden_ratio_conjugate = 0.618033988749895;
    let hue = (color_map.len() as f64 * golden_ratio_conjugate * 360.0) % 360.0;

    // Create a single-pixel Mat in HSV format
    let mut hsv_mat = unsafe { opencv::core::Mat::new_rows_cols(1, 1, opencv::core::CV_32FC3)? };

    // Set HSV values (hue, saturation, value)
    // Using maximum saturation and value for vibrant colors
    *hsv_mat.at_2d_mut::<opencv::core::Vec3f>(0, 0)? = opencv::core::Vec3f::from_array([
        (hue / 2.0) as f32, // OpenCV uses hue range [0, 180] instead of [0, 360]
        1.0 * 255.0,        // Full saturation
        1.0 * 255.0,        // Full value
    ]);

    // Convert HSV to BGR
    let mut bgr_mat = opencv::core::Mat::default();
    imgproc::cvt_color(
        &hsv_mat,
        &mut bgr_mat,
        imgproc::COLOR_HSV2BGR,
        0,
        opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )?;

    // Read BGR values
    let bgr = *bgr_mat.at_2d::<opencv::core::Vec3f>(0, 0)?;
    let color = opencv::core::Scalar::new(bgr[0] as f64, bgr[1] as f64, bgr[2] as f64, 0.0);

    color_map.insert(label.to_string(), color);
    Ok(color)
}

pub struct VideoAnnotator {
    capture: videoio::VideoCapture,
    output_writer: videoio::VideoWriter,
    frame_count: i32,
}

impl VideoAnnotator {
    pub fn new(video_path: &Path, output_path: &Path) -> Result<Self> {
        let capture =
            videoio::VideoCapture::from_file(video_path.to_str().unwrap(), videoio::CAP_ANY)?;

        if !capture.is_opened()? {
            return Err(opencv::Error::new(
                opencv::core::StsError as i32,
                "Could not open video file".to_string(),
            ));
        }

        let frame_count = capture.get(videoio::CAP_PROP_FRAME_COUNT)? as i32;
        let fps = capture.get(videoio::CAP_PROP_FPS)?;
        let width = capture.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
        let height = capture.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;

        let fourcc = videoio::VideoWriter::fourcc('m', 'p', '4', 'v')?;
        let output_writer = videoio::VideoWriter::new(
            output_path.to_str().unwrap(),
            fourcc,
            fps,
            opencv::core::Size::new(width, height),
            true,
        )?;

        Ok(Self {
            capture,
            output_writer,
            frame_count,
        })
    }

    pub fn annotate_video(&mut self, annotation: &Annotation) -> Result<()> {
        let mut frame = opencv::core::Mat::default();
        let mut frame_id = 0;
        let total_frames = self.frame_count;

        // Create progress tracking variables
        let mut frames_processed = 0;
        let progress_interval = total_frames / 100; // Update every 1%

        while self.capture.read(&mut frame)? {
            let mut output_frame = frame.clone();

            // Find annotation for current frame
            if let Some(frame_metadata) = annotation
                .frame_metadata
                .iter()
                .find(|m| m.frame_id == frame_id)
            {
                // Draw all bounding boxes for this frame
                for obj in &frame_metadata.frame_message.object_message {
                    if obj.bbox.len() == 4 {
                        // Define styling parameters
                        let text_color = opencv::core::Scalar::new(255.0, 255.0, 255.0, 0.0); // White text
                        let font_scale = 0.45; // Slightly larger
                        let text_thickness = 1;
                        let box_thickness = 2;
                        let font = imgproc::FONT_HERSHEY_SIMPLEX; // Changed back to SIMPLEX for cleaner look
                        let padding = 5; // Increased padding for more space

                        // Get color based on label
                        let color = get_color_for_label(&obj.label)?;

                        // Draw bounding box
                        imgproc::rectangle(
                            &mut output_frame,
                            opencv::core::Rect::new(
                                obj.bbox[0],
                                obj.bbox[1],
                                obj.bbox[2] - obj.bbox[0],
                                obj.bbox[3] - obj.bbox[1],
                            ),
                            color,
                            box_thickness,
                            imgproc::LINE_8,
                            0,
                        )?;

                        // Prepare label text with confidence score
                        let label =
                            format!("{} {:.0}%", obj.label, obj.bbox_confidence_score * 100.0);
                        let mut baseline = 0;

                        // Get text size for background rectangle
                        let text_size = imgproc::get_text_size(
                            &label,
                            font,
                            font_scale,
                            text_thickness,
                            &mut baseline,
                        )?;

                        // Calculate text position
                        let text_x = obj.bbox[0];
                        let text_y = obj.bbox[1];

                        // Draw background rectangle for text with more padding
                        imgproc::rectangle(
                            &mut output_frame,
                            opencv::core::Rect::new(
                                text_x,
                                text_y - (text_size.height + padding * 2), // Removed extra spacing
                                text_size.width + padding * 2,
                                text_size.height + padding * 2,
                            ),
                            color,
                            -1,
                            imgproc::LINE_8,
                            0,
                        )?;

                        // Draw text in white
                        imgproc::put_text(
                            &mut output_frame,
                            &label,
                            opencv::core::Point::new(
                                text_x + padding,
                                text_y - padding, // Adjusted to align perfectly with box
                            ),
                            font,
                            font_scale,
                            text_color,
                            text_thickness,
                            imgproc::LINE_AA,
                            false,
                        )?;
                    }
                }
            }

            // Write the frame
            self.output_writer.write(&output_frame)?;

            frame_id += 1;
            frames_processed += 1;

            // Update progress every 1%
            if frames_processed % progress_interval == 0 {
                let progress = (frames_processed as f32 / total_frames as f32 * 100.0) as i32;
                println!("Processing progress: {}%", progress);
            }
        }

        println!(
            "Video annotation completed: {} frames processed",
            frames_processed
        );
        Ok(())
    }
}


