use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use crate::core::{
    config::AppConfig,
    errors::{DomainError, DomainResult},
    traits::{GenerateParams, ImageGenerationPort},
};
use super::request::GeminiRequestBuilder;
use super::types::*;

pub struct GeminiImageClient {
    client: reqwest::Client,
    config: AppConfig,
}

impl GeminiImageClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().unwrap_or_else(|_| reqwest::Client::new()),
            config,
        }
    }

    fn map_model(&self, p: Option<&str>) -> String {
        match p {
            Some("nano-banana") => "gemini-2.5-flash-image".into(),
            Some("nano-banana2") | Some("nano-banana 2") => "gemini-3.1-flash-image-preview".into(),
            Some("nano-banana-pro") => "nano-banana-pro-preview".into(),
            Some(m) => m.into(),
            None => if self.config.image_model == "nano-banana" { "gemini-2.5-flash-image".into() } else { self.config.image_model.clone() },
        }
    }

    async fn extract_image(&self, img_id: String) -> DomainResult<(String, String)> {
        if img_id.starts_with("data:") {
            let parts: Vec<&str> = img_id.split(',').collect();
            if parts.len() != 2 { return Err(DomainError::Image("Invalid data URL".into())); }
            let mime = img_id.split(':').nth(1).and_then(|s| s.split(';').nth(0)).unwrap_or("image/png").into();
            Ok((mime, parts[1].into()))
        } else {
            let resp = self.client.get(&img_id).send().await?;
            let mime = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|h| h.to_str().ok()).unwrap_or("image/png").into();
            Ok((mime, BASE64_STANDARD.encode(resp.bytes().await?)))
        }
    }
}

#[async_trait]
impl ImageGenerationPort for GeminiImageClient {
    async fn list_models(&self) -> DomainResult<Vec<String>> {
        let url = format!("{}/models?key={}", self.config.base_url, self.config.api_key);
        let resp: GeminiModelsResponse = self.client.get(&url).send().await?.json().await.map_err(|e| DomainError::Image(e.to_string()))?;
        Ok(resp.models.into_iter().filter(|m| m.supported_generation_methods.contains(&"generateContent".into()))
            .map(|m| m.name.replace("models/", ""))
            .filter(|n| { let nl = n.to_lowercase(); nl.contains("image") || nl.contains("imagen") || nl.contains("banana") || nl.contains("lyria") })
            .collect())
    }

    async fn generate(&self, p: GenerateParams) -> DomainResult<Vec<String>> {
        let model = self.map_model(p.model.as_deref());
        let url = format!("{}/models/{}:generateContent?key={}", self.config.base_url, model, self.config.api_key);
        let req = GeminiRequestBuilder::build(p.prompt, None, p.seed);
        self.execute(&url, &req).await
    }

    async fn edit(&self, p: GenerateParams) -> DomainResult<Vec<String>> {
        let model = self.map_model(p.model.as_deref());
        let url = format!("{}/models/{}:generateContent?key={}", self.config.base_url, model, self.config.api_key);
        let img = self.extract_image(p.image_id.ok_or_else(|| DomainError::Image("Missing image".into()))?).await?;
        let req = GeminiRequestBuilder::build(p.prompt, Some(img), p.seed);
        self.execute(&url, &req).await
    }
}

impl GeminiImageClient {
    async fn execute(&self, url: &str, req: &GeminiRequest) -> DomainResult<Vec<String>> {
        let resp = self.client.post(url).json(req).send().await.map_err(DomainError::Http)?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() { return Err(DomainError::Image(format!("Gemini Error ({}): {}", status, body))); }
        let g_resp: GeminiResponse = serde_json::from_str(&body).map_err(|e| {
            tracing::error!("Gemini JSON decode error: {}. Body: {}", e, body);
            DomainError::Image(format!("error decoding response body: {}", e))
        })?;
        let reason = g_resp.candidates.first().and_then(|c| c.finish_reason.clone());
        let imgs: Vec<String> = g_resp.candidates.into_iter()
            .filter_map(|c| c.content)
            .flat_map(|c| c.parts).filter_map(|p| p.inline_data)
            .map(|d| format!("data:{};base64,{}", d.mime_type, d.data)).collect();
        if imgs.is_empty() {
            let r = reason.unwrap_or_else(|| "Unknown reason".to_string());
            tracing::warn!("Gemini returned no images. Reason: {}. Body: {}", r, body);
            return Err(DomainError::Image(format!("Model finished without image: {}", r)));
        }
        Ok(imgs)
    }
}
