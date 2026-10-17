//! Constructor-owned AOT InitializeInstanceElements, including foreign eval super.
use super::*;
use crate::gc_types::{
    ExecutableCode, ExecutableCodeSchema, FunctionContext, FunctionContextSchema, FunctionLocal,
    FunctionObject, FunctionObjectSchema, InitializeInstanceElementsHook, Nullable,
    ProgramInstanceLocals, ValueLocals,
};
use crate::runtime_helpers::{HelperParameters, InitializeInstanceElementsParameters};

fn has_instance_work(meta: &WasmFunctionMeta) -> bool {
    meta.class_instance_element_plan
        .as_ref()
        .is_some_and(|plan| !plan.elements.is_empty() || !plan.private_method_brands.is_empty())
}

impl FunctionBuilder<'_> {
    /// Nonconstructors and truly empty plans explicitly capture null. Private
    /// methods/accessors count as work even when no instance fields exist.
    pub(super) fn emit_instance_elements_initializer_capture(
        &self,
        meta: &WasmFunctionMeta,
        function: &mut Function,
    ) -> Result<FunctionLocal<InitializeInstanceElementsHook, Nullable>, EmitError> {
        let schema = self.runtime_schema();
        Ok(schema.reserve_function_local(function).initialize(
            schema.instance_elements_initializer_reference(
                self.runtime_helper_base()?,
                has_instance_work(meta),
                function,
            ),
            function,
        ))
    }

    pub(super) fn emit_initialize_super_instance_elements(
        &mut self,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // ExternalCaller is a real branch. It cannot project to a missing local
        // FunctionId and fall back to the eval program's lexical owner.
        let local_owner = match self.direct_eval_derived_owner() {
            Some(lila_ir::DirectEvalDerivedOwner::LocalFunction(owner)) => Some(owner.clone()),
            Some(lila_ir::DirectEvalDerivedOwner::ExternalCaller) => None,
            None => Some(
                self.lexical_derived_activation
                    .map(|activation| activation.owner_function_id.clone())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "super call requires its validated constructor owner",
                        )
                    })?,
            ),
        };
        if let Some(owner) = local_owner {
            let meta = self.functions.get(&owner).ok_or_else(|| {
                EmitError::unsupported("derived constructor owner is not planned")
            })?;
            if !has_instance_work(meta) {
                return Ok(());
            }
        }
        self.emit_active_constructor_instance_elements(receiver, function)
    }

    fn emit_active_constructor_instance_elements(
        &mut self,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let active = schema.reserve_value_local(function);
        // For an external caller this reads the retained ACTIVE_FUNCTION_CELL.
        // No update here replaces any of the four original derived cells.
        self.emit_get_derived_active_function_to_locals(&active, function)?;
        let constructor = schema.reserve_gc_local(function).initialize(
            active.cast_reference::<FunctionObject>(schema, function),
            function,
        );
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CONTEXT)
                .read(&constructor, schema, function)
                .reference(),
            function,
        );
        let hook = schema.reserve_function_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::INSTANCE_ELEMENTS_INITIALIZER)
                .read(&context, schema, function)
                .callable(),
            function,
        );
        hook.load(function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let callee = schema
            .reserve_function_local(function)
            .initialize(hook.load(function).require_non_null(function), function);
        // The actual constructor's P can differ from both current and calling P
        // after nested eval. It is rooted with that constructor's exact context.
        let defining_program = ProgramInstanceLocals::bind(
            schema,
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::DEFINING_PROGRAM)
                .read(&context, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        self.completion().store_call(
            schema.call_instance_elements_initializer(
                &callee,
                &defining_program,
                &context,
                receiver,
                function,
            ),
            function,
        );
        defining_program.clear(function);
        callee.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        hook.clear(function);
        context.clear(function);
        constructor.clear(function);
        active.clear(function);
        Ok(())
    }

    pub(crate) fn compile_initialize_instance_elements_hook(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::InitializeInstanceElements);
        let parameters =
            self.helper_parameters::<InitializeInstanceElementsParameters>(&mut function);
        let schema = self.runtime_schema();
        self.completion().initialize(&mut function);
        // A mismatched defining P is an internal invariant failure, never an
        // instruction to interpret a foreign FunctionId using this P's plans.
        parameters.caller_program.load(schema, &mut function);
        schema
            .struct_type::<FunctionContext>()
            .field(FunctionContextSchema::DEFINING_PROGRAM)
            .read(&parameters.caller_function_context, schema, &mut function)
            .reference()
            .require_non_null(&mut function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, &mut function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let constructor = schema.reserve_gc_local(&mut function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::ACTIVE_FUNCTION)
                .read(&parameters.caller_function_context, schema, &mut function)
                .reference()
                .require_non_null(&mut function),
            &mut function,
        );
        let code = schema.reserve_gc_local(&mut function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CODE)
                .read(&constructor, schema, &mut function)
                .reference(),
            &mut function,
        );
        let source_id = schema.reserve_i64_local(&mut function);
        schema
            .struct_type::<ExecutableCode>()
            .field(ExecutableCodeSchema::SOURCE_ID)
            .read(&code, schema, &mut function)
            .store_i64(source_id, &mut function);
        let done = self.open_frame(ControlFrameKind::Block, &mut function);
        // This helper is generated from this physical P's own constructor plans.
        let constructors = self
            .functions
            .values()
            .filter(|meta| has_instance_work(meta))
            .cloned()
            .collect::<Vec<_>>();
        for meta in constructors {
            source_id.load(&mut function);
            function.instruction(&Instruction::I64Const(meta.entry.source_id()));
            function.instruction(&Instruction::I64Eq);
            self.open_frame(ControlFrameKind::If, &mut function);
            self.emit_initialize_instance_elements(
                &meta,
                &parameters.caller_function_context,
                &parameters.receiver,
                &mut function,
            )?;
            self.emit_branch_to_target(done, &mut function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // A nonnull hook guarantees a nonempty matching plan. Unknown SOURCE_ID
        // is corrupt internal linkage and must never become a successful no-op.
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i64_local(source_id, &mut function);
        code.clear(&mut function);
        constructor.clear(&mut function);
        self.completion().emit(&mut function);
        self.clear_helper_function_context(&mut function);
        self.clear_entry_program(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
