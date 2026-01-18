use media_core::analysis::{
    SimilarityAnalyzer, MotionDetector, 
    SimilarityConfig, MotionConfig, AnalysisError
};
use tauri::command;
use std::path::Path;

// Helper to handle analysis errors
fn map_analysis_error(e: AnalysisError) -> String {
    format!("Analysis error: {:?}", e)
}

#[command]
pub async fn analyze_similarity(config: SimilarityConfig, input_path: String) -> Result<String, String> {
    // Run CPU-intensive task in a separate thread
    std::thread::spawn(move || {
        let mut analyzer = SimilarityAnalyzer::new(config).map_err(map_analysis_error)?;
        let output_dir = Path::new(&input_path).join("grouped_results");
        let results = analyzer.group_similar_images(Path::new(&input_path), &output_dir).map_err(map_analysis_error)?;
        
        serde_json::to_string(&results)
            .map_err(|e| format!("Serialization error: {}", e))
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

#[command]
pub async fn detect_motion(config: MotionConfig, video_path: String) -> Result<String, String> {
    std::thread::spawn(move || {
        let mut detector = MotionDetector::new(config).map_err(map_analysis_error)?;
        let output_dir = Path::new(&video_path).parent().unwrap_or(Path::new(".")).join("motion_events");
        let events = detector.process_video(Path::new(&video_path), &output_dir).map_err(map_analysis_error)?;
        
        serde_json::to_string(&events)
            .map_err(|e| format!("Serialization error: {}", e))
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

