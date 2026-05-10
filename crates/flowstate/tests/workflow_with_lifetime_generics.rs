use flowstate::{WorkflowState, prelude::*};

struct StrContainer<'a> {
    #[allow(unused)]
    my_str: &'a str,
}

#[derive(Workflow)]
#[flowstate(
    result = WorkflowResult,
    state_trait = MyWorkflowState,
)]
struct MyWorkflow<'workflow, State> {
    #[state]
    state: State,
    my_str: &'workflow str,
    my_str_container: StrContainer<'workflow>,
    message: String,
}

#[derive(State)]
struct StateA<'a, const N: usize, T>(&'a T)
where
    T: AsRef<str>;

impl<'workflow, const N: usize, T> MyWorkflowState<'workflow>
    for MyWorkflow<'workflow, StateA<'workflow, N, T>>
where
    T: AsRef<str>,
{
    fn next(mut self: Box<Self>) -> Transition<'workflow, WorkflowResult> {
        self.message.push_str(self.state.0.as_ref());
        self.message.push_str(self.my_str);

        self.transition_with(|state| StateB::<N, _>(state.0))
    }
}

#[derive(State)]
struct StateB<'a, const N: usize, T>(#[allow(unused)] &'a T);

impl<'workflow, const N: usize, T> MyWorkflowState<'workflow>
    for MyWorkflow<'workflow, StateB<'workflow, N, T>>
{
    fn next(mut self: Box<Self>) -> Transition<'workflow, WorkflowResult> {
        self.message.push_str(self.my_str_container.my_str);

        self.finish_with(|workflow| WorkflowResult {
            message: workflow.message,
        })
    }
}

struct WorkflowResult {
    message: String,
}

#[test]
fn test_basic_workflow_manual_impls() {
    let message = "Hello ".to_string();
    let workflow = MyWorkflow::new(
        StateA::<0, _>(&message),
        "world",
        StrContainer { my_str: "!" },
        String::new(),
    );
    let result = workflow.run();
    assert_eq!(result.message, "Hello world!");
}
