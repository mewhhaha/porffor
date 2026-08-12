//! `array` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_array_constructor_intrinsics(
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

        let from_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.from`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
        let from_async_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayFromAsync.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.fromAsync`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "fromAsync",
            from_async_meta,
            function,
        )?;
        let of_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.of`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "of", of_meta, function)?;
        let is_array_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIsArray.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.isArray`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isArray", is_array_meta, function)?;
        let key_local = self.reserve_temp_local();
        let getter_payload_local = self.reserve_temp_local();
        let getter_tag_local = self.reserve_temp_local();
        let species_meta = self
            .functions
            .get(&StandardBuiltinId::ArraySpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(getter_tag_local));
        self.emit_object_append_accessor_property_with_flags(
            object_local,
            key_local,
            Some((getter_payload_local, getter_tag_local)),
            None,
            false,
            true,
            function,
        )?;
        self.release_temp_local(getter_tag_local);
        self.release_temp_local(getter_payload_local);
        self.release_temp_local(key_local);
        let concat_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeConcat.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.concat`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_global_data(
            object_local,
            "toString",
            ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "concat", concat_meta, function)?;
        let join_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeJoin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.join`",
                )
        })?;
        self.emit_object_define_function_data(object_local, "join", join_meta, function)?;
        let slice_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeSlice.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.slice`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "slice", slice_meta, function)?;
        let splice_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeSplice.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.splice`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "splice", splice_meta, function)?;
        let fill_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFill.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.fill`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "fill", fill_meta, function)?;
        let sort_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeSort.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.sort`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "sort", sort_meta, function)?;
        let to_locale_string_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeToLocaleString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.toLocaleString`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "toLocaleString",
            to_locale_string_meta,
            function,
        )?;
        let flat_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFlat.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.flat`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "flat", flat_meta, function)?;
        let flat_map_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFlatMap.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.flatMap`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "flatMap", flat_map_meta, function)?;
        let at_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeAt.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.at`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "at", at_meta, function)?;
        let to_reversed_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeToReversed.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.toReversed`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "toReversed",
            to_reversed_meta,
            function,
        )?;
        let to_spliced_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeToSpliced.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.toSpliced`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "toSpliced",
            to_spliced_meta,
            function,
        )?;
        let to_sorted_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeToSorted.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.toSorted`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "toSorted", to_sorted_meta, function)?;
        let with_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeWith.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.with`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "with", with_meta, function)?;
        let reverse_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeReverse.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.reverse`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "reverse", reverse_meta, function)?;
        let copy_within_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeCopyWithin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.copyWithin`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "copyWithin",
            copy_within_meta,
            function,
        )?;
        let includes_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeIncludes.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.includes`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "includes", includes_meta, function)?;
        let index_of_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeIndexOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.indexOf`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "indexOf", index_of_meta, function)?;
        let last_index_of_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.lastIndexOf`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "lastIndexOf",
            last_index_of_meta,
            function,
        )?;
        let find_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFind.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.find`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "find", find_meta, function)?;
        let find_index_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFindIndex.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.findIndex`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "findIndex",
            find_index_meta,
            function,
        )?;
        let find_last_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFindLast.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.findLast`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "findLast", find_last_meta, function)?;
        let find_last_index_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.findLastIndex`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "findLastIndex",
            find_last_index_meta,
            function,
        )?;
        let every_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeEvery.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.every`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "every", every_meta, function)?;
        let some_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeSome.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.some`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "some", some_meta, function)?;
        let for_each_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeForEach.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.forEach`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "forEach", for_each_meta, function)?;
        let filter_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFilter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.filter`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "filter", filter_meta, function)?;
        let map_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeMap.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.map`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "map", map_meta, function)?;
        let reduce_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeReduce.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.reduce`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "reduce", reduce_meta, function)?;
        let reduce_right_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeReduceRight.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.reduceRight`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "reduceRight",
            reduce_right_meta,
            function,
        )?;
        let pop_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypePop.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.pop`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "pop", pop_meta, function)?;
        let push_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypePush.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.push`",
                )
        })?;
        self.emit_object_define_function_data(object_local, "push", push_meta, function)?;
        let shift_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeShift.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.shift`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "shift", shift_meta, function)?;
        let unshift_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeUnshift.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.unshift`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "unshift", unshift_meta, function)?;
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeKeys.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.keys`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "keys", keys_meta, function)?;
        let entries_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeEntries.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.entries`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "entries", entries_meta, function)?;
        let values_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.values`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            object_local,
            "values",
            &["Symbol.iterator"],
            values_meta,
            function,
        )?;

        let unscopables_local = self.reserve_temp_local();
        let unscopables_key_local = self.reserve_temp_local();
        let unscopables_value_local = self.reserve_temp_local();
        let unscopables_value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(unscopables_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(unscopables_value_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(unscopables_value_tag_local));
        for name in [
            "at",
            "copyWithin",
            "entries",
            "fill",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "flat",
            "flatMap",
            "includes",
            "keys",
            "toReversed",
            "toSorted",
            "toSpliced",
            "values",
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(unscopables_key_local));
            self.emit_object_append_data_property_with_flags(
                unscopables_local,
                unscopables_key_local,
                unscopables_value_local,
                unscopables_value_tag_local,
                true,
                true,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.unscopables"),
        ));
        function.instruction(&Instruction::LocalSet(unscopables_key_local));
        function.instruction(&Instruction::LocalGet(unscopables_local));
        function.instruction(&Instruction::LocalSet(unscopables_value_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(unscopables_value_tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            unscopables_key_local,
            unscopables_value_local,
            unscopables_value_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(unscopables_value_tag_local);
        self.release_temp_local(unscopables_value_local);
        self.release_temp_local(unscopables_key_local);
        self.release_temp_local(unscopables_local);

        Ok(())
    }
}
