mod value;

use std::{io, sync::Arc};

use indexmap::IndexMap;
pub use rmcp::handler::server::tool::Parameters;
use rmcp::{
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool,
    transport::{SseServer, sse_server::SseServerConfig},
};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;
use tokio_util::sync::CancellationToken;

use crate::mcp::value::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParam {
    pub query: String,
    pub params: Vec<Value>,
}

#[derive(Clone)]
pub struct PostgresMcpServer {
    client: Arc<tokio_postgres::Client>,
}

#[tool(tool_box)]
impl PostgresMcpServer {
    fn new(client: Arc<tokio_postgres::Client>) -> Self {
        Self { client }
    }

    #[tool(description = "Execute a query")]
    async fn query(
        &self,
        Parameters(params): Parameters<QueryParam>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let rows = match self
            .client
            .query(
                &params.query,
                params
                    .params
                    .iter()
                    .map(|p| p as &(dyn ToSql + Sync))
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .await
        {
            Ok(result) => result,
            Err(e) => return Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        };
        let rows = rows
            .into_iter()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| {
                        let name = column.name();
                        let value: Value = row.get(name);
                        (name.to_owned(), value)
                    })
                    .collect::<IndexMap<_, _>>()
            })
            .collect::<Vec<_>>();
        let Ok(serialized) = Content::json(&rows) else {
            return Err(rmcp::Error::internal_error("failed to serialize rows".to_string(), None));
        };
        Ok(CallToolResult::success(vec![serialized]))
    }
}

#[tool(tool_box)]
impl rmcp::ServerHandler for PostgresMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A PostgreSQL database".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn start(
    addr: &str,
    client: Arc<tokio_postgres::Client>,
) -> io::Result<CancellationToken> {
    let ctoken = CancellationToken::new();
    let config = SseServerConfig {
        bind: addr.parse().map_err(io::Error::other)?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: ctoken.clone(),
    };

    let sse_server = SseServer::serve_with_config(config).await?;
    sse_server.with_service(move || PostgresMcpServer::new(Arc::clone(&client)));
    Ok(ctoken)
}
