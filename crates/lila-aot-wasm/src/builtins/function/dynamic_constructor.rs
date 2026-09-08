use super::*;
use crate::functions::FunctionRealmRevokedRoute;
use lila_ir::{DynamicSourceRuntimeOperation, GeneratorPlanIr, ResumablePlanIr};

#[derive(Clone, Copy)]
enum EmptyDerivedFunction {
    Generator,
    Async,
    AsyncGenerator,
}

impl EmptyDerivedFunction {
    const ALL: [Self; 3] = [Self::Generator, Self::Async, Self::AsyncGenerator];

    const fn id(self) -> &'static str {
        match self {
            Self::Generator => "lila:empty:generator",
            Self::Async => "lila:empty:async",
            Self::AsyncGenerator => "lila:empty:async-generator",
        }
    }

    const fn protocol(self) -> FunctionProtocolIr {
        match self {
            Self::Generator => FunctionProtocolIr::Generator,
            Self::Async => FunctionProtocolIr::Async,
            Self::AsyncGenerator => FunctionProtocolIr::AsyncGenerator,
        }
    }

    const fn source(self) -> &'static str {
        match self {
            Self::Generator => "function* anonymous(\n) {\n\n}",
            Self::Async => "async function anonymous(\n) {\n\n}",
            Self::AsyncGenerator => "async function* anonymous(\n) {\n\n}",
        }
    }

    const fn constructor_global(self) -> u32 {
        match self {
            Self::Generator => GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
            Self::Async => ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
            Self::AsyncGenerator => ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
        }
    }

    const fn function_prototype_offset(self) -> u64 {
        match self {
            Self::Generator => HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
            Self::Async => HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET,
            Self::AsyncGenerator => HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
        }
    }

    const fn instance_prototype_offset(self) -> Option<u64> {
        match self {
            Self::Generator => Some(HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET),
            Self::Async => None,
            Self::AsyncGenerator => Some(HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET),
        }
    }

    fn body(self) -> FunctionIr {
        FunctionIr {
            id: self.id().to_string(),
            name: "anonymous".to_string(),
            to_string_representation: CallableToStringRepresentation::ExactSource(
                self.source().to_string(),
            ),
            protocol: self.protocol(),
            generator_plan: matches!(self, Self::Generator).then(|| GeneratorPlanIr {
                entry_state: 0,
                state_count: 1,
                suspension_points: Vec::new(),
            }),
            resumable_plan: matches!(self, Self::AsyncGenerator).then(|| ResumablePlanIr {
                entry_state: 0,
                state_count: 1,
                suspension_points: Vec::new(),
            }),
            strict: false,
            class_element_execution_kind: ClassElementExecutionKind::None,
            class_heritage_kind: ClassHeritageKind::None,
            is_static_class_member: false,
            is_derived_constructor: false,
            is_synthetic_default_derived_constructor: false,
            class_instance_element_plan: None,
            super_constructor_target: None,
            uses_super: false,
            this_before_super: false,
            lexical_derived_activation: None,
            private_name_ids: BTreeMap::new(),
            captures_private_environment: false,
            is_nested: false,
            is_expression: true,
            is_named_expression: false,
            captures_lexical_this: false,
            captures_lexical_arguments: false,
            params: Vec::new(),
            body: BlockIr {
                statements: Vec::new(),
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            return_kind: ValueKind::Object,
            return_shape: None,
            return_targets: FunctionTargetKnowledge::none(),
            constructor_instance: ValueInfo::undefined(),
            // Ordinary lowering owns this invocation state in every resumable
            // activation, including empty bodies, and assigns slots by name.
            owned_env_bindings: [
                LEXICAL_ARGUMENTS_NAME,
                LEXICAL_NEW_TARGET_NAME,
                LEXICAL_THIS_NAME,
            ]
            .into_iter()
            .zip(0_u32..)
            .map(|(name, slot)| OwnedEnvBindingIr {
                name: name.to_string(),
                slot,
            })
            .collect(),
            captured_bindings: Vec::new(),
        }
    }
}

