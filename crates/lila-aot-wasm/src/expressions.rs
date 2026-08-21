use super::*;
use crate::operations::NumericBinaryOperator;
use lila_ir::{OptionalChainCallReceiverIr, OptionalChainOperationIr, RegExpProgram};

#[derive(Debug)]
#[must_use = "a raw Super Property Reference must be consumed by GetValue"]
struct EvaluatedRawSuperPropertyReferenceLocals {
    base_payload: u32,
    base_tag: u32,
    receiver_payload: u32,
    receiver_tag: u32,
    referenced_name_payload: u32,
    referenced_name_tag: u32,
}

#[derive(Debug)]
#[must_use = "a coerced Super Property Reference must be consumed by PutValue"]
struct CoercedSuperPropertyReferenceLocals {
    base_payload: u32,
    base_tag: u32,
    receiver_payload: u32,
    receiver_tag: u32,
    property_key_payload: u32,
    property_key_tag: u32,
}

impl<'a> FunctionBuilder<'a> {
    /// Runs `body` with the object-write failure guards reading *this
    /// Reference's* `[[Strict]]` instead of the ambient strictness of the
    /// function being emitted.
    ///
    /// PutValue 3.d asks whether `V.[[Strict]]` is true, where `V` is the
    /// Reference the assignment evaluated — not whether the code currently
    /// being emitted is strict. The two agree only while Reference creation
    /// and Reference consumption sit in the same function body, which is
    /// exactly the assumption that fails for an outlined write helper shared
    /// by callers of both modes. `object_write_strict_flag_local` already
    /// exists for that helper; this makes every ordinary write use the same
    /// mechanism, sourced from the IR node rather than from
    /// `is_current_function_strict()`.
    ///
    /// The flag word comes from [`Strictness::helper_flag_word`], the only
    /// conversion from the type to a machine word, so no other boolean can
    /// reach this `I64Const`.
    ///
    /// The locals this reserves are held in an array whose **length is**
    /// `planning::REFERENCE_STRICTNESS_FLAG_LOCALS`, the same constant
    /// `count_expr_temp_locals` adds to every Reference-write arm's budget.
    /// That is the tie between the two: growing this guard to a second local
    /// is `error[E0308]` on the array type *and* on the destructuring below,
    /// rather than `reserve_temp_local`'s `assert!` firing in the middle of
    /// code generation. A `const _: () = assert!(CONST == 1)` would not have
    /// noticed — it compares the constant to a literal, not to what this
    /// function actually reserves.
    pub(crate) fn with_reference_strictness(
        &mut self,
        strictness: Strictness,
        function: &mut Function,
        body: impl FnOnce(&mut Self, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let reserved: [u32; crate::planning::REFERENCE_STRICTNESS_FLAG_LOCALS] =
            [self.reserve_temp_local()];
        let [strict_local] = reserved;
        function.instruction(&Instruction::I64Const(strictness.helper_flag_word()));
        function.instruction(&Instruction::LocalSet(strict_local));
        let previous_strict_local = self.object_write_strict_flag_local;
        self.object_write_strict_flag_local = Some(strict_local);
        let result = body(self, function);
        self.object_write_strict_flag_local = previous_strict_local;
        self.release_temp_local(strict_local);
        result
    }

    /// PutValue (6.2.5.6) for a Reference whose `[[Base]]` is the global object
    /// or unresolvable — [`lila_ir::ExprIr::GlobalPropertyWrite`] and the
    /// write-back halves of `GlobalPropertyUpdate` / `GlobalPropertyCompoundAssign`.
    ///
    /// Consumes the carried `[[Strict]]` for **both** of PutValue's strict
    /// throws, which is why it exists rather than the two halves being spelled
    /// separately at each arm:
    ///
    /// * step 2.a — unresolvable base, ReferenceError — via
    ///   `emit_global_property_write_checked`'s presence test;
    /// * step 3.d — `[[Set]]` answered `false`, TypeError — via the runtime
    ///   strictness guard `with_reference_strictness` installs, which the
    ///   `emit_object_write` inside the checked write then reads.
    ///
    /// `GlobalPropertyUpdate` and `GlobalPropertyCompoundAssign` used to bind
    /// `strictness: _` and call the *unchecked* `emit_global_property_write`,
    /// so `"use strict"; delete globalThis.g; g++;` silently created a property
    /// and `"use strict"; g -= 1;` on a non-writable global silently no-opped.
    /// A field constructed at three sites and read at none is what invariant I9
    /// was written to prohibit.
    fn emit_reference_global_property_write(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.with_reference_strictness(strictness, function, |emitter, function| {
            emitter.emit_global_property_write_checked(
                name,
                payload_local,
                tag_local,
                strictness,
                function,
            )
        })
    }

    /// Evaluate a super property's key expression without applying
    /// ToPropertyKey yet. SuperProperty evaluation needs this raw value before
    /// GetSuperBase/null checking; coercion is deliberately emitted by the
    /// caller only after that check.
    fn compile_super_property_key_expression_to_locals(
        &mut self,
        key: &PropertyKeyIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match key {
            PropertyKeyIr::StaticString(value) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            PropertyKeyIr::StringExpr(expr) => {
                self.compile_expr_to_locals(expr, payload_local, tag_local, function)?;
            }
            PropertyKeyIr::ArrayIndex(expr) => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            PropertyKeyIr::ArrayLength => {
                return Err(EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: object key kind",
                ));
            }
        }
        Ok(())
    }

