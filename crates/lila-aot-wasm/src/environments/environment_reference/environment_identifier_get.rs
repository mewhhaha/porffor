//! GetBindingValue consumes the selected Reference without repeating search.

use super::*;
use crate::runtime_helpers::{
    EnvironmentIdentifierGetSloppyArguments, EnvironmentIdentifierGetSloppyParameters,
    EnvironmentIdentifierGetStrictArguments, EnvironmentIdentifierGetStrictParameters,
    EnvironmentIdentifierTypeofSloppyArguments, EnvironmentIdentifierTypeofSloppyParameters,
    EnvironmentIdentifierTypeofStrictArguments, EnvironmentIdentifierTypeofStrictParameters,
    HelperParameters,
};

macro_rules! environment_identifier_get_helper {
    ($compile:ident, $id:ident, $parameters:ident, $strictness:ident, $read:ident) => {
        pub(crate) fn $compile(&mut self) -> Result<Function, EmitError> {
            let mut function = self.begin_helper_body(RuntimeHelperId::$id);
            let parameters = self.helper_parameters::<$parameters>(&mut function);
            self.push_scope();
            self.completion().initialize(&mut function);
            let schema = self.runtime_schema();
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
            let value = schema.reserve_value_local(&mut function);
            self.emit_environment_identifier_get_body(
                &reference,
                EnvironmentIdentifierRead::$read,
                &value,
                &mut function,
            )?;
            self.release_environment_identifier_reference(reference, &mut function);
            self.completion().commit_if_normal(&value, &mut function);
            value.clear(&mut function);
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
    pub(crate) fn emit_environment_identifier_get(
        &mut self,
        reference: &EnvironmentIdentifierReference,
        read: EnvironmentIdentifierRead,
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
                            &realm,
                            self.current_environment(),
                        ),
                        base,
                        function,
                    )
                    .store(self.completion(), function)
            };
        }
        match (reference.strictness, read) {
            (Strictness::Sloppy, EnvironmentIdentifierRead::Value) => {
                call!(EnvironmentIdentifierGetSloppyArguments)
            }
            (Strictness::Strict, EnvironmentIdentifierRead::Value) => {
                call!(EnvironmentIdentifierGetStrictArguments)
            }
            (Strictness::Sloppy, EnvironmentIdentifierRead::Typeof) => {
                call!(EnvironmentIdentifierTypeofSloppyArguments)
            }
            (Strictness::Strict, EnvironmentIdentifierRead::Typeof) => {
                call!(EnvironmentIdentifierTypeofStrictArguments)
            }
        }
        realm.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        value.copy_from(self.completion().value(), function);
        Ok(())
    }

    environment_identifier_get_helper!(
        compile_environment_identifier_get_sloppy_helper,
        EnvironmentIdentifierGetSloppy,
        EnvironmentIdentifierGetSloppyParameters,
        Sloppy,
        Value
    );
    environment_identifier_get_helper!(
        compile_environment_identifier_get_strict_helper,
        EnvironmentIdentifierGetStrict,
        EnvironmentIdentifierGetStrictParameters,
        Strict,
        Value
    );
    environment_identifier_get_helper!(
        compile_environment_identifier_typeof_sloppy_helper,
        EnvironmentIdentifierTypeofSloppy,
        EnvironmentIdentifierTypeofSloppyParameters,
        Sloppy,
        Typeof
    );
    environment_identifier_get_helper!(
        compile_environment_identifier_typeof_strict_helper,
        EnvironmentIdentifierTypeofStrict,
        EnvironmentIdentifierTypeofStrictParameters,
        Strict,
        Typeof
    );
}
