//! Compile proven empty direct eval to a shared, stateless Wasm completion kernel.
use super::*;
use crate::ValidatedRuntimeArtifact;
use lila_ir::EmptyRuntimeDirectEval;

pub(super) const EXPORT: &str = "lila.empty_direct_eval";

impl NativeProgramTemplate {
    /// Parsing, early errors and lowering of the actual source are mandatory
    /// before this proof is available. No source, object, closure, declaration,
    /// environment or caller operand can escape this empty program; its fresh
    /// P allocation is dead and its sole completion code is source-independent.
    pub fn for_empty_direct_eval(
        _proof: &EmptyRuntimeDirectEval<'_>,
        runtime: &ValidatedRuntimeArtifact,
    ) -> Self {
        Self {
            bytes: crate::module::empty_direct_eval_module(_proof, EXPORT).into(),
            runtime: Arc::clone(runtime.runtime()),
            images: Vec::new(),
            factories: Vec::new(),
            literal_operands: Vec::new(),
            payload: None,
            entry: TemplateEntry::EmptyDirectEval,
        }
    }
}

impl NativeProgramTemplate {
    pub fn permits_instance_reuse(&self) -> bool {
        match self.entry {
            TemplateEntry::Program => self
                .payload
                .as_ref()
                .is_some_and(|p| p.permits_instance_reuse()),
            TemplateEntry::EmptyDirectEval | TemplateEntry::PrimitiveDirectEval => true,
        }
    }
    pub fn empty_direct_eval_export(&self) -> Option<&'static str> {
        match self.entry {
            TemplateEntry::Program | TemplateEntry::PrimitiveDirectEval => None,
            TemplateEntry::EmptyDirectEval => Some(empty_direct_eval::EXPORT),
        }
    }
}
