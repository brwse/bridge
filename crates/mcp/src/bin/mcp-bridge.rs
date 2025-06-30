use brwse_bridge_cli::BridgeArgs;
use clap::Parser;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about = "MCP Bridge")]
struct Args {
    /// URL of the MCP server
    #[arg(long, env = "BRWSE_MCP_URL", default_value = "http://localhost:9000")]
    mcp_url: String,

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

    let mcp_bridge = brwse_bridge_mcp::bridge::McpBridge::new(args.mcp_url);
    let mcp_ct = brwse_bridge_mcp::bridge::start(&args.bridge.listen, mcp_bridge)
        .await
        .expect("failed to start MCP server");

    let _result = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal, stopping bridge...");

    mcp_ct.cancel();
}
