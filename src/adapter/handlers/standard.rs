use rust_mcp_sdk::schema::{CallToolRequestParams, CallToolResult, CallToolError};
use crate::{
    adapter::{handler::parse_args, tools::GenerateImageTool, handlers::shared},
    application::image::ImageGenerationService,
    core::traits::GenerateParams,
    cli::Flavor,
};
use std::sync::Arc;

pub async fn handle_generate_image(
    service: Arc<ImageGenerationService>,
    default_model: String,
    params: &CallToolRequestParams,
) -> Result<CallToolResult, CallToolError> {
    let tool: GenerateImageTool = parse_args(params)?;
    let model = tool.model.clone().unwrap_or_else(|| default_model.clone());

    let gen_params = GenerateParams::new(tool.prompt)
        .with_model(model.clone())
        .with_size(tool.size.clone().unwrap_or_else(|| "1024x1024".to_string()))
        .with_n(tool.n.unwrap_or(1));

    let urls = service.generate(gen_params).await.map_err(CallToolError::new)?;

    Ok(CallToolResult::text_content(vec![
        serde_json::to_string(&serde_json::json!({
            "success": true, "images": urls, "model": model,
        })).unwrap_or_default().into(),
    ]))
}

pub async fn dispatch(
    service: Arc<ImageGenerationService>,
    default_model: String,
    params: &CallToolRequestParams,
) -> Result<CallToolResult, CallToolError> {
    match params.name.as_str() {
        "generate_image" => handle_generate_image(service, default_model, params).await,
        "continue_edit" => shared::continue_edit::handle_continue_edit(service, default_model, params).await,
        "list_models" => Ok(shared::list_models::build_list_models_response(service, Flavor::Standard, default_model).await),
        _ => Ok(CallToolResult::text_content(vec![format!("Tool '{}' not supported", params.name).into()])),
    }
}
