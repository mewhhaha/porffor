use super::*;
use lila_ir::RegExpScopedModifier;

pub(super) const NODE_BYTES: u64 = 152;
pub(super) const TASK_BYTES: u64 = 24;
pub(super) const CLASS_BITMAP_BYTES: u64 = 8192;
pub(super) const UNBOUNDED: u64 = u64::MAX;
pub(super) const NODE_LAZY: u64 = 1;
pub(super) const NODE_ATOM_NULLABLE: u64 = 2;
pub(super) const NODE_OVERSIZED_BOUNDS: u64 = 4;
pub(super) const FLAG_IGNORE_CASE: u64 = RegExpScopedModifier::IgnoreCase.bit() as u64;
pub(super) const FLAG_MULTILINE: u64 = RegExpScopedModifier::Multiline.bit() as u64;
pub(super) const FLAG_DOT_ALL: u64 = RegExpScopedModifier::DotAll.bit() as u64;

#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum NodeKind {
    Atom = 1,
    Sequence,
    Root,
    Capture,
    NonCapture,
    PositiveLookahead,
    NegativeLookahead,
}

/// Parents are allocated before children. Zero is the absent node; the root is
/// node one. Group First/Last link Sequence nodes; Sequence First/Last link
/// quantified terms. Next links siblings only, never children.
#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum NodeWord {
    Kind = 0,
    Next = 8,
    First = 16,
    Last = 24,
    Parent = 32,
    Opcode = 40,
    Operand0 = 48,
    Operand1 = 56,
    CaptureStart = 64,
    CaptureEnd = 72,
    Minimum = 80,
    Maximum = 88,
    Flags = 96,
    SourceOffset = 104,
    AtomWidth = 112,
    Width = 120,
    IgnoreCase = 128,
    Multiline = 136,
    DotAll = 144,
}

/// Every field is an i64 Wasm local. Pointer locals refer only to the checked
/// workspace or immutable folding table; counts are not encoded in pointers.
/// The parser owns node_count, capture_count, range_count and cursor. The
/// lowerer owns instruction_count, split_count and repeatable_split_count.
pub(super) struct CompilerLocals {
    pub(super) heap_checkpoint: u32,
    pub(super) source_pointer: u32,
    pub(super) source_length: u32,
    pub(super) units: u32,
    pub(super) unit_count: u32,
    pub(super) unit_capacity: u32,
    pub(super) cursor: u32,
    pub(super) flags: u32,
    pub(super) nodes: u32,
    pub(super) node_count: u32,
    pub(super) node_capacity: u32,
    pub(super) capture_count: u32,
    pub(super) tasks: u32,
    pub(super) task_capacity: u32,
    pub(super) descriptor: u32,
    pub(super) instructions: u32,
    pub(super) instruction_count: u32,
    pub(super) ranges: u32,
    pub(super) range_count: u32,
    pub(super) split_count: u32,
    pub(super) repeatable_split_count: u32,
    pub(super) class_bitmap: u32,
    pub(super) folded_bitmap: u32,
    pub(super) graph_visited: u32,
    pub(super) graph_work: u32,
    pub(super) graph_colors: u32,
    pub(super) graph_edges: u32,
    pub(super) fold_table: u32,
    pub(super) fold_count: u32,
}

