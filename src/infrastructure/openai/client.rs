use async_openai::{
    Client,
    config::OpenAIConfig,
    types::images::{CreateImageRequestArgs, ImageModel, ImageSize},
};
use async_trait::async_trait;

use crate::{
    cli::Flavor,
    core::{
        config::AppConfig,
        errors::{DomainError, DomainResult},
        traits::{GenerateParams, ImageGenerationPort},
    },
};

pub struct OpenAiImageClient {
    client: Client<OpenAIConfig>,
    config: AppConfig,
}

impl OpenAiImageClient {
    pub fn new(config: AppConfig) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(&config.api_key)
            .with_api_base(&config.base_url);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let client = Client::with_config(openai_config).with_http_client(http_client);
        Self { client, config }
    }

    fn parse_size(&self, size: Option<&str>) -> ImageSize {
        match size {
            Some("256") => ImageSize::S256x256,
            Some("512") => ImageSize::S512x512,
            Some("1024") => ImageSize::S1024x1024,
            Some("1792x1024") => ImageSize::S1792x1024,
            Some("1024x1792") => ImageSize::S1024x1792,
            Some("1536x1024") => ImageSize::S1536x1024,
            Some("1024x1536") => ImageSize::S1024x1536,
            _ => ImageSize::S1024x1024,
        }
    }

    fn parse_model(&self, model: Option<&str>) -> ImageModel {
        let model_name = match model {
            Some("dall-e") => "dall-e-3",
            Some("gpt-image-1") => "gpt-image-1",
            Some(m) => m,
            None => &self.config.image_model,
        };
        ImageModel::Other(model_name.to_string())
    }

    async fn generate_standard(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        let n = params.n.unwrap_or(1);
        let size = self.parse_size(params.size.as_deref());
        let model = self.parse_model(params.model.as_deref());

        let request = CreateImageRequestArgs::default()
            .prompt(&params.prompt)
            .model(model)
            .n(n)
            .size(size)
            .build()
            .map_err(DomainError::OpenAi)?;

        let response = self.client.images().generate(request).await?;

        let urls: Vec<String> = response
            .data
            .iter()
            .filter_map(|img| match img.as_ref() {
                async_openai::types::images::Image::Url { url, .. } => Some(url.clone()),
                _ => None,
            })
            .collect();

        Ok(urls)
    }

    async fn generate_nano_banana(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        let ratio = params.ratio.as_deref().unwrap_or("1:1");
        let size = match ratio {
            "1:1" => ImageSize::S1024x1024,
            "16:9" => ImageSize::S1792x1024,
            "9:16" => ImageSize::S1024x1792,
            "21:9" => ImageSize::S1792x1024,
            "4:3" => ImageSize::S1024x1024,
            "3:4" => ImageSize::S1024x1024,
            _ => ImageSize::S1024x1024,
        };

        let n = params.n.unwrap_or(1);
        let model = self.parse_model(params.model.as_deref());

        let mut builder = CreateImageRequestArgs::default();
        builder.prompt(&params.prompt);
        builder.model(model);
        builder.n(n);
        builder.size(size);

        if let Some(seed) = params.seed {
            builder.user(format!("seed:{}", seed));
        }

        let request = builder.build().map_err(DomainError::OpenAi)?;

        let response = self.client.images().generate(request).await?;

        let urls: Vec<String> = response
            .data
            .iter()
            .filter_map(|img| match img.as_ref() {
                async_openai::types::images::Image::Url { url, .. } => Some(url.clone()),
                _ => None,
            })
            .collect();

        Ok(urls)
    }
}

#[async_trait]
impl ImageGenerationPort for OpenAiImageClient {
    async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        match self.config.flavor {
            Flavor::NanoBanana => self.generate_nano_banana(params).await,
            _ => self.generate_standard(params).await,
        }
    }

    async fn list_models(&self) -> DomainResult<Vec<String>> {
        Ok(vec!["gpt-image-1".to_string(), "dall-e".to_string()])
    }

    async fn edit(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        let image_data = params
            .image_id
            .clone()
            .ok_or_else(|| DomainError::Image("Missing image for edit".to_string()))?;

        // OpenAI edits/variations require actual files/bytes.
        // We need to download the image first.
        let bytes = if image_data.starts_with("data:") {
            let parts: Vec<&str> = image_data.split(',').collect();
            if parts.len() != 2 {
                return Err(DomainError::Image("Invalid image data".to_string()));
            }
            BASE64_STANDARD.decode(parts[1])
                .map_err(|e| DomainError::Image(format!("Base64 error: {}", e)))?
        } else {
            let resp = reqwest::get(&image_data).await.map_err(DomainError::Http)?;
            resp.bytes().await.map_err(DomainError::Http)?.to_vec()
        };

        use async_openai::types::images::{CreateImageVariationRequestArgs, DallE2ImageSize};

        let size = match self.parse_size(params.size.as_deref()) {
            ImageSize::S256x256 => DallE2ImageSize::S256x256,
            ImageSize::S512x512 => DallE2ImageSize::S512x512,
            _ => DallE2ImageSize::S1024x1024,
        };

        let request = CreateImageVariationRequestArgs::default()
            .image(async_openai::types::images::ImageInput::from_vec_u8("image.png".to_string(), bytes))
            .n(params.n.unwrap_or(1))
            .size(size)
            .build()
            .map_err(DomainError::OpenAi)?;

        let response = self.client.images().create_variation(request).await?;

        let urls: Vec<String> = response
            .data
            .iter()
            .filter_map(|img| match img.as_ref() {
                async_openai::types::images::Image::Url { url, .. } => Some(url.clone()),
                _ => None,
            })
            .collect();

        Ok(urls)
    }
}

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