pub(crate) fn append_empty_dynamic_function_bodies(script: &mut ScriptIr) {
    for kind in EmptyDerivedFunction::ALL {
        assert!(
            !script
                .functions
                .iter()
                .any(|function| function.id == kind.id()),
            "compiler-generated function identity `{}` is already occupied",
            kind.id(),
        );
        // Every Wasm artifact has heap-backed realm roots, and bootstrap
        // exposes all three constructors even without a direct source call.
        script.functions.push(kind.body());
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn compile_dynamic_function_constructor_builtin(
        &mut self,
        kind: DynamicFunctionKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let empty = match kind {
            DynamicFunctionKind::Ordinary => {
                return self.compile_function_constructor_builtin(function);
            }
            DynamicFunctionKind::Generator => EmptyDerivedFunction::Generator,
            DynamicFunctionKind::Async => EmptyDerivedFunction::Async,
            DynamicFunctionKind::AsyncGenerator => EmptyDerivedFunction::AsyncGenerator,
        };
        let body_meta =
            self.functions.get(empty.id()).cloned().unwrap_or_else(|| {
                panic!("missing compiler-generated empty body `{}`", empty.id())
            });
        let active_constructor_local = self.reserve_temp_local();
        let active_realm_local = self.reserve_temp_local();
        let active_intrinsics_local = self.reserve_temp_local();
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let function_prototype_tag_local = self.reserve_temp_local();
        let new_target_intrinsics_local = self.reserve_temp_local();
        let function_object_local = self.reserve_temp_local();
        let instance_prototype_local = self.reserve_temp_local();
        let instance_parent_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::GlobalGet(empty.constructor_global()));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(active_constructor_local));
        self.load_i64_to_local_from_offset(
            active_constructor_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            active_realm_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            active_realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            active_intrinsics_local,
            function,
        );
        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(active_constructor_local));
        function.instruction(&Instruction::LocalSet(new_target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(new_target_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            prototype_key_local,
            function_prototype_local,
            function_prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            function_prototype_local,
            function_prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(function_prototype_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        let realm_result =
            self.emit_get_function_realm(new_target_payload_local, new_target_tag_local, function);
        let prototype_realm = self.emit_route_function_realm_result(
            realm_result,
            FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                payload_local: self.result_local,
                tag_local: self.result_tag_local,
            },
            function,
        )?;
        self.load_i64_to_local_from_offset(
            prototype_realm.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            new_target_intrinsics_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            new_target_intrinsics_local,
            empty.function_prototype_offset(),
            function_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(function_prototype_tag_local));
        self.release_resolved_function_realm_local(prototype_realm);
        function.instruction(&Instruction::End);

        self.emit_function_value_payload(&body_meta, function)?;
        function.instruction(&Instruction::LocalSet(function_object_local));
        self.emit_store_function_defining_realm(
            function_object_local,
            active_realm_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            function_object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            function_prototype_tag_local,
            function,
        );
        if let Some(parent_offset) = empty.instance_prototype_offset() {
            self.load_i64_to_local_from_offset(
                function_object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                instance_prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                active_intrinsics_local,
                parent_offset,
                instance_parent_local,
                function,
            );
            self.store_i64_local_at_offset(
                instance_prototype_local,
                HEAP_PROTOTYPE_OFFSET,
                instance_parent_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(function_object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_reject_dynamic_source(DynamicSourceRuntimeOperation::Function(kind), function);
        function.instruction(&Instruction::End);

        self.release_temp_local(instance_parent_local);
        self.release_temp_local(instance_prototype_local);
        self.release_temp_local(function_object_local);
        self.release_temp_local(new_target_intrinsics_local);
        self.release_temp_local(function_prototype_tag_local);
        self.release_temp_local(function_prototype_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        self.release_temp_local(active_intrinsics_local);
        self.release_temp_local(active_realm_local);
        self.release_temp_local(active_constructor_local);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    #[test]
    fn empty_dynamic_bodies_match_the_source_lowering_execution_plans() {
        for kind in EmptyDerivedFunction::ALL {
            let parsed = parse(kind.source(), ParseOptions::script())
                .expect("canonical empty function source must parse");
            let lowered = lila_ir::lower(&parsed);
            assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
            let source_function = lowered
                .script
                .expect("script IR")
                .functions
                .into_iter()
                .find(|function| function.name == "anonymous")
                .expect("canonical function must be lowered");
            let generated = kind.body();
            assert_eq!(generated.protocol, source_function.protocol);
            assert_eq!(generated.generator_plan, source_function.generator_plan);
            assert_eq!(generated.resumable_plan, source_function.resumable_plan);
            assert_eq!(generated.body, source_function.body);
            assert_eq!(generated.return_kind, source_function.return_kind);
            assert_eq!(generated.strict, source_function.strict);
            assert_eq!(
                generated.owned_env_bindings, source_function.owned_env_bindings,
                "{:?} activation bindings must match ordinary lowering",
                generated.protocol
            );
            assert_eq!(
                generated.captured_bindings,
                source_function.captured_bindings
            );
        }
    }
}
