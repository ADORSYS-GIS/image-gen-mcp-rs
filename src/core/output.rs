use std::path::Path;
use tokio::fs;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

use crate::core::errors::{DomainError, DomainResult};

pub struct ImageSaver;

impl ImageSaver {
    /// Saves image data (URL or data URL) to a file.
    pub async fn save_image(
        data_or_url: &str,
        output_dir: &Path,
        prefix: &str,
    ) -> DomainResult<String> {
        // Ensure output directory exists
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).await?;
        }

        let cid = cuid2::create_id();
        let extension = "png"; // Default to png for now
        let filename = format!("{}_{}.{}", prefix, cid, extension);
        let file_path = output_dir.join(&filename);

        if data_or_url.starts_with("data:image/") {
            // Handle data URL (base64)
            let parts: Vec<&str> = data_or_url.split(",").collect();
            if parts.len() != 2 {
                return Err(DomainError::Image("Invalid data URL format".to_string()));
            }
            let base64_data = parts[1];
            let decoded = BASE64_STANDARD.decode(base64_data)
                .map_err(|e| DomainError::Image(format!("Base64 decoing error: {}", e)))?;
            
            fs::write(&file_path, decoded).await?;
        } else if data_or_url.starts_with("http") {
            // Handle remote URL
            let client = reqwest::Client::new();
            let response = client.get(data_or_url).send().await?;
            if !response.status().is_success() {
                return Err(DomainError::Image(format!("Failed to download image: {}", response.status())));
            }
            let bytes = response.bytes().await?;
            fs::write(&file_path, bytes).await?;
        } else {
            return Err(DomainError::Image("Unsupported image data format".to_string()));
        }

        Ok(file_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_data_url() {
        let dir = tempdir().unwrap();
        let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";
        let result = ImageSaver::save_image(data_url, dir.path(), "test").await;
        
        assert!(result.is_ok());
        let path_str = result.unwrap();
        let path = Path::new(&path_str);
        assert!(path.exists());
        assert!(fs::metadata(path).await.unwrap().len() > 0);
    }
}
