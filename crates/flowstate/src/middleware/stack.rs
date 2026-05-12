use crate::middleware::{AsyncWorkflowMiddleware, WorkflowMetadata, identity::IdentityMiddleware};

pub struct MiddlewareStack<Inner, Outer> {
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> AsyncWorkflowMiddleware for MiddlewareStack<Inner, Outer>
where
    Inner: AsyncWorkflowMiddleware,
    Outer: AsyncWorkflowMiddleware,
{
    fn wrap_workflow<'workflow, T: Send + 'workflow>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        fut: impl Future<Output = T> + Send + 'workflow,
    ) -> impl Future<Output = T> + Send + 'workflow {
        self.outer
            .wrap_workflow(metadata, self.inner.wrap_workflow(metadata, fut))
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
    pub fn layer<Middleware: AsyncWorkflowMiddleware>(
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
