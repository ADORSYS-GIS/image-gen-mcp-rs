use mimalloc::MiMalloc;
use std::sync::Arc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod adapter; mod application; mod cli; mod core; mod infrastructure;

use adapter::{handler::McpHandler, metadata::get_server_details};
use application::image::ImageGenerationService;
use cli::Cli;
use core::config::{AppConfig, TransportMode};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    tracing::info!("Starting Image Generation MCP Server v{}", env!("CARGO_PKG_VERSION"));
    tokio::select! {
        res = run() => { if let Err(e) = res { tracing::error!("App error: {}", e); std::process::exit(1); } }
        _ = tokio::signal::ctrl_c() => { tracing::info!("Shutdown gracefully..."); }
    }
}

async fn run() -> rust_mcp_sdk::error::SdkResult<()> {
    let cli = Cli::parse_config();
    let config = AppConfig::from_cli(cli.clone());
    if config.api_key.is_empty() { tracing::error!("API_KEY missing!"); } else { tracing::info!("Flavor: {:?}", config.flavor); }
    match config.transport_mode {
        TransportMode::Stdio => run_stdio(config).await, TransportMode::Http => run_http(config, cli.port).await,
    }
}

async fn run_stdio(config: AppConfig) -> rust_mcp_sdk::error::SdkResult<()> {
    use rust_mcp_sdk::{McpServer, StdioTransport, TransportOptions, mcp_server::{McpServerOptions, ToMcpServerHandler, server_runtime::create_server}};
    let handler = create_handler(config.clone());
    let server = create_server(McpServerOptions {
        server_details: get_server_details(config.flavor), transport: StdioTransport::new(TransportOptions::default())?,
        handler: handler.to_mcp_server_handler(), task_store: None, client_task_store: None, message_observer: None,
    });
    server.start().await
}

async fn run_http(config: AppConfig, port: u16) -> rust_mcp_sdk::error::SdkResult<()> {
    use rust_mcp_axum::{AxumServer, AxumServerOptions};
    use rust_mcp_sdk::{event_store::InMemoryEventStore, mcp_server::ToMcpServerHandler};
    let handler = create_handler(config.clone());
    let server = AxumServer::new(
        get_server_details(config.flavor),
        handler.to_mcp_server_handler(),
        AxumServerOptions {
            host: config.host,
            port,
            event_store: Some(Arc::new(InMemoryEventStore::default())),
            sse_support: true,
            ..Default::default()
        },
    );
    server.start().await
}

fn create_handler(config: AppConfig) -> McpHandler {
    let client = infrastructure::factory::ProviderFactory::create_client(config.clone());
    let service = Arc::new(ImageGenerationService::new(client, config.clone(), core::session::SessionStore::new()));
    McpHandler::new(service, config.image_model, config.flavor)
}
