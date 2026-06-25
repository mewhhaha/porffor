use super::super::*;

#[derive(Debug, Clone, Copy)]
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
                        function_targets: BTreeSet::new(),
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
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::ObjectLiteral(properties),
                )
            }
        }
    }

    pub(crate) fn compile_json_static_reviver_to_locals(
        &mut self,
        value: &JsonStaticValueIr,
        reviver: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reviver_payload_local = self.reserve_temp_local();
        let reviver_tag_local = self.reserve_temp_local();
        let root_payload_local = self.reserve_temp_local();
        let root_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let empty_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(reviver, reviver_payload_local, reviver_tag_local, function)?;
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
            JsonStaticPropertyKey::String(""),
            true,
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
        Ok(())
    }

    fn emit_json_static_internalize_property(
        &mut self,
        value: &JsonStaticValueIr,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key: JsonStaticPropertyKey<'_>,
        is_root_property: bool,
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
                function.instruction(&Instruction::I64Const(index as i64));
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
                        JsonStaticPropertyKey::ArrayIndex(index),
                        false,
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
                        JsonStaticPropertyKey::String(property_key),
                        false,
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
            is_root_property,
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.keys`",
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

        self.emit_json_apply_reviver_with_source(
            None,
            holder_payload_local,
            holder_tag_local,
            array_index_local,
            false,
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
        key: JsonStaticPropertyKey<'_>,
        is_root_property: bool,
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
                function.instruction(&Instruction::I64Const(index as i64));
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
                is_root_property,
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
                is_root_property,
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
                is_root_property,
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
        is_root_property: bool,
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
        self.set_completion_kind(CompletionKind::Normal, function);

        if is_root_property {
            self.release_temp_local(source_tag_local);
            self.release_temp_local(source_payload_local);
            self.release_temp_local(source_key_local);
            self.release_temp_local(context_tag_local);
            self.release_temp_local(context_payload_local);
            return Ok(());
        }

        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if let Some(array_index_local) = array_index_local {
            function.instruction(&Instruction::LocalGet(holder_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_delete(
                holder_payload_local,
                array_index_local,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::Else);
            self.emit_object_delete(
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                self.scratch_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            self.emit_object_delete(
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                self.scratch_local,
                function,
            )?;
        }
        function.instruction(&Instruction::Else);
        if let Some(array_index_local) = array_index_local {
            function.instruction(&Instruction::LocalGet(holder_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_create_data_property_silent(
                holder_payload_local,
                array_index_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_json_create_data_property(
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            self.emit_json_create_data_property(
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::End);

        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(context_tag_local);
        self.release_temp_local(context_payload_local);
        Ok(())
    }

    fn emit_json_create_data_property(
        &mut self,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let boxed_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(holder_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            holder_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_create_data_property_or_throw(
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            None,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_object_create_data_property_silent(
            holder_payload_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_create_data_property_or_throw(
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(boxed_kind_local);
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
        key: JsonStaticPropertyKey<'_>,
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

    pub(crate) fn emit_json_quote_string_payload(
        &mut self,
        string_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let dst_len_capacity_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let actual_len_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_len_capacity_local));
        function.instruction(&Instruction::LocalGet(dst_len_capacity_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(dst_len_capacity_local));
        self.emit_heap_alloc_from_local(dst_len_capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        Self::emit_store_u8_const_and_advance(b'"', dst_pos_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        self.emit_string_bytes_match_at_index_delta_i32(
            src_offset_local,
            src_len_local,
            index_local,
            0,
            b"\\uD834",
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_bytes_match_at_index_delta_i32(
            src_offset_local,
            src_len_local,
            index_local,
            6,
            b"\\uDF06",
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        for byte in [0xF0, 0x9D, 0x8C, 0x86] {
            Self::emit_store_u8_const_and_advance(byte, dst_pos_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        for byte in b"\\ud834" {
            Self::emit_store_u8_const_and_advance(*byte, dst_pos_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_string_bytes_match_at_index_delta_i32(
            src_offset_local,
            src_len_local,
            index_local,
            0,
            b"\\uDF06",
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        for byte in b"\\udf06" {
            Self::emit_store_u8_const_and_advance(*byte, dst_pos_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_local_and_advance(byte_local, dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'b', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b't', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'n', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'f', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'r', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'u', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'0', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'0', dst_pos_local, function);
        self.emit_store_json_lower_hex_digit_from_byte(
            byte_local,
            digit_local,
            4,
            dst_pos_local,
            function,
        );
        self.emit_store_json_lower_hex_digit_from_byte(
            byte_local,
            digit_local,
            0,
            dst_pos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        Self::emit_store_u8_local_and_advance(byte_local, dst_pos_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Self::emit_store_u8_const_and_advance(b'"', dst_pos_local, function);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_len_local));
        self.emit_pack_string_payload(dst_offset_local, actual_len_local, function);
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(digit_local);
        self.release_temp_local(actual_len_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(dst_len_capacity_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_store_u8_const_and_advance(
        byte: u8,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(byte as i32));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_store_u8_local_and_advance(
        byte_local: u32,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_string_bytes_match_at_index_delta_i32(
        &self,
        src_offset_local: u32,
        src_len_local: u32,
        index_local: u32,
        delta: i64,
        bytes: &[u8],
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(delta + bytes.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LeU);
        for (offset, byte) in bytes.iter().copied().enumerate() {
            function.instruction(&Instruction::LocalGet(src_offset_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I64Const(delta + offset as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Const(byte as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
        }
    }

    pub(crate) fn emit_store_json_lower_hex_digit_from_byte(
        &self,
        byte_local: u32,
        digit_local: u32,
        shift: i64,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_json_apply_replacer_with_this(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_indirect_call_from_locals(
            replacer_payload_local,
            replacer_tag_local,
            Some((this_payload_local, this_tag_local)),
            &[
                (key_payload_local, key_tag_local),
                (value_payload_local, value_tag_local),
            ],
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_omits_value_i32(&self, value_tag_local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(crate) fn emit_json_array_element_string_payload(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        value_string_local: u32,
        indent_level: u8,
        depth: u8,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_omits_value_i32(value_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(value_string_local));
        function.instruction(&Instruction::Else);
        self.emit_json_stringify_value_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level,
            depth,
            ancestor_payload_locals,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_property_key_payload_is_symbol_i32(
        &self,
        key_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol()")));
        function.instruction(&Instruction::I64Eq);
    }

    pub(crate) fn emit_json_apply_to_json(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toJSON")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        self.load_i64_to_local_from_offset(
            prototype_payload_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));
        self.emit_object_read(
            prototype_payload_local,
            method_tag_local,
            value_payload_local,
            value_tag_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_read(
            value_payload_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((value_payload_local, Some(value_tag_local))),
            &[(key_payload_local, key_tag_local)],
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(property_key_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_boxed_object_to_primitive_payload(
        &mut self,
        object_payload_local: u32,
        hint: ToPrimitiveHint,
        output_payload_local: u32,
        output_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let hook_names: &[&str] = match hint {
            ToPrimitiveHint::String => &["Symbol.toPrimitive", "toString", "valueOf"],
            ToPrimitiveHint::Default | ToPrimitiveHint::Number => {
                &["Symbol.toPrimitive", "valueOf", "toString"]
            }
        };
        let object_tag_local = self.reserve_temp_local();
        let hook_value_payload = self.reserve_temp_local();
        let hook_value_tag = self.reserve_temp_local();
        let call_result_payload = self.reserve_temp_local();
        let call_result_tag = self.reserve_temp_local();
        let primitive_result_local = self.reserve_temp_local();
        let call_attempted_local = self.reserve_temp_local();
        let own_present_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(primitive_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(call_attempted_local));

        for hook_name in hook_names {
            let key_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(primitive_result_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(hook_name)));
            function.instruction(&Instruction::LocalSet(key_local));
            if hint == ToPrimitiveHint::String && *hook_name != "Symbol.toPrimitive" {
                self.emit_object_own_data_field_read(
                    object_payload_local,
                    object_tag_local,
                    key_local,
                    own_present_local,
                    hook_value_payload,
                    hook_value_tag,
                    function,
                );
            } else {
                self.emit_object_read(
                    object_payload_local,
                    object_tag_local,
                    object_payload_local,
                    object_tag_local,
                    key_local,
                    hook_value_payload,
                    hook_value_tag,
                    function,
                )?;
            }
            if *hook_name == "Symbol.toPrimitive" {
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(call_attempted_local));
                self.emit_function_handle_call(
                    hook_value_payload,
                    hook_value_tag,
                    Some((object_payload_local, None)),
                    &[],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(output_payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(output_tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert object to primitive value",
                    output_payload_local,
                    output_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    "Cannot convert object to primitive value",
                    output_payload_local,
                    output_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(call_attempted_local));
                self.emit_function_handle_call(
                    hook_value_payload,
                    hook_value_tag,
                    Some((object_payload_local, None)),
                    &[],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(output_payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(output_tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            self.release_temp_local(key_local);
        }

        function.instruction(&Instruction::LocalGet(primitive_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(call_attempted_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            output_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            output_payload_local,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Cannot convert object to primitive value",
            output_payload_local,
            output_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(own_present_local);
        self.release_temp_local(call_attempted_local);
        self.release_temp_local(primitive_result_local);
        self.release_temp_local(call_result_tag);
        self.release_temp_local(call_result_payload);
        self.release_temp_local(hook_value_tag);
        self.release_temp_local(hook_value_payload);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    pub(crate) fn emit_json_gap_is_non_empty_i32(
        &self,
        gap_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(gap_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
    }

    pub(crate) fn emit_json_replacer_array_preflight(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_payload_local = self.reserve_temp_local();
        let len_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            replacer_payload_local,
            replacer_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            len_payload_local,
            len_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_value_to_number_payload(len_tag_local, len_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(len_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(len_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(len_local));

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
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get(
            replacer_payload_local,
            index_local,
            replacer_payload_local,
            replacer_tag_local,
            value_payload_local,
            value_tag_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            replacer_payload_local,
            replacer_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(len_tag_local);
        self.release_temp_local(len_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_indent_payload(
        &mut self,
        gap_payload_local: u32,
        indent_level: u8,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        for _ in 0..indent_level {
            self.emit_concat_string_payloads_local(output_local, gap_payload_local, function)?;
            function.instruction(&Instruction::LocalSet(output_local));
        }
        Ok(())
    }

    pub(crate) fn emit_json_append_newline_indent(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        indent_level: u8,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let token_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("\n")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_indent_payload(gap_payload_local, indent_level, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(token_local);
        Ok(())
    }

    pub(crate) fn emit_json_append_optional_newline_indent(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        indent_level: u8,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_gap_is_non_empty_i32(gap_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_append_colon(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_gap_is_non_empty_i32(gap_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(": ")));
        function.instruction(&Instruction::LocalSet(token_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(":")));
        function.instruction(&Instruction::LocalSet(token_local));
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        Ok(())
    }

    pub(crate) fn emit_json_throw_if_same_container(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        container_payload_local: u32,
        container_kind: ValueKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(container_kind.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalGet(container_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Converting circular structure to JSON",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_throw_if_in_container_stack(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        container_payload_local: u32,
        container_kind: ValueKind,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_throw_if_same_container(
            value_payload_local,
            value_tag_local,
            container_payload_local,
            container_kind,
            function,
        )?;
        for ancestor_payload_local in ancestor_payload_locals {
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::LocalGet(value_payload_local));
            function.instruction(&Instruction::LocalGet(*ancestor_payload_local));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                "Converting circular structure to JSON",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.emit_return_current_completion(function);
            function.instruction(&Instruction::End);
        }
        Ok(())
    }

    pub(crate) fn emit_json_replacer_allows_key(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        key_payload_local: u32,
        allowed_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_payload_local = self.reserve_temp_local();
        let list_tag_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_method_payload_local = self.reserve_temp_local();
        let element_method_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::LocalGet(replacer_payload_local));
        function.instruction(&Instruction::LocalSet(list_payload_local));
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::LocalSet(list_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_array_local));

        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
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
            replacer_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            boxed_kind_local,
            handler_tag_local,
            boxed_kind_local,
            handler_tag_local,
            property_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(list_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(list_tag_local));
        function.instruction(&Instruction::End);

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
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(list_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            list_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            list_payload_local,
            list_tag_local,
            list_payload_local,
            list_tag_local,
            property_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get(
            list_payload_local,
            index_local,
            list_payload_local,
            list_tag_local,
            element_payload_local,
            element_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            list_payload_local,
            list_tag_local,
            list_payload_local,
            list_tag_local,
            property_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(element_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            property_key_local,
            element_method_payload_local,
            element_method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            element_method_payload_local,
            element_method_tag_local,
            Some((element_payload_local, Some(element_tag_local))),
            &[],
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(element_payload_local, element_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(element_payload_local, key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(element_method_tag_local);
        self.release_temp_local(element_method_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(property_key_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(list_tag_local);
        self.release_temp_local(list_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_value_payload(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level: u8,
        depth: u8,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let proxy_array_payload_local = self.reserve_temp_local();
        let proxy_array_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_is_array_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_RAW_JSON as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("rawJSON")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            output_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BIGINT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Do not know how to serialize a BigInt",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_boxed_object_to_primitive_payload(
            value_payload_local,
            ToPrimitiveHint::Number,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_boxed_object_to_primitive_payload(
            value_payload_local,
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_primitive_to_string_payload(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BOOLEAN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
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

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_array_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_array_tag_local));
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
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            boxed_kind_local,
            proxy_handler_tag_local,
            boxed_kind_local,
            proxy_handler_tag_local,
            key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_array_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_array_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_is_array_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_target_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
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
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        if depth == 0 {
            function.instruction(&Instruction::I64Const(self.strings.payload("[]")));
            function.instruction(&Instruction::LocalSet(output_local));
        } else {
            self.emit_json_stringify_proxy_array_payload(
                proxy_array_payload_local,
                proxy_array_tag_local,
                replacer_payload_local,
                replacer_tag_local,
                gap_payload_local,
                output_local,
                indent_level,
                depth - 1,
                ancestor_payload_locals,
                function,
            )?;
        }
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        if depth == 0 {
            function.instruction(&Instruction::I64Const(self.strings.payload("{}")));
            function.instruction(&Instruction::LocalSet(output_local));
        } else {
            self.emit_json_stringify_object_payload(
                value_payload_local,
                replacer_payload_local,
                replacer_tag_local,
                gap_payload_local,
                output_local,
                indent_level,
                depth - 1,
                ancestor_payload_locals,
                function,
            )?;
        }
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        if depth == 0 {
            function.instruction(&Instruction::I64Const(self.strings.payload("[]")));
            function.instruction(&Instruction::LocalSet(output_local));
        } else {
            self.emit_json_stringify_array_payload(
                value_payload_local,
                replacer_payload_local,
                replacer_tag_local,
                gap_payload_local,
                output_local,
                indent_level,
                depth - 1,
                ancestor_payload_locals,
                function,
            )?;
        }
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_quote_string_payload(value_payload_local, output_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_number_to_string_payload(value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Do not know how to serialize a BigInt",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_is_array_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(proxy_array_tag_local);
        self.release_temp_local(proxy_array_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(brand_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_array_payload(
        &mut self,
        array_payload_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level: u8,
        depth: u8,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        let mut nested_ancestor_payload_locals = ancestor_payload_locals.to_vec();
        nested_ancestor_payload_locals.push(array_payload_local);

        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(array_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_index_get(
            array_payload_local,
            index_local,
            array_payload_local,
            array_tag_local,
            value_payload_local,
            value_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_container_stack(
            value_payload_local,
            value_tag_local,
            array_payload_local,
            ValueKind::Array,
            ancestor_payload_locals,
            function,
        )?;
        self.emit_json_array_element_string_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level.saturating_add(1),
            depth,
            &nested_ancestor_payload_locals,
            function,
        )?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(array_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(token_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_proxy_array_payload(
        &mut self,
        array_payload_local: u32,
        array_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level: u8,
        depth: u8,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let mut nested_ancestor_payload_locals = ancestor_payload_locals.to_vec();
        nested_ancestor_payload_locals.push(array_payload_local);

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            array_payload_local,
            array_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(length_tag_local, length_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(length_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_object_read(
            array_payload_local,
            array_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_container_stack(
            value_payload_local,
            value_tag_local,
            array_payload_local,
            ValueKind::Array,
            ancestor_payload_locals,
            function,
        )?;
        self.emit_json_array_element_string_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level.saturating_add(1),
            depth,
            &nested_ancestor_payload_locals,
            function,
        )?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(token_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_object_payload(
        &mut self,
        object_payload_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level: u8,
        depth: u8,
        ancestor_payload_locals: &[u32],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let first_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let keys_function_payload_local = self.reserve_temp_local();
        let keys_function_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_arg_payload_local = self.reserve_temp_local();
        let keys_arg_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let key_allowed_local = self.reserve_temp_local();
        let completed_local = self.reserve_temp_local();
        let duplicate_key_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let previous_key_payload_local = self.reserve_temp_local();
        let previous_key_tag_local = self.reserve_temp_local();
        let mut nested_ancestor_payload_locals = ancestor_payload_locals.to_vec();
        nested_ancestor_payload_locals.push(object_payload_local);

        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(completed_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("ownKeys")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            boxed_kind_local,
            proxy_handler_tag_local,
            boxed_kind_local,
            proxy_handler_tag_local,
            key_payload_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        self.emit_function_value_payload(&keys_meta, function)?;
        function.instruction(&Instruction::LocalSet(keys_function_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(keys_function_tag_local));
        self.emit_function_handle_call(
            keys_function_payload_local,
            keys_function_tag_local,
            None,
            &[(keys_arg_payload_local, keys_arg_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
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
        self.emit_json_replacer_allows_key(
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_allowed_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            keys_arg_payload_local,
            keys_arg_tag_local,
            keys_arg_payload_local,
            keys_arg_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            keys_arg_payload_local,
            keys_arg_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_container_stack(
            value_payload_local,
            value_tag_local,
            keys_arg_payload_local,
            ValueKind::Object,
            ancestor_payload_locals,
            function,
        )?;
        self.emit_json_omits_value_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level.saturating_add(1),
            depth,
            &nested_ancestor_payload_locals,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_index_get(
            replacer_payload_local,
            index_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            key_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_object_own_data_field_read(
            key_payload_local,
            key_tag_local,
            key_string_local,
            key_allowed_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((key_payload_local, Some(key_tag_local))),
            &[],
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            key_payload_local,
            key_tag_local,
            key_payload_local,
            key_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((key_payload_local, Some(key_tag_local))),
            &[],
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_index_get(
            replacer_payload_local,
            previous_index_local,
            replacer_payload_local,
            replacer_tag_local,
            previous_key_payload_local,
            previous_key_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(previous_key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(previous_key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            previous_key_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_object_own_data_field_read(
            previous_key_payload_local,
            previous_key_tag_local,
            key_string_local,
            key_allowed_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((previous_key_payload_local, Some(previous_key_tag_local))),
            &[],
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            previous_key_payload_local,
            previous_key_tag_local,
            previous_key_payload_local,
            previous_key_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((previous_key_payload_local, Some(previous_key_tag_local))),
            &[],
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(previous_key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            previous_key_payload_local,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(duplicate_key_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_container_stack(
            value_payload_local,
            value_tag_local,
            object_payload_local,
            ValueKind::Object,
            ancestor_payload_locals,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level.saturating_add(1),
            depth,
            &nested_ancestor_payload_locals,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(completed_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_json_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_replacer_allows_key(
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_allowed_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_container_stack(
            value_payload_local,
            value_tag_local,
            object_payload_local,
            ValueKind::Object,
            ancestor_payload_locals,
            function,
        )?;
        self.emit_json_omits_value_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level.saturating_add(1),
            depth,
            &nested_ancestor_payload_locals,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level.saturating_add(1),
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(previous_key_tag_local);
        self.release_temp_local(previous_key_payload_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(duplicate_key_local);
        self.release_temp_local(completed_local);
        self.release_temp_local(key_allowed_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(keys_arg_tag_local);
        self.release_temp_local(keys_arg_payload_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(keys_function_tag_local);
        self.release_temp_local(keys_function_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(token_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(first_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_raw_json_text(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let last_byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let compare_payload_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, first_byte_local, function);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, last_byte_local, function);
        for byte in [b'\t', b'\n', b'\r', b' '] {
            self.emit_or_byte_equals_flag(first_byte_local, byte, invalid_local, function);
            self.emit_or_byte_equals_flag(last_byte_local, byte, invalid_local, function);
        }
        for byte in [b'{', b'['] {
            self.emit_or_byte_equals_flag(first_byte_local, byte, invalid_local, function);
        }
        function.instruction(&Instruction::End);

        for text in [
            "undefined",
            "[object Object]",
            "NaN",
            "Infinity",
            "-Infinity",
        ] {
            function.instruction(&Instruction::LocalGet(invalid_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(text)));
            function.instruction(&Instruction::LocalSet(compare_payload_local));
            self.emit_string_payload_equality_i32(
                string_payload_local,
                compare_payload_local,
                function,
            );
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(invalid_local));
        }

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.rawJSON text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(compare_payload_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(last_byte_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_number_text(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_non_number_start_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'1' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_no_raw_string_controls(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let in_string_local = self.reserve_temp_local();
        let escaped_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_escape_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(escaped_local);
        self.release_temp_local(in_string_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_no_structural_trailing_commas(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let in_string_local = self.reserve_temp_local();
        let escaped_local = self.reserve_temp_local();
        let previous_significant_local = self.reserve_temp_local();
        let structural_depth_local = self.reserve_temp_local();
        let structural_stack_local = self.reserve_temp_local();
        let structural_mask_local = self.reserve_temp_local();
        let structured_seen_local = self.reserve_temp_local();
        let structured_closed_local = self.reserve_temp_local();
        let object_key_needs_colon_local = self.reserve_temp_local();
        let keyword_byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structured_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_escape_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::Else);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_structural_or_value_start_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(structured_seen_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::End);
        self.emit_increment_local(structural_depth_local, 1, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(structural_depth_local));
        function.instruction(&Instruction::LocalGet(structured_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(structured_closed_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'r' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 3, function);
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b's' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 4, function);
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'n' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 3, function);
        function.instruction(&Instruction::I64Const(b'n' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        self.emit_byte_is_digit_i32(keyword_byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(keyword_byte_local);
        self.release_temp_local(object_key_needs_colon_local);
        self.release_temp_local(structured_closed_local);
        self.release_temp_local(structured_seen_local);
        self.release_temp_local(structural_mask_local);
        self.release_temp_local(structural_stack_local);
        self.release_temp_local(structural_depth_local);
        self.release_temp_local(previous_significant_local);
        self.release_temp_local(escaped_local);
        self.release_temp_local(in_string_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_try_parse_json_string_text(
        &mut self,
        string_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        parsed_flag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let decoded_len_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let first_hex_local = self.reserve_temp_local();
        let second_hex_local = self.reserve_temp_local();
        let third_hex_local = self.reserve_temp_local();
        let fourth_hex_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        self.emit_increment_local(index_local, 1, function);
        self.emit_heap_alloc_from_local(string_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        for (escape, value) in [
            (b'"', b'"'),
            (b'\\', b'\\'),
            (b'/', b'/'),
            (b'b', 0x08),
            (b'f', 0x0c),
            (b'n', b'\n'),
            (b'r', b'\r'),
            (b't', b'\t'),
        ] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(escape as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(value as i64));
            function.instruction(&Instruction::LocalSet(codepoint_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            first_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(first_hex_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            second_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(second_hex_local, second_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            third_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(third_hex_local, third_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            fourth_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(fourth_hex_local, fourth_hex_local, function);
        self.emit_all_hex_valid_i32(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pack_four_hex_to_code_unit(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            codepoint_local,
            function,
        );
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        self.emit_increment_local(index_local, 4, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_byte_local(dst_pos_local, codepoint_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_store_byte_local(dst_pos_local, byte_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decoded_len_local));
        self.emit_pack_string_payload(dst_offset_local, decoded_len_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(fourth_hex_local);
        self.release_temp_local(third_hex_local);
        self.release_temp_local(second_hex_local);
        self.release_temp_local(first_hex_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(decoded_len_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_try_parse_json_keyword_text(
        &mut self,
        string_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        parsed_flag_local: u32,
        function: &mut Function,
    ) {
        let compare_payload_local = self.reserve_temp_local();

        for (text, payload, tag) in [
            ("null", 0, ValueKind::Null),
            ("false", 0, ValueKind::Boolean),
            ("true", 1, ValueKind::Boolean),
        ] {
            function.instruction(&Instruction::LocalGet(parsed_flag_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(text)));
            function.instruction(&Instruction::LocalSet(compare_payload_local));
            self.emit_string_payload_equality_i32(
                string_payload_local,
                compare_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(payload));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(tag.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(parsed_flag_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(compare_payload_local);
    }
}
