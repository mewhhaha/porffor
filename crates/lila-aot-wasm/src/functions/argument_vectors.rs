use super::*;
use crate::gc_types::{
    ArgumentListNode, ArgumentListNodeSchema, GcLocal, GcOperand, I32Local, Nullable, StoredValue,
    ValueArray, ValueLocals,
};

/// Only finish can publish the completed List. Append retains whole values;
/// the owner cannot be copied or mistaken for a JavaScript object.
#[must_use]
pub(crate) struct ArgumentListConstruction {
    head: GcLocal<ArgumentListNode, Nullable>,
    tail: GcLocal<ArgumentListNode, Nullable>,
    length: I32Local,
}

impl ArgumentListConstruction {
    pub(crate) fn new(schema: &crate::gc_types::RuntimeSchema, function: &mut Function) -> Self {
        let head = schema
            .reserve_gc_local(function)
            .initialize_null(schema, function);
        let tail = schema
            .reserve_gc_local(function)
            .initialize_null(schema, function);
        let length = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        length.store(function);
        Self { head, tail, length }
    }

    pub(crate) fn append(
        &self,
        value: &ValueLocals,
        schema: &crate::gc_types::RuntimeSchema,
        function: &mut Function,
    ) {
        self.length.load(function);
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        let node = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ArgumentListNode>().construct(
                (
                    GcOperand::reference(&stored, schema),
                    GcOperand::null(schema),
                ),
                function,
            ),
            function,
        );
        self.tail.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.head
            .replace(node.load(schema, function).nullable(), function);
        function.instruction(&Instruction::Else);
        let tail = schema.reserve_gc_local(function).initialize(
            self.tail.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .struct_type::<ArgumentListNode>()
            .field(ArgumentListNodeSchema::NEXT)
            .write(
                &tail,
                GcOperand::nullable_reference(&node, schema),
                schema,
                function,
            );
        tail.clear(function);
        function.instruction(&Instruction::End);
        self.tail
            .replace(node.load(schema, function).nullable(), function);
        self.length.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        self.length.store(function);
        node.clear(function);
        stored.clear(function);
    }

