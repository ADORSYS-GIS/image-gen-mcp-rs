use std::sync::Arc;
use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer, mcp_server::ServerHandler,
    schema::{CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError, Tool},
    tool_box,
};
use crate::{adapter::{handlers, tools::*}, application::image::ImageGenerationService, cli::Flavor};

tool_box!(StandardTools, [GenerateImageTool, ContinueEditTool, ListModelsTool]);
tool_box!(NanoBananaTools, [GenerateImageNanoTool, ContinueEditTool, ListModelsTool]);
tool_box!(OpenAiGenTools, [GenerateImageOpenAiTool, ContinueEditTool, ListModelsTool]);

pub struct McpHandler {
    image_service: Arc<ImageGenerationService>,
    default_model: String,
    flavor: Flavor,
}

impl McpHandler {
    pub fn new(service: Arc<ImageGenerationService>, default_model: String, flavor: Flavor) -> Self {
        Self { image_service: service, default_model, flavor }
    }

    fn tools(&self) -> Vec<Tool> {
        match self.flavor {
            Flavor::NanoBanana => NanoBananaTools::tools(),
            Flavor::OpenAiGen => OpenAiGenTools::tools(),
            Flavor::Standard => StandardTools::tools(),
        }
    }
}

pub fn parse_args<T: serde::de::DeserializeOwned>(params: &CallToolRequestParams) -> Result<T, CallToolError> {
    let args = params.arguments.as_ref().ok_or_else(|| CallToolError::invalid_arguments(&params.name, Some("Missing arguments".into())))?;
    serde_json::from_value(serde_json::Value::Object(args.clone())).map_err(|e| CallToolError::invalid_arguments(&params.name, Some(e.to_string())))
}

#[async_trait]
impl ServerHandler for McpHandler {
    async fn handle_list_tools_request(&self, _req: Option<PaginatedRequestParams>, _rt: Arc<dyn McpServer>) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult { tools: self.tools(), meta: None, next_cursor: None })
    }

    async fn handle_call_tool_request(&self, params: CallToolRequestParams, _rt: Arc<dyn McpServer>) -> Result<CallToolResult, CallToolError> {
        match self.flavor {
            Flavor::NanoBanana => handlers::nano_banana::dispatch(self.image_service.clone(), self.default_model.clone(), &params).await,
            Flavor::OpenAiGen => handlers::openai_gen::dispatch(self.image_service.clone(), self.default_model.clone(), &params).await,
            Flavor::Standard => handlers::standard::dispatch(self.image_service.clone(), self.default_model.clone(), &params).await,
        }
    }
}
