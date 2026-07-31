//! `iterator` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_iterator_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let prototype_object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let setter_payload_local = self.reserve_temp_local();
        let setter_tag_local = self.reserve_temp_local();
        let to_array_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeToArray.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.toArray`",
                )
            })?;
        let for_each_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeForEach.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.forEach`",
                )
            })?;
        let every_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeEvery.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.every`",
                )
            })?;
        let some_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeSome.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.some`",
                )
            })?;
        let find_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeFind.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.find`",
                )
            })?;
        let reduce_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeReduce.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.reduce`",
                )
            })?;
        let map_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeMap.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.map`",
                )
            })?;
        let filter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeFilter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.filter`",
                )
            })?;
        let flat_map_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeFlatMap.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.flatMap`",
                )
            })?;
        let take_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeTake.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.take`",
                )
            })?;
        let drop_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeDrop.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.drop`",
                )
            })?;
        let constructor_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeConstructorGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get %IteratorPrototype%.constructor`",
                )
            })?;
        let constructor_setter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeConstructorSetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `set %IteratorPrototype%.constructor`",
                )
            })?;
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.from`",
                )
            })?;
        let concat_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorConcat.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.concat`",
                )
            })?;
        let zip_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorZip.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.zip`",
                )
            })?;
        let zip_keyed_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorZipKeyed.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.zipKeyed`",
                )
            })?;
        let wrapper_return_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorFromWrapperReturn.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%WrapForValidIteratorPrototype%.return`",
                )
            })?;
        let wrapper_next_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorFromWrapperNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%WrapForValidIteratorPrototype%.next`",
                )
            })?;
        let helper_next_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorHelperNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorHelperPrototype%.next`",
                )
            })?;
        let helper_return_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorHelperReturn.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorHelperPrototype%.return`",
                )
            })?;
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorPrototype%[Symbol.iterator]`",
                )
            })?;
        let dispose_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeSymbolDispose.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorPrototype%[Symbol.dispose]`",
                )
            })?;
        let to_string_tag_getter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeToStringTagGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get %IteratorPrototype%[Symbol.toStringTag]`",
                )
            })?;
        let to_string_tag_setter_meta = self
            .functions
            .get(&StandardBuiltinId::IteratorPrototypeToStringTagSetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `set %IteratorPrototype%[Symbol.toStringTag]`",
                )
            })?;

        self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
        self.emit_object_define_function_data(object_local, "concat", concat_meta, function)?;
        self.emit_object_define_function_data(object_local, "zip", zip_meta, function)?;
        self.emit_object_define_function_data(object_local, "zipKeyed", zip_keyed_meta, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;

        function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "toArray",
            to_array_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "forEach",
            for_each_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "every",
            every_meta,
            function,
        )?;
        self.emit_object_define_function_data(prototype_object_local, "some", some_meta, function)?;
        self.emit_object_define_function_data(prototype_object_local, "find", find_meta, function)?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "reduce",
            reduce_meta,
            function,
        )?;
        self.emit_object_define_function_data(prototype_object_local, "map", map_meta, function)?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "filter",
            filter_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "flatMap",
            flat_map_meta,
            function,
        )?;
        self.emit_object_define_function_data(prototype_object_local, "take", take_meta, function)?;
        self.emit_object_define_function_data(prototype_object_local, "drop", drop_meta, function)?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(constructor_getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_function_value_payload(constructor_setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_object_local,
            key_local,
            Some((payload_local, tag_local)),
            Some((setter_payload_local, setter_tag_local)),
            false,
            true,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "Symbol.iterator",
            iterator_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "Symbol.dispose",
            dispose_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(to_string_tag_getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_function_value_payload(to_string_tag_setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(setter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_object_local,
            key_local,
            Some((payload_local, tag_local)),
            Some((setter_payload_local, setter_tag_local)),
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "next",
            wrapper_next_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "return",
            wrapper_return_meta,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "next",
            helper_next_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            prototype_object_local,
            "return",
            helper_return_meta,
            function,
        )?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Iterator Helper"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(setter_tag_local);
        self.release_temp_local(setter_payload_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_object_local);

        Ok(())
    }
}
