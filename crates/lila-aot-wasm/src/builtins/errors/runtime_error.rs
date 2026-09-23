use super::*;

impl FunctionBuilder<'_> {
    /// Allocate the error object for a runtime-thrown error.
    ///
    /// `message` is defined from the message, not from the name. That reads as
    /// a tautology; it is the repair. This function used to define `message`
    /// from `self.strings.payload(name)` and spell its message parameter
    /// `_message`, so not even an unused-parameter warning mentioned it, and
    /// every error the runtime threw reported `e.message === e.name`:
    ///
    /// ```text
    /// try { null.x } catch (e) { print(e.name); print(e.message); }
    /// // TypeError / TypeError     (before)
    /// // TypeError / Cannot read properties of null or undefined   (after)
    /// ```
    ///
    /// The repair is one token here and could not land alone.
    /// `StringPool::payload` takes `&self`, cannot extend the pool during
    /// emission, and panics with ``string `..` must exist in pool``; because
    /// this function never asked the pool for a message, the messages reaching
    /// only it were never required to be interned. `data.rs`'s
    /// `RUNTIME_ERROR_MESSAGE_LITERALS` is the other half, and the two are one
    /// patch: either alone is compile-time clean and run-time fatal.
    ///
    /// STANDING INSTRUCTION, unchanged in force: do **not** make the message
    /// fall back to the name when the pool lookup misses. That fallback is
    /// precisely the defect above, only harder to find — the program would run,
    /// report a plausible-looking `message`, and no test would notice. A miss
    /// must stay a named panic naming the missing string, which is what turns
    /// "someone added a message and forgot to intern it" into a one-line fix
    /// instead of an archaeology exercise.
    pub(crate) fn emit_runtime_error_object(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        if let (true, Some(kind)) = (
            self.has_source_execution_environment(),
            ErrorMessageConstructorKind::from_native_error_kind(kind),
        ) {
            self.emit_source_execution_realm_to_local(prototype_local, function);
            self.load_i64_to_local_from_offset(
                prototype_local,
                HEAP_REALM_INTRINSICS_OFFSET,
                prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                prototype_local,
                kind.prototype_slot().offset(),
                prototype_local,
                function,
            );
        } else {
            function.instruction(&Instruction::GlobalGet(error_prototype_global_index(kind)));
            function.instruction(&Instruction::LocalSet(prototype_local));
        }
        self.emit_fresh_native_error_object_call(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    /// Calls [`RuntimeHelperId::RuntimeErrorObject`] for a native error of
    /// `kind` whose [[Prototype]] the caller already resolved into
    /// `prototype_local`, leaving the fresh error in `payload_local` /
    /// `tag_local`.
    ///
    /// This is the only route to that composite: every runtime-thrown error
    /// is created by one shared body instead of an inline allocation and two
    /// property appends per throw site. The helper cannot throw, so both
    /// completion results are dropped and the caller's completion state is
    /// untouched until it publishes the Throw itself.
    fn emit_fresh_native_error_object_call(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let base = self.heap_alloc_function_index.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: heap value without memory",
            )
        })?;
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(kind.as_str())));
        // The message, not the name. See the doc comment on
        // `emit_runtime_error_object`: `payload(name)` here was the T24 defect.
        function.instruction(&Instruction::I64Const(self.strings.payload(message)));
        for _unused_slot in 3..7 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(
            RuntimeHelperId::RuntimeErrorObject.index(base),
        ));
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::Drop);
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        Ok(())
    }

    /// The only body of [`RuntimeHelperId::RuntimeErrorObject`].
    ///
    /// It allocates and appends only; it creates no runtime error of its own,
    /// so it cannot reach [`Self::emit_fresh_native_error_object_call`] and
    /// the helper has no inline seam to clear. Its body is never the main
    /// export, so the appends take the direct ordinary-object route and not
    /// the bootstrap-only `%Array.prototype%` arm, which a fresh object can
    /// never be.
    pub(crate) fn compile_runtime_error_object_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::RuntimeErrorObject);
        let prototype_param = 0;
        let name_param = 1;
        let message_param = 2;
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(prototype_param), None, &mut function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        // [[ErrorData]]. Without it a runtime-thrown error is not an error to
        // anything that reads the internal brand: `Object.prototype.toString`
        // answered "[object Object]" and `Error.isError` answered `false`, while
        // the same class constructed by user code answered "[object Error]" and
        // `true`. Same store as `emit_alloc_error_instance_from_locals`.
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            &mut function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(string_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            name_param,
            string_tag_local,
            true,
            false,
            true,
            &mut function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            message_param,
            string_tag_local,
            true,
            false,
            true,
            &mut function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(self.finish_function(function))
    }

    /// Publishes the two throw-diagnostic globals **together**.
    ///
    /// The name global existed alone, and an uncaught throw therefore reached
    /// the host as `TypeError: wasm-aot completion: object(handle@5397552)` — a
    /// raw linear-memory address that is not stable across builds and maps to
    /// no allocation site, so ~2,488 measured cases across ~1,743 addresses
    /// carried one bit of information between them.
    ///
    /// It is one function because the pairing is the invariant: a site that
    /// sets the name and forgets the message reports a *previous*, unrelated
    /// throw's message. `None` is the explicit "this throw carries no message"
    /// answer and clears the global; it is not the same as not calling this.
    /// The only site that may set either global without coming through here is
    /// `emit_capture_throw_error_name`, which reads both off a user-thrown
    /// value and zeroes the message on entry for the same reason.
    fn emit_set_thrown_error_text(
        &mut self,
        kind: NativeErrorKind,
        message: Option<&str>,
        function: &mut Function,
    ) {
        let name = kind.as_str();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        let message_payload = message.map_or(0, |message| self.strings.payload(message));
        function.instruction(&Instruction::I64Const(message_payload));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));
    }

    pub(crate) fn emit_throw_runtime_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        self.emit_throw_runtime_error_kind(kind, message, payload_local, tag_local, function)
    }

    fn emit_throw_runtime_error_kind(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name = kind.as_str();
        self.emit_runtime_error_object(kind, message, payload_local, tag_local, function)?;
        self.emit_set_thrown_error_text(kind, Some(message), function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        if self.has_source_execution_environment() {
            return self.emit_throw_runtime_error_kind(
                kind,
                message,
                payload_local,
                tag_local,
                function,
            );
        }
        let prototype_local = self.reserve_temp_local();
        let prototype_offset = error_realm_prototype_offset(kind);

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_kind(kind, message, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        if let Some(kind) = ErrorMessageConstructorKind::from_native_error_kind(kind) {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                HEAP_FUNCTION_DEFINING_REALM_OFFSET,
                prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                prototype_local,
                HEAP_REALM_INTRINSICS_OFFSET,
                prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                prototype_local,
                kind.prototype_slot().offset(),
                prototype_local,
                function,
            );
        } else {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                prototype_offset,
                prototype_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(error_prototype_global_index(kind)));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local_kind(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_type_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            TYPE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    /// Load `%TypeError.prototype%` of the executing builtin's Realm, the
    /// Realm a builtin's [[Call]] installs on its execution context
    /// (10.3.3 BuiltinCallOrConstruct steps 5-6).
    ///
    /// A builtin body's `current_env_local` is zero only for the entry Realm.
    /// Otherwise it is a function object whose defining Realm is that Realm:
    /// the builtin itself when reached through its function object, or the
    /// caller Realm's `%Function.prototype%` when source code calls a
    /// statically resolved builtin directly. `%Function.prototype%` is
    /// allocated before any native error prototype exists, so its per-object
    /// `HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET` snapshot is zero; only
    /// the defining Realm's intrinsic table answers for both environments.
    /// Every link is complete before user code runs, so a zero is a compiler
    /// bug and traps rather than borrowing another Realm's prototype.
    pub(crate) fn emit_load_active_builtin_realm_type_error_prototype(
        &mut self,
        prototype_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for offset in [
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            HEAP_REALM_INTRINSICS_OFFSET,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(prototype_local, offset, prototype_local, function);
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_throw_runtime_type_error_without_message(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        if self.has_source_execution_environment() {
            self.emit_source_execution_realm_to_local(prototype_local, function);
            self.load_i64_to_local_from_offset(
                prototype_local,
                HEAP_REALM_INTRINSICS_OFFSET,
                prototype_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                prototype_local,
                HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
                prototype_local,
                function,
            );
        } else {
            function.instruction(&Instruction::GlobalGet(error_prototype_global_index(
                NativeErrorKind::TypeError,
            )));
            function.instruction(&Instruction::LocalSet(prototype_local));
        }
        self.emit_throw_type_error_without_message_with_prototype_local(
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_type_error_without_message(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.has_source_execution_environment() {
            return self.emit_throw_runtime_type_error_without_message(
                payload_local,
                tag_local,
                function,
            );
        }
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(error_prototype_global_index(
            NativeErrorKind::TypeError,
        )));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(error_prototype_global_index(
            NativeErrorKind::TypeError,
        )));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_throw_type_error_without_message_with_prototype_local(
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    fn emit_throw_type_error_without_message_with_prototype_local(
        &mut self,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        self.store_i64_const_at_offset(
            payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_set_thrown_error_text(NativeErrorKind::TypeError, None, function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(TYPE_ERROR_NAME) as i64,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_range_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            RANGE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_current_function_realm_uri_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            URI_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_runtime_error_with_prototype_local(
        &mut self,
        name: &str,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        self.emit_throw_runtime_error_with_prototype_local_kind(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_throw_runtime_error_with_prototype_local_kind(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name = kind.as_str();
        // The same fresh error as `emit_runtime_error_object`, reached through
        // the realm-carrying prototype instead of the global one.
        self.emit_fresh_native_error_object_call(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_set_thrown_error_text(kind, Some(message), function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );
        Ok(())
    }

    /// Creates a fresh native error and routes its Throw completion to the
    /// innermost active catch/finally target, or returns it when there is none.
    ///
    /// The routing is deliberately delegated to
    /// [`Self::emit_propagate_current_throw`]. Whether this builder emits the
    /// main export or a user function does not affect a handler owned by the
    /// current body.
    pub(crate) fn emit_throw_runtime_error_to_active_handler(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(name, message, payload_local, tag_local, function)?;
        self.emit_propagate_current_throw(function);
        Ok(())
    }

    /// Capture, for the host, what a `throw` of an arbitrary value was.
    ///
    /// Two globals, read together by `render_wasmtime_completion`: the error's
    /// `name` (falling back to `constructor.name`, because a `Test262Error`
    /// carries no own `name`) and its `message`. The message half is what stops
    /// an uncaught user throw from reaching the host as nothing but
    /// `object(handle@5397552)`.
    ///
    /// **Both** globals are zeroed here, not by the caller. Zeroing the name
    /// used to be the caller's job — `control_flow.rs`'s `StatementIr::Throw`
    /// and `promise.rs`'s rejection path each emitted their own
    /// `I64Const(0); GlobalSet(name)` first — and that convention is exactly
    /// how a stale value from a previous throw reaches the host at whichever
    /// call site forgets. Both globals are module-lifetime, so a forgotten
    /// clear is not a missing diagnostic but a *wrong* one, attributed to the
    /// throw being captured now. Zeroing on entry makes it impossible to
    /// forget from outside this function; the emitted instruction sequence is
    /// unchanged, because the zero moved to exactly where the callers put it.
    pub(crate) fn emit_capture_throw_error_name(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        let message_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));

        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        // `.message`, read with the same non-calling data-property read as
        // `.name` so capturing a diagnostic can never run user code and change
        // the very completion it is describing. There is deliberately no
        // `constructor.message` fallback: `.name` has one because the error
        // classes put `name` on the prototype, whereas a missing `message` means
        // the thrown value simply has none, and inventing one would put the host
        // back to guessing.
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            message_payload_local,
            message_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(message_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(message_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(message_tag_local);
        self.release_temp_local(message_payload_local);
        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        Ok(())
    }

    /// Test262 compares the final thrown value's constructor name. Capture it
    /// after jobs and finalizers, independently of the diagnostic `.name`.
    /// Reflection here must not invoke accessors or Proxy traps.
    pub(crate) fn emit_capture_final_throw_constructor_name(&mut self, function: &mut Function) {
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(
            throw_error_constructor_name_global_index(self.uses_heap),
        ));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(self.result_tag_local, function);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            self.result_local,
            self.result_tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(constructor_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(
            throw_error_constructor_name_global_index(self.uses_heap),
        ));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
    }
}