#[derive(Clone, Copy)]
#[repr(i64)]
pub(super) enum CompileSyntax {
    InvalidFlags = 1,
    UnexpectedToken,
    UnclosedGroup,
    UnclosedClass,
    InvalidRange,
    InvalidEscape,
    InvalidQuantifier,
    InvalidModifiers,
}
#[derive(Clone, Copy)]
#[repr(i64)]
pub(super) enum CompileCapability {
    Unicode = 1,
    UnicodeSets,
    NamedGroups,
    NamedBackreference,
    Lookbehind,
}
#[derive(Clone, Copy)]
#[repr(i64)]
pub(super) enum CompileResource {
    AddressSpace = 1,
    MemoryGrowth,
    Nodes,
    Tasks,
    Instructions,
    Ranges,
    Repetition,
}
#[derive(Clone, Copy)]
pub(super) enum CompileFailure {
    Syntax(CompileSyntax),
    Unsupported(CompileCapability),
    Resource(CompileResource),
    Corrupt,
}
impl CompileFailure {
    pub(super) const fn status(self) -> RegExpCompilerStatus {
        match self {
            Self::Syntax(_) => RegExpCompilerStatus::SyntaxError,
            Self::Unsupported(_) => RegExpCompilerStatus::Unsupported,
            Self::Resource(_) => RegExpCompilerStatus::ResourceExhausted,
            Self::Corrupt => RegExpCompilerStatus::CorruptProgram,
        }
    }
    pub(super) const fn detail(self) -> i64 {
        match self {
            Self::Syntax(reason) => reason as i64,
            Self::Unsupported(reason) => reason as i64,
            Self::Resource(reason) => reason as i64,
            Self::Corrupt => 0,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_compile_failure(
        &self,
        compiler: &CompilerLocals,
        failure: CompileFailure,
        function: &mut Function,
    ) {
        self.emit_regexp_compile_release_tail(compiler.heap_checkpoint, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(failure.status().abi_word()));
        function.instruction(&Instruction::LocalGet(compiler.cursor));
        function.instruction(&Instruction::I64Const(failure.detail()));
        function.instruction(&Instruction::Return);
    }

    /// No observable call or nested allocation can escape this compiler's
    /// region. Clear released bytes before publishing the lower heap pointer:
    /// later allocation initializes some record fields by relying on zero memory.
    pub(super) fn emit_regexp_compile_release_tail(
        &self,
        retained_end: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(retained_end));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalGet(HEAP_PTR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalGet(retained_end));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        function.instruction(&Instruction::LocalGet(retained_end));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
    }

    pub(super) fn emit_regexp_node_address(
        &self,
        compiler: &CompilerLocals,
        node: u32,
        address: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(compiler.nodes));
        function.instruction(&Instruction::LocalGet(node));
        function.instruction(&Instruction::I64Const(NODE_BYTES as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(address));
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn reserve_regexp_compiler_locals(&mut self) -> CompilerLocals {
        CompilerLocals {
            heap_checkpoint: self.reserve_temp_local(),
            source_pointer: self.reserve_temp_local(),
            source_length: self.reserve_temp_local(),
            units: self.reserve_temp_local(),
            unit_count: self.reserve_temp_local(),
            unit_capacity: self.reserve_temp_local(),
            cursor: self.reserve_temp_local(),
            flags: self.reserve_temp_local(),
            nodes: self.reserve_temp_local(),
            node_count: self.reserve_temp_local(),
            node_capacity: self.reserve_temp_local(),
            capture_count: self.reserve_temp_local(),
            tasks: self.reserve_temp_local(),
            task_capacity: self.reserve_temp_local(),
            descriptor: self.reserve_temp_local(),
            instructions: self.reserve_temp_local(),
            instruction_count: self.reserve_temp_local(),
            ranges: self.reserve_temp_local(),
            range_count: self.reserve_temp_local(),
            split_count: self.reserve_temp_local(),
            repeatable_split_count: self.reserve_temp_local(),
            class_bitmap: self.reserve_temp_local(),
            folded_bitmap: self.reserve_temp_local(),
            graph_visited: self.reserve_temp_local(),
            graph_work: self.reserve_temp_local(),
            graph_colors: self.reserve_temp_local(),
            graph_edges: self.reserve_temp_local(),
            fold_table: self.reserve_temp_local(),
            fold_count: self.reserve_temp_local(),
        }
    }
    pub(super) fn release_regexp_compiler_locals(&mut self, compiler: CompilerLocals) {
        for local in [
            compiler.fold_count,
            compiler.fold_table,
            compiler.graph_edges,
            compiler.graph_colors,
            compiler.graph_work,
            compiler.graph_visited,
            compiler.folded_bitmap,
            compiler.class_bitmap,
            compiler.repeatable_split_count,
            compiler.split_count,
            compiler.range_count,
            compiler.ranges,
            compiler.instruction_count,
            compiler.instructions,
            compiler.descriptor,
            compiler.task_capacity,
            compiler.tasks,
            compiler.capture_count,
            compiler.node_capacity,
            compiler.node_count,
            compiler.nodes,
            compiler.flags,
            compiler.cursor,
            compiler.unit_capacity,
            compiler.unit_count,
            compiler.units,
            compiler.source_length,
            compiler.source_pointer,
            compiler.heap_checkpoint,
        ] {
            self.release_temp_local(local);
        }
    }
}
