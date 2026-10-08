use super::*;
use lila_ir::{REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION, REGEXP_OPCODE_NAMED_BACKREFERENCE};

pub(super) const NAME_ROW_BYTES: u64 = 32;

/// Parser-owned rows retain canonical names, capture IDs and the owning AST
/// group until duplicate-name and forward-reference validation has completed.
#[derive(Clone, Copy)]
#[repr(u64)]
enum NameWord {
    Payload = 0,
    Capture = 8,
    Owner = 16,
    Group = 24,
}

/// The only admission token accepted by lowering and publication. Construction
/// follows emitted checks for every duplicate name and resolution of every
/// named-reference operand; publication consumes the completed inventory.
/// Unresolved parser payloads cannot be passed to either emitter API.
pub(super) struct CompletedCaptureInventory {
    group_count: I64Local,
}

impl CompletedCaptureInventory {
    pub(super) fn emit_group_count(&self, function: &mut Function) {
        self.group_count.load(function);
    }
}

fn set(function: &mut Function, local: I64Local, value: u64) {
    function.instruction(&Instruction::I64Const(value as i64));
    local.store(function);
}
fn copy(function: &mut Function, target: I64Local, source: I64Local) {
    source.load(function);
    target.store(function);
}
fn load(function: &mut Function, address: I64Local, offset: u64, target: I64Local) {
    address.load(function);
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
    target.store(function);
}
fn store(function: &mut Function, address: I64Local, offset: u64, value: I64Local) {
    address.load(function);
    function.instruction(&Instruction::I32WrapI64);
    value.load(function);
    function.instruction(&Instruction::I64Store(MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
}
fn eq(function: &mut Function, local: I64Local, value: u64) {
    local.load(function);
    function.instruction(&Instruction::I64Const(value as i64));
    function.instruction(&Instruction::I64Eq);
}

impl FunctionBuilder<'_> {
    fn emit_regexp_name_row_address(
        &self,
        compiler: &CompilerLocals,
        row: I64Local,
        address: I64Local,
        function: &mut Function,
    ) {
        compiler.name_rows.load(function);
        row.load(function);
        function.instruction(&Instruction::I64Const(NAME_ROW_BYTES as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        address.store(function);
    }

    pub(super) fn emit_regexp_append_capture_name(
        &mut self,
        compiler: &CompilerLocals,
        owner: I64Local,
        payload: I64Local,
        function: &mut Function,
    ) {
        let address = self.runtime_schema().reserve_i64_local(function);
        compiler.name_count.load(function);
        compiler.unit_capacity.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::Nodes),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_name_row_address(compiler, compiler.name_count, address, function);
        store(function, address, NameWord::Payload as u64, payload);
        store(
            function,
            address,
            NameWord::Capture as u64,
            compiler.capture_count,
        );
        store(function, address, NameWord::Owner as u64, owner);
        self.emit_regexp_scratch_increment(compiler.name_count, 1, function);
        self.runtime_schema().release_i64_local(address, function);
    }

    /// Equality of canonical compiler-owned byte strings. These workspace
    /// addresses may use all 32 bits and are never property-key/symbol tags.
    fn emit_regexp_name_equal(
        &mut self,
        left: I64Local,
        right: I64Local,
        equal: I64Local,
        function: &mut Function,
    ) {
        let index = self.runtime_schema().reserve_i64_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        let lhs = self.runtime_schema().reserve_i64_local(function);
        let rhs = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        set(function, equal, 0);
        for (payload, pointer) in [(left, lhs), (right, rhs)] {
            payload.load(function);
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64ShrU);
            pointer.store(function);
        }
        left.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        length.store(function);
        length.load(function);
        right.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, equal, 1);
        set(function, index, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_scratch_byte(lhs, index, byte, function);
        rhs.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        byte.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, equal, 0);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [byte, rhs, lhs, length, index] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Two captures may share a name only if a common owning group places
    /// them in different Sequence arms. This is the runtime AST projection of
    /// the static parser's choice/arm divergence rule, checked pairwise.
    fn emit_regexp_capture_paths_diverge(
        &mut self,
        compiler: &CompilerLocals,
        left: I64Local,
        right: I64Local,
        diverges: I64Local,
        function: &mut Function,
    ) {
        let left_sequence = self.runtime_schema().reserve_i64_local(function);
        let right_sequence = self.runtime_schema().reserve_i64_local(function);
        let left_group = self.runtime_schema().reserve_i64_local(function);
        let right_group = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        set(function, diverges, 0);
        self.emit_regexp_node_address(compiler, left, address, function);
        load(function, address, NodeWord::Parent as u64, left_sequence);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, left_sequence, 0);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_node_address(compiler, left_sequence, address, function);
        load(function, address, NodeWord::Parent as u64, left_group);
        self.emit_regexp_node_address(compiler, right, address, function);
        load(function, address, NodeWord::Parent as u64, right_sequence);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, right_sequence, 0);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_node_address(compiler, right_sequence, address, function);
        load(function, address, NodeWord::Parent as u64, right_group);
        left_group.load(function);
        right_group.load(function);
        function.instruction(&Instruction::I64Eq);
        left_sequence.load(function);
        right_sequence.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, diverges, 1);
        function.instruction(&Instruction::End);
        self.emit_regexp_node_address(compiler, right_group, address, function);
        load(function, address, NodeWord::Parent as u64, right_sequence);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_regexp_node_address(compiler, left_group, address, function);
        load(function, address, NodeWord::Parent as u64, left_sequence);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            address,
            right_group,
            left_group,
            right_sequence,
            left_sequence,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }

    pub(super) fn emit_regexp_complete_capture_inventory(
        &mut self,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) -> CompletedCaptureInventory {
        let row = self.runtime_schema().reserve_i64_local(function);
        let prior = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let previous_address = self.runtime_schema().reserve_i64_local(function);
        let payload = self.runtime_schema().reserve_i64_local(function);
        let previous_payload = self.runtime_schema().reserve_i64_local(function);
        let owner = self.runtime_schema().reserve_i64_local(function);
        let previous_owner = self.runtime_schema().reserve_i64_local(function);
        let group = self.runtime_schema().reserve_i64_local(function);
        let equal = self.runtime_schema().reserve_i64_local(function);
        let diverges = self.runtime_schema().reserve_i64_local(function);
        set(function, row, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row.load(function);
        compiler.name_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_name_row_address(compiler, row, address, function);
        load(function, address, NameWord::Payload as u64, payload);
        load(function, address, NameWord::Owner as u64, owner);
        copy(function, group, compiler.unique_name_count);
        set(function, prior, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        prior.load(function);
        row.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_name_row_address(compiler, prior, previous_address, function);
        load(
            function,
            previous_address,
            NameWord::Payload as u64,
            previous_payload,
        );
        self.emit_regexp_name_equal(payload, previous_payload, equal, function);
        eq(function, equal, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(
            function,
            previous_address,
            NameWord::Owner as u64,
            previous_owner,
        );
        self.emit_regexp_capture_paths_diverge(compiler, owner, previous_owner, diverges, function);
        eq(function, diverges, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_node_address(compiler, owner, previous_address, function);
        load(
            function,
            previous_address,
            NodeWord::SourceOffset as u64,
            compiler.cursor,
        );
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::DuplicateGroupName),
            function,
        );
        function.instruction(&Instruction::End);
        // The node lookup above is only on the returning failure branch.
        load(function, previous_address, NameWord::Group as u64, group);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(prior, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        store(function, address, NameWord::Group as u64, group);
        group.load(function);
        compiler.unique_name_count.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.unique_name_count, 1, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(row, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // Forward references resolve only after the complete capture census
        // and canonical names have been validated. No unresolved payload is
        // admitted to instruction lowering.
        set(function, row, 1);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row.load(function);
        compiler.node_count.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_node_address(compiler, row, address, function);
        load(function, address, NodeWord::Opcode as u64, group);
        eq(function, group, REGEXP_OPCODE_NAMED_BACKREFERENCE);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(function, address, NodeWord::Operand0 as u64, payload);
        set(function, prior, 0);
        set(function, group, u64::MAX);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        prior.load(function);
        compiler.name_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_name_row_address(compiler, prior, previous_address, function);
        load(
            function,
            previous_address,
            NameWord::Payload as u64,
            previous_payload,
        );
        self.emit_regexp_name_equal(payload, previous_payload, equal, function);
        eq(function, equal, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(function, previous_address, NameWord::Group as u64, group);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(prior, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, group, u64::MAX);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(
            function,
            address,
            NodeWord::SourceOffset as u64,
            compiler.cursor,
        );
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::UnknownGroupName),
            function,
        );
        function.instruction(&Instruction::End);
        store(function, address, NodeWord::Operand0 as u64, group);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(row, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            diverges,
            equal,
            group,
            previous_owner,
            owner,
            previous_payload,
            payload,
            previous_address,
            address,
            prior,
            row,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
        CompletedCaptureInventory {
            group_count: compiler.unique_name_count,
        }
    }

    pub(super) fn emit_regexp_capture_table(
        &mut self,
        compiler: &CompilerLocals,
        captures: CompletedCaptureInventory,
        length: I64Local,
        named_offset: I64Local,
        function: &mut Function,
    ) {
        let table = self.runtime_schema().reserve_i64_local(function);
        let record = self.runtime_schema().reserve_i64_local(function);
        let candidate = self.runtime_schema().reserve_i64_local(function);
        let names = self.runtime_schema().reserve_i64_local(function);
        let group = self.runtime_schema().reserve_i64_local(function);
        let row = self.runtime_schema().reserve_i64_local(function);
        let address = self.runtime_schema().reserve_i64_local(function);
        let count = self.runtime_schema().reserve_i64_local(function);
        let value = self.runtime_schema().reserve_i64_local(function);
        let payload = self.runtime_schema().reserve_i64_local(function);
        let name_length = self.runtime_schema().reserve_i64_local(function);
        set(function, named_offset, 0);
        captures.emit_group_count(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, named_offset, length);
        compiler.descriptor.load(function);
        length.load(function);
        function.instruction(&Instruction::I64Add);
        table.store(function);
        for (offset, constant) in [(0, REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION), (24, 32)] {
            self.emit_regexp_scratch_store_word_const(table, offset, constant, function);
        }
        captures.emit_group_count(function);
        value.store(function);
        store(function, table, 8, value);
        store(function, table, 16, compiler.name_count);
        table.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        record.store(function);
        record.load(function);
        captures.emit_group_count(function);
        function.instruction(&Instruction::I64Const(24));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        candidate.store(function);
        candidate.load(function);
        compiler.name_count.load(function);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        names.store(function);
        set(function, group, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        group.load(function);
        captures.emit_group_count(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        candidate.load(function);
        table.load(function);
        function.instruction(&Instruction::I64Sub);
        value.store(function);
        store(function, record, 8, value);
        set(function, row, 0);
        set(function, count, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row.load(function);
        compiler.name_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_name_row_address(compiler, row, address, function);
        load(function, address, NameWord::Group as u64, value);
        value.load(function);
        group.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, count, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        load(function, address, NameWord::Payload as u64, payload);
        payload.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        name_length.store(function);
        names.load(function);
        table.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        name_length.load(function);
        function.instruction(&Instruction::I64Or);
        value.store(function);
        store(function, record, 0, value);
        names.load(function);
        function.instruction(&Instruction::I32WrapI64);
        payload.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        name_length.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        self.emit_regexp_scratch_increment_by_local(names, name_length, function);
        function.instruction(&Instruction::End);
        load(function, address, NameWord::Capture as u64, value);
        store(function, candidate, 0, value);
        self.emit_regexp_scratch_increment(candidate, 8, function);
        self.emit_regexp_scratch_increment(count, 1, function);
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(row, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        store(function, record, 16, count);
        self.emit_regexp_scratch_increment(record, 24, function);
        self.emit_regexp_scratch_increment(group, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        names.load(function);
        compiler.descriptor.load(function);
        function.instruction(&Instruction::I64Sub);
        length.store(function);
        function.instruction(&Instruction::End);
        for local in [
            name_length,
            payload,
            value,
            count,
            address,
            row,
            group,
            names,
            candidate,
            record,
            table,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
