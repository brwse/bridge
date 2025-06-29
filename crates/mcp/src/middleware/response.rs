#![expect(unused_variables, reason = "library code")]

use std::future::Future;

use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, CancelledNotificationParam, CompleteRequestParam,
        CompleteResult, GetPromptRequestParam, GetPromptResult, InitializeRequestParam,
        InitializeResult, ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
        ListToolsResult, PaginatedRequestParam, ProgressNotificationParam,
        ReadResourceRequestParam, ReadResourceResult, ServerInfo, SetLevelRequestParam,
        SubscribeRequestParam, UnsubscribeRequestParam,
    },
    service::{NotificationContext, RequestContext},
};

pub trait Middleware: 'static + Send + Sync {
    fn ping(
        &self,
        response: Result<(), rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::Error>> + Send {
        async { response }
    }

    fn initialize(
        &self,
        response: Result<InitializeResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, rmcp::Error>> + Send {
        async { response }
    }

    fn complete(
        &self,
        response: Result<CompleteResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CompleteResult, rmcp::Error>> + Send {
        async { response }
    }

    fn set_level(
        &self,
        response: Result<(), rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::Error>> + Send {
        async { response }
    }

    fn get_prompt(
        &self,
        response: Result<GetPromptResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, rmcp::Error>> + Send {
        async { response }
    }

    fn list_prompts(
        &self,
        response: Result<ListPromptsResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, rmcp::Error>> + Send {
        async { response }
    }

    fn list_resources(
        &self,
        response: Result<ListResourcesResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, rmcp::Error>> + Send {
        async { response }
    }

    fn list_resource_templates(
        &self,
        response: Result<ListResourceTemplatesResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, rmcp::Error>> + Send {
        async { response }
    }

    fn read_resource(
        &self,
        response: Result<ReadResourceResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, rmcp::Error>> + Send {
        async { response }
    }

    fn subscribe(
        &self,
        response: Result<(), rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::Error>> + Send {
        async { response }
    }

    fn unsubscribe(
        &self,
        response: Result<(), rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::Error>> + Send {
        async { response }
    }

    fn call_tool(
        &self,
        response: Result<CallToolResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::Error>> + Send {
        async { response }
    }

    fn list_tools(
        &self,
        response: Result<ListToolsResult, rmcp::Error>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::Error>> + Send {
        async { response }
    }
}

pub trait ServerHandlerExt: ServerHandler {
    fn with_response_middleware<M: Middleware>(self, middleware: M) -> WithMiddleware<Self, M> {
        WithMiddleware { inner: self, middleware }
    }
}

impl<T: ServerHandler> ServerHandlerExt for T {}

pub struct WithMiddleware<T, M> {
    inner: T,
    middleware: M,
}

impl<T: Clone, M: Clone> Clone for WithMiddleware<T, M> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), middleware: self.middleware.clone() }
    }
}

impl<T: ServerHandler, M: Middleware> ServerHandler for WithMiddleware<T, M> {
    async fn ping(&self, context: RequestContext<RoleServer>) -> Result<(), rmcp::Error> {
        let response = self.inner.ping(context.clone()).await;
        self.middleware.ping(response, context).await
    }

    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, rmcp::Error> {
        let response = self.inner.initialize(request, context.clone()).await;
        self.middleware.initialize(response, context).await
    }

    async fn complete(
        &self,
        request: CompleteRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, rmcp::Error> {
        let response = self.inner.complete(request, context.clone()).await;
        self.middleware.complete(response, context).await
    }

    async fn set_level(
        &self,
        request: SetLevelRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let response = self.inner.set_level(request, context.clone()).await;
        self.middleware.set_level(response, context).await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, rmcp::Error> {
        let response = self.inner.get_prompt(request, context.clone()).await;
        self.middleware.get_prompt(response, context).await
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::Error> {
        let response = self.inner.list_prompts(request, context.clone()).await;
        self.middleware.list_prompts(response, context).await
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::Error> {
        let response = self.inner.list_resources(request, context.clone()).await;
        self.middleware.list_resources(response, context).await
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, rmcp::Error> {
        let response = self.inner.list_resource_templates(request, context.clone()).await;
        self.middleware.list_resource_templates(response, context).await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::Error> {
        let response = self.inner.read_resource(request, context.clone()).await;
        self.middleware.read_resource(response, context).await
    }

    async fn subscribe(
        &self,
        request: SubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let response = self.inner.subscribe(request, context.clone()).await;
        self.middleware.subscribe(response, context).await
    }

    async fn unsubscribe(
        &self,
        request: UnsubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        let response = self.inner.unsubscribe(request, context.clone()).await;
        self.middleware.unsubscribe(response, context).await
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let response = self.inner.call_tool(request, context.clone()).await;
        self.middleware.call_tool(response, context).await
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let response = self.inner.list_tools(request, context.clone()).await;
        self.middleware.list_tools(response, context).await
    }

    async fn on_cancelled(
        &self,
        notification: CancelledNotificationParam,
        context: NotificationContext<RoleServer>,
    ) {
        self.inner.on_cancelled(notification, context).await
    }

    async fn on_progress(
        &self,
        notification: ProgressNotificationParam,
        context: NotificationContext<RoleServer>,
    ) {
        self.inner.on_progress(notification, context).await
    }

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        self.inner.on_initialized(context).await
    }

    async fn on_roots_list_changed(&self, context: NotificationContext<RoleServer>) {
        self.inner.on_roots_list_changed(context).await
    }

    fn get_info(&self) -> ServerInfo {
        self.inner.get_info()
    }
}
