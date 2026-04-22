use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

// Standard flavor tools
#[mcp_tool(
    name = "generate_image",
    title = "Generate Image",
    description = "Generate a new image from a text prompt using the configured model. Supports size parameter.",
    destructive_hint = false,
    idempotent_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenerateImageTool {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Size: 256, 512, 1024, 1792x1024, 1024x1792
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
}

// Nano-banana specific tools
#[mcp_tool(
    name = "generate_image_nano",
    title = "Generate Image (Nano-Banana)",
    description = "Generate image with nano-banana specific features: ratio and seed support.",
    destructive_hint = false,
    idempotent_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenerateImageNanoTool {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Aspect ratio: 1:1, 16:9, 9:16, 4:3, 3:4, 21:9
    #[serde(rename = "ratio", skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    /// Seed for reproducible generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

// OpenAI-gen specific tools
#[mcp_tool(
    name = "generate_image_openai",
    title = "Generate Image (OpenAI Style)",
    description = "Generate image with OpenAI-specific features: quality (standard/hd) and style (vivid/natural).",
    destructive_hint = false,
    idempotent_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenerateImageOpenAiTool {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Size: 1024x1024, 1792x1024, 1024x1792
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    /// Quality: standard or hd
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Style: vivid or natural
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

#[mcp_tool(
    name = "continue_edit",
    title = "Continue Editing",
    description = "Refine or edit an existing image by referencing its ID.",
    destructive_hint = false,
    idempotent_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ContinueEditTool {
    /// The ID of the image to edit (e.g. from a previous generation)
    pub image_id: String,
    /// What changes to apply to the image
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[mcp_tool(
    name = "list_models",
    title = "List Available Models",
    description = "List available image generation models and current configuration",
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListModelsTool {}
