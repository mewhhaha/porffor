//! `symbol` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_symbol_constructor_intrinsics(
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

        // `Symbol` has a [[Construct]] internal method per spec (it
        // may appear as an `extends` target; a `super()` call into
        // it throws) even though `new Symbol()` always throws, so
        // `constructable()` is true and the generic prototype/
        // constructor wiring above already defined the
        // non-writable/non-enumerable/non-configurable `prototype`
        // data property and the writable/non-enumerable/
        // configurable `Symbol.prototype.constructor` back-reference
        // (see the `else` branch gated on `builtin.constructable()`
        // near the top of this function). Fetch the prototype object
        // and continue with the well-known symbols, the global
        // registry, and `Symbol.prototype`'s remaining members.
        function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));

        // Symbol.prototype[Symbol.toStringTag] === "Symbol"
        // (non-writable, non-enumerable, configurable).
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Symbol")));
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

        for (key, value) in [
            ("iterator", "Symbol.iterator"),
            ("asyncIterator", "Symbol.asyncIterator"),
            ("hasInstance", "Symbol.hasInstance"),
            ("isConcatSpreadable", "Symbol.isConcatSpreadable"),
            ("match", "Symbol.match"),
            ("matchAll", "Symbol.matchAll"),
            ("replace", "Symbol.replace"),
            ("search", "Symbol.search"),
            ("species", "Symbol.species"),
            ("split", "Symbol.split"),
            ("toPrimitive", "Symbol.toPrimitive"),
            ("toStringTag", "Symbol.toStringTag"),
            ("unscopables", "Symbol.unscopables"),
            ("dispose", "Symbol.dispose"),
            ("asyncDispose", "Symbol.asyncDispose"),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(value)));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                false,
                function,
            )?;
        }

        // Global symbol registry backing `Symbol.for` / `Symbol.keyFor`:
        // a null-prototype ordinary object mapping description strings to
        // the canonical registered symbol values.
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::GlobalSet(SYMBOL_REGISTRY_GLOBAL_INDEX));

        if let Some(for_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolFor.function_id())
            .cloned()
        {
            self.emit_object_define_function_data(object_local, "for", &for_meta, function)?;
        }
        if let Some(key_for_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolKeyFor.function_id())
            .cloned()
        {
            self.emit_object_define_function_data(object_local, "keyFor", &key_for_meta, function)?;
        }

        // Symbol.prototype.toString / Symbol.prototype.valueOf:
        // ordinary writable/non-enumerable/configurable methods.
        function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        if let Some(to_string_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeToString.function_id())
            .cloned()
        {
            self.emit_object_define_function_data(
                prototype_object_local,
                "toString",
                &to_string_meta,
                function,
            )?;
        }
        if let Some(value_of_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeValueOf.function_id())
            .cloned()
        {
            self.emit_object_define_function_data(
                prototype_object_local,
                "valueOf",
                &value_of_meta,
                function,
            )?;
        }

        // Symbol.prototype.description: accessor (getter-only,
        // non-enumerable, configurable) — Object.getOwnPropertyDescriptor
        // must see a real getter function.
        if let Some(description_getter_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeDescriptionGetter.function_id())
            .cloned()
        {
            function.instruction(&Instruction::I64Const(self.strings.payload("description")));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(&description_getter_meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_accessor_property_with_flags(
                prototype_object_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }

        // Symbol.prototype[Symbol.toPrimitive]: non-writable,
        // non-enumerable, configurable data property.
        if let Some(to_primitive_meta) = self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeToPrimitive.function_id())
            .cloned()
        {
            function.instruction(&Instruction::I64Const(
                self.strings
                    .property_key_symbol_payload("Symbol.toPrimitive"),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(&to_primitive_meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
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
        }

        Ok(())
    }
}
