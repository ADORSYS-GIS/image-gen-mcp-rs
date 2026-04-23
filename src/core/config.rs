use std::path::PathBuf;

/// Configuration for the image generation service
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub base_url: String,
    pub image_model: String,
    pub transport_mode: TransportMode,
    pub host: String,
    pub flavor: crate::cli::Flavor,
    pub output_format: OutputFormat,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Stdio,
    Http,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Url,
    File,
}

impl AppConfig {
    pub fn from_cli(cli: crate::cli::Cli) -> Self {
        let flavor = cli.flavor();

        // Default to Google Gemini URL if nano-banana flavor is enabled and URL is the default
        let base_url = if flavor == crate::cli::Flavor::NanoBanana
            && cli.base_url == "https://api.openai.com/v1"
        {
            "https://generativelanguage.googleapis.com/v1beta".to_string()
        } else {
            cli.base_url
        };

        // Set flavor-specific default model if user didn't specify one
        // OpenAiGen flavor uses gpt-image-1 as default for semantic image editing
        // (dall-e-2/3 edits are inpainting-only, not semantic continuation)
        let image_model = if cli.image_model == "nano-banana" {
            match flavor {
                crate::cli::Flavor::OpenAiGen => "gpt-image-1".to_string(),
                _ => cli.image_model,
            }
        } else {
            cli.image_model
        };

        Self {
            api_key: cli.api_key,
            base_url,
            image_model,
            transport_mode: if cli.transport_mode == "http" {
                TransportMode::Http
            } else {
                TransportMode::Stdio
            },
            host: cli.host,
            flavor,
            output_format: if cli.output_format == "file" {
                OutputFormat::File
            } else {
                OutputFormat::Url
            },
            output_dir: PathBuf::from(cli.output_dir),
        }
    }
}
