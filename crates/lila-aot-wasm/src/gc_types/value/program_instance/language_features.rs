//! The immutable language-policy projection of one actual defining P.
use super::*;

impl ProgramInstanceLocals {
    /// Read the actual defining P, including a foreign constructor's class hook.
    pub(crate) fn emit_nonextensible_private_elements_enabled(
        &self,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        let policy =
            GcI32DomainLocal::new(schema, lila_ir::LanguageFeaturePolicy::default(), function);
        schema
            .struct_type::<ProgramInstance>()
            .field(ProgramInstanceSchema::LANGUAGE_FEATURE_POLICY)
            .read(&self.owner, schema, function)
            .store_domain(&policy, function);
        policy.load(function);
        function.instruction(&Instruction::I32Const(
            lila_ir::LanguageFeaturePolicy::default()
                .with_private_element_extensibility(
                    lila_ir::PrivateElementExtensibility::NonExtensibleAppliesToPrivate,
                )
                .wire_code(),
        ));
        function.instruction(&Instruction::I32Eq);
        policy.clear(schema, function);
    }
}
