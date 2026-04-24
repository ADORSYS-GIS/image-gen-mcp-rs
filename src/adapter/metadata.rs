use rust_mcp_sdk::schema::*;
use crate::cli::Flavor;

pub fn get_server_details(flavor: Flavor) -> InitializeResult {
    InitializeResult {
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
            flavor
        )),
        meta: None,
    }
}
