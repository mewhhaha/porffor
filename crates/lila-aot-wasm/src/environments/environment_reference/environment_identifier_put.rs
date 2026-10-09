//! PutValue consumes the original held Reference after RHS evaluation. Only
//! the complete write dispatch is shared; binding selection stays in P.

use super::*;
use crate::runtime_helpers::{
    EnvironmentIdentifierPutSloppyArguments, EnvironmentIdentifierPutSloppyParameters,
    EnvironmentIdentifierPutStrictArguments, EnvironmentIdentifierPutStrictParameters,
    HelperParameters,
};

macro_rules! environment_identifier_put_helper {
    ($compile:ident, $id:ident, $parameters:ident, $strictness:ident) => {
        pub(crate) fn $compile(&mut self) -> Result<Function, EmitError> {
            let mut function = self.begin_helper_body(RuntimeHelperId::$id);
            let parameters = self.helper_parameters::<$parameters>(&mut function);
            self.push_scope();
            self.completion().initialize(&mut function);
            let schema = self.runtime_schema();
            // Parameters retain all GC edges. These local copies allow the
            // existing owner to refresh delegates within the held record.
            let reference = EnvironmentIdentifierReference {
                key: schema
                    .reserve_gc_local(&mut function)
                    .initialize(parameters.name.load(schema, &mut function), &mut function),
                kind: schema.reserve_i32_local(&mut function),
                record: schema
                    .reserve_gc_local(&mut function)
                    .initialize(parameters.record.load(schema, &mut function), &mut function),
                entry: schema
                    .reserve_gc_local(&mut function)
                    .initialize(parameters.entry.load(schema, &mut function), &mut function),
                cell: schema
                    .reserve_gc_local(&mut function)
                    .initialize(parameters.cell.load(schema, &mut function), &mut function),
                base: schema.reserve_value_local(&mut function),
                strictness: Strictness::$strictness,
            };
            parameters.reference_kind.load(&mut function);
            reference.kind.store(&mut function);
            reference.base.copy_from(&parameters.base, &mut function);
            self.emit_environment_identifier_put_body(
                &reference,
                &parameters.assigned_value,
                &mut function,
            )?;
            self.release_environment_identifier_reference(reference, &mut function);
            // Abrupt paths above return their complete Completion directly;
            // the normal continuation retains the already evaluated RHS.
            self.completion()
                .set_normal(&parameters.assigned_value, &mut function);
            self.pop_scope();
            self.clear_helper_execution_realm(&mut function);
            parameters.release(&mut function);
            self.completion().emit(&mut function);
            function.instruction(&Instruction::End);
            Ok(self.finish_function(function))
        }
    };
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_environment_identifier_put(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        value: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = self.emit_execution_realm(function);
        let base = self.runtime_helper_base()?;
        macro_rules! call {
            ($arguments:ident) => {
                schema
                    .call_helper(
                        $arguments::new(
                            &reference.key,
                            reference.kind,
                            &reference.record,
                            &reference.entry,
                            &reference.cell,
                            &reference.base,
                            value,
                            &realm,
                            self.current_environment(),
                        ),
                        base,
                        function,
                    )
                    .store(self.completion(), function)
            };
        }
        match reference.strictness {
            Strictness::Sloppy => call!(EnvironmentIdentifierPutSloppyArguments),
            Strictness::Strict => call!(EnvironmentIdentifierPutStrictArguments),
        }
        realm.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    environment_identifier_put_helper!(
        compile_environment_identifier_put_sloppy_helper,
        EnvironmentIdentifierPutSloppy,
        EnvironmentIdentifierPutSloppyParameters,
        Sloppy
    );
    environment_identifier_put_helper!(
        compile_environment_identifier_put_strict_helper,
        EnvironmentIdentifierPutStrict,
        EnvironmentIdentifierPutStrictParameters,
        Strict
    );
}
