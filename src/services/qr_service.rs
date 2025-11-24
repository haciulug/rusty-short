use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::{ImageBuffer, ImageEncoder, Luma};
use qrcode::QrCode;

pub struct QrService;

impl QrService {
    pub fn generate_qr_code(url: &str) -> Result<Vec<u8>> {
        let code = QrCode::new(url)?;
        let image = code.render::<Luma<u8>>().build();
        
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        
        encoder.write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::L8,
        )?;
        
        Ok(buffer)
    }

    pub fn generate_qr_code_base64(url: &str) -> Result<String> {
        let png_data = Self::generate_qr_code(url)?;
        Ok(general_purpose::STANDARD.encode(png_data))
    }
}

