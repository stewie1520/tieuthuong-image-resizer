use axum::Json;
use crate::error::AppError;
use crate::models::{ResizeRequest, ResizeResponse};
use crate::s3::{S3Client, parse_s3_url, generate_resized_key};
use crate::image_processor::ImageProcessor;

pub async fn resize_image(
    Json(payload): Json<ResizeRequest>,
) -> Result<Json<ResizeResponse>, AppError> {
    tracing::info!(
        "Resize request: url={}, width={}, height={}, mode={:?}",
        payload.s3_url,
        payload.width,
        payload.height,
        payload.object_mode
    );

    if payload.width == 0 || payload.height == 0 {
        return Err(AppError::InvalidS3Url(
            "Width and height must be greater than 0".to_string(),
        ));
    }

    let (bucket, original_key) = parse_s3_url(&payload.s3_url)?;

    let s3_client = S3Client::new().await;

    let resized_key = generate_resized_key(&original_key, payload.width, payload.height);

    if s3_client.check_object_exists(&bucket, &resized_key).await {
        let resized_url = format!("s3://{}/{}", bucket, resized_key);
        tracing::info!("Resized image already exists at {}, returning cached URL", resized_url);
        
        return Ok(Json(ResizeResponse {
            original_url: payload.s3_url,
            resized_url,
            width: payload.width,
            height: payload.height,
            object_mode: payload.object_mode,
        }));
    }

    let image_data = s3_client.download_image(&payload.s3_url).await?;

    let (resized_data, content_type) = ImageProcessor::resize(
        image_data,
        payload.width,
        payload.height,
        payload.object_mode,
    )?;

    let resized_url = s3_client
        .upload_image(&bucket, &resized_key, resized_data, &content_type)
        .await?;

    tracing::info!("Successfully resized and uploaded image to {}", resized_url);

    Ok(Json(ResizeResponse {
        original_url: payload.s3_url,
        resized_url,
        width: payload.width,
        height: payload.height,
        object_mode: payload.object_mode,
    }))
}
