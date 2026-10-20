use super::*;

#[must_use = "Promise combinator function Realm is retained until publication completes"]
pub(super) struct PromiseCombinatorElementFunctionMaterializationContext {
    internal: PromiseInternalFunctionMaterializationContext,
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_current_function_promise_combinator_element_materialization_context(
        &mut self,
        function: &mut Function,
    ) -> PromiseCombinatorElementFunctionMaterializationContext {
        PromiseCombinatorElementFunctionMaterializationContext {
            internal: self
                .emit_current_function_promise_internal_function_materialization_context(function),
        }
    }
    pub(super) fn emit_promise_combinator_element_function_value(
        &mut self,
        entry: PromiseInternalFunction<'_>,
        context: &PromiseCombinatorElementFunctionMaterializationContext,
        function: &mut Function,
    ) -> Result<GcLocal<FunctionObject>, EmitError> {
        self.emit_promise_internal_function_value(entry, &context.internal, function)
    }
    pub(super) fn release_promise_combinator_element_function_materialization_context(
        &mut self,
        context: PromiseCombinatorElementFunctionMaterializationContext,
        function: &mut Function,
    ) {
        self.release_promise_internal_function_materialization_context(context.internal, function);
    }

    /// The spec values/errors List remains compiler-private. Reactions replace
    /// slots in this rooted list; ArrayCreate happens only after completion.
    pub(super) fn emit_reserve_promise_combinator_list_entry(
        &self,
        shared: &GcLocal<PromiseCombinatorShared>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let source = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseCombinatorShared>()
                .field(PromiseCombinatorSharedSchema::VALUES)
                .read(shared, schema, function)
                .reference(),
            function,
        );
        let old_length = schema.reserve_i32_local(function);
        let new_length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        schema
            .array_type::<ValueArray>()
            .length(&source, schema, function);
        old_length.store(function);
        old_length.load(function);
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        old_length.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        new_length.store(function);
        let undefined = schema.reserve_value_local(function);
        undefined.set_undefined(function);
        let initial = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&undefined, function),
            function,
        );
        let destination = schema.reserve_gc_local(function).initialize(
            schema.array_type::<ValueArray>().filled(
                GcOperand::reference(&initial, schema),
                new_length,
                function,
            ),
            function,
        );
        function.instruction(&Instruction::I32Const(0));
        index.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        old_length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        let entry = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .read(&source, index, schema, function)
                .reference(),
            function,
        );
        schema.array_type::<ValueArray>().write(
            &destination,
            index,
            GcOperand::reference(&entry, schema),
            schema,
            function,
        );
        entry.clear(function);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema
            .struct_type::<PromiseCombinatorShared>()
            .field(PromiseCombinatorSharedSchema::VALUES)
            .write(
                shared,
                GcOperand::reference(&destination, schema),
                schema,
                function,
            );
        destination.clear(function);
        initial.clear(function);
        undefined.clear(function);
        schema.release_i32_local(index, function);
        schema.release_i32_local(new_length, function);
        schema.release_i32_local(old_length, function);
        source.clear(function);
    }
    pub(super) fn emit_store_promise_combinator_list_entry(
        &self,
        shared: &GcLocal<PromiseCombinatorShared>,
        index: I64Local,
        value: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        index.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let position = schema.reserve_i32_local(function);
        index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        position.store(function);
        let list = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PromiseCombinatorShared>()
                .field(PromiseCombinatorSharedSchema::VALUES)
                .read(shared, schema, function)
                .reference(),
            function,
        );
        let entry = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(value, function),
            function,
        );
        schema.array_type::<ValueArray>().write(
            &list,
            position,
            GcOperand::reference(&entry, schema),
            schema,
            function,
        );
        entry.clear(function);
        list.clear(function);
        schema.release_i32_local(position, function);
    }
}
