use std::sync::Arc;

use crate::{application::image::ImageGenerationService, cli::Flavor};

pub async fn build_list_models_response(
    service: Arc<ImageGenerationService>,
    flavor: Flavor,
    default_model: String,
) -> rust_mcp_sdk::schema::CallToolResult {
    let (friendly_models, flavor_name) = match flavor {
        Flavor::NanoBanana => (vec!["nano-banana", "nano-banana2"], "nano-banana"),
        Flavor::OpenAiGen => (vec!["gpt-image-1", "dall-e"], "openai-gen"),
        Flavor::Standard => (
            vec!["nano-banana", "nano-banana2", "gpt-image-1", "dall-e"],
            "standard",
        ),
    };

    let live_models = service.list_models().await.unwrap_or_default();

    rust_mcp_sdk::schema::CallToolResult::text_content(vec![
        serde_json::to_string(&serde_json::json!({
            "success": true,
            "flavor": flavor_name,
            "models": friendly_models,
            "available_api_models": live_models,
            "default_model": default_model,
        }))
        .unwrap_or_default()
        .into(),
    ])
}