    fn compile_super_property_read_to_locals(
        &mut self,
        key: &PropertyKeyIr,
        receiver: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let super_base_local = self.reserve_temp_local();
        let super_base_tag_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_super_property_key_expression_to_locals(
            key,
            key_local,
            key_tag_local,
            function,
        )?;
        self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
        self.emit_throw_if_null_super_base(super_base_local, super_base_tag_local, function)?;
        self.emit_value_to_property_key_locals(key_local, key_tag_local, function)?;
        self.emit_object_read_with_key_tag(
            super_base_local,
            super_base_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            Some(key_tag_local),
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(super_base_tag_local);
        self.release_temp_local(super_base_local);
        Ok(())
    }

    fn compile_super_property_write_to_locals(
        &mut self,
        key: &PropertyKeyIr,
        receiver: &TypedExpr,
        value: &TypedExpr,
        strictness: Strictness,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let super_base_local = self.reserve_temp_local();
        let super_base_tag_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();

        self.compile_expr_to_locals(
            receiver,
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.compile_super_property_key_expression_to_locals(
            key,
            key_local,
            key_tag_local,
            function,
        )?;
        self.emit_load_super_base(super_base_local, super_base_tag_local, function)?;
        self.emit_throw_if_null_super_base(super_base_local, super_base_tag_local, function)?;
        self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
        self.emit_value_to_property_key_locals(key_local, key_tag_local, function)?;
        self.emit_ordinary_set_result_via_helper(
            super_base_local,
            super_base_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_local,
            key_tag_local,
            payload_local,
            tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.with_reference_strictness(strictness, function, |emitter, function| {
            emitter.emit_object_write_set_failure_else("Cannot assign to super property", function)
        })?;
        function.instruction(&Instruction::End);

        self.release_temp_local(set_result_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(super_base_tag_local);
        self.release_temp_local(super_base_local);
        Ok(())
    }

    fn evaluate_raw_super_property_reference(
        &mut self,
        receiver: &TypedExpr,
        referenced_name: &PropertyKeyIr,
        function: &mut Function,
    ) -> Result<EvaluatedRawSuperPropertyReferenceLocals, EmitError> {
        let base_payload = self.reserve_temp_local();
        let base_tag = self.reserve_temp_local();
        let receiver_payload = self.reserve_temp_local();
        let receiver_tag = self.reserve_temp_local();
        let referenced_name_payload = self.reserve_temp_local();
        let referenced_name_tag = self.reserve_temp_local();

        self.compile_expr_to_locals(receiver, receiver_payload, receiver_tag, function)?;
        self.compile_super_property_key_expression_to_locals(
            referenced_name,
            referenced_name_payload,
            referenced_name_tag,
            function,
        )?;
        self.emit_load_super_base(base_payload, base_tag, function)?;
        self.emit_throw_if_null_super_base(base_payload, base_tag, function)?;

        Ok(EvaluatedRawSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            referenced_name_payload,
            referenced_name_tag,
        })
    }

    fn emit_get_value_from_raw_super_property_reference(
        &mut self,
        reference: EvaluatedRawSuperPropertyReferenceLocals,
        value_payload: u32,
        value_tag: u32,
        function: &mut Function,
    ) -> Result<CoercedSuperPropertyReferenceLocals, EmitError> {
        let EvaluatedRawSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            referenced_name_payload: property_key_payload,
            referenced_name_tag: property_key_tag,
        } = reference;

        self.emit_value_to_property_key_locals(property_key_payload, property_key_tag, function)?;
        self.emit_object_read_with_key_tag(
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            Some(property_key_tag),
            value_payload,
            value_tag,
            function,
        )?;

        Ok(CoercedSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
        })
    }

    fn emit_put_value_from_coerced_super_property_reference(
        &mut self,
        reference: CoercedSuperPropertyReferenceLocals,
        value_payload: u32,
        value_tag: u32,
        set_result: u32,
        strictness: Strictness,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let CoercedSuperPropertyReferenceLocals {
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
        } = reference;

        self.emit_ordinary_set_result_via_helper(
            base_payload,
            base_tag,
            receiver_payload,
            receiver_tag,
            property_key_payload,
            property_key_tag,
            value_payload,
            value_tag,
            set_result,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.with_reference_strictness(strictness, function, |emitter, function| {
            emitter.emit_object_write_set_failure_else("Cannot assign to super property", function)
        })?;
        function.instruction(&Instruction::End);

        self.release_temp_local(property_key_tag);
        self.release_temp_local(property_key_payload);
        self.release_temp_local(receiver_tag);
        self.release_temp_local(receiver_payload);
        self.release_temp_local(base_tag);
        self.release_temp_local(base_payload);
        Ok(())
    }

    fn compile_super_property_mutation_to_locals(
        &mut self,
        mutation: &SuperPropertyMutationIr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // These result locals sit below the reference carrier so PutValue can
        // consume and release the carrier while retaining both numeric values
        // until a successful Set selects the prefix/postfix result.
        let old_value_payload = self.reserve_temp_local();
        let old_value_tag = self.reserve_temp_local();
        let new_value_payload = self.reserve_temp_local();
        let new_value_tag = self.reserve_temp_local();
        let set_result = self.reserve_temp_local();

        let raw_reference = self.evaluate_raw_super_property_reference(
            mutation.receiver(),
            mutation.referenced_name(),
            function,
        )?;
        let coerced_reference = self.emit_get_value_from_raw_super_property_reference(
            raw_reference,
            old_value_payload,
            old_value_tag,
            function,
        )?;

        match mutation.operation() {
            SuperPropertyMutationOperationIr::NumericUpdate {
                op,
                return_mode,
                value_kind,
            } => {
                match value_kind {
                    ValueKind::Dynamic => self.emit_value_to_numeric_locals(
                        old_value_payload,
                        old_value_tag,
                        function,
                    )?,
                    ValueKind::Number => {
                        self.emit_value_to_number_payload(
                            old_value_tag,
                            old_value_payload,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalSet(old_value_payload));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                        function.instruction(&Instruction::LocalSet(old_value_tag));
                        self.emit_return_current_completion_if_throw(function);
                    }
                    ValueKind::BigInt => {}
                    ValueKind::Undefined
                    | ValueKind::Null
                    | ValueKind::Boolean
                    | ValueKind::String
                    | ValueKind::Symbol
                    | ValueKind::Object
                    | ValueKind::Array
                    | ValueKind::Function
                    | ValueKind::Arguments => {
                        unreachable!("numeric Super update requires Number, BigInt, or Dynamic")
                    }
                }
                self.emit_update_delta_from_locals(
                    *op,
                    *value_kind,
                    old_value_payload,
                    old_value_tag,
                    function,
                );
                function.instruction(&Instruction::LocalSet(new_value_payload));
                function.instruction(&Instruction::LocalGet(old_value_tag));
                function.instruction(&Instruction::LocalSet(new_value_tag));

                self.emit_put_value_from_coerced_super_property_reference(
                    coerced_reference,
                    new_value_payload,
                    new_value_tag,
                    set_result,
                    mutation.strictness(),
                    function,
                )?;

                let (result_payload, result_tag) = match return_mode {
                    UpdateReturnMode::Prefix => (new_value_payload, new_value_tag),
                    UpdateReturnMode::Postfix => (old_value_payload, old_value_tag),
                };
                function.instruction(&Instruction::LocalGet(result_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(result_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            SuperPropertyMutationOperationIr::EagerCompound {
                old_value_binding,
                result,
            } => {
                self.push_scope();
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(
                        old_value_binding.clone(),
                        BindingStorage::Dynamic {
                            tag_local: old_value_tag,
                            payload_local: old_value_payload,
                        },
                    );
                let compile_result =
                    self.compile_expr_to_locals(result, new_value_payload, new_value_tag, function);
                self.pop_scope();
                compile_result?;

                self.emit_put_value_from_coerced_super_property_reference(
                    coerced_reference,
                    new_value_payload,
                    new_value_tag,
                    set_result,
                    mutation.strictness(),
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(new_value_payload));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(new_value_tag));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
        }

        self.release_temp_local(set_result);
        self.release_temp_local(new_value_tag);
        self.release_temp_local(new_value_payload);
        self.release_temp_local(old_value_tag);
        self.release_temp_local(old_value_payload);
        Ok(())
    }

    pub(crate) fn compile_expr_payload(
        &mut self,
        expr: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let emits_own_dynamic_result = matches!(
            expr.expr,
            ExprIr::UpdateIdentifier {
                value_kind: ValueKind::Dynamic,
                ..
            } | ExprIr::GlobalPropertyUpdate {
                value_kind: ValueKind::Dynamic,
                ..
            }
        );
        if !expr.possible_kinds.is_singleton() && !emits_own_dynamic_result {
            self.compile_expr_to_locals(expr, self.scratch_local, self.result_tag_local, function)?;
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            return Ok(());
        }

        match &expr.expr {
            ExprIr::DynamicImport {
                specifier,
                options,
                referrer,
                ..
            } => {
                self.emit_dynamic_import(*referrer, specifier, options.as_deref(), function)?;
            }
            ExprIr::ImportMeta { module } => {
                self.emit_import_meta(*module, function)?;
            }
            ExprIr::ModuleNamespace { module } => {
                self.emit_module_namespace(*module, function)?;
            }
            ExprIr::Undefined | ExprIr::ArrayHole | ExprIr::Null => {
                self.emit_undefined_payload(function);
            }
            ExprIr::Boolean(value) => {
                function.instruction(&Instruction::I64Const(i64::from(*value)));
            }
            ExprIr::Number(bits) => {
                function.instruction(&Instruction::I64Const(*bits as i64));
            }
            ExprIr::BigInt(value) => {
                if value.requires_arbitrary_precision_storage {
                    let (sign, limbs) = value.signed_magnitude_limbs();
                    self.emit_alloc_bigint_literal(sign, &limbs, function)?;
                    return Ok(());
                }
                function.instruction(&Instruction::I64Const(value.wrapping_payload() as i64));
            }
            ExprIr::Symbol { description } => {
                // Allocate a symbol record and record its `[[Description]]`
                // (undefined, or a coerced string) so `.description` can read
                // it back. The record handle itself is the symbol value and
                // gives each `Symbol()` a unique identity.
                let handle_local = self.reserve_temp_local();
                self.emit_heap_alloc_const(HEAP_SYMBOL_RECORD_SIZE, function)?;
                function.instruction(&Instruction::LocalSet(handle_local));
                match description {
                    None => {
                        self.store_i64_const_at_offset(
                            handle_local,
                            HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                            ValueKind::Undefined.tag() as u64,
                            function,
                        );
                    }
                    Some(description) => {
                        let desc_payload_local = self.reserve_temp_local();
                        let desc_tag_local = self.reserve_temp_local();
                        self.compile_expr_to_locals(
                            description,
                            desc_payload_local,
                            desc_tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            desc_payload_local,
                            desc_tag_local,
                            function,
                        )?;
                        self.store_i64_local_at_offset(
                            handle_local,
                            HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
                            desc_tag_local,
                            function,
                        );
                        self.store_i64_local_at_offset(
                            handle_local,
                            HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
                            desc_payload_local,
                            function,
                        );
                        self.release_temp_local(desc_tag_local);
                        self.release_temp_local(desc_payload_local);
                    }
                }
                function.instruction(&Instruction::LocalGet(handle_local));
                self.release_temp_local(handle_local);
            }
            ExprIr::String(value) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
            }
            ExprIr::TemplateObject(template) => {
                function.instruction(&Instruction::GlobalGet(
                    self.template_object_global_index(template.site_id),
                ));
            }
            ExprIr::RegExpLiteral {
                source,
                flags,
                program,
            } => {
                self.compile_regexp_literal_payload(source, flags, program.as_ref(), function)?;
            }
            ExprIr::FunctionValue(function_id) => {
                self.emit_function_identity_payload(function_id, function)?;
            }
            ExprIr::JsonParseStaticReviver { value, reviver } => {
                self.compile_json_static_reviver_to_locals(
                    value,
                    reviver,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::This => {
                self.compile_this_payload(function)?;
            }
            ExprIr::Arguments => {
                let storage = self.lookup_binding(LEXICAL_ARGUMENTS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing `arguments` binding",
                    )
                })?;
                self.read_binding_payload(storage, function)?;
            }
            ExprIr::ObjectLiteral(properties) => {
                self.compile_object_literal_payload(properties, function)?;
            }
            ExprIr::ArrayLiteral(elements) => {
                self.compile_array_literal_payload(elements, function)?;
            }
            ExprIr::ArrayAccumulation(accumulation) => {
                self.compile_array_accumulation_payload(accumulation, function)?;
            }
            ExprIr::Identifier(name) => {
                if name == LEXICAL_THIS_NAME {
                    self.compile_this_payload(function)?;
                    return Ok(());
                }
                if name == GLOBAL_THIS_NAME {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    return Ok(());
                }
                if self.should_read_script_global_property(name) {
                    let storage = self.lookup_binding(name);
                    self.emit_global_property_read(
                        name,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if let Some(storage) = storage {
                        function.instruction(&Instruction::LocalGet(self.result_tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.read_binding_to_locals(
                            storage,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::End);
                    }
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_read(
                        name,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                if let Some(host) = self.compiled_host_builtin_by_name(name) {
                    let meta = self.functions.get(&host.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown host function `{}`",
                            host.as_str()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_payload(storage, function)?;
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.emit_global_property_read(
                    name,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::GlobalIdentifierRead { name } => {
                self.emit_global_identifier_read(
                    name,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::AssignIdentifier { name, value } => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    let value_local = self.reserve_temp_local();
                    let tag_local = self.reserve_temp_local();
                    self.compile_expr_to_locals(value, value_local, tag_local, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        value_local,
                        tag_local,
                        function,
                    )?;
                    self.emit_global_property_write(name, value_local, tag_local, function)?;
                    function.instruction(&Instruction::LocalGet(value_local));
                    self.release_temp_local(tag_local);
                    self.release_temp_local(value_local);
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(value, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.write_binding_from_locals(storage, value_local, tag_local, function);
                self.mirror_binding_to_global_object(name, storage, function)?;
                function.instruction(&Instruction::LocalGet(value_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::GlobalPropertyWrite {
                name,
                value,
                strictness,
                ..
            } => {
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.compile_expr_to_locals(value, value_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(value_local, tag_local, function)?;
                self.emit_reference_global_property_write(
                    name,
                    value_local,
                    tag_local,
                    *strictness,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(value_local));
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::PropertyWrite {
                target,
                key,
                value,
                strictness,
            } => {
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_write_payload(target, key, value, function)
                })?;
            }
            ExprIr::PropertyUpdate {
                target,
                key,
                op,
                return_mode,
                value_kind,
                strictness,
            } => {
                let scratch_local = self.scratch_local;
                let result_tag_local = self.result_tag_local;
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_update_to_locals(
                        target,
                        key,
                        *op,
                        *return_mode,
                        *value_kind,
                        scratch_local,
                        result_tag_local,
                        function,
                    )
                })?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PropertyCompoundAssign {
                target,
                key,
                op,
                value,
                strictness,
            } => {
                let scratch_local = self.scratch_local;
                let result_tag_local = self.result_tag_local;
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_compound_assign_to_locals(
                        target,
                        key,
                        *op,
                        value,
                        scratch_local,
                        result_tag_local,
                        function,
                    )
                })?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::UpdateIdentifier {
                name,
                op,
                return_mode,
                value_kind,
            } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.read_binding_to_locals(storage, value_local, tag_local, function)?;
                if *value_kind == ValueKind::Dynamic {
                    self.emit_value_to_numeric_locals(value_local, tag_local, function)?;
                } else if *value_kind == ValueKind::Number {
                    self.emit_value_to_number_payload(tag_local, value_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_return_current_completion_if_throw(function);
                }
                match return_mode {
                    UpdateReturnMode::Prefix => {
                        self.emit_update_delta_from_locals(
                            *op,
                            *value_kind,
                            value_local,
                            tag_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalSet(self.scratch_local));
                        if *value_kind == ValueKind::Dynamic {
                            function.instruction(&Instruction::LocalGet(tag_local));
                        } else {
                            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
                        }
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        self.write_binding_from_locals(
                            storage,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        );
                        self.mirror_binding_to_global_object(name, storage, function)?;
                        function.instruction(&Instruction::LocalGet(self.scratch_local));
                    }
                    UpdateReturnMode::Postfix => {
                        let old_value_local = self.reserve_temp_local();
                        function.instruction(&Instruction::LocalGet(value_local));
                        function.instruction(&Instruction::LocalSet(old_value_local));
                        self.emit_update_delta_from_locals(
                            *op,
                            *value_kind,
                            value_local,
                            tag_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalSet(value_local));
                        if *value_kind != ValueKind::Dynamic {
                            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                        self.write_binding_from_locals(storage, value_local, tag_local, function);
                        self.mirror_binding_to_global_object(name, storage, function)?;
                        function.instruction(&Instruction::LocalGet(tag_local));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        function.instruction(&Instruction::LocalGet(old_value_local));
                        self.release_temp_local(old_value_local);
                    }
                };
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::CompoundAssignIdentifier { name, op, value } => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                let temp_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let rhs_tag_local = self.reserve_temp_local();
                self.read_binding_to_locals(storage, temp_local, tag_local, function)?;
                self.compile_expr_to_locals(value, self.scratch_local, rhs_tag_local, function)?;
                if matches!(op, ArithmeticBinaryOp::Add) {
                    let lhs_string_local = self.reserve_temp_local();
                    let rhs_string_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(rhs_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_value_to_string_payload(temp_local, tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(lhs_string_local));
                    self.emit_value_to_string_payload(self.scratch_local, rhs_tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(rhs_string_local));
                    self.emit_concat_string_payloads_local(
                        lhs_string_local,
                        rhs_string_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_number_payload(tag_local, temp_local, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.emit_value_to_number_payload(rhs_tag_local, self.scratch_local, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Add);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                    function.instruction(&Instruction::End);
                    self.release_temp_local(rhs_string_local);
                    self.release_temp_local(lhs_string_local);
                } else if matches!(op, ArithmeticBinaryOp::Mod) {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                } else if matches!(op, ArithmeticBinaryOp::Exp) {
                    let output_local = self.reserve_temp_local();
                    self.emit_number_pow_payload(
                        temp_local,
                        self.scratch_local,
                        output_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(output_local));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.release_temp_local(output_local);
                } else {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => unreachable!(),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                        ArithmeticBinaryOp::Exp => unreachable!(),
                    };
                }
                if !matches!(op, ArithmeticBinaryOp::Add | ArithmeticBinaryOp::Exp) {
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                }
                if matches!(op, ArithmeticBinaryOp::Exp) {
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                }
                self.write_binding_from_locals(
                    storage,
                    self.scratch_local,
                    rhs_tag_local,
                    function,
                );
                self.mirror_binding_to_global_object(name, storage, function)?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                self.release_temp_local(rhs_tag_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(temp_local);
            }
            ExprIr::GlobalPropertyUpdate {
                name,
                op,
                return_mode,
                value_kind,
                strictness,
            } => {
                let value_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                self.emit_global_property_read(name, value_local, tag_local, function)?;
                if *value_kind == ValueKind::Dynamic {
                    self.emit_value_to_numeric_locals(value_local, tag_local, function)?;
                } else if *value_kind == ValueKind::Number {
                    self.emit_value_to_number_payload(tag_local, value_local, function)?;
                    function.instruction(&Instruction::LocalSet(value_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_return_current_completion_if_throw(function);
                }
                match return_mode {
                    UpdateReturnMode::Prefix => {
                        self.emit_update_delta_from_locals(
                            *op,
                            *value_kind,
                            value_local,
                            tag_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalSet(value_local));
                        if *value_kind != ValueKind::Dynamic {
                            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                        self.emit_reference_global_property_write(
                            name,
                            value_local,
                            tag_local,
                            *strictness,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalGet(tag_local));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        function.instruction(&Instruction::LocalGet(value_local));
                    }
                    UpdateReturnMode::Postfix => {
                        let old_value_local = self.reserve_temp_local();
                        function.instruction(&Instruction::LocalGet(value_local));
                        function.instruction(&Instruction::LocalSet(old_value_local));
                        self.emit_update_delta_from_locals(
                            *op,
                            *value_kind,
                            value_local,
                            tag_local,
                            function,
                        );
                        function.instruction(&Instruction::LocalSet(value_local));
                        if *value_kind != ValueKind::Dynamic {
                            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
                            function.instruction(&Instruction::LocalSet(tag_local));
                        }
                        self.emit_reference_global_property_write(
                            name,
                            value_local,
                            tag_local,
                            *strictness,
                            function,
                        )?;
                        function.instruction(&Instruction::LocalGet(tag_local));
                        function.instruction(&Instruction::LocalSet(self.result_tag_local));
                        function.instruction(&Instruction::LocalGet(old_value_local));
                        self.release_temp_local(old_value_local);
                    }
                }
                self.release_temp_local(tag_local);
                self.release_temp_local(value_local);
            }
            ExprIr::GlobalPropertyCompoundAssign {
                name,
                op,
                value,
                strictness,
            } => {
                let temp_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let rhs_tag_local = self.reserve_temp_local();
                self.emit_global_property_read(name, temp_local, tag_local, function)?;
                self.compile_expr_to_locals(value, self.scratch_local, rhs_tag_local, function)?;
                if matches!(op, ArithmeticBinaryOp::Add) {
                    let lhs_string_local = self.reserve_temp_local();
                    let rhs_string_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(rhs_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_value_to_string_payload(temp_local, tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(lhs_string_local));
                    self.emit_value_to_string_payload(self.scratch_local, rhs_tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(rhs_string_local));
                    self.emit_concat_string_payloads_local(
                        lhs_string_local,
                        rhs_string_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_number_payload(tag_local, temp_local, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.emit_value_to_number_payload(rhs_tag_local, self.scratch_local, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Add);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                    function.instruction(&Instruction::End);
                    self.release_temp_local(rhs_string_local);
                    self.release_temp_local(lhs_string_local);
                } else if matches!(op, ArithmeticBinaryOp::Mod) {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                } else if matches!(op, ArithmeticBinaryOp::Exp) {
                    let output_local = self.reserve_temp_local();
                    self.emit_number_pow_payload(
                        temp_local,
                        self.scratch_local,
                        output_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(output_local));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    self.release_temp_local(output_local);
                } else {
                    function.instruction(&Instruction::LocalGet(temp_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => unreachable!(),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                        ArithmeticBinaryOp::Exp => unreachable!(),
                    };
                }
                if !matches!(op, ArithmeticBinaryOp::Add | ArithmeticBinaryOp::Exp) {
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                }
                if matches!(op, ArithmeticBinaryOp::Exp) {
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(rhs_tag_local));
                }
                // The checked strict write performs a HasProperty operation
                // before Set and may use `scratch_local` internally. Move the
                // completed RHS into the now-dead old-value local so the
                // Reference payload cannot be clobbered before consumption.
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::LocalSet(temp_local));
                self.emit_reference_global_property_write(
                    name,
                    temp_local,
                    rhs_tag_local,
                    *strictness,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(temp_local));
                self.release_temp_local(rhs_tag_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(temp_local);
            }
            ExprIr::UnaryNumber { op, expr } => {
                self.compile_expr_to_number_payload(expr, function)?;
                match op {
                    UnaryNumericOp::Plus => {}
                    UnaryNumericOp::Minus => {
                        function.instruction(&Instruction::F64ReinterpretI64);
                        function.instruction(&Instruction::F64Neg);
                        function.instruction(&Instruction::I64ReinterpretF64);
                    }
                }
            }
            ExprIr::UnaryBitwiseNumeric { op, expr } => {
                self.compile_unary_bitwise_numeric_to_locals(
                    *op,
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Void { expr } => {
                self.compile_expr_to_locals(
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_undefined_payload(function);
            }
            ExprIr::DeleteValue { expr } => {
                self.compile_expr_to_locals(
                    expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(1));
            }
            ExprIr::DeleteIdentifier { kind, .. } => {
                let value = if matches!(kind, DeleteIdentifierKindIr::NonDeletable) {
                    0
                } else {
                    1
                };
                function.instruction(&Instruction::I64Const(value));
            }
            ExprIr::DeleteGlobalProperty { name, strictness } => {
                let result_local = self.reserve_temp_local();
                self.emit_global_property_delete(name, result_local, *strictness, function)?;
                function.instruction(&Instruction::LocalGet(result_local));
                self.release_temp_local(result_local);
            }
            ExprIr::DeleteProperty {
                target,
                key,
                strictness,
            } => {
                self.compile_delete_property_i32(target, key, *strictness, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::TypeOf { expr } => {
                self.compile_typeof_payload(expr, function)?;
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
            }
            ExprIr::NewTarget => {
                self.compile_new_target_to_locals(
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::LogicalNot { expr } => {
                self.compile_truthy_i32(expr, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::SpecOperation {
                operation,
                operands,
            } => {
                self.compile_spec_operation_payload(*operation, operands, function)?;
            }
            ExprIr::BinaryNumber { op, lhs, rhs } => {
                if expr.kind == ValueKind::BigInt {
                    // Both operands are statically BigInt, but each may be
                    // inline or heap-backed, and an inline-inline operation can
                    // still overflow into a heap result. The shared helper owns
                    // that whole decision; the runtime tag it reports is left in
                    // `result_tag_local` for `compile_expr_to_locals`.
                    self.compile_bigint_arithmetic_to_locals(
                        BigIntHelperOp::from_arithmetic(*op),
                        lhs,
                        rhs,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                if matches!(op, ArithmeticBinaryOp::Exp) {
                    self.compile_expr_payload(lhs, function)?;
                    self.compile_expr_payload(rhs, function)?;
                    let lhs_local = self.reserve_temp_local();
                    let rhs_local = self.reserve_temp_local();
                    let output_local = self.reserve_temp_local();
                    function.instruction(&Instruction::LocalSet(rhs_local));
                    function.instruction(&Instruction::LocalSet(lhs_local));
                    self.emit_number_pow_payload(lhs_local, rhs_local, output_local, function)?;
                    function.instruction(&Instruction::LocalGet(output_local));
                    self.release_temp_local(output_local);
                    self.release_temp_local(rhs_local);
                    self.release_temp_local(lhs_local);
                } else if matches!(op, ArithmeticBinaryOp::Mod) {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.result_local));
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::LocalGet(self.result_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.result_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                    function.instruction(&Instruction::I64ReinterpretF64);
                } else {
                    self.compile_expr_payload(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_payload(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => function.instruction(&Instruction::F64Add),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                        ArithmeticBinaryOp::Exp => unreachable!(),
                    };
                    function.instruction(&Instruction::I64ReinterpretF64);
                }
            }
            ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {
                if matches!(op, ArithmeticBinaryOp::Exp)
                    && expr.possible_kinds.contains(ValueKind::BigInt)
                {
                    self.compile_coercive_binary_number_to_locals(
                        *op,
                        lhs,
                        rhs,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                if expr.kind == ValueKind::BigInt {
                    let lhs_payload = self.reserve_temp_local();
                    let lhs_tag = self.reserve_temp_local();
                    let rhs_payload = self.reserve_temp_local();
                    let rhs_tag = self.reserve_temp_local();
                    self.compile_expr_to_primitive_locals(
                        lhs,
                        ToPrimitiveHint::Number,
                        lhs_payload,
                        lhs_tag,
                        function,
                    )?;
                    self.compile_expr_to_primitive_locals(
                        rhs,
                        ToPrimitiveHint::Number,
                        rhs_payload,
                        rhs_tag,
                        function,
                    )?;
                    self.emit_is_bigint_tag_i32(lhs_tag, function);
                    self.emit_is_bigint_tag_i32(rhs_tag, function);
                    function.instruction(&Instruction::I32And);
                    self.open_frame(ControlFrameKind::If, function);
                    self.emit_bigint_binary_op_to_locals(
                        BigIntHelperOp::from_arithmetic(*op),
                        lhs_payload,
                        lhs_tag,
                        rhs_payload,
                        rhs_tag,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.pop_control(ControlFrameKind::If);
                    function.instruction(&Instruction::Else);
                    self.emit_throw_runtime_error(
                        TYPE_ERROR_NAME,
                        "Cannot mix BigInt and other types",
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    self.release_temp_local(rhs_tag);
                    self.release_temp_local(rhs_payload);
                    self.release_temp_local(lhs_tag);
                    self.release_temp_local(lhs_payload);
                    return Ok(());
                }
                if matches!(op, ArithmeticBinaryOp::Exp) {
                    let lhs_local = self.reserve_temp_local();
                    let rhs_local = self.reserve_temp_local();
                    let output_local = self.reserve_temp_local();

                    self.compile_operand_pair_to_number_locals(
                        NumericBinaryOperator::Arithmetic(*op),
                        lhs,
                        rhs,
                        lhs_local,
                        rhs_local,
                        function,
                    )?;
                    self.emit_number_pow_payload(lhs_local, rhs_local, output_local, function)?;
                    function.instruction(&Instruction::LocalGet(output_local));

                    self.release_temp_local(output_local);
                    self.release_temp_local(rhs_local);
                    self.release_temp_local(lhs_local);
                } else if matches!(op, ArithmeticBinaryOp::Mod) {
                    // `%` needs both operands twice, so each has to be spilled
                    // to a local. The spill slots must be freshly reserved
                    // temporaries: `self.result_local`/`self.scratch_local` are
                    // shared scratch that any nested emission (a string
                    // comparison, a call, a throw propagation) is free to
                    // clobber, which would silently corrupt the already
                    // evaluated left operand while the right one is compiled.
                    let lhs_local = self.reserve_temp_local();
                    let rhs_local = self.reserve_temp_local();
                    self.compile_operand_pair_to_number_locals(
                        NumericBinaryOperator::Arithmetic(*op),
                        lhs,
                        rhs,
                        lhs_local,
                        rhs_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(lhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(lhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(rhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Div);
                    function.instruction(&Instruction::F64Trunc);
                    function.instruction(&Instruction::LocalGet(rhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::F64Mul);
                    function.instruction(&Instruction::F64Sub);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    self.release_temp_local(rhs_local);
                    self.release_temp_local(lhs_local);
                } else {
                    let lhs_local = self.reserve_temp_local();
                    let rhs_local = self.reserve_temp_local();
                    self.compile_operand_pair_to_number_locals(
                        NumericBinaryOperator::Arithmetic(*op),
                        lhs,
                        rhs,
                        lhs_local,
                        rhs_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(lhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    function.instruction(&Instruction::LocalGet(rhs_local));
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        ArithmeticBinaryOp::Add => function.instruction(&Instruction::F64Add),
                        ArithmeticBinaryOp::Sub => function.instruction(&Instruction::F64Sub),
                        ArithmeticBinaryOp::Mul => function.instruction(&Instruction::F64Mul),
                        ArithmeticBinaryOp::Div => function.instruction(&Instruction::F64Div),
                        ArithmeticBinaryOp::Mod => unreachable!(),
                        ArithmeticBinaryOp::Exp => unreachable!(),
                    };
                    function.instruction(&Instruction::I64ReinterpretF64);
                    self.release_temp_local(rhs_local);
                    self.release_temp_local(lhs_local);
                }
            }
            ExprIr::BitwiseNumeric { op, lhs, rhs } => {
                self.compile_bitwise_numeric_to_locals(
                    *op,
                    lhs,
                    rhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::StringFromCharCode { code } => {
                self.compile_string_from_char_code_payload(code, function)?;
            }
            ExprIr::StringCharCodeAt { target, index } => {
                self.compile_string_char_code_at_payload(target, index, function)?;
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.compile_coercive_add_to_locals(
                    lhs,
                    rhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::StringConcat { lhs, rhs } => {
                self.compile_string_concat_payload(lhs, rhs, function)?;
            }
            ExprIr::CompareNumber { op, lhs, rhs } => {
                self.compile_expr_payload(lhs, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                self.compile_expr_payload(rhs, function)?;
                function.instruction(&Instruction::F64ReinterpretI64);
                match op {
                    RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
                    RelationalBinaryOp::LessThanOrEqual => {
                        function.instruction(&Instruction::F64Le)
                    }
                    RelationalBinaryOp::GreaterThan => function.instruction(&Instruction::F64Gt),
                    RelationalBinaryOp::GreaterThanOrEqual => {
                        function.instruction(&Instruction::F64Ge)
                    }
                };
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::CompareValue { op, lhs, rhs } => {
                if lhs.possible_kinds.is_subset_of(
                    KindSet::PRIMITIVE_ONLY
                        .without(ValueKind::String)
                        .without(ValueKind::BigInt),
                ) && rhs.possible_kinds.is_subset_of(
                    KindSet::PRIMITIVE_ONLY
                        .without(ValueKind::String)
                        .without(ValueKind::BigInt),
                ) {
                    self.compile_expr_to_number_payload_nonstring(lhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    self.compile_expr_to_number_payload_nonstring(rhs, function)?;
                    function.instruction(&Instruction::F64ReinterpretI64);
                    match op {
                        RelationalBinaryOp::LessThan => function.instruction(&Instruction::F64Lt),
                        RelationalBinaryOp::LessThanOrEqual => {
                            function.instruction(&Instruction::F64Le)
                        }
                        RelationalBinaryOp::GreaterThan => {
                            function.instruction(&Instruction::F64Gt)
                        }
                        RelationalBinaryOp::GreaterThanOrEqual => {
                            function.instruction(&Instruction::F64Ge)
                        }
                    };
                } else {
                    self.compile_compare_value_i32(*op, lhs, rhs, function)?;
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::StrictEquality { op, lhs, rhs } => {
                self.compile_strict_equality_i32(lhs, rhs, function)?;
                if matches!(op, EqualityBinaryOp::StrictNotEqual) {
                    function.instruction(&Instruction::I32Eqz);
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::LooseEquality { op, lhs, rhs } => {
                if !lhs.possible_kinds.contains(ValueKind::String)
                    && !rhs.possible_kinds.contains(ValueKind::String)
                {
                    self.compile_loose_equality_nonstring_i32(lhs, rhs, function)?;
                } else {
                    self.compile_loose_equality_i32(lhs, rhs, function)?;
                }
                if matches!(op, EqualityBinaryOp::LooseNotEqual) {
                    function.instruction(&Instruction::I32Eqz);
                }
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_truthy_tagged_i32(
                    self.result_tag_local,
                    self.scratch_local,
                    function,
                )?;
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
                    LogicalBinaryOp::Or => {}
                    LogicalBinaryOp::Coalesce => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And => {}
                    LogicalBinaryOp::Or => {
                        self.compile_expr_to_locals(
                            rhs,
                            self.scratch_local,
                            self.result_tag_local,
                            function,
                        )?;
                    }
                    LogicalBinaryOp::Coalesce => {}
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_truthy_i32(condition, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_expr_to_locals(
                    then_expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.compile_expr_to_locals(
                    else_expr,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_expr_payload(rhs, function)?;
            }
            ExprIr::MaterializeBinding { name, value, body } => {
                self.compile_materialized_binding_to_locals(
                    name,
                    value,
                    body,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::ArrayDestructure {
                value,
                pattern,
                evaluation,
            } => {
                self.compile_array_destructure_to_locals(
                    value,
                    pattern,
                    *evaluation,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                self.compile_object_destructure_to_locals(
                    value,
                    pattern,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(
                    name,
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::SpreadArgument(_) => {
                return Err(EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: spread argument outside call",
                ));
            }
            ExprIr::AssertSameValue {
                actual,
                expected,
                message,
            } => {
                self.emit_assert_same_value(actual, expected, message, function)?;
                function.instruction(&Instruction::I64Const(0));
            }
            ExprIr::RuntimeThrow { name, message } => {
                self.emit_throw_runtime_error(
                    name.as_str(),
                    message,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                static_regexp_compilation,
            } => {
                self.emit_indirect_call(
                    callee,
                    this_arg.as_deref(),
                    args,
                    static_regexp_compilation.as_ref(),
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::Construct {
                callee,
                args,
                static_regexp_compilation,
            } => {
                self.emit_construct(
                    callee,
                    args,
                    static_regexp_compilation.as_ref(),
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::ClassDefinition(class) => {
                self.compile_class_definition_payload(class, function)?;
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.emit_method_call(
                    receiver,
                    key,
                    args,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.emit_instanceof_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::In { lhs, rhs } => {
                self.emit_in_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
            }
            ExprIr::PropertyRead { target, key } => {
                self.compile_property_read_to_locals(
                    target,
                    key,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                self.compile_optional_property_chain_to_locals(
                    target,
                    chain,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::SuperConstruct { args } => {
                let ctor_payload_local = self.reserve_temp_local();
                let ctor_tag_local = self.reserve_temp_local();
                let new_target_payload_local = self.reserve_temp_local();
                let new_target_tag_local = self.reserve_temp_local();
                self.emit_prepare_super_construct_to_locals(
                    new_target_payload_local,
                    new_target_tag_local,
                    ctor_payload_local,
                    ctor_tag_local,
                    function,
                )?;
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_super_construct_with_prepared_arg_vector(
                    ctor_payload_local,
                    ctor_tag_local,
                    new_target_payload_local,
                    new_target_tag_local,
                    argc_local,
                    argv_local,
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.result_local));
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(new_target_tag_local);
                self.release_temp_local(new_target_payload_local);
                self.release_temp_local(ctor_tag_local);
                self.release_temp_local(ctor_payload_local);
            }
            ExprIr::SuperPropertyRead { key, receiver } => {
                self.compile_super_property_read_to_locals(
                    key,
                    receiver,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::SuperPropertyWrite {
                key,
                receiver,
                value,
                strictness,
            } => {
                self.compile_super_property_write_to_locals(
                    key,
                    receiver,
                    value,
                    *strictness,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::SuperPropertyMutation(mutation) => {
                self.compile_super_property_mutation_to_locals(
                    mutation,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateRead {
                target,
                private_name_id,
            } => {
                self.compile_private_read_to_locals(
                    target,
                    *private_name_id,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateWrite {
                target,
                private_name_id,
                value,
            } => {
                self.compile_private_write_to_locals(
                    target,
                    *private_name_id,
                    value,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
            ExprIr::PrivateIn {
                private_name_id,
                rhs,
            } => {
                let rhs_payload_local = self.reserve_temp_local();
                let rhs_tag_local = self.reserve_temp_local();
                let token_local = self.reserve_temp_local();
                let result_local = self.reserve_temp_local();

                self.compile_expr_to_locals(rhs, rhs_payload_local, rhs_tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    rhs_payload_local,
                    rhs_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(result_local));
                self.emit_is_heap_object_like_tag_i32(rhs_tag_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_private_name_token_to_local(*private_name_id, token_local, function)?;
                self.emit_private_brand_has_i32(
                    rhs_payload_local,
                    token_local,
                    result_local,
                    function,
                );
                function.instruction(&Instruction::Else);
                self.emit_throw_runtime_error(
                    "TypeError",
                    "right-hand side of private in is not an object",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(result_local);
                self.release_temp_local(token_local);
                self.release_temp_local(rhs_tag_local);
                self.release_temp_local(rhs_payload_local);
            }
        }
        Ok(())
    }

    pub(crate) fn compile_string_from_char_code_payload(
        &mut self,
        code: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let code_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();

        self.compile_expr_to_number_payload(code, function)?;
        function.instruction(&Instruction::LocalSet(code_local));
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_uint32_i64_from_number_payload(code_local, code_local, function);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(0xffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(code_local));

        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(0x800));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_from_local(len_local, function)?;
        function.instruction(&Instruction::LocalSet(offset_local));

        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xC0));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(0x3f));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0xE0));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x3f));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(code_local));
        function.instruction(&Instruction::I64Const(0x3f));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x80));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_pack_string_payload(offset_local, len_local, function);

        self.release_temp_local(offset_local);
        self.release_temp_local(len_local);
        self.release_temp_local(code_local);
        Ok(())
    }

    pub(crate) fn compile_string_char_code_at_payload(
        &mut self,
        target: &TypedExpr,
        index: &TypedExpr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_local = self.reserve_temp_local();
        let string_offset_local = self.reserve_temp_local();
        let string_byte_len_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_payload_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();
        let byte_index_local = self.reserve_temp_local();
        let unit_index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let unit_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();

        self.compile_expr_payload(target, function)?;
        function.instruction(&Instruction::LocalSet(string_local));
        self.emit_unpack_string_payload(
            string_local,
            string_offset_local,
            string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            string_offset_local,
            string_byte_len_local,
            string_len_local,
            function,
        );

        self.compile_expr_to_number_payload(index, function)?;
        function.instruction(&Instruction::LocalSet(index_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::I64TruncSatF64S);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(string_byte_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, byte_index_local, byte_local, function);
        self.emit_decode_utf8_scalar_at_index(
            string_offset_local,
            byte_index_local,
            string_byte_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_advance_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xFFFF));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(temp_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::LocalGet(temp_local));
        function.instruction(&Instruction::I64Const(0x3FF));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(byte_index_local));
        function.instruction(&Instruction::LocalGet(unit_index_local));
        function.instruction(&Instruction::LocalGet(unit_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(unit_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));

        for local in [
            temp_local,
            unit_advance_local,
            advance_local,
            codepoint_local,
            byte_local,
            unit_index_local,
            byte_index_local,
            result_local,
            index_local,
            index_payload_local,
            string_len_local,
            string_byte_len_local,
            string_offset_local,
            string_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(crate) fn compile_optional_property_chain_to_locals(
        &mut self,
        target: &TypedExpr,
        chain: &[OptionalChainOperationIr],
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let reference_receiver_local = self.reserve_temp_local();
        let reference_receiver_tag_local = self.reserve_temp_local();
        let has_reference_local = self.reserve_temp_local();
        let call_this_local = self.reserve_temp_local();
        let call_this_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(target, receiver_local, receiver_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            receiver_local,
            receiver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_reference_local));
        function.instruction(&Instruction::LocalGet(receiver_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::LocalSet(tag_local));

        // Every property access uses runtime tag dispatch. Besides handling
        // the result of an earlier operation, this is important for optional
        // access on a statically primitive base: the property lookup still
        // follows that primitive's boxed/prototype semantics after the
        // nullish check.
        let dynamic_receiver = TypedExpr::from_info(
            ValueInfo {
                kind: ValueKind::Dynamic,
                possible_kinds: KindSet::all_runtime_tags(),
                heap_shape: None,
                function_targets: BTreeSet::new(),
            },
            ExprIr::Undefined,
        );

        // Each block is one contiguous short-circuit segment. A grouped call
        // starts a fresh segment so a short in the preceding segment resumes
        // at that call, while a short inside the fresh segment still skips all
        // of its later operations. Optional computed keys are emitted after
        // their short-circuit test; ordinary computed keys retain the usual
        // key-evaluation-before-RequireObjectCoercible order.
        self.open_frame(ControlFrameKind::Block, function);
        for operation in chain {
            match operation {
                OptionalChainOperationIr::Property { key, shorted } => {
                    if *shorted {
                        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                        self.open_frame(ControlFrameKind::If, function);
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(receiver_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(receiver_tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(has_reference_local));
                        // Leave both this `if` and the active chain segment.
                        function.instruction(&Instruction::Br(1));
                        self.pop_control(ControlFrameKind::If);
                        function.instruction(&Instruction::End);
                    } else if !matches!(key, PropertyKeyIr::StringExpr(_)) {
                        // Only `?.` short-circuits. A later ordinary property
                        // access still performs RequireObjectCoercible.
                        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                        self.open_frame(ControlFrameKind::If, function);
                        self.emit_throw_runtime_error(
                            TYPE_ERROR_NAME,
                            "Cannot read properties of null or undefined",
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.emit_propagate_throw_from_locals_if_needed(
                            payload_local,
                            tag_local,
                            function,
                        )?;
                        self.pop_control(ControlFrameKind::If);
                        function.instruction(&Instruction::End);
                    }

                    // Preserve the Reference base for a following call. The
                    // property key/getter can then overwrite the current value
                    // without losing the eventual call receiver.
                    function.instruction(&Instruction::LocalGet(receiver_local));
                    function.instruction(&Instruction::LocalSet(reference_receiver_local));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::LocalSet(reference_receiver_tag_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(has_reference_local));

                    self.compile_property_read_from_locals(
                        &dynamic_receiver,
                        key,
                        receiver_local,
                        receiver_tag_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        payload_local,
                        tag_local,
                        function,
                    )?;

                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::LocalSet(receiver_local));
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::LocalSet(receiver_tag_local));
                }
                OptionalChainOperationIr::PrivateProperty {
                    private_name_id,
                    shorted,
                } => {
                    if *shorted {
                        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                        self.open_frame(ControlFrameKind::If, function);
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(receiver_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(receiver_tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(has_reference_local));
                        function.instruction(&Instruction::Br(1));
                        self.pop_control(ControlFrameKind::If);
                        function.instruction(&Instruction::End);
                    }

                    function.instruction(&Instruction::LocalGet(receiver_local));
                    function.instruction(&Instruction::LocalSet(reference_receiver_local));
                    function.instruction(&Instruction::LocalGet(receiver_tag_local));
                    function.instruction(&Instruction::LocalSet(reference_receiver_tag_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::LocalSet(has_reference_local));

                    let receiver_name = "$optional.private.receiver";
                    self.push_scope();
                    self.binding_scopes
                        .last_mut()
                        .expect("binding scope stack must exist")
                        .insert(
                            receiver_name.to_string(),
                            BindingStorage::Dynamic {
                                tag_local: receiver_tag_local,
                                payload_local: receiver_local,
                            },
                        );
                    let receiver = TypedExpr::from_info(
                        dynamic_receiver.value_info(),
                        ExprIr::Identifier(receiver_name.to_string()),
                    );
                    let result = self.compile_private_read_to_locals(
                        &receiver,
                        *private_name_id,
                        payload_local,
                        tag_local,
                        function,
                    );
                    self.pop_scope();
                    result?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        payload_local,
                        tag_local,
                        function,
                    )?;

                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::LocalSet(receiver_local));
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::LocalSet(receiver_tag_local));
                }
                OptionalChainOperationIr::Call {
                    args,
                    receiver,
                    shorted,
                    boundary_before,
                } => {
                    if *boundary_before {
                        self.pop_control(ControlFrameKind::Block);
                        function.instruction(&Instruction::End);
                        self.open_frame(ControlFrameKind::Block, function);
                    }

                    if *shorted {
                        // Optional call tests the callee before evaluating any
                        // arguments and exits the active contiguous segment.
                        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
                        self.open_frame(ControlFrameKind::If, function);
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(receiver_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(receiver_tag_local));
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(has_reference_local));
                        function.instruction(&Instruction::Br(1));
                        self.pop_control(ControlFrameKind::If);
                        function.instruction(&Instruction::End);
                    }

                    // Arguments precede the callable check for a non-nullish
                    // callee, including ordinary (non-shorted) call operations.
                    let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;

                    match receiver {
                        OptionalChainCallReceiverIr::ReferenceOrUndefined => {
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::LocalSet(call_this_local));
                            function.instruction(&Instruction::I64Const(
                                ValueKind::Undefined.tag() as i64,
                            ));
                            function.instruction(&Instruction::LocalSet(call_this_tag_local));
                            function.instruction(&Instruction::LocalGet(has_reference_local));
                            function.instruction(&Instruction::I64Const(0));
                            function.instruction(&Instruction::I64Ne);
                            self.open_frame(ControlFrameKind::If, function);
                            function.instruction(&Instruction::LocalGet(reference_receiver_local));
                            function.instruction(&Instruction::LocalSet(call_this_local));
                            function
                                .instruction(&Instruction::LocalGet(reference_receiver_tag_local));
                            function.instruction(&Instruction::LocalSet(call_this_tag_local));
                            self.pop_control(ControlFrameKind::If);
                            function.instruction(&Instruction::End);
                        }
                        OptionalChainCallReceiverIr::CurrentThis => {
                            self.compile_this_to_locals(
                                call_this_local,
                                call_this_tag_local,
                                function,
                            )?;
                        }
                    }

                    self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
                        receiver_local,
                        receiver_tag_local,
                        call_this_local,
                        call_this_tag_local,
                        argc_local,
                        argv_local,
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.set_completion_kind(CompletionKind::Normal, function);
                    self.release_temp_local(argv_local);
                    self.release_temp_local(argc_local);

                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(has_reference_local));
                    function.instruction(&Instruction::LocalGet(payload_local));
                    function.instruction(&Instruction::LocalSet(receiver_local));
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::LocalSet(receiver_tag_local));
                }
            }
        }
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);

        self.release_temp_local(call_this_tag_local);
        self.release_temp_local(call_this_local);
        self.release_temp_local(has_reference_local);
        self.release_temp_local(reference_receiver_tag_local);
        self.release_temp_local(reference_receiver_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_local);
        Ok(())
    }

    pub(crate) fn emit_regexp_program_slots(
        &mut self,
        object_local: u32,
        program: Option<&RegExpProgram>,
        function: &mut Function,
    ) {
        let (
            program_ptr,
            instruction_count,
            capture_count,
            split_count,
            repeatable_split_count,
            named_group_table_ptr,
        ) = program
            .map(|program| {
                let reference = self.strings.regexp_program(program);
                (
                    reference.ptr,
                    reference.instruction_count,
                    reference.capture_count,
                    reference.split_count,
                    reference.repeatable_split_count,
                    reference.named_group_table_ptr,
                )
            })
            .unwrap_or((0, 0, 0, 0, 0, 0));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_PROGRAM_PTR_OFFSET,
            program_ptr as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_PROGRAM_INSTRUCTION_COUNT_OFFSET,
            instruction_count as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
            capture_count as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
            split_count as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
            repeatable_split_count as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_NAMED_GROUP_TABLE_PTR_OFFSET,
            named_group_table_ptr as u64,
            function,
        );
    }

    /// Installs the compiled RegExp program for a `(source, flags)` pair that is
    /// only known at run time, by looking the pair up **by string value** in the
    /// AOT-built runtime program table.
    ///
    /// # The lookup is total, and that is the point
    ///
    /// Four outcomes, and every one of them is now reachable code:
    ///
    /// * **`Program` row** — install the six program slots. Unchanged.
    /// * **`Rejected` row** — the compile-time RegExp compiler saw this exact
    ///   pattern and answered `InvalidSyntax`, so this is a spec SyntaxError
    ///   (`RegExpInitialize` step 3.b). Before batch 7 the row did not exist at
    ///   all (`data.rs` `continue`d past every compile failure), the loop below
    ///   had no else arm, and the caller went on to publish a live RegExp whose
    ///   `instruction_count` is 0 — `assert.throws(SyntaxError, () =>
    ///   r.compile("(?<x>a)(?<x>b)"))` therefore measured *no throw at all*.
    ///   That is the one Bug-outcome failure in the batch-7 sweep.
    /// * **`Unsupported` row** — the pattern is legal ECMAScript and Lila's
    ///   RegExp compiler cannot build a program for it yet. Behaves exactly like
    ///   a total miss; throwing here would invent a SyntaxError for a pattern
    ///   the spec accepts.
    /// * **total miss** — the pair is genuinely not in the table, i.e. the
    ///   pattern or the flags string was computed at run time and never named as
    ///   a literal anywhere. The slots stay zeroed and the caller proceeds.
    ///
    /// # Why a total miss deliberately does *not* throw
    ///
    /// The obvious symmetry ("no row, no program, so throw") is wrong here, and
    /// the reason is measured rather than argued. A zeroed program does not mean
    /// `exec` silently answers `null`: `emit_regexp_prototype_exec_from_locals`
    /// tries the compiled program first, then falls through to
    /// `emit_regexp_exec_simple_from_locals` — a real fallback matcher covering
    /// `.`, escapes, two-alternative patterns and the `g`/`i`/`y`/`u` flags —
    /// and only if *that* declines does it throw
    /// `TypeError: RegExp.prototype.exec unsupported pattern`. Measured on this
    /// head: `new RegExp("(?<" + "y>c)")` builds an object with the right
    /// `source` and its `test` call throws that TypeError; it does not answer
    /// `false`. So throwing SyntaxError on every miss would convert answers the
    /// fallback matcher gets *right* into spurious SyntaxErrors, including for
    /// the common `new RegExp("a", computedFlags)` shape whose flags string is
    /// simply not a script literal. That trade is a regression, not a fix.
    ///
    /// The half this closes is therefore stated exactly: **a pattern the
    /// compiler has seen and rejected now throws**; a valid pattern the compiler
    /// has never seen keeps today's behaviour. Widening the first half is a
    /// candidate-collection problem (see `runtime_regexp_argument_literals`),
    /// not an emitter problem, and that is where it was widened.
    pub(crate) fn emit_runtime_regexp_program_slots(
        &mut self,
        object_local: u32,
        source_payload_local: u32,
        flags_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_regexp_program_slots(object_local, None, function);
        if self.strings.runtime_regexp_program_count == 0 {
            return Ok(());
        }
        // The stride and every offset below come from the word indices
        // `StringPool::append_runtime_regexp_program_table` assigns through, so
        // this reader cannot fall out of step with that writer. They used to be
        // literal `72`/`64`/`16`…`56` here, agreeing with the writer's array
        // order only by inspection.
        let index_local = self.reserve_temp_local();
        let record_ptr_local = self.reserve_temp_local();
        let candidate_payload_local = self.reserve_temp_local();
        let entry_kind_local = self.reserve_temp_local();

        // `RUNTIME_REGEXP_ENTRY_KIND_PROGRAM` is 0 and is also what a total miss
        // leaves behind, so the post-loop test asks only about `Rejected`. The
        // two are deliberately not merged: see the doc comment.
        function.instruction(&Instruction::I64Const(
            RuntimeRegExpEntryKind::Program.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(entry_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(
            self.strings.runtime_regexp_program_count as i64,
        ));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(
            self.strings.runtime_regexp_program_table_ptr as i64,
        ));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(RUNTIME_REGEXP_RECORD_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(record_ptr_local));
        function.instruction(&Instruction::LocalGet(record_ptr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(
            runtime_regexp_record_offset(RUNTIME_REGEXP_RECORD_SOURCE_WORD),
        )));
        function.instruction(&Instruction::LocalSet(candidate_payload_local));
        self.emit_string_payload_equality_i32(
            source_payload_local,
            candidate_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(record_ptr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(
            runtime_regexp_record_offset(RUNTIME_REGEXP_RECORD_FLAGS_WORD),
        )));
        function.instruction(&Instruction::LocalSet(candidate_payload_local));
        self.emit_string_payload_equality_i32(
            flags_payload_local,
            candidate_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        // The row matched. Read its discriminant BEFORE deciding what to do
        // with it — this is the else arm the loop never had.
        function.instruction(&Instruction::LocalGet(record_ptr_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(Self::memarg8(
            runtime_regexp_record_offset(RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD),
        )));
        function.instruction(&Instruction::LocalSet(entry_kind_local));
        function.instruction(&Instruction::LocalGet(entry_kind_local));
        function.instruction(&Instruction::I64Const(
            RuntimeRegExpEntryKind::Program.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (record_word, heap_offset) in [
            (
                RUNTIME_REGEXP_RECORD_PROGRAM_PTR_WORD,
                HEAP_REGEXP_PROGRAM_PTR_OFFSET,
            ),
            (
                RUNTIME_REGEXP_RECORD_INSTRUCTION_COUNT_WORD,
                HEAP_REGEXP_PROGRAM_INSTRUCTION_COUNT_OFFSET,
            ),
            (
                RUNTIME_REGEXP_RECORD_CAPTURE_COUNT_WORD,
                HEAP_REGEXP_PROGRAM_CAPTURE_COUNT_OFFSET,
            ),
            (
                RUNTIME_REGEXP_RECORD_SPLIT_COUNT_WORD,
                HEAP_REGEXP_PROGRAM_SPLIT_COUNT_OFFSET,
            ),
            (
                RUNTIME_REGEXP_RECORD_REPEATABLE_SPLIT_COUNT_WORD,
                HEAP_REGEXP_PROGRAM_REPEATABLE_SPLIT_COUNT_OFFSET,
            ),
            (
                RUNTIME_REGEXP_RECORD_NAMED_GROUP_TABLE_PTR_WORD,
                HEAP_REGEXP_NAMED_GROUP_TABLE_PTR_OFFSET,
            ),
        ] {
            function.instruction(&Instruction::LocalGet(record_ptr_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(Self::memarg8(
                runtime_regexp_record_offset(record_word),
            )));
            function.instruction(&Instruction::LocalSet(candidate_payload_local));
            self.store_i64_local_at_offset(
                object_local,
                heap_offset,
                candidate_payload_local,
                function,
            );
        }
        function.instruction(&Instruction::End);
        // Leaves the search with `entry_kind_local` holding this row's kind.
        // Branch depth is unchanged from before the discriminant existed: at
        // this point the enclosing labels are flags-If (0), source-If (1),
        // Loop (2), Block (3).
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

        // A `Rejected` row is the compile-time compiler's `InvalidSyntax`
        // verdict on this exact pattern, reaching run time. `Unsupported` rows
        // and total misses both fall through without throwing — the first
        // because the pattern is legal, the second because the runtime fallback
        // matcher may still answer it correctly. The doc comment says why at
        // length; do not "simplify" this into `!= PROGRAM`.
        //
        // Which kinds throw is decided ONCE, by an exhaustive match over the
        // closed domain (`RuntimeRegExpEntryKind::throws_syntax_error`), and the
        // comparison chain is derived from `ALL` rather than transcribed here.
        // Before that, this reader tested a raw `u64` against two of the three
        // constants, so a fourth kind would have compiled cleanly and fallen
        // through as a silent miss — the very class the closed writer type was
        // introduced to remove. Today exactly one kind throws, so this emits the
        // same four instructions it always did.
        let throwing_kind_words = RuntimeRegExpEntryKind::ALL
            .iter()
            .copied()
            .filter(|kind| kind.throws_syntax_error())
            .map(RuntimeRegExpEntryKind::word)
            .collect::<Vec<_>>();
        // Bound rather than `?`-ed. The four `release_temp_local` calls below
        // are the temp-local stack's only unwind, and returning past them makes
        // the *next* release trip `release_temp_local`'s `assert_eq!`, replacing
        // a readable `EmitError` with an unrelated compiler panic. The `End`
        // must be emitted on the error path too, or `code_sink`'s depth
        // accounting is left unbalanced as well.
        let mut thrown = Ok(());
        if !throwing_kind_words.is_empty() {
            for (position, word) in throwing_kind_words.iter().enumerate() {
                function.instruction(&Instruction::LocalGet(entry_kind_local));
                function.instruction(&Instruction::I64Const(*word as i64));
                function.instruction(&Instruction::I64Eq);
                if position > 0 {
                    function.instruction(&Instruction::I32Or);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            thrown = self.emit_throw_runtime_error_to_active_handler(
                SYNTAX_ERROR_NAME,
                "Invalid regular expression pattern",
                self.result_local,
                self.result_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(entry_kind_local);
        self.release_temp_local(candidate_payload_local);
        self.release_temp_local(record_ptr_local);
        self.release_temp_local(index_local);
        thrown
    }

    fn compile_regexp_literal_payload(
        &mut self,
        source: &str,
        flags: &str,
        program: Option<&RegExpProgram>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(REGEXP_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_REGEXP,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            self.strings.payload(source) as u64,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            self.strings.payload(flags) as u64,
            function,
        );
        self.emit_regexp_program_slots(object_local, program, function);

        function.instruction(&Instruction::I64Const(self.strings.payload("lastIndex")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data_with_configurable(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            true,
            false,
            false,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn compile_expr_to_binding(
        &mut self,
        expr: &TypedExpr,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(expr, payload_local, tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        match storage {
            BindingStorage::Fixed { .. } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                self.store_payload_to_binding(storage, function);
            }
            BindingStorage::Dynamic {
                tag_local: binding_tag_local,
                payload_local: binding_payload_local,
            } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(binding_payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::LocalSet(binding_tag_local));
            }
            BindingStorage::EnvSlot { .. } => {
                self.write_binding_from_locals(storage, payload_local, tag_local, function);
            }
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    fn compile_materialized_binding_to_locals(
        &mut self,
        name: &str,
        value: &TypedExpr,
        body: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let binding_payload_local = self.reserve_temp_local();
        let binding_tag_local = self.reserve_temp_local();
        self.compile_expr_to_locals(value, binding_payload_local, binding_tag_local, function)?;
        self.emit_propagate_throw_from_locals_if_needed(
            binding_payload_local,
            binding_tag_local,
            function,
        )?;
        self.push_scope();
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(
                name.to_string(),
                BindingStorage::Dynamic {
                    tag_local: binding_tag_local,
                    payload_local: binding_payload_local,
                },
            );
        self.compile_expr_to_locals(body, payload_local, tag_local, function)?;
        self.pop_scope();
        self.release_temp_local(binding_tag_local);
        self.release_temp_local(binding_payload_local);
        Ok(())
    }

    pub(crate) fn compile_expr_to_locals(
        &mut self,
        expr: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if matches!(
            expr.expr,
            ExprIr::Undefined | ExprIr::ArrayHole | ExprIr::Null
        ) {
            self.emit_undefined_payload(function);
            function.instruction(&Instruction::LocalSet(payload_local));
            let value_kind = if matches!(expr.expr, ExprIr::Null) {
                ValueKind::Null
            } else {
                ValueKind::Undefined
            };
            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            return Ok(());
        }

        if let ExprIr::BigInt(value) = &expr.expr {
            if value.requires_arbitrary_precision_storage {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
                function.instruction(&Instruction::LocalSet(tag_local));
                return Ok(());
            }
        }

        if matches!(&expr.expr, ExprIr::This)
            || matches!(&expr.expr, ExprIr::Identifier(name) if name == LEXICAL_THIS_NAME)
        {
            self.compile_this_to_locals(payload_local, tag_local, function)?;
            return Ok(());
        }

        if let ExprIr::Identifier(name) = &expr.expr {
            if self.should_read_script_global_property(name) {
                let storage = self.lookup_binding(name);
                self.emit_global_property_read(name, payload_local, tag_local, function)?;
                if let Some(storage) = storage {
                    function.instruction(&Instruction::LocalGet(tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
                    function.instruction(&Instruction::End);
                }
                return Ok(());
            }
        }

        let identifier_storage_is_runtime_dynamic = matches!(
            &expr.expr,
            ExprIr::Identifier(name)
                if matches!(
                    self.lookup_binding(name),
                    Some(BindingStorage::Dynamic { .. } | BindingStorage::EnvSlot { .. })
                )
        );
        if expr.possible_kinds.is_singleton()
            && !expr_result_tag_is_runtime_dynamic(&expr.expr)
            && !identifier_storage_is_runtime_dynamic
        {
            self.compile_expr_payload(expr, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(expr.kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            return Ok(());
        }

        match &expr.expr {
            ExprIr::This => {
                self.compile_this_to_locals(payload_local, tag_local, function)?;
            }
            ExprIr::Arguments => {
                let storage = self.lookup_binding(LEXICAL_ARGUMENTS_NAME).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing `arguments` binding",
                    )
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
            }
            ExprIr::NewTarget => {
                self.compile_new_target_to_locals(payload_local, tag_local, function)?;
            }
            ExprIr::SpecOperation {
                operation,
                operands,
            } => {
                self.compile_spec_operation_to_locals(
                    *operation,
                    operands,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::UnaryBitwiseNumeric { op, expr } => {
                self.compile_unary_bitwise_numeric_to_locals(
                    *op,
                    expr,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::Identifier(name) => {
                if name == LEXICAL_THIS_NAME {
                    self.compile_this_to_locals(payload_local, tag_local, function)?;
                    return Ok(());
                }
                if name == GLOBAL_THIS_NAME {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    return Ok(());
                }
                if self.should_read_script_global_property(name) {
                    let storage = self.lookup_binding(name);
                    self.emit_global_property_read(name, payload_local, tag_local, function)?;
                    if let Some(storage) = storage {
                        function.instruction(&Instruction::LocalGet(tag_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
                        function.instruction(&Instruction::End);
                    }
                    return Ok(());
                }
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.emit_global_property_read(name, payload_local, tag_local, function)?;
                    return Ok(());
                }
                if let Some(host) = self.compiled_host_builtin_by_name(name) {
                    let meta = self.functions.get(&host.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown host function `{}`",
                            host.as_str()
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
            }
            ExprIr::GlobalPropertyRead { name } => {
                self.emit_global_property_read(name, payload_local, tag_local, function)?;
            }
            ExprIr::GlobalIdentifierRead { name } => {
                self.emit_global_identifier_read(name, payload_local, tag_local, function)?;
            }
            ExprIr::AssignIdentifier { name, value } => {
                if self.is_script_global_binding(name) && self.lookup_binding(name).is_none() {
                    self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    self.emit_global_property_write(name, payload_local, tag_local, function)?;
                    return Ok(());
                }
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: unbound identifier `{name}`"
                    ))
                })?;
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.write_binding_from_locals(storage, payload_local, tag_local, function);
                self.mirror_binding_to_global_object(name, storage, function)?;
            }
            ExprIr::GlobalPropertyWrite {
                name,
                value,
                strictness,
                ..
            } => {
                self.compile_expr_to_locals(value, payload_local, tag_local, function)?;
                self.emit_propagate_throw_from_locals_if_needed(
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.emit_reference_global_property_write(
                    name,
                    payload_local,
                    tag_local,
                    *strictness,
                    function,
                )?;
            }
            ExprIr::PropertyRead { target, key } => {
                self.compile_property_read_to_locals(
                    target,
                    key,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                self.compile_optional_property_chain_to_locals(
                    target,
                    chain,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::PropertyWrite {
                target,
                key,
                value,
                strictness,
            } => {
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_write_to_locals(
                        target,
                        key,
                        value,
                        payload_local,
                        tag_local,
                        function,
                    )
                })?;
            }
            ExprIr::PropertyUpdate {
                target,
                key,
                op,
                return_mode,
                value_kind,
                strictness,
            } => {
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_update_to_locals(
                        target,
                        key,
                        *op,
                        *return_mode,
                        *value_kind,
                        payload_local,
                        tag_local,
                        function,
                    )
                })?;
            }
            ExprIr::PropertyCompoundAssign {
                target,
                key,
                op,
                value,
                strictness,
            } => {
                self.with_reference_strictness(*strictness, function, |emitter, function| {
                    emitter.compile_property_compound_assign_to_locals(
                        target,
                        key,
                        *op,
                        value,
                        payload_local,
                        tag_local,
                        function,
                    )
                })?;
            }
            ExprIr::SuperConstruct { args } => {
                let ctor_payload_local = self.reserve_temp_local();
                let ctor_tag_local = self.reserve_temp_local();
                let new_target_payload_local = self.reserve_temp_local();
                let new_target_tag_local = self.reserve_temp_local();
                self.emit_prepare_super_construct_to_locals(
                    new_target_payload_local,
                    new_target_tag_local,
                    ctor_payload_local,
                    ctor_tag_local,
                    function,
                )?;
                let (argc_local, argv_local) = self.emit_call_args_vector(args, function)?;
                self.emit_super_construct_with_prepared_arg_vector(
                    ctor_payload_local,
                    ctor_tag_local,
                    new_target_payload_local,
                    new_target_tag_local,
                    argc_local,
                    argv_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                self.release_temp_local(argv_local);
                self.release_temp_local(argc_local);
                self.release_temp_local(new_target_tag_local);
                self.release_temp_local(new_target_payload_local);
                self.release_temp_local(ctor_tag_local);
                self.release_temp_local(ctor_payload_local);
            }
            ExprIr::SuperPropertyRead { key, receiver } => self
                .compile_super_property_read_to_locals(
                    key,
                    receiver,
                    payload_local,
                    tag_local,
                    function,
                )?,
            ExprIr::SuperPropertyWrite {
                key,
                receiver,
                value,
                strictness,
            } => self.compile_super_property_write_to_locals(
                key,
                receiver,
                value,
                *strictness,
                payload_local,
                tag_local,
                function,
            )?,
            ExprIr::SuperPropertyMutation(mutation) => self
                .compile_super_property_mutation_to_locals(
                    mutation,
                    payload_local,
                    tag_local,
                    function,
                )?,
            ExprIr::LogicalShortCircuit { op, lhs, rhs } => {
                self.compile_expr_to_locals(lhs, payload_local, tag_local, function)?;
                match op {
                    LogicalBinaryOp::Coalesce => {
                        self.compile_nullish_tagged_i32(tag_local, function)?;
                    }
                    LogicalBinaryOp::And | LogicalBinaryOp::Or => {
                        self.compile_truthy_tagged_i32(tag_local, payload_local, function)?;
                    }
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                match op {
                    LogicalBinaryOp::And => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                    LogicalBinaryOp::Or => {}
                    LogicalBinaryOp::Coalesce => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                }
                function.instruction(&Instruction::Else);
                match op {
                    LogicalBinaryOp::And => {}
                    LogicalBinaryOp::Or => {
                        self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
                    }
                    LogicalBinaryOp::Coalesce => {}
                }
                function.instruction(&Instruction::End);
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_truthy_i32(condition, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_expr_to_locals(then_expr, payload_local, tag_local, function)?;
                function.instruction(&Instruction::Else);
                self.compile_expr_to_locals(else_expr, payload_local, tag_local, function)?;
                function.instruction(&Instruction::End);
            }
            ExprIr::Comma { lhs, rhs } => {
                self.compile_expr_to_locals(
                    lhs,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;
            }
            ExprIr::MaterializeBinding { name, value, body } => {
                self.compile_materialized_binding_to_locals(
                    name,
                    value,
                    body,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::ArrayDestructure {
                value,
                pattern,
                evaluation,
            } => {
                self.compile_array_destructure_to_locals(
                    value,
                    pattern,
                    *evaluation,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                self.compile_object_destructure_to_locals(
                    value,
                    pattern,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::UpdateIdentifier { .. }
            | ExprIr::GlobalPropertyUpdate { .. }
            | ExprIr::GlobalPropertyCompoundAssign { .. } => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                if expr_result_tag_is_runtime_dynamic(&expr.expr) {
                    function.instruction(&Instruction::LocalGet(self.result_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(expr.kind.tag() as i64));
                }
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {
                self.compile_coercive_binary_number_to_locals(
                    *op,
                    lhs,
                    rhs,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::BitwiseNumeric { op, lhs, rhs } => {
                self.compile_bitwise_numeric_to_locals(
                    *op,
                    lhs,
                    rhs,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::BinaryNumber { .. } => {
                self.compile_expr_payload(expr, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                if expr.kind == ValueKind::BigInt {
                    // The BigInt path reports inline vs heap-backed at runtime.
                    function.instruction(&Instruction::LocalGet(self.result_tag_local));
                } else {
                    function.instruction(&Instruction::I64Const(expr.kind.tag() as i64));
                }
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.compile_coercive_add_to_locals(lhs, rhs, payload_local, tag_local, function)?;
            }
            ExprIr::CallNamed { name, args } => {
                self.emit_call(name, args, payload_local, tag_local, function)?;
            }
            ExprIr::SpreadArgument(_) => {
                return Err(EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: spread argument outside call",
                ));
            }
            ExprIr::AssertSameValue {
                actual,
                expected,
                message,
            } => {
                self.emit_assert_same_value(actual, expected, message, function)?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::RuntimeThrow { name, message } => {
                self.emit_throw_runtime_error(
                    name.as_str(),
                    message,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::JsonParseStaticReviver { value, reviver } => {
                self.compile_json_static_reviver_to_locals(
                    value,
                    reviver,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                static_regexp_compilation,
            } => {
                self.emit_indirect_call(
                    callee,
                    this_arg.as_deref(),
                    args,
                    static_regexp_compilation.as_ref(),
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::Construct {
                callee,
                args,
                static_regexp_compilation,
            } => {
                self.emit_construct(
                    callee,
                    args,
                    static_regexp_compilation.as_ref(),
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.emit_method_call(receiver, key, args, payload_local, tag_local, function)?;
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.emit_instanceof_i32(lhs, rhs, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            ExprIr::PrivateRead {
                target,
                private_name_id,
            } => {
                self.compile_private_read_to_locals(
                    target,
                    *private_name_id,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            ExprIr::PrivateWrite {
                target,
                private_name_id,
                value,
            } => {
                self.compile_private_write_to_locals(
                    target,
                    *private_name_id,
                    value,
                    payload_local,
                    tag_local,
                    function,
                )?;
            }
            _ => {
                return Err(EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: dynamic expression form {:?}",
                    expr.expr
                )));
            }
        }
        Ok(())
    }
}
