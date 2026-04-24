use std::sync::Arc;

use crate::core::{
    config::AppConfig,
    errors::DomainResult,
    traits::{GenerateParams, ImageGenerationPort},
};

pub struct ImageGenerationService {
    client: Arc<dyn ImageGenerationPort>,
    config: AppConfig,
    session: crate::core::session::SessionStore,
}

impl ImageGenerationService {
    pub fn new(
        client: Arc<dyn ImageGenerationPort>,
        config: AppConfig,
        session: crate::core::session::SessionStore,
    ) -> Self {
        Self {
            client,
            config,
            session,
        }
    }

    pub async fn generate(&self, params: GenerateParams) -> DomainResult<Vec<serde_json::Value>> {
        let results = self.client.generate(params).await?;
        self.process_and_store_results(results).await
    }

    pub async fn edit(&self, mut params: GenerateParams) -> DomainResult<Vec<serde_json::Value>> {
        let image_id = params
            .image_id
            .clone()
            .ok_or_else(|| crate::core::errors::DomainError::Image("image_id is required for editing".to_string()))?;

        let image_data = self.session.get(&image_id).await.ok_or_else(|| {
            crate::core::errors::DomainError::Image(format!("Image ID {} not found in session", image_id))
        })?;

        // Replace image_id with actual data/URL for the infrastructure layer
        params.image_id = Some(image_data);

        let results = self.client.edit(params).await?;
        self.process_and_store_results(results).await
    }

    async fn process_and_store_results(&self, results: Vec<String>) -> DomainResult<Vec<serde_json::Value>> {
        let mut final_results = Vec::new();

        for data in results {
            let id = cuid2::create_id();
            self.session.store(id.clone(), data.clone()).await;

            let output = if self.config.output_format == crate::core::config::OutputFormat::File {
                crate::core::output::ImageSaver::save_image(
                    &data,
                    &self.config.output_dir,
                    "gen",
                )
                .await?
            } else {
                data
            };

            // Return structured JSON
            final_results.push(serde_json::json!({
                "id": id,
                "url": output,
            }));
        }

        Ok(final_results)
    }

    pub async fn list_models(&self) -> DomainResult<Vec<String>> {
        self.client.list_models().await
    }
}
