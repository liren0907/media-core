use image::GenericImageView;

pub struct ImageHandler;

impl ImageHandler {
    pub fn read_image_info(file_path: &str) -> Result<String, String> {
        match image::open(file_path) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let color_type = img.color();
                let info = format!(
                    "Image details:\nWidth: {}\nHeight: {}\nColor Type: {:?}",
                    width, height, color_type
                );
                println!("{}", info);
                Ok(info)
            }
            Err(e) => {
                println!("Failed to open image: {}", e);
                Err("Failed to open image".into())
            }
        }
    }
}
