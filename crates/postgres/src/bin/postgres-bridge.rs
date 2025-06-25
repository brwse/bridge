use std::{process, sync::Arc};

use brwse_bridge_cli::{BridgeArgs, setup_registry};
use clap::Parser;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about = "Postgres Bridge - PostgreSQL protocol bridge")]
struct Args {
    /// PostgreSQL server address
    #[arg(long, env = "BRWSE_DATABASE_URL")]
    database_url: String,

    #[command(flatten)]
    bridge: BridgeArgs,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Setup registry
    if let Some(registry) = args.bridge.registry {
        setup_registry(&registry).await;
    }

    // Build the PostgreSQL bridge
    info!("Starting PostgreSQL bridge on {} -> {:?}", args.bridge.listen, args.database_url);

    let (client, connection) =
        match tokio_postgres::connect(&args.database_url, tokio_postgres::NoTls).await {
            Ok((client, connection)) => (client, connection),
            Err(e) => {
                error!("Failed to connect to PostgreSQL: {}", e);
                process::exit(1);
            }
        };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("PostgreSQL connection error: {}", e);
        }
    });

    let mcp_ct = brwse_bridge_postgres::mcp::start(&args.bridge.listen, Arc::new(client))
        .await
        .expect("failed to start MCP server");

    let _result = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal, stopping bridge...");

    mcp_ct.cancel();
}
