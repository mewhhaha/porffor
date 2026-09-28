use super::*;
use lila_ir::FunctionProtocolIr;

/// The only source functions allowed to expose legacy reflection
/// properties are sloppy ordinary declarations and expressions. Every other
/// ECMAScript or native entry is a barrier for its immediate callee.
#[derive(Clone, Copy)]
pub(crate) enum LegacyActivationMode {
    Exposable,
    Barrier,
}

impl LegacyActivationMode {
    /// Keep source-function reflection eligibility identical at activation
    /// entry and when planning whether its arguments object may be elided.
    pub(crate) const fn for_source(protocol: FunctionProtocolIr, strict: bool) -> Self {
        match protocol {
            FunctionProtocolIr::OrdinaryCallAndConstruct if !strict => Self::Exposable,
            FunctionProtocolIr::OrdinaryCallOnly
            | FunctionProtocolIr::OrdinaryCallAndConstruct
            | FunctionProtocolIr::Arrow
            | FunctionProtocolIr::Generator
            | FunctionProtocolIr::Async
            | FunctionProtocolIr::AsyncArrow
            | FunctionProtocolIr::AsyncGenerator
            | FunctionProtocolIr::ModuleActivation
            | FunctionProtocolIr::AsyncModuleActivation
            | FunctionProtocolIr::ObjectMethod(_)
            | FunctionProtocolIr::ObjectGetter
            | FunctionProtocolIr::ObjectSetter
            | FunctionProtocolIr::ClassConstructor
            | FunctionProtocolIr::ClassMethod(_)
            | FunctionProtocolIr::ClassGetter
            | FunctionProtocolIr::ClassSetter => Self::Barrier,
        }
    }

    pub(crate) const fn is_exposable(self) -> bool {
        matches!(self, Self::Exposable)
    }
}

/// Wasm locals owned by one invocation. Keeping the saved values in the Wasm
/// stack handles recursive calls without heap allocations or a second stack.
#[derive(Clone, Copy)]
pub(crate) struct LegacyActivationLocals {
    pub(crate) mode: LegacyActivationMode,
    previous: u32,
    callee: u32,
    saved_caller_payload: u32,
    saved_caller_tag: u32,
    saved_arguments_payload: u32,
    saved_arguments_tag: u32,
    entry: u32,
    buffer: u32,
    length: u32,
    index: u32,
    key: u32,
    tag: u32,
}

impl LegacyActivationLocals {
    pub(crate) const COUNT: u32 = 12;

    pub(crate) const fn at(base: u32, mode: LegacyActivationMode) -> Self {
        Self {
            mode,
            previous: base,
            callee: base + 1,
            saved_caller_payload: base + 2,
            saved_caller_tag: base + 3,
            saved_arguments_payload: base + 4,
            saved_arguments_tag: base + 5,
            entry: base + 6,
            buffer: base + 7,
            length: base + 8,
            index: base + 9,
            key: base + 10,
            tag: base + 11,
        }
    }
}

