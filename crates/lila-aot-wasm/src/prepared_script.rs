//! Finite independently compiled Script dispatch owns rooted Realm restoration.
use super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::{
    CompletionLocals, DirectEvalExecutionContext, Environment, GcLocal, I32Local, Nullable,
    PrivateEnvironment, RealmRecord, RealmRecordSchema, StoredValue, StringValue, ValueLocals,
};
use lila_ir::NativeErrorKind;

/// Direct eval requires its caller-context owner and cannot enter this path.
#[derive(Clone, Copy)]
pub(crate) enum GlobalPreparedScriptKind {
    IndirectEval,
    RealmScript,
}

impl GlobalPreparedScriptKind {
    fn script_kind(self) -> PreparedScriptKind {
        match self {
            Self::IndirectEval => PreparedScriptKind::IndirectEval,
            Self::RealmScript => PreparedScriptKind::RealmScript,
        }
    }

    fn unsupported_operation(self) -> lila_ir::DynamicSourceRuntimeOperation {
        match self {
            Self::IndirectEval => lila_ir::DynamicSourceRuntimeOperation::Eval,
            Self::RealmScript => lila_ir::DynamicSourceRuntimeOperation::RealmEvalScript,
        }
    }
}

/// These are the two independently compiled global-source entry owners.
/// ShadowRealm evaluation must supply its receiver's Realm and retain the
/// distinction between parsing failure and target execution failure.
#[derive(Clone, Copy)]
enum PreparedScriptDispatch<'a> {
    Global(GlobalPreparedScriptKind),
    ShadowRealm {
        realm: &'a GcLocal<RealmRecord>,
        source_parsed: I32Local,
    },
}

impl PreparedScriptDispatch<'_> {
    fn script_kind(self) -> PreparedScriptKind {
        match self {
            Self::Global(kind) => kind.script_kind(),
            Self::ShadowRealm { .. } => PreparedScriptKind::ShadowRealmEvaluate,
        }
    }

    fn unsupported_operation(self) -> lila_ir::DynamicSourceRuntimeOperation {
        match self {
            Self::Global(kind) => kind.unsupported_operation(),
            Self::ShadowRealm { .. } => lila_ir::DynamicSourceRuntimeOperation::ShadowRealmEvaluate,
        }
    }
}

/// What the program's Script hook did with a source string.
#[derive(Clone, Copy)]
enum PreparedScriptStatus {
    /// No prepared Script has this source; nothing ran.
    Unmatched,
    /// A prepared Script whose source does not parse.
    SyntaxError,
    /// A prepared Script ran in the target Realm.
    Entered,
}

impl PreparedScriptStatus {
    const fn code(self) -> i32 {
        match self {
            Self::Unmatched => 0,
            Self::SyntaxError => 1,
            Self::Entered => 2,
        }
    }
}

/// The wire word naming a global Script entry kind to the program hook.
/// Direct eval has its own caller-context owner and no word.
fn script_kind_code(kind: &PreparedScriptKind) -> Option<i32> {
    match kind {
        PreparedScriptKind::IndirectEval => Some(0),
        PreparedScriptKind::RealmScript => Some(1),
        PreparedScriptKind::ShadowRealmEvaluate => Some(2),
        PreparedScriptKind::DirectEval(_) => None,
    }
}

#[must_use = "restore the caller Realm before publishing a Script completion"]
struct PreparedScriptRealmExecution {
    saved_realm: GcLocal<RealmRecord>,
    realm: GcLocal<RealmRecord>,
    environment: GcLocal<Environment, Nullable>,
    this_value: ValueLocals,
    undefined: ValueLocals,
    private_environment: GcLocal<PrivateEnvironment, Nullable>,
    direct_eval_context: GcLocal<DirectEvalExecutionContext, Nullable>,
}

impl PreparedScriptRealmExecution {
    fn inputs(&self) -> crate::function_entry::PreparedScriptInputs<'_> {
        crate::function_entry::PreparedScriptInputs::new(
            &self.environment,
            &self.this_value,
            &self.undefined,
            &self.environment,
            &self.private_environment,
            &self.direct_eval_context,
        )
    }

    fn restore(self, builder: &FunctionBuilder<'_>, function: &mut Function) {
        builder.replace_current_realm(&self.saved_realm, function);
        self.direct_eval_context.clear(function);
        self.private_environment.clear(function);
        self.undefined.clear(function);
        self.this_value.clear(function);
        self.environment.clear(function);
        self.realm.clear(function);
        self.saved_realm.clear(function);
    }
}

