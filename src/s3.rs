use aws_sdk_s3::Client;
use aws_config;
use bytes::Bytes;
use url::Url;

use crate::error::AppError;

pub struct S3Client {
    client: Client,
}

impl S3Client {
    pub async fn new() -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn download_image(&self, s3_url: &str) -> Result<Bytes, AppError> {
        let (bucket, key) = parse_s3_url(s3_url)?;
        
        tracing::info!("Downloading from S3: bucket={}, key={}", bucket, key);
        
        let response = self
            .client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| AppError::S3Error(format!("Failed to download from S3: {}", e)))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AppError::S3Error(format!("Failed to read S3 response body: {}", e)))?;

        Ok(data.into_bytes())
    }

    pub async fn check_object_exists(&self, bucket: &str, key: &str) -> bool {
        tracing::info!("Checking if object exists: bucket={}, key={}", bucket, key);
        
        match self.client.head_object().bucket(bucket).key(key).send().await {
            Ok(_) => {
                tracing::info!("Object exists: bucket={}, key={}", bucket, key);
                true
            }
            Err(_) => {
                tracing::info!("Object does not exist: bucket={}, key={}", bucket, key);
                false
            }
        }
    }

    pub async fn upload_image(
        &self,
        bucket: &str,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<String, AppError> {
        tracing::info!("Uploading to S3: bucket={}, key={}", bucket, key);
        
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::S3Error(format!("Failed to upload to S3: {}", e)))?;

        let url = format!("s3://{}/{}", bucket, key);
        Ok(url)
    }
}

pub fn parse_s3_url(s3_url: &str) -> Result<(String, String), AppError> {
    let url = Url::parse(s3_url)
        .map_err(|e| AppError::InvalidS3Url(format!("Invalid URL format: {}", e)))?;

    let (bucket, key) = match url.scheme() {
        "s3" => {
            let bucket = url
                .host_str()
                .ok_or_else(|| AppError::InvalidS3Url("Missing bucket name".to_string()))?
                .to_string();

            let key = url.path().trim_start_matches('/').to_string();
            (bucket, key)
        }
        "https" | "http" => {
            let host = url
                .host_str()
                .ok_or_else(|| AppError::InvalidS3Url("Missing host".to_string()))?;

            if host.starts_with("s3.") || host.starts_with("s3-") {
                let path = url.path().trim_start_matches('/');
                let parts: Vec<&str> = path.splitn(2, '/').collect();
                
                if parts.len() < 2 {
                    return Err(AppError::InvalidS3Url(
                        "Invalid path-style S3 URL format".to_string(),
                    ));
                }
                
                (parts[0].to_string(), parts[1].to_string())
            } else if host.contains(".s3.") || host.contains(".s3-") {
                let bucket = host.split('.').next()
                    .ok_or_else(|| AppError::InvalidS3Url("Cannot extract bucket name".to_string()))?
                    .to_string();
                
                let key = url.path().trim_start_matches('/').to_string();
                (bucket, key)
            } else {
                return Err(AppError::InvalidS3Url(
                    "URL does not appear to be a valid S3 URL".to_string(),
                ));
            }
        }
        _ => {
            return Err(AppError::InvalidS3Url(
                "URL must use s3://, https://, or http:// scheme".to_string(),
            ));
        }
    };
    
    if key.is_empty() {
        return Err(AppError::InvalidS3Url("Missing object key".to_string()));
    }

    Ok((bucket, key))
}

pub fn generate_resized_key(original_key: &str, width: u32, height: u32) -> String {
    let extension = std::path::Path::new(original_key)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");

    let stem = std::path::Path::new(original_key)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let parent = std::path::Path::new(original_key)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let filename = format!("{}_{}x{}.{}", stem, width, height, extension);
    
    if parent.is_empty() {
        filename
    } else {
        format!("{}/{}", parent, filename)
    }
}
