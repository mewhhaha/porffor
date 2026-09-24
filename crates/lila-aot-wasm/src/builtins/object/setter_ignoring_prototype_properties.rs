//! The accessor setters specified as `SetterThatIgnoresPrototypeProperties(
//! this value, home, p, v)`:
//!
//! - ECMA-262 27.1.4.1 `Iterator.prototype.constructor` and 27.1.4.14
//!   `Iterator.prototype[%Symbol.toStringTag%]`, whose home object is
//!   `%Iterator.prototype%`;
//! - proposal-error-stack-accessor `set Error.prototype.stack`, whose home
//!   object is `%Error.prototype%` and which first requires `v` to be a String.
//!
//! The home object is always the setter's own realm's intrinsic, derived from
//! the builtin's realm environment like every other realm-sensitive builtin
//! body. The realm environment must stay the builtin's trusted realm record:
//! the body calls further builtins (`[[GetOwnProperty]]`, `Set`,
//! `CreateDataPropertyOrThrow`) that consume it.

use super::super::errors::ActiveBuiltinRealmPrototype;
use super::*;

/// The closed set of builtin setters whose algorithm is
/// `SetterThatIgnoresPrototypeProperties`. Each row names its home object,
/// its property key and any precondition on `v` together.
#[derive(Clone, Copy)]
pub(crate) enum SetterIgnoringPrototypeProperties {
    IteratorConstructor,
    IteratorToStringTag,
    ErrorStack,
}

/// What the setter requires of `v` before it reaches
/// `SetterThatIgnoresPrototypeProperties`.
enum SetterValueDomain {
    Any,
    /// `set Error.prototype.stack` step 3: if `v` is not a String, throw a
    /// TypeError.
    String {
        not_string_message: &'static str,
    },
}

impl SetterIgnoringPrototypeProperties {
    const fn incompatible_receiver_message(self) -> &'static str {
        match self {
            Self::IteratorConstructor => {
                "Iterator.prototype.constructor setter called on incompatible receiver"
            }
            Self::IteratorToStringTag => {
                "Iterator.prototype[Symbol.toStringTag] setter called on incompatible receiver"
            }
            Self::ErrorStack => "Error.prototype.stack setter called on incompatible receiver",
        }
    }

    const fn home_receiver_message(self) -> &'static str {
        match self {
            Self::IteratorConstructor => {
                "Cannot assign to read only property 'constructor' of %Iterator.prototype%"
            }
            Self::IteratorToStringTag => {
                "Cannot assign to read only property Symbol.toStringTag of %Iterator.prototype%"
            }
            Self::ErrorStack => "Cannot assign to read only property 'stack' of %Error.prototype%",
        }
    }

    const fn value_domain(self) -> SetterValueDomain {
        match self {
            Self::IteratorConstructor | Self::IteratorToStringTag => SetterValueDomain::Any,
            Self::ErrorStack => SetterValueDomain::String {
                not_string_message: "Error.prototype.stack setter value must be a string",
            },
        }
    }

    /// The property key `p`, as a tagged value and as the property-key
    /// payload the object model indexes by (identical for every kind).
    fn emit_key_to_locals(
        self,
        builder: &FunctionBuilder<'_>,
        key_payload_local: u32,
        key_tag_local: u32,
        function: &mut Function,
    ) {
        let (payload, tag) = match self {
            Self::IteratorConstructor => {
                (builder.strings.payload("constructor"), ValueKind::String)
            }
            Self::IteratorToStringTag => (
                builder
                    .strings
                    .property_key_symbol_payload("Symbol.toStringTag"),
                ValueKind::Symbol,
            ),
            Self::ErrorStack => (builder.strings.payload("stack"), ValueKind::String),
        };
        function.instruction(&Instruction::I64Const(payload));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(tag.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
    }

    /// `home`: the executing setter's own realm's intrinsic prototype.
    fn emit_home_to_local(
        self,
        builder: &mut FunctionBuilder<'_>,
        home_local: u32,
        function: &mut Function,
    ) {
        match self {
            Self::IteratorConstructor | Self::IteratorToStringTag => {
                function.instruction(&Instruction::LocalGet(builder.current_env_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(home_local));
                function.instruction(&Instruction::Else);
                builder.emit_load_function_defining_realm_iterator_prototype(
                    builder.current_env_local,
                    home_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
            Self::ErrorStack => builder.emit_load_active_builtin_realm_prototype(
                ActiveBuiltinRealmPrototype::Error,
                home_local,
                function,
            ),
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// The setter body for `setter`, with `v` the first argument, ending in
    /// `SetterThatIgnoresPrototypeProperties ( thisValue, home, p, v )`.
    pub(crate) fn emit_setter_ignoring_prototype_properties(
        &mut self,
        setter: SetterIgnoringPrototypeProperties,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self
            .this_payload_local
            .ok_or_else(|| EmitError::unsupported("missing prototype-ignoring setter receiver"))?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("missing prototype-ignoring setter receiver tag")
        })?;
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "prototype-ignoring setter requires Object.getOwnPropertyDescriptor",
                )
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("prototype-ignoring setter requires Object.defineProperty")
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

        // The caller's own precondition on `v`, which its algorithm places
        // after the Object check on `this` and before the home check.
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        match setter.value_domain() {
            SetterValueDomain::Any => {}
            SetterValueDomain::String { not_string_message } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    not_string_message,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
            }
        }

        // 2. If SameValue(thisValue, home) is true, throw a TypeError
        //    (emulating a strict-mode write to a non-writable data property).
        setter.emit_home_to_local(self, home_local, function);
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
        //    the generic Object.defineProperty body. Its descriptor object is
        //    the owner's private CreateDataProperty carrier, whose null
        //    prototype keeps ToPropertyDescriptor's HasProperty/Get reads
        //    unobservable.
        self.emit_create_data_property_descriptor_carrier(
            TaggedLocals::new(value_payload_local, value_tag_local),
            descriptor_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));
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

        // 6. Return unused. (Both callers' setters then return undefined.)
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
