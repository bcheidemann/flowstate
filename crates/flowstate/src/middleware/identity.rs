use crate::middleware::{AsyncWorkflowMiddleware, WorkflowMiddleware};

#[derive(Default)]
pub struct IdentityMiddleware;

impl WorkflowMiddleware for IdentityMiddleware {}

impl AsyncWorkflowMiddleware for IdentityMiddleware {}