impl FunctionBuilder<'_> {
    fn emit_enter_prepared_script_realm(
        &mut self,
        realm: GcLocal<RealmRecord>,
        function: &mut Function,
    ) -> PreparedScriptRealmExecution {
        let schema = self.runtime_schema();
        let saved_realm = self.load_current_realm(function);
        let environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(
                schema
                    .struct_type::<RealmRecord>()
                    .field(RealmRecordSchema::GLOBAL_ENVIRONMENT)
                    .read(&realm, schema, function)
                    .reference()
                    .require_non_null(function)
                    .nullable(),
                function,
            );
        let global_this = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<RealmRecord>()
                .field(RealmRecordSchema::GLOBAL_THIS)
                .read(&realm, schema, function)
                .reference()
                .require_non_null(function),
            function,
        );
        let this_value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&global_this, &this_value, schema, function);
        global_this.clear(function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        let private_environment = schema
            .reserve_gc_local::<PrivateEnvironment, Nullable>(function)
            .initialize_null(schema, function);
        let direct_eval_context = schema
            .reserve_gc_local::<DirectEvalExecutionContext, Nullable>(function)
            .initialize_null(schema, function);
        self.replace_current_realm(&realm, function);
        PreparedScriptRealmExecution {
            saved_realm,
            realm,
            environment,
            this_value,
            undefined,
            private_environment,
            direct_eval_context,
        }
    }

    pub(crate) fn emit_module_prelude(
        &mut self,
        id: lila_ir::StaticScriptId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let planned = self
            .functions
            .get(&id.function_id())
            .ok_or_else(|| EmitError::unsupported("Module prelude has no planned Script entry"))?
            .entry
            .clone();
        let entry = planned.prepared_script()?;
        let realm = self.load_current_realm(function);
        let execution = self.emit_enter_prepared_script_realm(realm, function);
        let pending = schema.reserve_completion(function);
        pending.store_call(
            entry.emit_call(execution.inputs(), schema, function),
            function,
        );
        execution.restore(self, function);
        self.completion().copy_from(&pending, function);
        pending.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        self.completion().set_normal(&undefined, function);
        undefined.clear(function);
        Ok(())
    }

    pub(crate) fn emit_prepared_script_dispatch(
        &mut self,
        kind: GlobalPreparedScriptKind,
        source: &GcLocal<StringValue>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_prepared_script_dispatch_with_owner(
            PreparedScriptDispatch::Global(kind),
            source,
            output,
            function,
        )
    }

    /// The native evaluate owner has already validated receiver and String
    /// input. Syntax errors are produced before entering the receiver Realm;
    /// execution completions return only after restoring the active caller.
    pub(crate) fn emit_evaluate_shadow_realm_source(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        source: &GcLocal<StringValue>,
        output: &CompletionLocals,
        source_parsed: I32Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I32Const(0));
        source_parsed.store(function);
        self.emit_prepared_script_dispatch_with_owner(
            PreparedScriptDispatch::ShadowRealm {
                realm,
                source_parsed,
            },
            source,
            output,
            function,
        )
    }

    /// The program's hook owns matching and execution of the prepared Scripts;
    /// the runtime only supplies the target Realm and reads the status back.
    fn emit_prepared_script_dispatch_with_owner(
        &mut self,
        dispatch: PreparedScriptDispatch<'_>,
        source: &GcLocal<StringValue>,
        output: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::runtime_helpers::{PreparedScriptArguments, ProgramHook};
        let schema = self.runtime_schema();
        let kind = schema.reserve_i32_local(function);
        let code = script_kind_code(&dispatch.script_kind())
            .expect("a global Script dispatch is never a direct eval");
        kind.set_constant(code, function);
        let status = schema.reserve_i32_local(function);
        status.set_constant(PreparedScriptStatus::Unmatched.code(), function);
        let realm = match dispatch {
            PreparedScriptDispatch::Global(_) => schema
                .reserve_gc_local(function)
                .initialize(self.emit_current_function_realm(function), function),
            PreparedScriptDispatch::ShadowRealm { realm, .. } => schema
                .reserve_gc_local(function)
                .initialize(realm.load(schema, function), function),
        };
        self.emit_program_hook_dispatch(
            ProgramHook::PreparedScript,
            |_, hook, function| {
                schema
                    .call_hook(
                        hook,
                        PreparedScriptArguments::new(kind, source, &realm),
                        function,
                    )
                    .store(output, status, function);
                Ok(())
            },
            |_, _| Ok(()),
            function,
        )?;
        realm.clear(function);
        status.load(function);
        function.instruction(&Instruction::I32Const(
            PreparedScriptStatus::Unmatched.code(),
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_reject_dynamic_source(dispatch.unsupported_operation(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if let PreparedScriptDispatch::ShadowRealm { source_parsed, .. } = dispatch {
            status.load(function);
            function.instruction(&Instruction::I32Const(PreparedScriptStatus::Entered.code()));
            function.instruction(&Instruction::I32Eq);
            source_parsed.store(function);
        }
        schema.release_i32_local(status, function);
        schema.release_i32_local(kind, function);
        Ok(())
    }

    /// The program half of Script dispatch: compare the source against the
    /// Scripts prepared for this entry kind, then enter the target Realm and
    /// run the matching unit. A miss reports `Unmatched` and does nothing.
    pub(crate) fn compile_prepared_script_hook(&mut self) -> Result<Function, EmitError> {
        use crate::runtime_helpers::{HelperParameters, PreparedScriptParameters};
        let mut function = self.begin_helper_body(RuntimeHelperId::PreparedScript);
        let parameters = self.helper_parameters::<PreparedScriptParameters>(&mut function);
        self.push_scope();
        let schema = self.runtime_schema();
        let output = schema.reserve_completion(&mut function);
        output.initialize(&mut function);
        let status = schema.reserve_i32_local(&mut function);
        status.set_constant(PreparedScriptStatus::Unmatched.code(), &mut function);
        let folding = schema.reserve_i32_local(&mut function);
        folding.set_constant(0, &mut function);
        let equal = schema.reserve_i32_local(&mut function);
        let entries = self.functions.prepared_scripts().to_vec();
        let done = self.open_frame(ControlFrameKind::Block, &mut function);
        for script in entries {
            let Some(code) = script_kind_code(&script.kind) else {
                continue;
            };
            parameters.kind.load(&mut function);
            function.instruction(&Instruction::I32Const(code));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, &mut function);
            let expected = schema.reserve_gc_local(&mut function).initialize(
                self.emit_interned_string_reference(&script.source, &mut function)?,
                &mut function,
            );
            self.emit_gc_string_equality(
                &parameters.source,
                &expected,
                folding,
                equal,
                &mut function,
            );
            expected.clear(&mut function);
            equal.load(&mut function);
            self.open_frame(ControlFrameKind::If, &mut function);
            // A ShadowRealm that fails to parse never enters its Realm.
            let execution = match (&script.kind, &script.outcome) {
                (
                    PreparedScriptKind::ShadowRealmEvaluate,
                    PreparedScriptOutcome::DeferredSyntaxError { .. },
                ) => None,
                _ => {
                    let realm = schema
                        .reserve_gc_local(&mut function)
                        .initialize(parameters.realm.load(schema, &mut function), &mut function);
                    Some(self.emit_enter_prepared_script_realm(realm, &mut function))
                }
            };
            match &script.outcome {
                PreparedScriptOutcome::DeferredSyntaxError { .. } => {
                    let error_realm = self.load_current_realm(&mut function);
                    let prototype = schema.reserve_value_local(&mut function);
                    self.emit_load_non_array_realm_intrinsic(
                        &error_realm,
                        NonArrayRealmIntrinsicSlot::SyntaxErrorPrototype,
                        &prototype,
                        &mut function,
                    );
                    let message = self.strings.source_runtime_error_message(
                        SourceRuntimeErrorMessage::PreparedScript(&script.outcome),
                    )?;
                    self.emit_throw_runtime_error_with_prototype(
                        NativeErrorKind::SyntaxError,
                        message,
                        &prototype,
                        &output,
                        &mut function,
                    )?;
                    prototype.clear(&mut function);
                    error_realm.clear(&mut function);
                    status.set_constant(PreparedScriptStatus::SyntaxError.code(), &mut function);
                }
                PreparedScriptOutcome::Executable(unit) => {
                    let planned = self
                        .functions
                        .get(&unit.id.function_id())
                        .ok_or_else(|| {
                            EmitError::unsupported("prepared global Script has no planned entry")
                        })?
                        .entry
                        .clone();
                    let entry = planned.prepared_script()?;
                    output.store_call(
                        entry.emit_call(
                            execution
                                .as_ref()
                                .expect("executable source entered its Realm")
                                .inputs(),
                            schema,
                            &mut function,
                        ),
                        &mut function,
                    );
                    status.set_constant(PreparedScriptStatus::Entered.code(), &mut function);
                }
            }
            if let Some(execution) = execution {
                execution.restore(self, &mut function);
            }
            self.emit_branch_to_target(done, &mut function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, &mut function);
        schema.release_i32_local(equal, &mut function);
        schema.release_i32_local(folding, &mut function);
        output.clear(&mut function);
        self.pop_scope();
        parameters.release(&mut function);
        self.completion().emit(&mut function);
        status.load(&mut function);
        schema.release_i32_local(status, &mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
