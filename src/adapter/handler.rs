#![allow(clippy::enum_variant_names)]

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
    serde_json::from_value(serde_json::Value::Object(args.clone()))
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
            "continue_edit" => {
                handlers::shared::continue_edit::handle_continue_edit(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "list_models" => {
                Ok(handlers::shared::list_models::build_list_models_response(
                    self.image_service.clone(),
                    self.flavor,
                    self.default_model.clone(),
                )
                .await)
            }
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
            "continue_edit" => {
                handlers::shared::continue_edit::handle_continue_edit(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "list_models" => {
                Ok(handlers::shared::list_models::build_list_models_response(
                    self.image_service.clone(),
                    self.flavor,
                    self.default_model.clone(),
                )
                .await)
            }
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
            "continue_edit" => {
                handlers::shared::continue_edit::handle_continue_edit(
                    self.image_service.clone(),
                    self.default_model.clone(),
                    params,
                )
                .await
            }
            "list_models" => {
                Ok(handlers::shared::list_models::build_list_models_response(
                    self.image_service.clone(),
                    self.flavor,
                    self.default_model.clone(),
                )
                .await)
            }
            _ => Ok(CallToolResult::text_content(vec![
                format!("Tool '{}' not supported", params.name).into(),
            ])),
        }
    }
}
