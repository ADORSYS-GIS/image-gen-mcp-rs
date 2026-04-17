use async_openai::{
    Client,
    config::OpenAIConfig,
    types::images::{CreateImageRequestArgs, ImageModel, ImageResponseFormat, ImageSize},
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

        let client = Client::with_config(openai_config);
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
        model
            .map(|m| ImageModel::Other(m.to_string()))
            .unwrap_or(ImageModel::Other(self.config.image_model.clone()))
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
            .response_format(ImageResponseFormat::Url)
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

        // Build using builder to avoid missing fields issues
        let mut builder = CreateImageRequestArgs::default();
        builder.prompt(&params.prompt);
        builder.model(model);
        builder.n(n);
        builder.size(size);
        builder.response_format(ImageResponseFormat::Url);

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

    async fn download(&self, url: &str) -> DomainResult<Vec<u8>> {
        let response = reqwest::get(url).await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
