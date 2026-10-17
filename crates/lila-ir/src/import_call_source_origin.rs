use crate::{ModuleUnitId, RuntimeDirectEvalImportOrigin};

/// Where an ImportCall's resolution authority lives. An external caller record
/// is never reinterpreted as a numeric record in this ProgramInstance's graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportCallSourceOriginIr {
    Local(Option<ModuleUnitId>),
    ExternalCaller(RuntimeDirectEvalImportOrigin),
}
