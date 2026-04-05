use porffor_ir::ProgramIr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CArtifact {
    pub source: String,
}

pub fn emit(_program: &ProgramIr) -> Result<CArtifact, String> {
    Err("C backend scaffold exists, but shared IR emission is not implemented yet".to_string())
}
