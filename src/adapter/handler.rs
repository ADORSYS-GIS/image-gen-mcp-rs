use std::sync::Arc;

use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{
        CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult,
        PaginatedRequestParams, RpcError, Tool,
    },
    tool_box,
};

use crate::{
    adapter::{handlers, tools::*},
    application::image::ImageGenerationService,
    cli::Flavor,
    core::traits::GenerateParams,
};

// Tool sets per flavor
tool_box!(
    StandardTools,
    [GenerateImageTool, ContinueEditTool, ListModelsTool]
);
tool_box!(
    NanoBananaTools,
    [GenerateImageNanoTool, ContinueEditTool, ListModelsTool]
);
tool_box!(
    OpenAiGenTools,
    [GenerateImageOpenAiTool, ContinueEditTool, ListModelsTool]
);

pub struct McpHandler {
    image_service: Arc<ImageGenerationService>,
    default_model: String,
    flavor: Flavor,
}

impl McpHandler {
    pub fn new(
        service: Arc<ImageGenerationService>,
        default_model: String,
        flavor: Flavor,
    ) -> Self {
        Self {
            image_service: service,
            default_model,
            flavor,
        }
    }

    fn tools(&self) -> Vec<Tool> {
        match self.flavor {
            Flavor::NanoBanana => NanoBananaTools::tools(),
            Flavor::OpenAiGen => OpenAiGenTools::tools(),
            Flavor::Standard => StandardTools::tools(),
        }
    }
}

pub fn parse_args<T: serde::de::DeserializeOwned>(
    params: &CallToolRequestParams,
) -> std::result::Result<T, CallToolError> {
    let args = params.arguments.as_ref().ok_or_else(|| {
        CallToolError::invalid_arguments(&params.name, Some("Missing arguments".into()))
    })?;
    let json = serde_json::Value::Object(args.clone());
    serde_json::from_value(json)
        .map_err(|e| CallToolError::invalid_arguments(&params.name, Some(e.to_string())))
}

#[async_trait]
impl ServerHandler for McpHandler {
    async fn handle_list_tools_request(
        &self,
        _req: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: self.tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match self.flavor {
            Flavor::NanoBanana => self.handle_nano_banana(&params).await,
            Flavor::OpenAiGen => self.handle_openai_gen(&params).await,
            Flavor::Standard => self.handle_standard(&params).await,
        }
    }
}

// Shared handlers
impl McpHandler {
    async fn handle_continue_edit(
        &self,
        params: &CallToolRequestParams,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool: ContinueEditTool = parse_args(params)?;
        let gen_params = GenerateParams::new(format!(
            "{} {}",
            tool.context_prompt, tool.additional_prompt
        ))
        .with_model(tool.model.unwrap_or_else(|| self.default_model.clone()))
        .with_n(1);
        let urls = self
            .image_service
            .generate(gen_params)
            .await
            .map_err(CallToolError::new)?;
        Ok(CallToolResult::text_content(vec![
            serde_json::to_string(&serde_json::json!({
                "success": true, "images": urls,
            }))
            .unwrap_or_default()
            .into(),
        ]))
    }

    async fn handle_list_models(
        &self,
        _params: &CallToolRequestParams,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let (models, flavor_name) = match self.flavor {
            Flavor::NanoBanana => (vec!["nano-banana", "gpt-image-1"], "nano-banana"),
            Flavor::OpenAiGen => (vec!["dall-e-3", "dall-e-2", "gpt-image-1"], "openai-gen"),
            Flavor::Standard => (
                vec!["nano-banana", "dall-e-3", "dall-e-2", "gpt-image-1"],
                "standard",
            ),
        };
        Ok(CallToolResult::text_content(vec![serde_json::to_string(&serde_json::json!({
            "success": true, "flavor": flavor_name, "models": models, "default_model": self.default_model,
        })).unwrap_or_default().into()]))
    }
}

// Flavor-specific handlers
impl McpHandler {
    async fn handle_standard(
        &self,
        params: &CallToolRequestParams,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "generate_image" => {
                handlers::standard::handle_generate_image(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "continue_edit" => self.handle_continue_edit(params).await,
            "list_models" => self.handle_list_models(params).await,
            _ => Ok(CallToolResult::text_content(vec![
                format!("Tool '{}' not supported", params.name).into(),
            ])),
        }
    }

    async fn handle_nano_banana(
        &self,
        params: &CallToolRequestParams,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "generate_image_nano" => {
                handlers::nano_banana::handle_generate_image_nano(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "continue_edit" => self.handle_continue_edit(params).await,
            "list_models" => self.handle_list_models(params).await,
            _ => Ok(CallToolResult::text_content(vec![
                format!("Tool '{}' not supported", params.name).into(),
            ])),
        }
    }

    async fn handle_openai_gen(
        &self,
        params: &CallToolRequestParams,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "generate_image_openai" => {
                handlers::openai_gen::handle_generate_image_openai(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "continue_edit" => self.handle_continue_edit(params).await,
            "list_models" => self.handle_list_models(params).await,
            _ => Ok(CallToolResult::text_content(vec![
                format!("Tool '{}' not supported", params.name).into(),
            ])),
        }
    }
}
