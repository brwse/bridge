use std::{process, sync::Arc};

use brwse_bridge_cli::{BridgeArgs, setup_registry};
use clap::Parser;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about = "HTTP Bridge - HTTP API protocol bridge for OpenAPI specs")]
struct Args {
    /// Path to OpenAPI specification file (JSON or YAML)
    #[arg(long, env = "BRWSE_OPENAPI_SPEC_PATH")]
    openapi_spec: String,

    /// Base URL for the API (overrides spec's servers)
    #[arg(long, env = "BRWSE_API_BASE_URL")]
    base_url: Option<String>,

    /// Default timeout for HTTP requests in seconds
    #[arg(long, default_value = "30", env = "BRWSE_HTTP_TIMEOUT")]
    timeout: u64,

    #[command(flatten)]
    bridge: BridgeArgs,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Setup registry
    if args.bridge.registry.br_token.is_some() {
        setup_registry(&args.bridge.registry).await;
    }

    // Load and parse OpenAPI spec
    info!("Loading OpenAPI spec from: {}", args.openapi_spec);

    let spec = match brwse_bridge_http::openapi::load_spec(&args.openapi_spec).await {
        Ok(spec) => Arc::new(spec),
        Err(e) => {
            error!("Failed to load OpenAPI spec: {}", e);
            process::exit(1);
        }
    };

    info!("OpenAPI spec loaded: {} (v{})", spec.info.title, spec.info.version);

    // Determine base URL
    let base_url = args
        .base_url
        .or_else(|| spec.servers.first().map(|s| s.url.clone()))
        .unwrap_or_else(|| {
            error!("No base URL provided and no servers found in OpenAPI spec");
            process::exit(1);
        });

    info!("Using base URL: {}", base_url);

    // Build the HTTP bridge
    info!("Starting HTTP bridge on {} -> {}", args.bridge.listen, base_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(args.timeout))
        .build()
        .expect("Failed to build HTTP client");

    let mcp_ct =
        brwse_bridge_http::bridge::start(&args.bridge.listen, spec, base_url, Arc::new(client))
            .await
            .expect("failed to start MCP server");

    let _result = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal, stopping bridge...");

    mcp_ct.cancel();
}
