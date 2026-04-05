use porffor_ir::ProgramIr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub invariant_note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError {
    message: String,
}

impl EmitError {
    pub fn not_ready() -> Self {
        Self {
            message: "AOT Wasm backend skeleton exists, but real direct JS->Wasm codegen is not implemented yet".to_string(),
        }
    }
}

impl core::fmt::Display for EmitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EmitError {}

pub fn emit(_program: &ProgramIr) -> Result<WasmArtifact, EmitError> {
    Err(EmitError::not_ready())
}
