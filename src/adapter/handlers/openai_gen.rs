use rust_mcp_sdk::schema::CallToolRequestParams;

use crate::{
    adapter::{handler::parse_args, tools::GenerateImageOpenAiTool},
    application::image::ImageGenerationService,
    core::traits::GenerateParams,
};

use std::sync::Arc;

pub async fn handle_generate_image_openai(
    service: Arc<ImageGenerationService>,
    default_model: String,
    params: &CallToolRequestParams,
) -> std::result::Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::CallToolError>
{
    let tool: GenerateImageOpenAiTool = parse_args(params)?;
    let model = tool.model.clone().unwrap_or_else(|| default_model.clone());
    let size = tool.size.clone().unwrap_or_else(|| "1024x1024".to_string());
    let quality = tool
        .quality
        .clone()
        .unwrap_or_else(|| "standard".to_string());
    let style = tool.style.clone().unwrap_or_else(|| "vivid".to_string());

    let gen_params = GenerateParams::new(tool.prompt)
        .with_model(model.clone())
        .with_size(size)
        .with_n(tool.n.unwrap_or(1))
        .with_quality(quality.clone())
        .with_style(style.clone());

    let urls = service
        .generate(gen_params)
        .await
        .map_err(rust_mcp_sdk::schema::CallToolError::new)?;

    Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
        serde_json::to_string(&serde_json::json!({
            "success": true,
            "images": urls,
            "model": model,
            "quality": quality,
            "style": style,
        }))
        .unwrap_or_default()
        .into(),
    ]))
}
