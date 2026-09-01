use super::*;

enum JsonStaticPropertyKey<'a> {
    String(&'a str),
    ArrayIndex(usize),
}

fn json_static_primitive_source(value: &JsonStaticValueIr) -> Option<&str> {
    match value {
        JsonStaticValueIr::Null { source }
        | JsonStaticValueIr::Boolean { source, .. }
        | JsonStaticValueIr::Number { source, .. }
        | JsonStaticValueIr::String { source, .. } => Some(source.as_str()),
        JsonStaticValueIr::Array(_) | JsonStaticValueIr::Object(_) => None,
    }
}

impl<'a> FunctionBuilder<'a> {
    fn json_static_value_to_expr(value: &JsonStaticValueIr) -> TypedExpr {
        match value {
            JsonStaticValueIr::Null { .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Null), ExprIr::Null)
            }
            JsonStaticValueIr::Boolean { value, .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(*value))
            }
            JsonStaticValueIr::Number { bits, .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(*bits))
            }
            JsonStaticValueIr::String { value, .. } => TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String(value.clone()),
            ),
            JsonStaticValueIr::Array(values) => {
                let elements = values
                    .iter()
                    .map(Self::json_static_value_to_expr)
                    .collect::<Vec<_>>();
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Array,
                        possible_kinds: KindSet::from_kind(ValueKind::Array),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::ArrayLiteral(elements),
                )
            }
            JsonStaticValueIr::Object(properties) => {
                let properties = properties
                    .iter()
                    .map(|(key, value)| ObjectPropertyIr::Data {
                        key: key.clone(),
                        value: Self::json_static_value_to_expr(value),
                        is_shorthand: false,
                    })
                    .collect::<Vec<_>>();
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Object,
                        possible_kinds: KindSet::from_kind(ValueKind::Object),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::none(),
                    },
                    ExprIr::ObjectLiteral(properties),
                )
            }
        }
    }

    pub(crate) fn compile_json_static_reviver_to_locals(
        &mut self,
        callee: &TypedExpr,
        input: &TypedExpr,
        value: &JsonStaticValueIr,
        reviver: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let discarded_payload_local = self.reserve_temp_local();
        let discarded_tag_local = self.reserve_temp_local();
        let reviver_payload_local = self.reserve_temp_local();
        let reviver_tag_local = self.reserve_temp_local();
        let root_payload_local = self.reserve_temp_local();
        let root_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let empty_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            callee,
            discarded_payload_local,
            discarded_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            discarded_payload_local,
            discarded_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(
            input,
            discarded_payload_local,
            discarded_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            discarded_payload_local,
            discarded_tag_local,
            function,
        )?;
        self.compile_expr_to_locals(reviver, reviver_payload_local, reviver_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            reviver_payload_local,
            reviver_tag_local,
            function,
        )?;
        let value_expr = Self::json_static_value_to_expr(value);
        self.compile_expr_to_locals(&value_expr, value_payload_local, value_tag_local, function)?;

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(root_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(root_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(empty_key_local));
        self.emit_object_define_enumerable_data(
            root_payload_local,
            empty_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.emit_json_static_internalize_property(
            value,
            root_payload_local,
            root_tag_local,
            &JsonStaticPropertyKey::String(""),
            JsonReviverPropertyRole::Root,
            reviver_payload_local,
            reviver_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(empty_key_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(root_tag_local);
        self.release_temp_local(root_payload_local);
        self.release_temp_local(reviver_tag_local);
        self.release_temp_local(reviver_payload_local);
        self.release_temp_local(discarded_tag_local);
        self.release_temp_local(discarded_payload_local);
        Ok(())
    }

    fn emit_json_static_internalize_property(
        &mut self,
        value: &JsonStaticValueIr,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key: &JsonStaticPropertyKey<'_>,
        role: JsonReviverPropertyRole,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.emit_json_static_key_payload(key, key_payload_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));

        match key {
            JsonStaticPropertyKey::ArrayIndex(index) => {
                let index_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(*index as i64));
                function.instruction(&Instruction::LocalSet(index_local));
                self.emit_array_index_get_with_prototype(
                    holder_payload_local,
                    index_local,
                    holder_payload_local,
                    holder_tag_local,
                    current_payload_local,
                    current_tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            JsonStaticPropertyKey::String(_) => {
                self.emit_object_read(
                    holder_payload_local,
                    holder_tag_local,
                    holder_payload_local,
                    holder_tag_local,
                    key_payload_local,
                    current_payload_local,
                    current_tag_local,
                    function,
                )?;
            }
        }
        self.emit_propagate_throw_from_locals_if_needed(
            current_payload_local,
            current_tag_local,
            function,
        )?;

        match value {
            JsonStaticValueIr::Array(values) => {
                function.instruction(&Instruction::LocalGet(current_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for (index, element) in values.iter().enumerate() {
                    self.emit_json_static_internalize_property(
                        element,
                        current_payload_local,
                        current_tag_local,
                        &JsonStaticPropertyKey::ArrayIndex(index),
                        JsonReviverPropertyRole::Nested,
                        reviver_payload_local,
                        reviver_tag_local,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                }
                if values.is_empty() {
                    self.emit_json_static_maybe_internalize_dynamic_value(
                        current_payload_local,
                        current_tag_local,
                        reviver_payload_local,
                        reviver_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::Else);
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            JsonStaticValueIr::Object(properties) => {
                function.instruction(&Instruction::LocalGet(current_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for property_index in Self::json_static_object_property_order(properties) {
                    let (property_key, property_value) = &properties[property_index];
                    self.emit_json_static_internalize_property(
                        property_value,
                        current_payload_local,
                        current_tag_local,
                        &JsonStaticPropertyKey::String(property_key),
                        JsonReviverPropertyRole::Nested,
                        reviver_payload_local,
                        reviver_tag_local,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                }
                if properties.is_empty() {
                    self.emit_json_static_maybe_internalize_dynamic_value(
                        current_payload_local,
                        current_tag_local,
                        reviver_payload_local,
                        reviver_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::Else);
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            JsonStaticValueIr::Null { .. }
            | JsonStaticValueIr::Boolean { .. }
            | JsonStaticValueIr::Number { .. }
            | JsonStaticValueIr::String { .. } => {
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
            }
        }

        self.emit_json_static_apply_reviver(
            value,
            holder_payload_local,
            holder_tag_local,
            key,
            &role,
            key_payload_local,
            key_tag_local,
            current_payload_local,
            current_tag_local,
            reviver_payload_local,
            reviver_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    fn emit_json_static_maybe_internalize_dynamic_value(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let is_array_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_arg_payload_local = self.reserve_temp_local();
        let keys_arg_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

        self.emit_json_static_is_array_like(
            value_payload_local,
            value_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(element_tag_local, element_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_json_static_internalize_dynamic_property(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            Some(index_local),
            reviver_payload_local,
            reviver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("ownKeys")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            self.scratch_local,
            proxy_handler_tag_local,
            self.scratch_local,
            proxy_handler_tag_local,
            key_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        self.emit_direct_js_call(
            &keys_meta,
            None,
            &[(keys_arg_payload_local, keys_arg_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            keys_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_static_internalize_dynamic_property(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            None,
            reviver_payload_local,
            reviver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(keys_arg_tag_local);
        self.release_temp_local(keys_arg_payload_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(is_array_local);
        Ok(())
    }

    fn emit_json_static_internalize_dynamic_property(
        &mut self,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        array_index_local: Option<u32>,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        if let Some(array_index_local) = array_index_local {
            function.instruction(&Instruction::LocalGet(holder_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_index_get_with_prototype(
                holder_payload_local,
                array_index_local,
                holder_payload_local,
                holder_tag_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                holder_payload_local,
                holder_tag_local,
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            self.emit_object_read(
                holder_payload_local,
                holder_tag_local,
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        self.emit_propagate_current_completion_if_throw(function);

        self.emit_json_apply_reviver_with_source(
            None,
            holder_payload_local,
            holder_tag_local,
            array_index_local,
            &JsonReviverPropertyRole::Nested,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            reviver_payload_local,
            reviver_tag_local,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    fn emit_json_static_is_array_like(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        is_array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_json_static_apply_reviver(
        &mut self,
        value: &JsonStaticValueIr,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key: &JsonStaticPropertyKey<'_>,
        role: &JsonReviverPropertyRole,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_index_local = match key {
            JsonStaticPropertyKey::ArrayIndex(index) => {
                let local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(*index as i64));
                function.instruction(&Instruction::LocalSet(local));
                Some(local)
            }
            JsonStaticPropertyKey::String(_) => None,
        };

        let source = json_static_primitive_source(value);
        let result = if source.is_some() {
            self.emit_json_static_current_matches_value_i32(
                value,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_json_apply_reviver_with_source(
                source,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_json_apply_reviver_with_source(
                None,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            Ok(())
        } else {
            self.emit_json_apply_reviver_with_source(
                None,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )
        };

        if let Some(array_index_local) = array_index_local {
            self.release_temp_local(array_index_local);
        }
        result
    }

    fn emit_json_static_current_matches_value_i32(
        &mut self,
        value: &JsonStaticValueIr,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match value {
            JsonStaticValueIr::Null { .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
            }
            JsonStaticValueIr::Boolean { value, .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::I64Const(i64::from(*value)));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            JsonStaticValueIr::Number { bits, .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::I64Const(*bits as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            JsonStaticValueIr::String { value, .. } => {
                let expected_payload_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    value_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::I32And);
                self.release_temp_local(expected_payload_local);
            }
            JsonStaticValueIr::Array(_) | JsonStaticValueIr::Object(_) => {
                function.instruction(&Instruction::I32Const(0));
            }
        }
        Ok(())
    }

    fn emit_json_apply_reviver_with_source(
        &mut self,
        source: Option<&str>,
        holder_payload_local: u32,
        holder_tag_local: u32,
        array_index_local: Option<u32>,
        role: &JsonReviverPropertyRole,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_payload_local = self.reserve_temp_local();
        let context_tag_local = self.reserve_temp_local();
        let source_key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(context_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(context_tag_local));

        if let Some(source) = source {
            function.instruction(&Instruction::I64Const(self.strings.payload("source")));
            function.instruction(&Instruction::LocalSet(source_key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(source)));
            function.instruction(&Instruction::LocalSet(source_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(source_tag_local));
            self.emit_object_define_enumerable_data(
                context_payload_local,
                source_key_local,
                source_payload_local,
                source_tag_local,
                function,
            )?;
        }

        self.emit_indirect_call_from_locals(
            reviver_payload_local,
            reviver_tag_local,
            Some((holder_payload_local, holder_tag_local)),
            &[
                (key_payload_local, key_tag_local),
                (value_payload_local, value_tag_local),
                (context_payload_local, context_tag_local),
            ],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_json_apply_reviver_result(
            role,
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            array_index_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;

        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(context_tag_local);
        self.release_temp_local(context_payload_local);
        Ok(())
    }

    fn json_static_object_property_order(properties: &[(String, JsonStaticValueIr)]) -> Vec<usize> {
        let mut integer_indices = Vec::new();
        let mut string_indices = Vec::new();

        for (index, (key, _)) in properties.iter().enumerate() {
            if let Some(array_index) = Self::json_static_array_index_key(key) {
                integer_indices.push((array_index, index));
            } else {
                string_indices.push(index);
            }
        }

        integer_indices.sort_by_key(|(array_index, _)| *array_index);
        integer_indices
            .into_iter()
            .map(|(_, index)| index)
            .chain(string_indices)
            .collect()
    }

    fn json_static_array_index_key(key: &str) -> Option<u32> {
        if key.is_empty() || (key.len() > 1 && key.starts_with('0')) {
            return None;
        }
        let value = key.parse::<u32>().ok()?;
        (value != u32::MAX && value.to_string() == key).then_some(value)
    }

    fn emit_json_static_key_payload(
        &mut self,
        key: &JsonStaticPropertyKey<'_>,
        key_payload_local: u32,
        function: &mut Function,
    ) {
        match key {
            JsonStaticPropertyKey::String(key) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            }
            JsonStaticPropertyKey::ArrayIndex(index) => {
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(&index.to_string()),
                ));
            }
        }
        function.instruction(&Instruction::LocalSet(key_payload_local));
    }
}
