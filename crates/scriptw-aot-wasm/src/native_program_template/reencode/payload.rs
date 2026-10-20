use super::*;

impl Encoder<'_> {
    pub(super) fn program_payload(
        &self,
        function: &mut wasm_encoder::Function,
    ) -> Result<(), reencode::Error<String>> {
        let contexts = self
            .plan
            .contexts
            .as_ref()
            .expect("owned mode has checked contexts");
        match contexts.bodies.get(&self.body_index) {
            Some(crate::native_program_contexts::FunctionContext::Bootstrap { .. }) => {
                function.instruction(&Instruction::LocalGet(0));
            }
            Some(crate::native_program_contexts::FunctionContext::Program { local, .. }) => {
                crate::gc_types::native_payload::load(function, *local, contexts.schema);
            }
            None => {
                return Err(reencode::Error::UserError(
                    "source payload site has no checked program owner".into(),
                ))
            }
        }
        Ok(())
    }
}
