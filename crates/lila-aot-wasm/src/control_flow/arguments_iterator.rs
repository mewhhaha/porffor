//! Shared Arguments exotic lookup used by synchronous iterator consumers.
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_arguments_iterator_method_to_locals(
        &mut self,
        source_payload: u32,
        source_tag: u32,
        method_payload: u32,
        method_tag: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Presence is separate from value: an own undefined/null iterator must
        // still throw, and an accessor must be invoked exactly once by [[Get]].
        // Arguments shares the named-property table layout with Array.
        let key_local = self.reserve_temp_local();
        let found_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_array_named_prop_read(
            source_payload,
            key_local,
            method_payload,
            method_tag,
            Some(found_local),
            function,
        );
        function.instruction(&Instruction::LocalGet(found_local));
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let source_name = "$sync.iterator.arguments.source";
        self.push_scope();
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(
                source_name.to_string(),
                BindingStorage::Dynamic {
                    tag_local: source_tag,
                    payload_local: source_payload,
                },
            );
        let source = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Arguments,
                possible_kinds: KindSet::from_kind(ValueKind::Arguments),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::none(),
            },
            ExprIr::Identifier(source_name.to_string()),
        );
        let method = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: FunctionTargetKnowledge::unknown(),
            },
            ExprIr::PropertyRead {
                target: Box::new(source),
                key: PropertyKeyIr::StaticString("Symbol.iterator".to_string()),
            },
        );
        self.compile_expr_to_locals(&method, method_payload, method_tag, function)?;
        self.pop_scope();
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            source_payload,
            source_tag,
            source_payload,
            source_tag,
            key_local,
            method_payload,
            method_tag,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(found_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
