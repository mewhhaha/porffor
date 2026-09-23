//! The two "weird" accessors on `%Iterator.prototype%` (ECMA-262 27.1.4.1
//! `Iterator.prototype.constructor` and 27.1.4.14
//! `Iterator.prototype[%Symbol.toStringTag%]`).
//!
//! Both setters are `SetterThatIgnoresPrototypeProperties(this value,
//! %Iterator.prototype%, p, v)`. The home object is the setter's own realm's
//! `%Iterator.prototype%`, derived from the builtin's realm environment like
//! every other realm-sensitive builtin body. The realm environment must stay
//! the builtin's trusted realm record: the body calls further builtins
//! (`[[GetOwnProperty]]`, `Set`, `CreateDataPropertyOrThrow`) that consume it.

use super::*;

/// The property a `%Iterator.prototype%` weird setter writes.
#[derive(Clone, Copy)]
pub(crate) enum IteratorPrototypeWeirdSetter {
    Constructor,
    ToStringTag,
}

impl IteratorPrototypeWeirdSetter {
    const fn incompatible_receiver_message(self) -> &'static str {
        match self {
            Self::Constructor => {
                "Iterator.prototype.constructor setter called on incompatible receiver"
            }
            Self::ToStringTag => {
                "Iterator.prototype[Symbol.toStringTag] setter called on incompatible receiver"
            }
        }
    }

    const fn home_receiver_message(self) -> &'static str {
        match self {
            Self::Constructor => {
                "Cannot assign to read only property 'constructor' of %Iterator.prototype%"
            }
            Self::ToStringTag => {
                "Cannot assign to read only property Symbol.toStringTag of %Iterator.prototype%"
            }
        }
    }

    /// The property key `p`, as a tagged value and as the property-key
    /// payload the object model indexes by (identical for both kinds).
    fn emit_key_to_locals(
        self,
        builder: &FunctionBuilder<'_>,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        let (payload, tag) = match self {
            Self::Constructor => (builder.strings.payload("constructor"), ValueKind::String),
            Self::ToStringTag => (
                builder
                    .strings
                    .property_key_symbol_payload("Symbol.toStringTag"),
                ValueKind::Symbol,
            ),
        };
        function.instruction(&Instruction::I64Const(payload));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(tag.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
    }
}

impl<'a> FunctionBuilder<'a> {
    /// `SetterThatIgnoresPrototypeProperties ( thisValue, home, p, v )`
    /// (ECMA-262 27.1.4.1.2 / 27.1.4.14.2), with `home` the current realm's
    /// `%Iterator.prototype%` and `v` the first argument.
    pub(crate) fn emit_iterator_prototype_weird_setter(
        &mut self,
        setter: IteratorPrototypeWeirdSetter,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported("missing %Iterator.prototype% weird setter receiver")
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing %Iterator.prototype% weird setter receiver tag")
        })?;
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "%Iterator.prototype% weird setter requires Object.getOwnPropertyDescriptor",
                )
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "%Iterator.prototype% weird setter requires Object.defineProperty",
                )
            })?;
        let home_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();

        // 1. If thisValue is not an Object, throw a TypeError exception.
        self.emit_is_heap_object_like_tag_i32(this_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            setter.incompatible_receiver_message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // 2. If SameValue(thisValue, home) is true, throw a TypeError
        //    (emulating a strict-mode write to a non-writable data property).
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(home_local));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_iterator_prototype(
            self.current_env_local,
            home_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalGet(home_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            setter.home_receiver_message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // 3. Let desc be ? thisValue.[[GetOwnProperty]](p). The descriptor
        //    builtin owns ordinary, exotic and Proxy [[GetOwnProperty]].
        setter.emit_key_to_locals(self, key_payload_local, key_tag_local, function);
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_direct_js_call(
            &get_own_property_descriptor_meta,
            None,
            &[
                (this_payload_local, this_tag_local),
                (key_payload_local, key_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // 4. If desc is undefined, perform ? CreateDataPropertyOrThrow(
        //    thisValue, p, v). Set would find this very accessor again.
        //    CreateDataPropertyOrThrow is DefinePropertyOrThrow with a
        //    complete, all-true data descriptor, and `thisValue` may be any
        //    object kind (Array, Function, Proxy, ...), so this goes through
        //    the generic Object.defineProperty body. Its descriptor object
        //    has a null prototype, keeping ToPropertyDescriptor's
        //    HasProperty/Get reads unobservable.
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
        self.emit_object_define_local_data(
            descriptor_payload_local,
            "value",
            value_payload_local,
            value_tag_local,
            function,
        )?;
        for field in ["writable", "enumerable", "configurable"] {
            self.emit_object_define_bool_data(descriptor_payload_local, field, true, function)?;
        }
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (this_payload_local, this_tag_local),
                (key_payload_local, key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        // 5. Else, perform ? Set(thisValue, p, v, true).
        self.emit_object_write_strict(
            this_payload_local,
            this_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);

        // 6. Return unused.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        for local in [
            descriptor_tag_local,
            descriptor_payload_local,
            value_tag_local,
            value_payload_local,
            key_tag_local,
            key_payload_local,
            home_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