    pub(super) fn append_list(
        &self,
        list: &GcLocal<ValueArray>,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) {
        let schema = builder.runtime_schema();
        let index = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        let value = schema.reserve_value_local(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        schema
            .array_type::<ValueArray>()
            .length(list, schema, function);
        count.store(function);
        let done = builder.open_frame(ControlFrameKind::Block, function);
        let next = builder.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        builder.open_frame(ControlFrameKind::If, function);
        builder.emit_branch_to_target(done, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        builder.emit_argument_vector_entry_to_value(list, index, &value, function);
        self.append(&value, schema, function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        builder.emit_branch_to_target(next, function);
        builder.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        builder.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        value.clear(function);
        schema.release_i32_local(count, function);
        schema.release_i32_local(index, function);
    }

    pub(crate) fn finish(
        self,
        builder: &mut FunctionBuilder<'_>,
        function: &mut Function,
    ) -> GcLocal<ValueArray> {
        let schema = builder.runtime_schema();
        let value = schema.reserve_value_local(function);
        value.set_undefined(function);
        let empty = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&value, function),
            function,
        );
        let list = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().filled(
                GcOperand::reference(&empty, schema),
                self.length,
                function,
            ),
            function,
        );
        let cursor = schema
            .reserve_gc_local::<ArgumentListNode, Nullable>(function)
            .initialize(self.head.load(schema, function), function);
        let index = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let done = builder.open_frame(ControlFrameKind::Block, function);
        let next = builder.open_frame(ControlFrameKind::Loop, function);
        cursor.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        builder.open_frame(ControlFrameKind::If, function);
        builder.emit_branch_to_target(done, function);
        builder.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let node = schema.reserve_gc_local(function).initialize(
            cursor.load(schema, function).require_non_null(function),
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ArgumentListNode>()
                .field(ArgumentListNodeSchema::VALUE)
                .read(&node, schema, function)
                .reference(),
            function,
        );
        schema.array_type::<ValueArray>().write(
            &list,
            index,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        cursor.replace(
            schema
                .struct_type::<ArgumentListNode>()
                .field(ArgumentListNodeSchema::NEXT)
                .read(&node, schema, function)
                .reference(),
            function,
        );
        stored.clear(function);
        node.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        builder.emit_branch_to_target(next, function);
        builder.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        builder.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i32_local(index, function);
        cursor.clear(function);
        empty.clear(function);
        value.clear(function);
        schema.release_i32_local(self.length, function);
        self.tail.clear(function);
        self.head.clear(function);
        list
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_argument_list_capture(
        &mut self,
        capture: &lila_ir::ArgumentListCaptureIr,
        function: &mut Function,
    ) -> Result<GcLocal<ValueArray>, EmitError> {
        self.emit_call_args_vector(capture.arguments(), function)
    }

    pub(super) fn emit_spread_argument_into_list(
        &mut self,
        source_expression: &TypedExpr,
        list: &ArgumentListConstruction,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::emit::AccessorThrowRouting;
        let schema = self.runtime_schema();
        let source = schema.reserve_value_local(function);
        let method = schema.reserve_value_local(function);
        let iterator = schema.reserve_value_local(function);
        let next_method = schema.reserve_value_local(function);
        let step = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let empty = self.emit_pre_evaluated_arg_vector(&[], function);
        self.compile_expr_to_value(source_expression, &source, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        let symbol = schema.reserve_gc_local(function).initialize(
            self.emit_well_known_symbol_reference(lila_ir::WellKnownSymbol::Iterator, function)?,
            function,
        );
        let iterator_key =
            crate::operations::PropertyKeyLocals::from_symbol(schema, &symbol, function);
        self.emit_object_read_with_throw_routing(
            &source,
            &source,
            &iterator_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        iterator_key.clear(function);
        symbol.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        method.copy_from(pending.value(), function);
        self.emit_is_callable_i32(&method, function)?;
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::SPREAD_ARGUMENT_IS_NOT_ITERABLE,
            &pending,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.emit_function_or_proxy_call_with_argv(&method, &source, &empty, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        iterator.copy_from(pending.value(), function);
        self.emit_object_value_i32(&iterator, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::SPREAD_ITERATOR_METHOD_MUST_RETURN_OBJECT,
            &pending,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let next_key = self.emit_function_string_key("next", function)?;
        self.emit_object_read_with_throw_routing(
            &iterator,
            &iterator,
            &next_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        next_key.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        next_method.copy_from(pending.value(), function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        self.emit_function_or_proxy_call_with_argv(
            &next_method,
            &iterator,
            &empty,
            &pending,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        step.copy_from(pending.value(), function);
        self.emit_object_value_i32(&step, function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(
            NativeErrorKind::TypeError,
            RuntimeErrorMessage::SPREAD_ITERATOR_NEXT_RESULT_MUST_BE_OBJECT,
            &pending,
            function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let done_key = self.emit_function_string_key("done", function)?;
        self.emit_object_read_with_throw_routing(
            &step,
            &step,
            &done_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        done_key.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        self.compile_truthy_tagged_i32(pending.value(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let value_key = self.emit_function_string_key("value", function)?;
        self.emit_object_read_with_throw_routing(
            &step,
            &step,
            &value_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        value_key.clear(function);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        list.append(pending.value(), schema, function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        empty.clear(function);
        pending.clear(function);
        step.clear(function);
        next_method.clear(function);
        iterator.clear(function);
        method.clear(function);
        source.clear(function);
        Ok(())
    }

    /// The callback can only consume the completed internal argument List.
    /// Length coercion and every indexed Get finish before callable dispatch.
    pub(crate) fn emit_with_array_like_argument_vector(
        &mut self,
        input: &ValueLocals,
        not_object: RuntimeErrorMessage,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
        consume: impl FnOnce(
            &mut Self,
            &GcLocal<ValueArray>,
            &crate::gc_types::CompletionLocals,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        use crate::emit::AccessorThrowRouting;
        use crate::runtime_helpers::{NumberToStringArguments, ValueToNumberArguments};
        let schema = self.runtime_schema();
        let pending = schema.reserve_completion(function);
        let length_value = schema.reserve_value_local(function);
        let element = schema.reserve_value_local(function);
        let count = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let bits = schema.reserve_i64_local(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        result.initialize(function);
        function.instruction(&Instruction::I32Const(0));
        for tag in [
            WasmRuntimeValueTag::Object,
            WasmRuntimeValueTag::Array,
            WasmRuntimeValueTag::Function,
            WasmRuntimeValueTag::Arguments,
        ] {
            input.tag().load(function);
            function.instruction(&Instruction::I32Const(tag as i32));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_runtime_error(NativeErrorKind::TypeError, not_object, result, function)?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let length_key = self.emit_function_string_key("length", function)?;
        self.emit_object_read_with_throw_routing(
            input,
            input,
            &length_key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        length_key.clear(function);
        self.emit_bind_metadata_abrupt_exit(&pending, result, exit, function);
        length_value.copy_from(pending.value(), function);
        let base = self.runtime_helper_base.ok_or_else(|| {
            EmitError::unsupported("compiler invariant: conversion helper catalog is absent")
        })?;
        schema
            .call_helper(
                ValueToNumberArguments::new(&length_value, self.current_environment()),
                base,
                function,
            )
            .store(&pending, function);
        self.emit_bind_metadata_abrupt_exit(&pending, result, exit, function);
        function.instruction(&Instruction::I32Const(0));
        count.store(function);
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Gt);
        self.open_frame(ControlFrameKind::If, function);
        pending.value().scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Const(Ieee64::from(
            9_007_199_254_740_991.0,
        )));
        function.instruction(&Instruction::F64Min);
        function.instruction(&Instruction::I64ReinterpretF64);
        bits.store(function);
        // A List larger than Wasm's actual array count cannot be allocated.
        // This is a runtime resource failure; it must not wrap or truncate into
        // a shorter list whose later indexed Gets would be silently omitted.
        bits.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(u32::MAX as f64)));
        function.instruction(&Instruction::F64Gt);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        bits.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I32TruncF64U);
        count.store(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        element.set_undefined(function);
        let empty = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&element, function),
            function,
        );
        let list = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().filled(
                GcOperand::reference(&empty, schema),
                count,
                function,
            ),
            function,
        );
        empty.clear(function);
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        let next = self.open_frame(ControlFrameKind::Loop, function);
        index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_branch_to_target(done, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::F64ConvertI32U);
        function.instruction(&Instruction::I64ReinterpretF64);
        bits.store(function);
        let text = schema
            .call_helper(NumberToStringArguments::new(bits), base, function)
            .bind(schema, schema.reserve_gc_local(function), function);
        let key = crate::operations::PropertyKeyLocals::from_string(schema, &text, function);
        text.clear(function);
        self.emit_object_read_with_throw_routing(
            input,
            input,
            &key,
            &pending,
            AccessorThrowRouting::LeaveInCompletion,
            function,
        )?;
        key.clear(function);
        self.emit_bind_metadata_abrupt_exit(&pending, result, exit, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(pending.value(), function),
            function,
        );
        schema.array_type::<ValueArray>().write(
            &list,
            index,
            GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        stored.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        self.emit_branch_to_target(next, function);
        self.pop_control(ControlFrameKind::Loop);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        consume(self, &list, result, function)?;
        list.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i64_local(bits, function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(count, function);
        element.clear(function);
        length_value.clear(function);
        pending.clear(function);
        Ok(())
    }

    pub(crate) fn emit_builtin_arg_is_present_i32(&self, index: usize, function: &mut Function) {
        let index = i64::try_from(index).expect("builtin argument index fits its ABI");
        let entry = self
            .body_entry_locals()
            .expect("builtin owns its declared callable entry");
        entry.argument_count().load(function);
        function.instruction(&Instruction::I64Const(index));
        function.instruction(&Instruction::I64GtU);
    }

    pub(crate) fn emit_builtin_arg_to_value(
        &self,
        index: usize,
        destination: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index = u32::try_from(index).expect("builtin argument index fits a GC array");
        let index_local = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(index as i32));
        index_local.store(function);
        let entry = self
            .body_entry_locals()
            .expect("builtin owns its declared callable entry");
        self.emit_argument_vector_entry_to_value(
            entry.arguments(),
            index_local,
            destination,
            function,
        );
        schema.release_i32_local(index_local, function);
    }

    /// An omitted argument is canonical Undefined. In-range entries copy all
    /// three parts together, including the original strong reference.
    pub(crate) fn emit_argument_vector_entry_to_value(
        &self,
        arguments: &GcLocal<ValueArray>,
        index: I32Local,
        destination: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let array = schema.array_type::<ValueArray>();
        index.load(function);
        array.length(arguments, schema, function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        let stored = schema.reserve_gc_local(function).initialize(
            array.read(arguments, index, schema, function).reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, destination, schema, function);
        stored.clear(function);
        function.instruction(&Instruction::Else);
        destination.set_undefined(function);
        function.instruction(&Instruction::End);
    }

    /// Internal argument-list slicing allocates a ValueArray, independently of
    /// the Array exotic that implements an observable source rest parameter.
    pub(crate) fn emit_builtin_argument_vector_tail(
        &self,
        start: u32,
        function: &mut Function,
    ) -> GcLocal<ValueArray> {
        let schema = self.runtime_schema();
        let entry = self
            .body_entry_locals()
            .expect("builtin owns its declared callable entry");
        let source = entry.arguments();
        let array = schema.array_type::<ValueArray>();
        let length = schema.reserve_i32_local(function);
        let count = schema.reserve_i32_local(function);
        let destination_index = schema.reserve_i32_local(function);
        let source_index = schema.reserve_i32_local(function);
        array.length(source, schema, function);
        length.store(function);
        length.load(function);
        function.instruction(&Instruction::I32Const(start as i32));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        length.load(function);
        function.instruction(&Instruction::I32Const(start as i32));
        function.instruction(&Instruction::I32Sub);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        count.store(function);
        let initial_value = schema.reserve_value_local(function);
        initial_value.set_undefined(function);
        let initial = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&initial_value, function),
            function,
        );
        initial_value.clear(function);
        let destination = schema.reserve_gc_local(function).initialize(
            array.filled(GcOperand::reference(&initial, schema), count, function),
            function,
        );
        initial.clear(function);
        function.instruction(&Instruction::I32Const(0));
        destination_index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        destination_index.load(function);
        count.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        destination_index.load(function);
        function.instruction(&Instruction::I32Const(start as i32));
        function.instruction(&Instruction::I32Add);
        source_index.store(function);
        let element = schema.reserve_gc_local(function).initialize(
            array
                .read(source, source_index, schema, function)
                .reference(),
            function,
        );
        array.write(
            &destination,
            destination_index,
            GcOperand::reference(&element, schema),
            schema,
            function,
        );
        element.clear(function);
        destination_index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        destination_index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(source_index, function);
        schema.release_i32_local(destination_index, function);
        schema.release_i32_local(count, function);
        schema.release_i32_local(length, function);
        destination
    }

    pub(crate) fn emit_pre_evaluated_arg_vector(
        &self,
        arguments: &[&ValueLocals],
        function: &mut Function,
    ) -> GcLocal<ValueArray> {
        let schema = self.runtime_schema();
        let stored: Vec<GcLocal<StoredValue>> = arguments
            .iter()
            .map(|value| {
                schema.reserve_gc_local(function).initialize(
                    schema
                        .struct_type::<StoredValue>()
                        .from_value(value, function),
                    function,
                )
            })
            .collect();
        let vector = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().fixed(
                stored
                    .iter()
                    .map(|value| GcOperand::reference(value, schema)),
                function,
            ),
            function,
        );
        for value in stored.into_iter().rev() {
            value.clear(function);
        }
        vector
    }
}
