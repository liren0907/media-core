pub struct DialogHandler;

impl DialogHandler {
    pub fn select_video_file() -> Result<String, String> {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("Video Files", &["mp4"])
            .pick_file()
        {
            println!("Selected file: {}", file.to_string_lossy());
            let file_path = file.to_string_lossy().to_string();
            Ok(file_path)
        } else {
            println!("No file selected");
            Err("No file selected".into())
        }
    }
}
