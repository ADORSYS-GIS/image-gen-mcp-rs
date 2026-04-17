use rust_mcp_sdk::schema::CallToolRequestParams;

use crate::{
    adapter::{handler::parse_args, tools::GenerateImageNanoTool},
    application::image::ImageGenerationService,
    core::traits::GenerateParams,
};

use std::sync::Arc;

pub async fn handle_generate_image_nano(
    service: Arc<ImageGenerationService>,
    default_model: String,
    params: &CallToolRequestParams,
) -> std::result::Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::CallToolError>
{
    let tool: GenerateImageNanoTool = parse_args(params)?;
    let model = tool.model.clone().unwrap_or_else(|| default_model.clone());
    let ratio = tool.ratio.clone().unwrap_or_else(|| "1:1".to_string());

    let gen_params = GenerateParams::new(tool.prompt)
        .with_model(model.clone())
        .with_ratio(ratio.clone())
        .with_n(tool.n.unwrap_or(1))
        .with_seed(tool.seed.unwrap_or(0));

    let urls = service
        .generate(gen_params)
        .await
        .map_err(rust_mcp_sdk::schema::CallToolError::new)?;

    Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
        serde_json::to_string(&serde_json::json!({
            "success": true,
            "images": urls,
            "model": model,
            "ratio": ratio,
            "seed": tool.seed,
        }))
        .unwrap_or_default()
        .into(),
    ]))
}
