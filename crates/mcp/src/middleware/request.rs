#![expect(unused_variables, reason = "library code")]

use rmcp::{
    RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CancelledNotificationParam, CompleteRequestParam, CompleteResult,
        GetPromptRequestParam, InitializeRequestParam, InitializeResult, PaginatedRequestParam,
        ProgressNotificationParam, ReadResourceRequestParam, ServerInfo, SetLevelRequestParam,
        SubscribeRequestParam, UnsubscribeRequestParam,
    },
    service::{NotificationContext, RequestContext},
};

pub trait Middleware: 'static + Send + Sync {
    fn ping(
        &self,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::Error>> + Send {
        async { Ok(()) }
    }

    fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }
    fn complete(
        &self,
        request: CompleteRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CompleteRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn set_level(
        &self,
        request: SetLevelRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<SetLevelRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }
    fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn list_prompts(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<Option<PaginatedRequestParam>, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn list_resources(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<Option<PaginatedRequestParam>, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<Option<PaginatedRequestParam>, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn subscribe(
        &self,
        request: SubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<SubscribeRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn unsubscribe(
        &self,
        request: UnsubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<UnsubscribeRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolRequestParam, rmcp::Error>> + Send {
        async { Ok(request) }
    }

    fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<Option<PaginatedRequestParam>, rmcp::Error>> + Send {
        async { Ok(request) }
    }
}

pub trait ServerHandlerExt: ServerHandler {
    fn with_request_middleware<M: Middleware>(self, middleware: M) -> WithMiddleware<Self, M> {
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
        self.inner.ping(context).await
    }

    async fn initialize(
        &self,
        mut request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, rmcp::Error> {
        request = self.middleware.initialize(request, context.clone()).await?;
        self.inner.initialize(request, context).await
    }

    async fn complete(
        &self,
        mut request: CompleteRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, rmcp::Error> {
        request = self.middleware.complete(request, context.clone()).await?;
        self.inner.complete(request, context).await
    }

    async fn set_level(
        &self,
        mut request: SetLevelRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        request = self.middleware.set_level(request, context.clone()).await?;
        self.inner.set_level(request, context).await
    }

    async fn get_prompt(
        &self,
        mut request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::GetPromptResult, rmcp::Error> {
        request = self.middleware.get_prompt(request, context.clone()).await?;
        self.inner.get_prompt(request, context).await
    }

    async fn list_prompts(
        &self,
        mut request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListPromptsResult, rmcp::Error> {
        request = self.middleware.list_prompts(request, context.clone()).await?;
        self.inner.list_prompts(request, context).await
    }

    async fn list_resources(
        &self,
        mut request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::Error> {
        request = self.middleware.list_resources(request, context.clone()).await?;
        self.inner.list_resources(request, context).await
    }

    async fn list_resource_templates(
        &self,
        mut request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListResourceTemplatesResult, rmcp::Error> {
        request = self.middleware.list_resource_templates(request, context.clone()).await?;
        self.inner.list_resource_templates(request, context).await
    }

    async fn read_resource(
        &self,
        mut request: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::Error> {
        request = self.middleware.read_resource(request, context.clone()).await?;
        self.inner.read_resource(request, context).await
    }

    async fn subscribe(
        &self,
        mut request: SubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        request = self.middleware.subscribe(request, context.clone()).await?;
        self.inner.subscribe(request, context).await
    }

    async fn unsubscribe(
        &self,
        mut request: UnsubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<(), rmcp::Error> {
        request = self.middleware.unsubscribe(request, context.clone()).await?;
        self.inner.unsubscribe(request, context).await
    }

    async fn call_tool(
        &self,
        mut request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::Error> {
        request = self.middleware.call_tool(request, context.clone()).await?;
        self.inner.call_tool(request, context).await
    }

    async fn list_tools(
        &self,
        mut request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::Error> {
        request = self.middleware.list_tools(request, context.clone()).await?;
        self.inner.list_tools(request, context).await
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
