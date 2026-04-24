use std::sync::Arc;

use crate::{
    core::{config::AppConfig, traits::ImageGenerationPort},
    infrastructure::{google::client::GeminiImageClient, openai::OpenAiImageClient},
};

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_client(config: AppConfig) -> Arc<dyn ImageGenerationPort> {
        // Simple heuristic: if base_url contains googleapis, use Gemini client
        if config.base_url.contains("googleapis.com") {
            Arc::new(GeminiImageClient::new(config))
        } else {
            Arc::new(OpenAiImageClient::new(config))
        }
    }
}
