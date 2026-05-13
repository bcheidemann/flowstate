use std::ops::ControlFlow;

#[cfg(all(feature = "async", feature = "unstable_middleware"))]
use crate::middleware::AsyncWorkflowMiddleware;
#[cfg(feature = "unstable_middleware")]
use crate::middleware::{WorkflowMetadata, WorkflowMiddleware, WorkflowStateMetadata};
#[cfg(feature = "async")]
use crate::{AsyncContext, AsyncState, AsyncTransition};
use crate::{Context, State, Transition};

pub trait Workflow {
    fn workflow_name(&self) -> String;

    fn record_workflow_context(&self, _ctx: &mut Context) {}

    fn state(&self) -> &dyn State;
}

/// A [`Workflow`] in a specific state.
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
pub trait WorkflowState<'workflow, Result>: Workflow {
    fn name(&self) -> String {
        self.state().name()
    }

    fn record_context(&self, ctx: &mut Context) {
        self.state().record_context(ctx)
    }

    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// Return [`self.finish(result)`](Workflow::finish) or
    /// [`self.finish_with(|workflow| result)`](Workflow::finish_with) to
    /// terminate the workflow with a result.
    fn next(self: Box<Self>) -> Transition<'workflow, Result>;

    fn run(self) -> Result
    where
        Self: WorkflowState<'workflow, Result> + Sized,
    {
        let mut workflow: Box<dyn WorkflowState<Result>> = Box::new(self);

        loop {
            match workflow.next() {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    #[cfg(feature = "unstable_middleware")]
    fn run_with_middleware<Middleware>(self, middleware: Middleware) -> Result
    where
        Self: WorkflowState<'workflow, Result> + Sized,
        Middleware: WorkflowMiddleware,
    {
        run_with_middleware(self, middleware)
    }

    fn finish(self, result: Result) -> Transition<'workflow, Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }

    fn finish_with<Fn>(self, map_fn: Fn) -> Transition<'workflow, Result>
    where
        Self: Sized,
        Fn: FnOnce(Self) -> Result,
    {
        ControlFlow::Break(map_fn(self))
    }
}

#[cfg(feature = "unstable_middleware")]
fn run_with_middleware<'workflow, W, M, R>(initial_state: W, middleware: M) -> R
where
    W: WorkflowState<'workflow, R>,
    M: WorkflowMiddleware,
{
    let workflow_name = initial_state.workflow_name();
    let metadata = WorkflowMetadata {
        name: &workflow_name,
    };
    let mut ctx = Context::default();
    initial_state.record_workflow_context(&mut ctx);
    let run_workflow_fn = || run_with_middleware_impl(initial_state, &middleware);

    middleware.wrap_workflow(&metadata, &ctx, run_workflow_fn)()
}

#[cfg(feature = "unstable_middleware")]
fn run_with_middleware_impl<'workflow, W, M, R>(initial_workflow_state: W, middleware: &M) -> R
where
    W: WorkflowState<'workflow, R>,
    M: WorkflowMiddleware,
{
    let mut workflow_state: Box<dyn WorkflowState<R>> = Box::new(initial_workflow_state);

    loop {
        let workflow_state_name = workflow_state.name();
        let metadata = WorkflowStateMetadata {
            name: &workflow_state_name,
        };
        let mut ctx = Context::default();
        workflow_state.record_context(&mut ctx);

        match middleware.wrap_state(&metadata, &ctx, || workflow_state.next())() {
            ControlFlow::Continue(next) => workflow_state = next,
            ControlFlow::Break(result) => return result,
        }
    }
}

#[cfg(feature = "async")]
pub trait AsyncWorkflow {
    fn workflow_name(&self) -> String;

    fn record_workflow_context(&self, _ctx: &mut AsyncContext) {}

    fn state(&self) -> &dyn AsyncState;
}

/// An async [`Workflow`] in a specific state.
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncWorkflowState<'workflow, Result>: AsyncWorkflow + Send {
    fn name(&self) -> String {
        self.state().name()
    }

    fn record_context(&self, ctx: &mut AsyncContext) {
        self.state().record_context(ctx)
    }

    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// Return [`self.finish(result)`](Workflow::finish) or
    /// [`self.finish_with(|workflow| result)`](Workflow::finish_with) to
    /// terminate the workflow with a result.
    async fn next(self: Box<Self>) -> AsyncTransition<'workflow, Result>;

    async fn run(self) -> Result
    where
        Self: AsyncWorkflowState<'workflow, Result> + Sized,
    {
        let mut workflow: Box<dyn AsyncWorkflowState<Result>> = Box::new(self);

        loop {
            match workflow.next().await {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    #[cfg(feature = "unstable_middleware")]
    async fn run_with_middleware<Middleware>(self, middleware: Middleware) -> Result
    where
        Self: AsyncWorkflowState<'workflow, Result> + Sized,
        Middleware: AsyncWorkflowMiddleware + Send + Sync,
        Result: Send,
    {
        run_with_middleware_async(self, middleware).await
    }

    fn finish(self, result: Result) -> AsyncTransition<'workflow, Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }

    fn finish_with<Fn>(self, map_fn: Fn) -> AsyncTransition<'workflow, Result>
    where
        Self: Sized,
        Fn: FnOnce(Self) -> Result,
    {
        ControlFlow::Break(map_fn(self))
    }
}

#[cfg(all(feature = "async", feature = "unstable_middleware"))]
async fn run_with_middleware_async<'workflow, W, M, R>(initial_state: W, middleware: M) -> R
where
    W: AsyncWorkflowState<'workflow, R> + Sized,
    M: AsyncWorkflowMiddleware + Send + Sync,
    R: Send,
{
    let workflow_name = initial_state.workflow_name();
    let metadata = WorkflowMetadata {
        name: &workflow_name,
    };
    let mut ctx = AsyncContext::default();
    initial_state.record_workflow_context(&mut ctx);
    let run_workflow_fut = run_with_middleware_async_impl(initial_state, &middleware);

    middleware
        .wrap_workflow(&metadata, &ctx, run_workflow_fut)
        .await
}

#[cfg(all(feature = "async", feature = "unstable_middleware"))]
async fn run_with_middleware_async_impl<'workflow, W, M, R>(
    initial_workflow_state: W,
    middleware: &M,
) -> R
where
    W: AsyncWorkflowState<'workflow, R> + Sized,
    M: AsyncWorkflowMiddleware,
    R: Send,
{
    let mut workflow_state: Box<dyn AsyncWorkflowState<R>> = Box::new(initial_workflow_state);

    loop {
        let workflow_state_name = workflow_state.name();
        let metadata = WorkflowStateMetadata {
            name: &workflow_state_name,
        };
        let mut ctx = AsyncContext::default();
        workflow_state.record_context(&mut ctx);

        match middleware
            .wrap_state(&metadata, &ctx, workflow_state.next())
            .await
        {
            ControlFlow::Continue(next) => workflow_state = next,
            ControlFlow::Break(result) => return result,
        }
    }
}
