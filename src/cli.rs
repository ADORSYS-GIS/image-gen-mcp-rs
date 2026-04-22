use clap::Parser;

/// MCP Server for Image Generation and Editing
///
/// Supports OpenAI-compatible endpoints including nanobanana and gpt-image-gen.
#[derive(Parser, Debug, Clone)]
#[command(name = "image-mcp")]
#[command(version)]
#[command(about = "MCP server for image generation via OpenAI-compatible APIs")]
pub struct Cli {
    /// OpenAI-compatible API key
    #[arg(long, env = "OPENAI_API_KEY")]
    pub api_key: String,

    /// Base URL for OpenAI-compatible API
    #[arg(
        long,
        env = "OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    pub base_url: String,

    /// Default image generation model
    #[arg(long, env = "IMAGE_MODEL", default_value = "nano-banana")]
    pub image_model: String,

    /// Transport mode: stdio or http
    #[arg(long, env = "TRANSPORT_MODE", default_value = "stdio")]
    pub transport_mode: String,

    /// HTTP server host (when transport_mode is http)
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    pub host: String,

    /// HTTP server port (when transport_mode is http)
    #[arg(long, env = "PORT", default_value = "8080")]
    pub port: u16,

    /// Enable nano-banana flavor with ratio and seed support
    #[arg(long, env = "NANO_BANANA", default_value = "false")]
    pub nano_banana: bool,

    /// Enable OpenAI generation flavor with quality and style support
    #[arg(long, env = "OPENAI_GEN", default_value = "false")]
    pub openai_gen: bool,

    /// Output format: url or file
    #[arg(long, env = "OUTPUT_FORMAT", default_value = "url")]
    pub output_format: String,

    /// Output directory for files (when output_format is file)
    #[arg(long, env = "OUTPUT_DIR", default_value = "./outputs")]
    pub output_dir: String,
}

impl Cli {
    pub fn parse_config() -> Self {
        Self::parse()
    }

    pub fn flavor(&self) -> Flavor {
        if self.nano_banana {
            Flavor::NanoBanana
        } else if self.openai_gen {
            Flavor::OpenAiGen
        } else {
            Flavor::Standard
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Standard,
    NanoBanana,
    OpenAiGen,
}
