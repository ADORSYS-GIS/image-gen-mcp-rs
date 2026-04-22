use mimalloc::MiMalloc;
use std::sync::Arc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod adapter;
mod application;
mod cli;
mod core;
mod infrastructure;

use adapter::handler::McpHandler;
use application::image::ImageGenerationService;
use cli::Cli;
use core::config::{AppConfig, TransportMode};
// mod adapter, core, infrastructure handled below

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> rust_mcp_sdk::error::SdkResult<()> {
    let cli = Cli::parse_config();
    let config = AppConfig::from_cli(cli.clone());

    match config.transport_mode {
        TransportMode::Stdio => run_stdio_server(config).await,
        TransportMode::Http => run_http_server(config, cli.port).await,
    }
}

async fn run_stdio_server(config: AppConfig) -> rust_mcp_sdk::error::SdkResult<()> {
    use rust_mcp_sdk::{
        McpServer, StdioTransport, TransportOptions,
        mcp_server::{
            McpServerOptions, ServerRuntime, ToMcpServerHandler, server_runtime::create_server,
        },
        schema::*,
    };

    let handler = create_handler(config.clone());

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "image-mcp".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("Image Generation MCP Server".into()),
            description: Some("MCP server for image generation via OpenAI-compatible APIs".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: Some(format!(
            "Flavor: {:?}. Generate images via OpenAI-compatible providers.",
            config.flavor
        )),
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let server: Arc<ServerRuntime> = create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });

    server.start().await
}

async fn run_http_server(config: AppConfig, port: u16) -> rust_mcp_sdk::error::SdkResult<()> {
    use rust_mcp_sdk::{
        event_store::InMemoryEventStore,
        mcp_server::{HyperServerOptions, ToMcpServerHandler, hyper_server},
        schema::*,
    };

    let handler = create_handler(config.clone());

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "image-mcp".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("Image Generation MCP Server".into()),
            description: Some("MCP server for image generation via OpenAI-compatible APIs".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: Some(format!(
            "Flavor: {:?}. Generate images via OpenAI-compatible providers.",
            config.flavor
        )),
        meta: None,
    };

    let server = hyper_server::create_server(
        server_details,
        handler.to_mcp_server_handler(),
        HyperServerOptions {
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
    let image_client = infrastructure::factory::ProviderFactory::create_client(config.clone());
    let session = core::session::SessionStore::new();
    let image_service = Arc::new(ImageGenerationService::new(
        image_client,
        config.clone(),
        session,
    ));

    McpHandler::new(image_service, config.image_model, config.flavor)
}
