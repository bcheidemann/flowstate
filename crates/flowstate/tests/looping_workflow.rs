use flowstate::{Transition, Workflow, WorkflowState};

#[derive(Workflow)]
#[flowstate(result = usize)]
struct CountLengthOfVecWorkflow<State> {
    #[state]
    state: State,
    vec: Vec<u8>,
}

struct Count(usize);

impl WorkflowState<usize> for CountLengthOfVecWorkflow<Count> {
    fn next(mut self: Box<Self>) -> Transition<usize> {
        let current_count = self.state.0;

        if self.vec.pop().is_some() {
            self.transition(Count(current_count + 1))
        } else {
            self.result(current_count)
        }
    }
}

#[test]
fn test_looping_workflow() {
    let workflow = CountLengthOfVecWorkflow::new(Count(0), vec![1, 2, 3, 4]);

    let result = workflow.run();

    assert_eq!(result, 4);
}
