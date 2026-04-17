use std::sync::Arc;

use crate::core::{
    errors::DomainResult,
    traits::{GenerateParams, ImageGenerationPort},
};

pub struct ImageGenerationService {
    client: Arc<dyn ImageGenerationPort>,
}

impl ImageGenerationService {
    pub fn new(client: Arc<dyn ImageGenerationPort>) -> Self {
        Self { client }
    }

    pub async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<String>> {
        self.client.generate(params).await
    }
}
