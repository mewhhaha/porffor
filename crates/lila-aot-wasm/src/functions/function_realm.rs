use super::*;
use crate::gc_types::{
    BoundFunction, BoundFunctionSchema, CompletionLocals, FunctionContext, FunctionContextSchema,
    FunctionObject, FunctionObjectSchema, GcLocal, GcNullability, I32Local, Nullable, ProxyObject,
    ProxyObjectSchema, RealmRecord, RuntimeSchema, StoredValue, StoredValueSchema, ValueLocals,
};

enum FunctionRealmOutcome {
    Resolved,
    Revoked,
    Invalid,
}
impl FunctionRealmOutcome {
    const fn runtime_code(&self) -> i32 {
        match self {
            Self::Resolved => 0,
            Self::Revoked => 1,
            Self::Invalid => 2,
        }
    }
}

/// A realm cannot escape before both non-resolved outcomes are routed.
#[must_use]
pub(crate) struct FunctionRealmResultLocals {
    realm: GcLocal<RealmRecord, Nullable>,
    outcome: I32Local,
}

#[must_use]
pub(crate) struct ResolvedFunctionRealmLocal(GcLocal<RealmRecord>);
impl ResolvedFunctionRealmLocal {
    pub(crate) fn realm(&self) -> &GcLocal<RealmRecord> {
        &self.0
    }
}

/// Invalid representation always traps. A revoked callback may use its
/// captured current Realm; constructor fallback produces the required throw.
pub(crate) enum FunctionRealmRevokedRoute<'a> {
    UseCurrentRealm {
        realm: &'a GcLocal<RealmRecord>,
    },
    ThrowTypeErrorAndReturn {
        result: &'a CompletionLocals,
    },
    ThrowTypeErrorAndBranch {
        result: &'a CompletionLocals,
        target: ControlTarget,
    },
}

fn stored_into(
    stored: GcLocal<StoredValue>,
    value: &ValueLocals,
    schema: &RuntimeSchema,
    function: &mut Function,
) {
    schema
        .struct_type::<StoredValue>()
        .read_into(&stored, value, schema, function);
    stored.clear(function);
}

impl FunctionBuilder<'_> {
    /// Follow strong bound/proxy targets without property access. Function
    /// identity is tested with concrete GC types; tag Function is insufficient
    /// to distinguish an ordinary callable from a bound exotic.
    pub(crate) fn emit_get_function_realm(
        &self,
        source: &ValueLocals,
        function: &mut Function,
    ) -> FunctionRealmResultLocals {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local::<RealmRecord, Nullable>(function)
            .initialize_null(schema, function);
        let outcome = schema.reserve_i32_local(function);
        let current = schema.reserve_value_local(function);
        current.copy_from(source, function);
        function.instruction(&Instruction::I32Const(
            FunctionRealmOutcome::Invalid.runtime_code(),
        ));
        outcome.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        current.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Function.tag()));
        function.instruction(&Instruction::I32Eq);
        current.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<BoundFunction>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bound = schema.reserve_gc_local(function).initialize(
            current.cast_reference::<BoundFunction>(schema, function),
            function,
        );
        let target = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<BoundFunction>()
                .field(BoundFunctionSchema::TARGET)
                .read(&bound, schema, function)
                .reference(),
            function,
        );
        stored_into(target, &current, schema, function);
        bound.clear(function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        current.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Object.tag()));
        function.instruction(&Instruction::I32Eq);
        current.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ProxyObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        let proxy = schema.reserve_gc_local(function).initialize(
            current.cast_reference::<ProxyObject>(schema, function),
            function,
        );
        let proxy_type = schema.struct_type::<ProxyObject>();
        let handler = schema.reserve_gc_local(function).initialize(
            proxy_type
                .field(ProxyObjectSchema::HANDLER)
                .read(&proxy, schema, function)
                .reference(),
            function,
        );
        let handler_tag = schema.reserve_i32_local(function);
        schema
            .struct_type::<StoredValue>()
            .field(StoredValueSchema::TAG)
            .read(&handler, schema, function)
            .store(handler_tag, function);
        handler.clear(function);
        let target = schema.reserve_gc_local(function).initialize(
            proxy_type
                .field(ProxyObjectSchema::TARGET)
                .read(&proxy, schema, function)
                .reference(),
            function,
        );
        stored_into(target, &current, schema, function);
        proxy.clear(function);
        handler_tag.load(function);
        schema.release_i32_local(handler_tag, function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null.tag()));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I32Const(
            FunctionRealmOutcome::Revoked.runtime_code(),
        ));
        outcome.store(function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        current.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Function.tag()));
        function.instruction(&Instruction::I32Eq);
        current.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<FunctionObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        let callable = schema.reserve_gc_local(function).initialize(
            current.cast_reference::<FunctionObject>(schema, function),
            function,
        );
        let context = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CONTEXT)
                .read(&callable, schema, function)
                .reference(),
            function,
        );
        realm.replace(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::REALM)
                .read(&context, schema, function)
                .reference()
                .nullable(),
            function,
        );
        context.clear(function);
        callable.clear(function);
        function.instruction(&Instruction::I32Const(
            FunctionRealmOutcome::Resolved.runtime_code(),
        ));
        outcome.store(function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        current.clear(function);
        FunctionRealmResultLocals { realm, outcome }
    }

    pub(crate) fn emit_route_function_realm_result(
        &mut self,
        result: FunctionRealmResultLocals,
        revoked_route: FunctionRealmRevokedRoute<'_>,
        function: &mut Function,
    ) -> Result<ResolvedFunctionRealmLocal, EmitError> {
        let schema = self.runtime_schema();
        result.outcome.load(function);
        function.instruction(&Instruction::I32Const(
            FunctionRealmOutcome::Revoked.runtime_code(),
        ));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        match revoked_route {
            FunctionRealmRevokedRoute::UseCurrentRealm { realm } => {
                result
                    .realm
                    .replace(realm.load(schema, function).nullable(), function);
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn { result: completion } => {
                self.emit_throw_runtime_error(
                    NativeErrorKind::TypeError,
                    RuntimeErrorMessage::CANNOT_GET_FUNCTION_REALM_FROM_A_REVOKED_PROXY,
                    completion,
                    function,
                )?;
                completion.emit(function);
                function.instruction(&Instruction::Return);
            }
            FunctionRealmRevokedRoute::ThrowTypeErrorAndBranch {
                result: completion,
                target,
            } => {
                self.emit_throw_runtime_error(
                    NativeErrorKind::TypeError,
                    RuntimeErrorMessage::CANNOT_GET_FUNCTION_REALM_FROM_A_REVOKED_PROXY,
                    completion,
                    function,
                )?;
                self.emit_branch_to_target(target, function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.outcome.load(function);
        function.instruction(&Instruction::I32Const(
            FunctionRealmOutcome::Invalid.runtime_code(),
        ));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let realm = schema.reserve_gc_local(function).initialize(
            result
                .realm
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        schema.release_i32_local(result.outcome, function);
        result.realm.clear(function);
        Ok(ResolvedFunctionRealmLocal(realm))
    }

    pub(crate) fn release_resolved_function_realm_local(
        &self,
        realm: ResolvedFunctionRealmLocal,
        function: &mut Function,
    ) {
        realm.0.clear(function);
    }
}
