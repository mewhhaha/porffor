//! One real Wasm factory executes the proven scalar operands of each eval.
use super::*;
use crate::ValidatedRuntimeArtifact;

pub(super) const EXPORT: &str = "lila.primitive_direct_eval";

impl NativeProgramTemplate {
    pub fn for_primitive_direct_eval(
        proof: &lila_ir::PrimitiveRuntimeDirectEval<'_>,
        runtime: &ValidatedRuntimeArtifact,
    ) -> Self {
        Self {
            bytes: crate::module::primitive_direct_eval_module(proof, EXPORT).into(),
            runtime: Arc::clone(runtime.runtime()),
            images: Vec::new(),
            factories: Vec::new(),
            literal_operands: Vec::new(),
            payload: None,
            entry: TemplateEntry::PrimitiveDirectEval,
        }
    }

    pub fn primitive_direct_eval_export(&self) -> Option<&'static str> {
        match self.entry {
            TemplateEntry::Program | TemplateEntry::EmptyDirectEval => None,
            TemplateEntry::PrimitiveDirectEval => Some(EXPORT),
        }
    }
}