impl FunctionBuilder<'_> {
    /// Locate the non-configurable own data property. Named-property buffers
    /// may grow during arbitrary user code, so no entry address is retained
    /// across calls or between activation entry and exit. Equivalent descriptor
    /// updates are allowed, so compare the stored key by string content.
    fn emit_legacy_property_entry(&self, object: u32, name: &'static str, function: &mut Function) {
        let locals = self
            .legacy_activation_locals
            .expect("legacy activation locals");
        self.load_i64_to_local_from_offset(object, HEAP_PTR_OFFSET, locals.buffer, function);
        self.load_i64_to_local_from_offset(object, HEAP_LEN_OFFSET, locals.length, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(locals.index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(locals.index));
        function.instruction(&Instruction::LocalGet(locals.length));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(locals.buffer));
        function.instruction(&Instruction::LocalGet(locals.index));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(locals.entry));
        self.load_i64_to_local_from_offset(
            locals.entry,
            HEAP_OBJECT_KEY_OFFSET,
            locals.key,
            function,
        );
        // Fast path for the original static key, followed by the same semantic
        // string comparison used by ordinary own-property lookup. A Symbol
        // carrier cannot match either of these string names.
        function.instruction(&Instruction::LocalGet(locals.key));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(locals.key));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(locals.key));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        for _ in 0..5 {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Call(
            self.string_equality_helper_function_index()
                .expect("legacy function properties require the string equality helper"),
        ));
        for _ in 0..3 {
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(locals.index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(locals.index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_read_legacy_property(
        &self,
        object: u32,
        name: &'static str,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) {
        let locals = self
            .legacy_activation_locals
            .expect("legacy activation locals");
        self.emit_legacy_property_entry(object, name, function);
        self.load_i64_to_local_from_offset(
            locals.entry,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload,
            function,
        );
        self.load_i64_to_local_from_offset(
            locals.entry,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            tag,
            function,
        );
    }

    fn emit_write_legacy_property(
        &self,
        object: u32,
        name: &'static str,
        payload: u32,
        tag: u32,
        function: &mut Function,
    ) {
        let locals = self
            .legacy_activation_locals
            .expect("legacy activation locals");
        self.emit_legacy_property_entry(object, name, function);
        self.store_i64_local_at_offset(
            locals.entry,
            HEAP_OBJECT_DATA_PAYLOAD_OFFSET,
            payload,
            function,
        );
        self.store_i64_local_at_offset(locals.entry, HEAP_OBJECT_DATA_TAG_OFFSET, tag, function);
    }

    fn emit_write_legacy_null_property(
        &self,
        object: u32,
        name: &'static str,
        function: &mut Function,
    ) {
        let locals = self
            .legacy_activation_locals
            .expect("legacy activation locals");
        self.emit_legacy_property_entry(object, name, function);
        self.store_i64_const_at_offset(locals.entry, HEAP_OBJECT_DATA_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            locals.entry,
            HEAP_OBJECT_DATA_TAG_OFFSET,
            ValueKind::Null.tag() as u64,
            function,
        );
    }

    pub(crate) fn emit_begin_legacy_activation(&mut self, function: &mut Function) {
        let Some(locals) = self.legacy_activation_locals else {
            return;
        };
        function.instruction(&Instruction::GlobalGet(LEGACY_ACTIVE_CALLER_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(locals.previous));
        match locals.mode {
            LegacyActivationMode::Barrier => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::GlobalSet(LEGACY_ACTIVE_CALLER_GLOBAL_INDEX));
            }
            LegacyActivationMode::Exposable => {
                self.load_i64_to_local_from_offset(
                    self.class_function_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
                    locals.callee,
                    function,
                );
                self.emit_read_legacy_property(
                    locals.callee,
                    "caller",
                    locals.saved_caller_payload,
                    locals.saved_caller_tag,
                    function,
                );
                self.emit_read_legacy_property(
                    locals.callee,
                    "arguments",
                    locals.saved_arguments_payload,
                    locals.saved_arguments_tag,
                    function,
                );
                function.instruction(&Instruction::LocalGet(locals.previous));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(locals.tag));
                self.emit_write_legacy_property(
                    locals.callee,
                    "caller",
                    locals.previous,
                    locals.tag,
                    function,
                );
                self.emit_write_legacy_null_property(locals.callee, "arguments", function);
                function.instruction(&Instruction::LocalGet(locals.callee));
                function.instruction(&Instruction::GlobalSet(LEGACY_ACTIVE_CALLER_GLOBAL_INDEX));
            }
        }
    }

    pub(crate) fn emit_publish_legacy_arguments(
        &mut self,
        arguments_payload: u32,
        function: &mut Function,
    ) {
        let Some(locals) = self.legacy_activation_locals else {
            return;
        };
        if !matches!(locals.mode, LegacyActivationMode::Exposable) {
            return;
        }
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(locals.tag));
        self.emit_write_legacy_property(
            locals.callee,
            "arguments",
            arguments_payload,
            locals.tag,
            function,
        );
    }

    /// `Function.prototype.call`, `Function.prototype.apply`, and
    /// `Reflect.apply` perform PrepareForTailCall after validating and
    /// collecting arguments. Their native context censors callbacks during
    /// that work, but is no longer the target's parent when Call begins.
    pub(crate) fn emit_prepare_legacy_tail_call(&self, function: &mut Function) {
        let Some(locals) = self.legacy_activation_locals else {
            return;
        };
        debug_assert!(matches!(locals.mode, LegacyActivationMode::Barrier));
        function.instruction(&Instruction::LocalGet(locals.previous));
        function.instruction(&Instruction::GlobalSet(LEGACY_ACTIVE_CALLER_GLOBAL_INDEX));
    }

    pub(crate) fn emit_end_legacy_activation(&self, function: &mut Function) {
        let Some(locals) = self.legacy_activation_locals else {
            return;
        };
        if let Some(frame) = self.deferred_arguments_frame {
            crate::gc_types::legacy_arguments::emit_get(
                function,
                frame,
                crate::gc_types::legacy_arguments::Field::Parent,
            );
            function.instruction(&Instruction::GlobalSet(
                crate::gc_types::legacy_arguments::GLOBAL_INDEX,
            ));
        }
        if matches!(locals.mode, LegacyActivationMode::Exposable) {
            // The same source function may be recursively active; restore the
            // immediately enclosing invocation's values before returning.
            self.emit_write_legacy_property(
                locals.callee,
                "caller",
                locals.saved_caller_payload,
                locals.saved_caller_tag,
                function,
            );
            self.emit_write_legacy_property(
                locals.callee,
                "arguments",
                locals.saved_arguments_payload,
                locals.saved_arguments_tag,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(locals.previous));
        function.instruction(&Instruction::GlobalSet(LEGACY_ACTIVE_CALLER_GLOBAL_INDEX));
    }
}
