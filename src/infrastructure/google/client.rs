use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::{
    config::AppConfig,
    errors::{DomainError, DomainResult},
    traits::{GenerateParams, ImageGenerationPort},
};

pub struct GeminiImageClient {
    client: reqwest::Client,
    config: AppConfig,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_data: Option<InlineData>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    response_modalities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CandidatePart {
    inline_data: Option<InlineData>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

impl GeminiImageClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            config,
        }
    }

    fn map_model(&self, model: Option<&str>) -> String {
        match model {
            Some("nano-banana") | Some("nano-banana 1") => {
                "gemini-2.5-flash-image".to_string()
            }
            Some("nano-banana 2") | Some("nano-banana2") | Some("naano banana 2") => {
                "gemini-3.1-flash-image-preview".to_string()
            }
            Some("nano-banana-pro") => "nano-banana-pro-preview".to_string(),
            Some(m) => m.to_string(),
            None => {
                if self.config.image_model == "nano-banana" {
                    "gemini-2.5-flash-image".to_string()
                } else {
                    self.config.image_model.clone()
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct GeminiModelsResponse {
    models: Vec<GeminiModelInfo>,
}

#[derive(Deserialize)]
struct GeminiModelInfo {
    name: String,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}

#[async_trait]
impl ImageGenerationPort for GeminiImageClient {
    async fn list_models(&self) -> DomainResult<Vec<String>> {
        let url = format!(
            "{}/models?key={}",
            self.config.base_url, self.config.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(DomainError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DomainError::Image(format!(
                "Gemini API error ({}): {}",
                status, error_text
            )));
        }

        let models_resp: GeminiModelsResponse = response
            .json()
            .await
            .map_err(|e| DomainError::Image(format!("JSON error: {}", e)))?;

        let models = models_resp
            .models
            .into_iter()
            .filter(|m| {
                m.supported_generation_methods
                    .contains(&"generateContent".to_string())
            })
            .map(|m| m.name.replace("models/", ""))
            // Filter for models that are likely image generation models
            .filter(|name| {
                let name_lower = name.to_lowercase();
                name_lower.contains("image") || 
                name_lower.contains("imagen") || 
                name_lower.contains("banana") ||
                name_lower.contains("lyria")
            })
            .collect();

        Ok(models)
    }

    async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        let model = self.map_model(params.model.as_deref());
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.base_url, model, self.config.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: Some(params.prompt),
                    inline_data: None,
                }],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["IMAGE".to_string()],
                seed: params.seed,
            },
        };

        self.execute_request(&url, &request).await
    }

    async fn edit(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        let model = self.map_model(params.model.as_deref());
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.base_url, model, self.config.api_key
        );

        let image_data = params
            .image_id
            .clone()
            .ok_or_else(|| DomainError::Image("Missing image for edit".to_string()))?;

        // Extract base64 and mime type from data URL
        let (mime_type, data) = if image_data.starts_with("data:") {
            let parts: Vec<&str> = image_data.split(',').collect();
            if parts.len() != 2 {
                return Err(DomainError::Image("Invalid image data".to_string()));
            }
            let mime = image_data
                .split(':')
                .nth(1)
                .and_then(|s| s.split(';').nth(0))
                .unwrap_or("image/png")
                .to_string();
            (mime, parts[1].to_string())
        } else {
            // It's a URL, we need to download it
            let resp = self.client.get(&image_data).send().await?;
            let mime = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("image/png")
                .to_string();
            let bytes = resp.bytes().await?;
            (mime, BASE64_STANDARD.encode(bytes))
        };

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part {
                        text: None,
                        inline_data: Some(InlineData { mime_type, data }),
                    },
                    Part {
                        text: Some(params.prompt),
                        inline_data: None,
                    },
                ],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["IMAGE".to_string()],
                seed: params.seed,
            },
        };

        self.execute_request(&url, &request).await
    }
}

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

impl GeminiImageClient {
    async fn execute_request(&self, url: &str, request: &GeminiRequest) -> DomainResult<Vec<String>> {
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(DomainError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(DomainError::Image(format!(
                "Gemini API error ({}): {}",
                status, error_text
            )));
        }

        let gemini_resp: GeminiResponse = response
            .json()
            .await
            .map_err(|e| DomainError::Image(format!("JSON error: {}", e)))?;

        let mut images = Vec::new();
        for candidate in gemini_resp.candidates {
            for part in candidate.content.parts {
                if let Some(inline_data) = part.inline_data {
                    images.push(format!(
                        "data:{};base64,{}",
                        inline_data.mime_type, inline_data.data
                    ));
                }
            }
        }

        Ok(images)
    }
}
