use core::time::Duration;
use std::{io, sync::Arc};

use rmcp::{
    RoleClient, RoleServer, ServerHandler, ServiceExt,
    model::{ClientInfo, InitializeRequestParam},
    service::RunningService,
    transport::{SseClientTransport, SseServer, sse_server::SseServerConfig},
};
use tokio::sync::{Mutex, OwnedMappedMutexGuard, OwnedMutexGuard};
use tokio_util::sync::CancellationToken;

pub struct McpBridge {
    url: String,
    client: Arc<Mutex<Option<RunningService<RoleClient, InitializeRequestParam>>>>,
}

impl McpBridge {
    pub fn new(url: String) -> Self {
        Self { url, client: Arc::new(Mutex::new(None)) }
    }

    async fn client(
        &self,
    ) -> Result<
        OwnedMappedMutexGuard<
            Option<RunningService<RoleClient, InitializeRequestParam>>,
            RunningService<RoleClient, InitializeRequestParam>,
        >,
        rmcp::Error,
    > {
        let Ok(client) =
            OwnedMutexGuard::try_map(Arc::clone(&self.client).lock_owned().await, |client| {
                client.as_mut()
            })
        else {
            return Err(rmcp::Error::invalid_request("Client not initialized", None));
        };
        Ok(client)
    }
}

impl Clone for McpBridge {
    fn clone(&self) -> Self {
        // We don't clone the client because a new clone means a new proxy to the
        // MCP server.
        Self { url: self.url.clone(), client: Arc::clone(&self.client) }
    }
}

impl rmcp::ServerHandler for McpBridge {
    async fn ping(
        &self,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let client = self.client().await?;
        client
            .send_request(rmcp::model::ClientRequest::PingRequest(
                rmcp::model::PingRequest::default(),
            ))
            .await
            .map_err(service_error_to_mcp_error)?;
        Ok(())
    }

    async fn initialize(
        &self,
        request: rmcp::model::InitializeRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::InitializeResult, rmcp::Error> {
        let transport = SseClientTransport::start(self.url.clone())
            .await
            .expect("failed to connect to MCP server");
        let client_info = ClientInfo {
            protocol_version: request.protocol_version,
            capabilities: request.capabilities,
            client_info: request.client_info,
        };
        let service = client_info.serve(transport).await.expect("failed to connect to MCP server");
        let peer_info = service.peer_info().expect("peer info not found").clone();

        self.client.lock().await.replace(service);
        Ok(peer_info)
    }

    async fn complete(
        &self,
        request: rmcp::model::CompleteRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CompleteResult, rmcp::Error> {
        let client = self.client().await?;
        client.complete(request).await.map_err(service_error_to_mcp_error)
    }

    async fn set_level(
        &self,
        request: rmcp::model::SetLevelRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let client = self.client().await?;
        client.set_level(request).await.map_err(service_error_to_mcp_error)
    }

    async fn get_prompt(
        &self,
        request: rmcp::model::GetPromptRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::GetPromptResult, rmcp::Error> {
        let client = self.client().await?;
        client.get_prompt(request).await.map_err(service_error_to_mcp_error)
    }

    async fn list_prompts(
        &self,
        request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListPromptsResult, rmcp::Error> {
        let client = self.client().await?;
        client.list_prompts(request).await.map_err(service_error_to_mcp_error)
    }

    async fn list_resources(
        &self,
        request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::Error> {
        let client = self.client().await?;
        client.list_resources(request).await.map_err(service_error_to_mcp_error)
    }

    async fn list_resource_templates(
        &self,
        request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourceTemplatesResult, rmcp::Error> {
        let client = self.client().await?;
        client.list_resource_templates(request).await.map_err(service_error_to_mcp_error)
    }

    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::Error> {
        let client = self.client().await?;
        client.read_resource(request).await.map_err(service_error_to_mcp_error)
    }

    async fn subscribe(
        &self,
        request: rmcp::model::SubscribeRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let client = self.client().await?;
        client.subscribe(request).await.map_err(service_error_to_mcp_error)
    }

    async fn unsubscribe(
        &self,
        request: rmcp::model::UnsubscribeRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let client = self.client().await?;
        client.unsubscribe(request).await.map_err(service_error_to_mcp_error)
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::Error> {
        let client = self.client().await?;
        client.call_tool(request).await.map_err(service_error_to_mcp_error)
    }

    async fn list_tools(
        &self,
        request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::Error> {
        let client = self.client().await?;
        client.list_tools(request).await.map_err(service_error_to_mcp_error)
    }

    async fn on_cancelled(
        &self,
        notification: rmcp::model::CancelledNotificationParam,
        _context: rmcp::service::NotificationContext<RoleServer>,
    ) {
        if let Ok(client) = self.client().await {
            let _ignore = client.notify_cancelled(notification).await;
        }
    }

    async fn on_progress(
        &self,
        notification: rmcp::model::ProgressNotificationParam,
        _context: rmcp::service::NotificationContext<RoleServer>,
    ) {
        if let Ok(client) = self.client().await {
            let _ignore = client.notify_progress(notification).await;
        }
    }

    async fn on_initialized(&self, _context: rmcp::service::NotificationContext<RoleServer>) {
        if let Ok(client) = self.client().await {
            let _ignore = client.notify_initialized().await;
        }
    }

    async fn on_roots_list_changed(
        &self,
        _context: rmcp::service::NotificationContext<RoleServer>,
    ) {
        if let Ok(client) = self.client().await {
            let _ignore = client.notify_roots_list_changed().await;
        }
    }
}

fn service_error_to_mcp_error(e: rmcp::ServiceError) -> rmcp::Error {
    match e {
        rmcp::ServiceError::McpError(error_data) => error_data,
        rmcp::ServiceError::TransportSend(error) => {
            rmcp::Error::internal_error(error.to_string(), None)
        }
        rmcp::ServiceError::TransportClosed => {
            rmcp::Error::internal_error("Transport closed", None)
        }
        rmcp::ServiceError::UnexpectedResponse => {
            rmcp::Error::internal_error("Unexpected response", None)
        }
        rmcp::ServiceError::Cancelled { reason } => {
            rmcp::Error::internal_error(reason.unwrap_or("Cancelled".to_string()), None)
        }
        rmcp::ServiceError::Timeout { timeout } => {
            rmcp::Error::internal_error(format!("Timeout after {timeout:?}"), None)
        }
        _ => rmcp::Error::internal_error("Unexpected error", None),
    }
}

pub async fn start<T>(addr: &str, service: T) -> io::Result<CancellationToken>
where
    T: ServerHandler + Clone,
{
    let ctoken = CancellationToken::new();
    let config = SseServerConfig {
        bind: addr.parse().map_err(io::Error::other)?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: ctoken.clone(),
        sse_keep_alive: Some(Duration::from_secs(30)),
    };

    let sse_server = SseServer::serve_with_config(config).await?;
    sse_server.with_service(move || service.clone());
    Ok(ctoken)
}
