use bytes::Bytes;
use image::{DynamicImage, ImageFormat, GenericImageView};
use std::io::Cursor;

use crate::error::AppError;
use crate::models::ObjectMode;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn resize(
        image_data: Bytes,
        width: u32,
        height: u32,
        object_mode: ObjectMode,
    ) -> Result<(Bytes, String), AppError> {
        let img = image::load_from_memory(&image_data)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to decode image: {}", e)))?;

        let resized = match object_mode {
            ObjectMode::Cover => Self::resize_cover(img, width, height),
            ObjectMode::Contain => Self::resize_contain(img, width, height),
            ObjectMode::Fill => Self::resize_fill(img, width, height),
            ObjectMode::ScaleDown => Self::resize_scale_down(img, width, height),
        };

        let format = ImageFormat::Jpeg;
        let content_type = "image/jpeg";

        let mut buffer = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut buffer), format)
            .map_err(|e| AppError::ImageProcessingError(format!("Failed to encode image: {}", e)))?;

        Ok((Bytes::from(buffer), content_type.to_string()))
    }

    fn resize_cover(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        let (img_width, img_height) = img.dimensions();
        let img_aspect = img_width as f64 / img_height as f64;
        let target_aspect = width as f64 / height as f64;

        let (scale_width, scale_height) = if img_aspect > target_aspect {
            (((height as f64) * img_aspect) as u32, height)
        } else {
            (width, ((width as f64) / img_aspect) as u32)
        };

        let scaled = img.resize_exact(
            scale_width,
            scale_height,
            image::imageops::FilterType::Lanczos3,
        );

        let x_offset = (scale_width.saturating_sub(width)) / 2;
        let y_offset = (scale_height.saturating_sub(height)) / 2;

        DynamicImage::ImageRgba8(image::imageops::crop_imm(
            &scaled.to_rgba8(),
            x_offset,
            y_offset,
            width,
            height,
        ).to_image())
    }

    fn resize_contain(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    fn resize_fill(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    }

    fn resize_scale_down(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        let (img_width, img_height) = img.dimensions();
        
        if img_width <= width && img_height <= height {
            return img;
        }

        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    }
}
