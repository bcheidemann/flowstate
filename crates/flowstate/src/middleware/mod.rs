#[cfg(feature = "async")]
use crate::AsyncContext;
use crate::Context;

pub mod identity;
pub mod stack;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WorkflowMetadata<'a> {
    pub name: &'a str,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WorkflowStateMetadata<'a> {
    pub name: &'a str,
}

pub trait WorkflowMiddleware {
    #[inline(always)]
    fn wrap_workflow<'workflow, Result>(
        &self,
        _metadata: &'workflow WorkflowMetadata<'workflow>,
        _ctx: &Context,
        next: impl FnOnce() -> Result,
    ) -> impl FnOnce() -> Result {
        next
    }

    #[inline(always)]
    fn wrap_state<'state, Transition>(
        &self,
        _metadata: &'state WorkflowStateMetadata<'state>,
        _ctx: &Context,
        next: impl FnOnce() -> Transition,
    ) -> impl FnOnce() -> Transition {
        next
    }
}

#[cfg(feature = "async")]
pub trait AsyncWorkflowMiddleware {
    fn wrap_workflow<'workflow, Result: Send + 'workflow>(
        &self,
        _metadata: &'workflow WorkflowMetadata<'workflow>,
        _ctx: &AsyncContext,
        fut: impl Future<Output = Result> + Send + 'workflow,
    ) -> impl Future<Output = Result> + Send + 'workflow {
        fut
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        _metadata: &'state WorkflowStateMetadata<'state>,
        _ctx: &AsyncContext,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        fut
    }
}
