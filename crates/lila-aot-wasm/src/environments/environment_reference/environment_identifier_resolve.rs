//! ResolveBinding selects a held Reference before later source effects.
//! Search, Proxy traps and abrupt completion live in one shared runtime body.

use super::*;
use crate::runtime_helpers::{
    EnvironmentIdentifierResolveArguments, EnvironmentIdentifierResolveParameters, HelperParameters,
};

impl FunctionBuilder<'_> {
    pub(super) fn emit_resolve_environment_identifier_from_record(
        &mut self,
        name: &GcLocal<StringValue>,
        strictness: Strictness,
        record: GcLocal<Environment, Nullable>,
        function: &mut Function,
    ) -> Result<EnvironmentIdentifierReference, EmitError> {
        let schema = self.runtime_schema();
        let key = schema
            .reserve_gc_local(function)
            .initialize(name.load(schema, function), function);
        let kind = schema.reserve_i32_local(function);
        let base = schema.reserve_value_local(function);
        let realm = self.emit_execution_realm(function);
        let helper_base = self.runtime_helper_base()?;
        let (selected_record, entry, cell) = schema
            .call_helper(
                EnvironmentIdentifierResolveArguments::new(
                    &key,
                    &record,
                    &realm,
                    self.current_environment(),
                ),
                helper_base,
                function,
            )
            .bind(schema, self.completion(), kind, &base, function);
        record.clear(function);
        realm.clear(function);
        // All Reference results are rooted before an abrupt branch can abandon
        // this expression. The caller never repeats the environment search.
        self.emit_propagate_current_throw_if_needed(function);
        Ok(EnvironmentIdentifierReference {
            key,
            kind,
            record: selected_record,
            entry,
            cell,
            base,
            strictness,
        })
    }

    pub(crate) fn compile_environment_identifier_resolve_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::EnvironmentIdentifierResolve);
        let parameters =
            self.helper_parameters::<EnvironmentIdentifierResolveParameters>(&mut function);
        self.push_scope();
        self.completion().initialize(&mut function);
        let schema = self.runtime_schema();
        let record = schema
            .reserve_gc_local(&mut function)
            .initialize(parameters.record.load(schema, &mut function), &mut function);
        // A search trap can throw. Route it to this exit, which returns the
        // complete resolution ABI rather than the five-word JS call ABI.
        let exit = self.open_frame(ControlFrameKind::Block, &mut function);
        self.throw_handler_stack.push(exit);
        let reference = self.emit_resolve_environment_identifier_body_from_record(
            &parameters.name,
            Strictness::Sloppy,
            record,
            &mut function,
        )?;
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().emit(&mut function);
        reference.kind.load(&mut function);
        reference.record.load(schema, &mut function);
        reference.entry.load(schema, &mut function);
        reference.cell.load(schema, &mut function);
        reference.base.emit(&mut function);
        self.release_environment_identifier_reference(reference, &mut function);
        self.pop_scope();
        self.clear_helper_execution_realm(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
