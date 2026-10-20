//! Native P code can be shared; every source/data/GC owner remains fresh.
use crate::{
    RuntimeArtifact, RuntimeDirectEvalArtifact, RuntimeFunctionArtifact,
    RuntimeGlobalScriptArtifact,
};
use std::sync::Arc;
mod contexts;
mod empty_direct_eval;
mod error;
mod primitive_direct_eval;
use error::invalid;
pub use error::NativeProgramTemplateError;
mod literal_operands;
mod payload;
mod plan;
mod reencode;
mod references;
mod share_bodies;
mod unused_function_types;
pub use literal_operands::NativeProgramLiteralOperand;
pub use payload::{NativeProgramPayload, NativeProgramPayloadDirectory};
#[cfg(test)]
mod tests;
pub const NATIVE_PROGRAM_DATA_MODULE: &str = "scriptw.native_program.data";

/// Packed Wasm storage, not a JavaScript value or a guessed GC shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeProgramArrayWidth {
    I8,
    I16,
}
impl NativeProgramArrayWidth {
    pub const fn bytes(self) -> usize {
        match self {
            Self::I8 => 1,
            Self::I16 => 2,
        }
    }
}
/// Every invocation constructs a fresh array of the actual imported result type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeProgramArrayConstruction {
    Whole,
    Slice,
}

#[derive(Clone, Debug)]
pub struct NativeProgramDataImage {
    pub(crate) name: String,
    pub(crate) bytes: Arc<[u8]>,
}
impl NativeProgramDataImage {
    pub fn import_name(&self) -> &str {
        &self.name
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
#[derive(Clone, Debug)]
pub struct NativeProgramArrayFactory {
    pub(crate) name: String,
    pub(crate) width: NativeProgramArrayWidth,
    pub(crate) construction: NativeProgramArrayConstruction,
    pub(crate) target_array_type: u32,
}
impl NativeProgramArrayFactory {
    pub fn import_name(&self) -> &str {
        &self.name
    }
    pub const fn width(&self) -> NativeProgramArrayWidth {
        self.width
    }
    pub const fn construction(&self) -> NativeProgramArrayConstruction {
        self.construction
    }
}

/// Only checked native-artifact producers can mint this code/data package.
/// Byte images are source-specific operands. They never replace the fresh
/// arrays allocated by the original Wasm instructions at their original call.
#[derive(Debug)]
pub struct NativeProgramTemplate {
    bytes: Arc<[u8]>,
    runtime: Arc<RuntimeArtifact>,
    images: Vec<NativeProgramDataImage>,
    factories: Vec<NativeProgramArrayFactory>,
    literal_operands: Vec<NativeProgramLiteralOperand>,
    payload: Option<NativeProgramPayload>,
    entry: TemplateEntry,
}
#[derive(Debug)]
enum TemplateEntry {
    Program,
    EmptyDirectEval,
    PrimitiveDirectEval,
}
impl NativeProgramTemplate {
    pub fn for_function(
        artifact: &RuntimeFunctionArtifact,
    ) -> Result<Self, NativeProgramTemplateError> {
        Self::from_native(artifact.bytes(), artifact.runtime())
    }
    pub fn for_global_script(
        artifact: &RuntimeGlobalScriptArtifact,
    ) -> Result<Self, NativeProgramTemplateError> {
        Self::from_native(artifact.bytes(), artifact.runtime())
    }
    pub fn for_direct_eval(
        artifact: &RuntimeDirectEvalArtifact,
    ) -> Result<Self, NativeProgramTemplateError> {
        Self::from_native(artifact.bytes(), artifact.runtime())
    }
    fn from_native(
        bytes: &[u8],
        runtime: &Arc<RuntimeArtifact>,
    ) -> Result<Self, NativeProgramTemplateError> {
        let Some(plan) = plan::Plan::read(bytes)? else {
            return Ok(Self {
                bytes: Arc::from(bytes),
                runtime: Arc::clone(runtime),
                images: Vec::new(),
                factories: Vec::new(),
                literal_operands: Vec::new(),
                payload: None,
                entry: TemplateEntry::Program,
            });
        };
        let payload = NativeProgramPayload::from_plan(&plan)?;
        let module = share_bodies::encode(reencode::encode(bytes, &plan)?, &plan)?;
        let module = unused_function_types::encode(module)?;
        Ok(Self {
            bytes: module.into(),
            runtime: Arc::clone(runtime),
            images: plan.images,
            factories: plan.factories,
            literal_operands: plan.literal_operands,
            payload,
            entry: TemplateEntry::Program,
        })
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn runtime(&self) -> &Arc<RuntimeArtifact> {
        &self.runtime
    }
    pub fn data_images(&self) -> &[NativeProgramDataImage] {
        &self.images
    }
    pub fn literal_operands(&self) -> &[NativeProgramLiteralOperand] {
        &self.literal_operands
    }
    pub fn array_factories(&self) -> &[NativeProgramArrayFactory] {
        &self.factories
    }
    pub fn program_payload(&self) -> Option<&NativeProgramPayload> {
        self.payload.as_ref()
    }
}
