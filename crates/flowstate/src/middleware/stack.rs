#[cfg(feature = "async")]
use crate::{AsyncContext, middleware::AsyncWorkflowMiddleware};
use crate::{
    Context,
    middleware::{WorkflowMetadata, WorkflowMiddleware, identity::IdentityMiddleware},
};

pub struct MiddlewareStack<Inner, Outer> {
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> WorkflowMiddleware for MiddlewareStack<Inner, Outer>
where
    Inner: WorkflowMiddleware,
    Outer: WorkflowMiddleware,
{
    fn wrap_workflow<'workflow, Result>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        ctx: &Context,
        next: impl FnOnce() -> Result,
    ) -> impl FnOnce() -> Result {
        self.outer
            .wrap_workflow(metadata, ctx, self.inner.wrap_workflow(metadata, ctx, next))
    }

    fn wrap_state<'state, Transition>(
        &self,
        metadata: &'state super::WorkflowStateMetadata<'state>,
        ctx: &Context,
        next: impl FnOnce() -> Transition,
    ) -> impl FnOnce() -> Transition {
        self.outer
            .wrap_state(metadata, ctx, self.inner.wrap_state(metadata, ctx, next))
    }
}

#[cfg(feature = "async")]
impl<Inner, Outer> AsyncWorkflowMiddleware for MiddlewareStack<Inner, Outer>
where
    Inner: AsyncWorkflowMiddleware,
    Outer: AsyncWorkflowMiddleware,
{
    fn wrap_workflow<'workflow, Result: Send + 'workflow>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        ctx: &AsyncContext,
        fut: impl Future<Output = Result> + Send + 'workflow,
    ) -> impl Future<Output = Result> + Send + 'workflow {
        self.outer
            .wrap_workflow(metadata, ctx, self.inner.wrap_workflow(metadata, ctx, fut))
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        metadata: &'state super::WorkflowStateMetadata<'state>,
        ctx: &AsyncContext,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        self.outer
            .wrap_state(metadata, ctx, self.inner.wrap_state(metadata, ctx, fut))
    }
}

pub struct MiddlewareStackBuilder<Stack>(Stack);

impl MiddlewareStackBuilder<IdentityMiddleware> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MiddlewareStackBuilder<IdentityMiddleware> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Stack> MiddlewareStackBuilder<Stack> {
    /// Adds a new layer to the middleware stack.
    ///
    /// Layers are executed in the order they are added.
    pub fn layer<Middleware>(
        self,
        middleware: Middleware,
    ) -> MiddlewareStackBuilder<MiddlewareStack<Middleware, Stack>> {
        MiddlewareStackBuilder(MiddlewareStack {
            inner: middleware,
            outer: self.0,
        })
    }

    pub fn build(self) -> Stack {
        self.0
    }
}
