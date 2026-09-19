use super::*;

pub(super) fn set(function: &mut Function, local: u32, value: u64) {
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::LocalSet(local));
}
pub(super) fn copy(function: &mut Function, target: u32, source: u32) {
    function.instruction(&Instruction::LocalGet(source));
    function.instruction(&Instruction::LocalSet(target));
}
pub(super) fn load(function: &mut Function, pointer: u32, offset: u64, target: u32) {
    function.instruction(&Instruction::LocalGet(pointer));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
    function.instruction(&Instruction::LocalSet(target));
}
pub(super) fn store(function: &mut Function, pointer: u32, offset: u64, source: u32) {
    function.instruction(&Instruction::LocalGet(pointer));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::LocalGet(source));
    function.instruction(&Instruction::I64Store(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
}
pub(super) fn store_const(function: &mut Function, pointer: u32, offset: u64, value: u64) {
    function.instruction(&Instruction::LocalGet(pointer));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::I64Store(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
}
pub(super) fn eq(function: &mut Function, local: u32, value: u64) {
    function.instruction(&Instruction::LocalGet(local));
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::I64Eq);
}
pub(super) fn between(function: &mut Function, local: u32, first: u64, last: u64) {
    function.instruction(&Instruction::LocalGet(local));
    function.instruction(&Instruction::I64Const(first as i64));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::LocalGet(local));
    function.instruction(&Instruction::I64Const(last as i64));
    function.instruction(&Instruction::I64LeU);
    function.instruction(&Instruction::I32And);
}
pub(super) fn peek(
    compiler: &CompilerLocals,
    function: &mut Function,
    cursor: u32,
    delta: u64,
    target: u32,
) {
    function.instruction(&Instruction::LocalGet(cursor));
    function.instruction(&Instruction::I64Const(delta as i64));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalGet(compiler.unit_count));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::LocalGet(compiler.units));
    function.instruction(&Instruction::LocalGet(cursor));
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
    function.instruction(&Instruction::LocalSet(target));
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_parser_new_node(
        &mut self,
        compiler: &CompilerLocals,
        kind: NodeKind,
        parent: u32,
        node: u32,
        address: u32,
        function: &mut Function,
    ) {
        self.emit_increment_local(compiler.node_count, 1, function);
        function.instruction(&Instruction::LocalGet(compiler.node_count));
        function.instruction(&Instruction::LocalGet(compiler.node_capacity));
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
        store_const(function, address, NodeWord::Minimum as u64, 1);
        store_const(function, address, NodeWord::Maximum as u64, 1);
        store(
            function,
            address,
            NodeWord::SourceOffset as u64,
            compiler.cursor,
        );
        function.instruction(&Instruction::LocalGet(address));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(compiler.capture_count));
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
        parent: u32,
        child: u32,
        function: &mut Function,
    ) {
        let address = self.reserve_temp_local();
        let last = self.reserve_temp_local();
        let last_address = self.reserve_temp_local();
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
        self.release_temp_local(last_address);
        self.release_temp_local(last);
        self.release_temp_local(address);
    }
}
