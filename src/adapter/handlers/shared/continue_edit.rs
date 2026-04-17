use rust_mcp_sdk::schema::CallToolRequestParams;

use crate::{
    adapter::handler::parse_args,
    application::image::ImageGenerationService,
    core::traits::GenerateParams,
};

use std::sync::Arc;

pub async fn handle_continue_edit(
    service: Arc<ImageGenerationService>,
    default_model: String,
    params: &CallToolRequestParams,
) -> std::result::Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::CallToolError> {
    let tool: crate::adapter::tools::ContinueEditTool = parse_args(params)?;
    let gen_params = GenerateParams::new(format!(
        "{} {}",
        tool.context_prompt, tool.additional_prompt
    ))
    .with_model(tool.model.unwrap_or(default_model))
    .with_n(1);

    let urls = service
        .generate(gen_params)
        .await
        .map_err(rust_mcp_sdk::schema::CallToolError::new)?;

    Ok(rust_mcp_sdk::schema::CallToolResult::text_content(vec![
        serde_json::to_string(&serde_json::json!({
            "success": true,
            "images": urls,
        }))
        .unwrap_or_default()
        .into(),
    ]))
}
