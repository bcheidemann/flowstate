use std::ops::ControlFlow;

/// A workflow that progresses through a series of [`WorkflowState`]s and
/// produces a [`Workflow::Result`].
///
/// Implement this trait on the workflow's container type. Call [`Workflow::run`]
/// to drive the workflow to completion and obtain the final result.
pub trait Workflow {
    type Result;

    fn run(self) -> Self::Result
    where
        Self: WorkflowState<Self::Result> + Sized,
    {
        let mut workflow: Box<dyn WorkflowState<Self::Result>> = Box::new(self);

        loop {
            match workflow.next() {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    fn result(
        self,
        result: Self::Result,
    ) -> ControlFlow<Self::Result, Box<dyn WorkflowState<Self::Result>>>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }
}

/// A single state in a [`Workflow`].
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
pub trait WorkflowState<Result> {
    fn next(self: Box<Self>) -> ControlFlow<Result, Box<dyn WorkflowState<Result>>>;
}
