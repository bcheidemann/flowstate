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
    fn wrap_workflow<'workflow, Result>(
        &self,
        _metadata: &'workflow WorkflowMetadata<'workflow>,
        next: impl FnOnce() -> Result,
    ) -> impl FnOnce() -> Result {
        next
    }

    fn wrap_state<'state, Transition>(
        &self,
        _metadata: &'state WorkflowStateMetadata<'state>,
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
        fut: impl Future<Output = Result> + Send + 'workflow,
    ) -> impl Future<Output = Result> + Send + 'workflow {
        fut
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        _metadata: &'state WorkflowStateMetadata<'state>,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        fut
    }
}
