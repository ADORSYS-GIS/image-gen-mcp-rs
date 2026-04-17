use std::sync::Arc;

use crate::core::{
    errors::DomainResult,
    traits::{GenerateParams, ImageGenerationPort},
};

pub struct ImageGenerationService {
    client: Arc<dyn ImageGenerationPort>,
    default_model: String,
}

impl ImageGenerationService {
    pub fn new(client: Arc<dyn ImageGenerationPort>, default_model: String) -> Self {
        Self {
            client,
            default_model,
        }
    }

    pub async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        self.client.generate(params).await
    }

    pub async fn continue_editing(
        &self,
        context_prompt: &str,
        additional_prompt: &str,
        model: Option<&str>,
    ) -> DomainResult<Vec<String>> {
        let combined = format!("{} {}", context_prompt, additional_prompt);
        let params = GenerateParams::new(combined)
            .with_model(model.unwrap_or(&self.default_model).to_string());

        self.client.generate(params).await
    }
}
