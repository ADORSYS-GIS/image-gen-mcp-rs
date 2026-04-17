/// Configuration for the image generation service
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub base_url: String,
    pub image_model: String,
    pub embedding_model: String,
    pub transport_mode: TransportMode,
    pub host: String,
    pub flavor: crate::cli::Flavor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Stdio,
    Http,
}

impl AppConfig {
    pub fn from_cli(cli: crate::cli::Cli) -> Self {
        let flavor = cli.flavor();
        Self {
            api_key: cli.api_key,
            base_url: cli.base_url,
            image_model: cli.image_model,
            embedding_model: cli.embedding_model,
            transport_mode: if cli.transport_mode == "http" {
                TransportMode::Http
            } else {
                TransportMode::Stdio
            },
            host: cli.host,
            flavor,
        }
    }
}
