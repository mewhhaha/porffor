//! Independent AOT units for CreateDynamicFunction's syntax-proven arguments.

use crate::{DynamicFunctionKind, FunctionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedDynamicFunction {
    pub kind: DynamicFunctionKind,
    pub arguments: Vec<String>,
    pub outcome: PreparedDynamicFunctionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedDynamicFunctionOutcome {
    Compiled { function_id: FunctionId },
    SyntaxError { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DynamicFunctionSourceAdmission {
    Intrinsic,
    RuntimeCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicFunctionSource {
    pub(crate) admission: DynamicFunctionSourceAdmission,
    pub(crate) kind: DynamicFunctionKind,
    pub(crate) arguments: Vec<String>,
}
