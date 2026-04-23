use async_openai::{
    Client, config::OpenAIConfig,
    types::images::{CreateImageRequestArgs, ImageSize, CreateImageVariationRequestArgs, DallE2ImageSize, ImageInput},
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use crate::{
    cli::Flavor,
    core::{config::AppConfig, errors::{DomainError, DomainResult}, traits::{GenerateParams, ImageGenerationPort}},
};
use super::utils::OpenAiUtils;

pub struct OpenAiImageClient {
    client: Client<OpenAIConfig>,
    config: AppConfig,
}

impl OpenAiImageClient {
    pub fn new(c: AppConfig) -> Self {
        let cfg = OpenAIConfig::new().with_api_key(&c.api_key).with_api_base(&c.base_url);
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
            .build().unwrap_or_default();
        Self { client: Client::with_config(cfg).with_http_client(http), config: c }
    }

    async fn gen_int(&self, p: GenerateParams, size: ImageSize) -> DomainResult<Vec<String>> {
        let model = OpenAiUtils::parse_model(p.model.as_deref(), &self.config);
        let mut builder = CreateImageRequestArgs::default();
        builder.prompt(&p.prompt).model(model).n(p.n.unwrap_or(1)).size(size);
        if let Some(q) = OpenAiUtils::parse_quality(p.quality.as_deref()) { builder.quality(q); }
        if let Some(s) = OpenAiUtils::parse_style(p.style.as_deref()) { builder.style(s); }
        if let Some(seed) = p.seed { builder.user(format!("seed:{}", seed)); }
        let resp = self.client.images().generate(builder.build().map_err(DomainError::OpenAi)?).await?;
        Ok(resp.data.iter().filter_map(|img| match img.as_ref() {
            async_openai::types::images::Image::Url { url, .. } => Some(url.clone()),
            async_openai::types::images::Image::B64Json { b64_json, .. } => {
                Some(format!("data:image/png;base64,{}", b64_json))
            }
        }).collect())
    }
}

#[async_trait]
impl ImageGenerationPort for OpenAiImageClient {
    async fn generate(&self, p: GenerateParams) -> DomainResult<Vec<String>> {
        let size = if matches!(self.config.flavor, Flavor::NanoBanana) {
            match p.ratio.as_deref().unwrap_or("1:1") {
                "16:9" | "21:9" => ImageSize::S1792x1024, "9:16" => ImageSize::S1024x1792, _ => ImageSize::S1024x1024
            }
        } else { OpenAiUtils::parse_size(p.size.as_deref()) };
        self.gen_int(p, size).await
    }

    async fn list_models(&self) -> DomainResult<Vec<String>> { Ok(vec!["gpt-image-1".into(), "dall-e".into()]) }

    async fn edit(&self, p: GenerateParams) -> DomainResult<Vec<String>> {
        let img_id = p.image_id.clone().ok_or_else(|| DomainError::Image("Missing image".into()))?;
        let bytes = if img_id.starts_with("data:") {
            BASE64_STANDARD.decode(img_id.split(',').nth(1).ok_or_else(|| DomainError::Image("Invalid data URL".into()))?).map_err(|e| DomainError::Image(e.to_string()))?
        } else { reqwest::get(&img_id).await.map_err(DomainError::Http)?.bytes().await.map_err(DomainError::Http)?.to_vec() };

        let size = match OpenAiUtils::parse_size(p.size.as_deref()) {
            ImageSize::S256x256 => "256x256", ImageSize::S512x512 => "512x512", _ => "1024x1024",
        };

        // Refined manual multipart request
        let mut form = reqwest::multipart::Form::new()
            .part("image", reqwest::multipart::Part::bytes(bytes).file_name("image.png").mime_str("image/png").unwrap());
        
        if p.n.unwrap_or(1) != 1 { form = form.text("n", p.n.unwrap_or(1).to_string()); }
        if size != "1024x1024" { form = form.text("size", size.to_string()); }
        form = form.text("response_format", "url");

        let url = format!("{}/images/variations", self.config.base_url.trim_end_matches('/'));
        let resp = reqwest::Client::new().post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .multipart(form).send().await.map_err(DomainError::Http)?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(DomainError::Image(format!("OpenAI Provider Error ({}): {}", status, body)));
        }

        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            tracing::error!("OpenAI Variation decode error: {}. Body: {}", e, body);
            DomainError::Image(format!("error decoding response body: {}", e))
        })?;

        // Extract URLs or Base64 from the data array
        let imgs = json["data"].as_array().map(|arr| {
            arr.iter().filter_map(|item| {
                item["url"].as_str().map(|s| s.to_string())
                    .or_else(|| item["b64_json"].as_str().map(|s| format!("data:image/png;base64,{}", s)))
            }).collect()
        }).unwrap_or_default();

        Ok(imgs)
    }
}
