use image::GenericImageView;
use anyhow::Result;

pub struct ImageService;

impl ImageService {
    pub fn compress(data: &[u8], max_size: u32) -> Result<Vec<u8>> {
        let img = image::load_from_memory(data)?;
        let (width, height) = img.dimensions();
        
        let resized = if width > max_size || height > max_size {
            img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        let encoder = match webp::Encoder::from_image(&resized) {
            Ok(encoder) => encoder,
            Err(e) => return Err(anyhow::anyhow!("{}", e)),
        };
        let webp = encoder.encode(75.0);
        
        Ok(webp.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImage;
    
    #[test]
    fn test_compress_image() {
        let mut img = image::DynamicImage::new_rgb8(100, 100);
        for x in 0..100 {
            for y in 0..100 {
                img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            }
        }
        
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Jpeg).unwrap();
        let data = buffer.into_inner();
        
        let compressed = ImageService::compress(&data, 50).unwrap();
        assert!(!compressed.is_empty());
        
        let decoded = image::load_from_memory(&compressed).unwrap();
        assert!(decoded.width() <= 50);
        assert!(decoded.height() <= 50);
    }
}
