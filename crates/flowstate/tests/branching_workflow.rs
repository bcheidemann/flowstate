use std::ops::ControlFlow;

use flowstate::{Transition, Workflow, WorkflowState};

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
            DiskDriveCommand::Open => match self.state {
                DiskDriveState::Closed => self.state = DiskDriveState::Opening,
                _ => {}
            },
            DiskDriveCommand::Wait => match self.state {
                DiskDriveState::Opening => self.state = DiskDriveState::Open,
                DiskDriveState::Closing => self.state = DiskDriveState::Closed,
                _ => {}
            },
            DiskDriveCommand::Close => match self.state {
                DiskDriveState::Open => self.state = DiskDriveState::Closing,
                _ => {}
            },
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

struct ValidateDriveInitialState;

impl WorkflowState<EjectDiskResult> for EjectDiskWorkflow<ValidateDriveInitialState> {
    fn next(self: Box<Self>) -> Transition<EjectDiskResult> {
        match self.drive.state {
            DiskDriveState::Opening => self.transition(WaitForDriveOpen),
            DiskDriveState::Open => {
                // TODO: This should be possible through self.result but because
                //       it takes ownership of self, we can't have moved out of
                //       self. Maybe a self.map_result(|workflow| ...) helper?
                ControlFlow::Break(self.drive)
            }
            DiskDriveState::Closing => self.transition(WaitForDriveClosed),
            DiskDriveState::Closed => self.transition(OpenDrive),
        }
    }
}

struct WaitForDriveClosed;

impl WorkflowState<EjectDiskResult> for EjectDiskWorkflow<WaitForDriveClosed> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Wait);

        self.transition(OpenDrive)
    }
}

struct OpenDrive;

impl WorkflowState<EjectDiskResult> for EjectDiskWorkflow<OpenDrive> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Open);

        self.transition(WaitForDriveOpen)
    }
}

struct WaitForDriveOpen;

impl WorkflowState<EjectDiskResult> for EjectDiskWorkflow<WaitForDriveOpen> {
    fn next(mut self: Box<Self>) -> Transition<EjectDiskResult> {
        self.drive.issue_command(DiskDriveCommand::Wait);

        // TODO: This should be possible through self.result but because
        //       it takes ownership of self, we can't have moved out of
        //       self. Maybe a self.map_result(|workflow| ...) helper?
        ControlFlow::Break(self.drive)
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
