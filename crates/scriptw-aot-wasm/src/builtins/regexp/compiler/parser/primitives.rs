use super::*;

pub(super) fn set(function: &mut Function, local: I64Local, value: u64) {
    function.instruction(&Instruction::I64Const(value as i64));
    local.store(function);
}
pub(super) fn copy(function: &mut Function, target: I64Local, source: I64Local) {
    source.load(function);
    target.store(function);
}
pub(super) fn load(function: &mut Function, pointer: I64Local, offset: u64, target: I64Local) {
    pointer.load(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
    target.store(function);
}
pub(super) fn store(function: &mut Function, pointer: I64Local, offset: u64, source: I64Local) {
    pointer.load(function);
    function.instruction(&Instruction::I32WrapI64);
    source.load(function);
    function.instruction(&Instruction::I64Store(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
}
pub(super) fn store_const(function: &mut Function, pointer: I64Local, offset: u64, value: u64) {
    pointer.load(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::I64Store(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
}
pub(super) fn eq(function: &mut Function, local: I64Local, value: u64) {
    local.load(function);
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::I64Eq);
}
pub(super) fn between(function: &mut Function, local: I64Local, first: u64, last: u64) {
    local.load(function);
    function.instruction(&Instruction::I64Const(first as i64));
    function.instruction(&Instruction::I64GeU);
    local.load(function);
    function.instruction(&Instruction::I64Const(last as i64));
    function.instruction(&Instruction::I64LeU);
    function.instruction(&Instruction::I32And);
}
pub(super) fn peek(
    compiler: &CompilerLocals,
    function: &mut Function,
    cursor: I64Local,
    delta: u64,
    target: I64Local,
) {
    cursor.load(function);
    function.instruction(&Instruction::I64Const(delta as i64));
    function.instruction(&Instruction::I64Add);
    compiler.unit_count.load(function);
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    compiler.units.load(function);
    cursor.load(function);
    function.instruction(&Instruction::I64Const(delta as i64));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I64Const(8));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::I64Const(-1));
    function.instruction(&Instruction::End);
    target.store(function);
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_new_node(
        &mut self,
        compiler: &CompilerLocals,
        kind: NodeKind,
        parent: I64Local,
        node: I64Local,
        address: I64Local,
        function: &mut Function,
    ) {
        self.emit_regexp_scratch_increment(compiler.node_count, 1, function);
        compiler.node_count.load(function);
        compiler.node_capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Nodes),
            function,
        );
        function.instruction(&Instruction::End);
        copy(function, node, compiler.node_count);
        self.emit_regexp_node_address(compiler, node, address, function);
        store_const(function, address, NodeWord::Kind as u64, kind as u64);
        store(function, address, NodeWord::Parent as u64, parent);
        // A group's direction belongs to its containing sequence. Its child
        // sequences inherit ordinary groups, but assertions choose their own
        // direction independently of the containing assertion.
        let parent_address = self.runtime_schema().reserve_i64_local(function);
        let parent_kind = self.runtime_schema().reserve_i64_local(function);
        let direction = self.runtime_schema().reserve_i64_local(function);
        set(function, direction, 0);
        eq(function, parent, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_node_address(compiler, parent, parent_address, function);
        load(function, parent_address, NodeWord::Kind as u64, parent_kind);
        load(
            function,
            parent_address,
            NodeWord::Direction as u64,
            direction,
        );
        for (group, child_direction) in [
            (NodeKind::PositiveLookahead, 0),
            (NodeKind::NegativeLookahead, 0),
            (NodeKind::PositiveLookbehind, 1),
            (NodeKind::NegativeLookbehind, 1),
        ] {
            eq(function, parent_kind, group as u64);
            function.instruction(&Instruction::If(BlockType::Empty));
            set(function, direction, child_direction);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        store(function, address, NodeWord::Direction as u64, direction);
        self.runtime_schema().release_i64_local(direction, function);
        self.runtime_schema()
            .release_i64_local(parent_kind, function);
        self.runtime_schema()
            .release_i64_local(parent_address, function);
        store_const(function, address, NodeWord::Minimum as u64, 1);
        store_const(function, address, NodeWord::Maximum as u64, 1);
        store_const(
            function,
            address,
            NodeWord::MaximumKind as u64,
            RegExpRepeatMaximumKind::Finite.word(),
        );
        for word in [
            NodeWord::MinimumDigitsStart,
            NodeWord::MinimumDigitsEnd,
            NodeWord::MaximumDigitsStart,
            NodeWord::MaximumDigitsEnd,
        ] {
            store_const(function, address, word as u64, 0);
        }
        store(
            function,
            address,
            NodeWord::SourceOffset as u64,
            compiler.cursor,
        );
        address.load(function);
        function.instruction(&Instruction::I32WrapI64);
        compiler.capture_count.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: NodeWord::CaptureStart as u64,
            align: 3,
            memory_index: 0,
        }));
    }
    pub(super) fn emit_regexp_parser_append(
        &mut self,
        compiler: &CompilerLocals,
        parent: I64Local,
        child: I64Local,
        function: &mut Function,
    ) {
        let address = self.runtime_schema().reserve_i64_local(function);
        let last = self.runtime_schema().reserve_i64_local(function);
        let last_address = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_node_address(compiler, parent, address, function);
        load(function, address, NodeWord::Last as u64, last);
        eq(function, last, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        store(function, address, NodeWord::First as u64, child);
        function.instruction(&Instruction::Else);
        self.emit_regexp_node_address(compiler, last, last_address, function);
        store(function, last_address, NodeWord::Next as u64, child);
        function.instruction(&Instruction::End);
        store(function, address, NodeWord::Last as u64, child);
        self.runtime_schema()
            .release_i64_local(last_address, function);
        self.runtime_schema().release_i64_local(last, function);
        self.runtime_schema().release_i64_local(address, function);
    }
}
