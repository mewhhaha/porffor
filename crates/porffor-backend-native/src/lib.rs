use porffor_ir::ProgramIr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeArtifact {
    pub target_triple: Option<String>,
}

pub fn emit(_program: &ProgramIr, target_triple: Option<&str>) -> Result<NativeArtifact, String> {
    Err(format!(
        "native backend scaffold exists for target {:?}, but shared IR emission is not implemented yet",
        target_triple
    ))
}
