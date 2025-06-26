mod value;

use std::{io, sync::Arc};

use assert2::let_assert;
use indexmap::IndexMap;
pub use rmcp::handler::server::tool::Parameters;
use rmcp::{
    RoleServer,
    model::{
        CallToolRequestParam, CallToolResult, Content, ListToolsResult, PaginatedRequestParam,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    transport::{SseServer, sse_server::SseServerConfig},
};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio_postgres::types::ToSql;
use tokio_util::sync::CancellationToken;

use crate::{mcp::value::Value, schema::remove_excess};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(transform = remove_excess)]
pub struct QueryParam {
    /// The SQL query to execute.
    pub query: String,
    /// The parameters to pass to the query.
    pub params: Vec<Value>,
}

#[derive(Clone)]
pub struct PostgresMcpServer {
    client: Arc<tokio_postgres::Client>,
}

impl PostgresMcpServer {
    fn new(client: Arc<tokio_postgres::Client>) -> Self {
        Self { client }
    }

    async fn query(&self, params: QueryParam) -> Result<CallToolResult, rmcp::Error> {
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

impl rmcp::ServerHandler for PostgresMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A PostgreSQL database".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let schema = schema_for!(QueryParam);
        let_assert!(JsonValue::Object(schema) = schema.to_value());
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![Tool::new("query", "Query the database", Arc::new(schema))],
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let arguments = request.arguments.map(JsonValue::Object).unwrap_or_default();
        let params = serde_json::from_value::<QueryParam>(arguments).map_err(|e| {
            rmcp::Error::invalid_params(format!("failed to parse arguments: {e}"), None)
        })?;

        // Execute tool directly from spec
        self.query(params).await
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
