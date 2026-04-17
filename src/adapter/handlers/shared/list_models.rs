use crate::cli::Flavor;

pub fn build_list_models_response(
    flavor: Flavor,
    default_model: String,
) -> rust_mcp_sdk::schema::CallToolResult {
    let (models, flavor_name) = match flavor {
        Flavor::NanoBanana => (vec!["nano-banana", "gpt-image-1"], "nano-banana"),
        Flavor::OpenAiGen => (vec!["dall-e-3", "dall-e-2", "gpt-image-1"], "openai-gen"),
        Flavor::Standard => (
            vec!["nano-banana", "dall-e-3", "dall-e-2", "gpt-image-1"],
            "standard",
        ),
    };

    rust_mcp_sdk::schema::CallToolResult::text_content(vec![serde_json::to_string(
        &serde_json::json!({
            "success": true,
            "flavor": flavor_name,
            "models": models,
            "default_model": default_model,
        }),
    )
    .unwrap_or_default()
    .into()])
}
