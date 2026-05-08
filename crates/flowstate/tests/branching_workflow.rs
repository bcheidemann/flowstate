use flowstate::prelude::*;

#[derive(Clone, Copy)]
enum DiskDriveState {
    Opening,
    Open,
    Closing,
    Closed,
}

#[derive(Clone, Copy)]
enum DiskDriveCommand {
    Open,
    Wait,
    #[allow(dead_code)]
    Close,
}

struct DiskDrive {
    history: Vec<(DiskDriveState, DiskDriveCommand)>,
    state: DiskDriveState,
}

impl DiskDrive {
    fn issue_command(&mut self, command: DiskDriveCommand) {
        self.history.push((self.state, command));

        match command {
            DiskDriveCommand::Open => {
                if let DiskDriveState::Closed = self.state {
                    self.state = DiskDriveState::Opening
                }
            }
            DiskDriveCommand::Wait => match self.state {
                DiskDriveState::Opening => self.state = DiskDriveState::Open,
                DiskDriveState::Closing => self.state = DiskDriveState::Closed,
                _ => {}
            },
            DiskDriveCommand::Close => {
                if let DiskDriveState::Open = self.state {
                    self.state = DiskDriveState::Closing
                }
            }
        }
    }
}

#[derive(Workflow)]
#[flowstate(result = EjectDiskResult)]
struct EjectDiskWorkflow<State> {
    #[state]
    _state: State,
    drive: DiskDrive,
}

type EjectDiskResult = DiskDrive;

#[derive(State)]
struct ValidateDriveInitialState;

impl EjectDiskWorkflowState for EjectDiskWorkflow<ValidateDriveInitialState> {
    fn next(self: Box<Self>) -> Transition<EjectDiskResult> {
        match self.drive.state {
            DiskDriveState::Opening => self.transition(WaitForDriveOpen),
            DiskDriveState::Open => self.finish_with(|workflow| workflow.drive),
            DiskDriveState::Closing => self.transition(WaitForDriveClosed),
            DiskDriveState::Closed => self.transition(OpenDrive),
        }
    }
}

#[derive(State)]
struct WaitForDriveClosed;

impl EjectDiskWorkflowState for EjectDiskWorkflow<WaitForDriveClosed> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Wait);

        self.transition(OpenDrive)
    }
}

#[derive(State)]
struct OpenDrive;

impl EjectDiskWorkflowState for EjectDiskWorkflow<OpenDrive> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Open);

        self.transition(WaitForDriveOpen)
    }
}

#[derive(State)]
struct WaitForDriveOpen;

impl EjectDiskWorkflowState for EjectDiskWorkflow<WaitForDriveOpen> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Wait);

        self.finish_with(|workflow| workflow.drive)
    }
}

#[test]
fn test_branching_workflow_1() {
    let drive = DiskDrive {
        state: DiskDriveState::Open,
        history: vec![],
    };
    let workflow = EjectDiskWorkflow::new(ValidateDriveInitialState, drive);

    let result = workflow.run();

    assert_eq!(result.history.len(), 0);
    assert!(matches!(result.state, DiskDriveState::Open));
}

#[test]
fn test_branching_workflow_2() {
    let drive = DiskDrive {
        state: DiskDriveState::Closed,
        history: vec![],
    };
    let workflow = EjectDiskWorkflow::new(ValidateDriveInitialState, drive);

    let result = workflow.run();

    assert_eq!(result.history.len(), 2);
    assert!(matches!(
        result.history[0],
        (DiskDriveState::Closed, DiskDriveCommand::Open)
    ));
    assert!(matches!(
        result.history[1],
        (DiskDriveState::Opening, DiskDriveCommand::Wait)
    ));
    assert!(matches!(result.state, DiskDriveState::Open));
}

#[test]
fn test_branching_workflow_3() {
    let drive = DiskDrive {
        state: DiskDriveState::Closing,
        history: vec![],
    };
    let workflow = EjectDiskWorkflow::new(ValidateDriveInitialState, drive);

    let result = workflow.run();

    assert_eq!(result.history.len(), 3);
    assert!(matches!(
        result.history[0],
        (DiskDriveState::Closing, DiskDriveCommand::Wait)
    ));
    assert!(matches!(
        result.history[1],
        (DiskDriveState::Closed, DiskDriveCommand::Open)
    ));
    assert!(matches!(
        result.history[2],
        (DiskDriveState::Opening, DiskDriveCommand::Wait)
    ));
    assert!(matches!(result.state, DiskDriveState::Open));
}
