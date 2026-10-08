use super::*;
use lila_ir::{RegExpScopedModifier, RegExpUnicodeMode};

pub(super) const NODE_BYTES: u64 = 200;
pub(super) const TASK_BYTES: u64 = 24;
pub(super) const CLASS_SET_FRAME_BYTES: u64 = 64;
pub(super) const NODE_LAZY: u64 = 1;
pub(super) const NODE_ATOM_NULLABLE: u64 = 2;
/// Written only by the checked bottom-up width pass, never by the parser.
pub(super) const NODE_PURE_EPSILON: u64 = 4;
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
    PositiveLookbehind,
    NegativeLookbehind,
    /// A completed finite string/code-point set backed by its private payload.
    FiniteClassSet,
}

impl NodeKind {
    pub(super) const ALL: [Self; 10] = [
        Self::Atom,
        Self::Sequence,
        Self::Root,
        Self::Capture,
        Self::NonCapture,
        Self::PositiveLookahead,
        Self::NegativeLookahead,
        Self::PositiveLookbehind,
        Self::NegativeLookbehind,
        Self::FiniteClassSet,
    ];

    pub(super) const fn composes_pure_epsilon(self) -> bool {
        match self {
            Self::Sequence | Self::Root | Self::NonCapture => true,
            Self::Atom
            | Self::Capture
            | Self::PositiveLookahead
            | Self::NegativeLookahead
            | Self::PositiveLookbehind
            | Self::NegativeLookbehind
            | Self::FiniteClassSet => false,
        }
    }
}

/// This checkpoint-owned payload is published only by completed finite-set
/// preparation. Width and instruction lowering consume the same closed words.
#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum FiniteClassSetAtomWord {
    RowsPointer = 0,
    RowCount = 8,
    SingletonOpcode = 16,
    SingletonOperand0 = 24,
    SingletonOperand1 = 32,
    ContainsEmpty = 40,
}

impl FiniteClassSetAtomWord {
    pub(super) const BYTES: u64 = 48;
}

/// Each prepared multi-code-point member retains one ordinary matcher
/// instruction per position; source-sized parser nodes are never expanded.
#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum FiniteClassSetStringWord {
    InstructionsPointer = 0,
    CodePointLength = 8,
}

impl FiniteClassSetStringWord {
    pub(super) const BYTES: u64 = 16;
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
    /// Exact zero/one/many classification, never the numeric repetition count.
    Minimum = 80,
    Maximum = 88,
    Flags = 96,
    SourceOffset = 104,
    AtomWidth = 112,
    Width = 120,
    IgnoreCase = 128,
    Multiline = 136,
    DotAll = 144,
    /// Direction of this node in its containing sequence; assertions own a
    /// separately selected child direction. Only 0 (forward) and 1 (reverse).
    Direction = 152,
    MaximumKind = 160,
    MinimumDigitsStart = 168,
    MinimumDigitsEnd = 176,
    MaximumDigitsStart = 184,
    MaximumDigitsEnd = 192,
}

/// These words only select the existing zero/one/star/consuming-plus layouts.
/// Counted repetition always consumes the original canonical decimal span.
#[derive(Clone, Copy)]
#[repr(u64)]
pub(super) enum BoundClass {
    Zero = 0,
    One = 1,
    Many = 2,
}

/// Every field is an i64 Wasm local. Pointer locals refer only to the checked
/// workspace or immutable folding table; counts are not encoded in pointers.
/// The parser owns node_count, capture_count, range_count and cursor. The
/// lowerer owns instruction_count, split_count and repeatable_split_count.
pub(in crate::builtins) struct CompilerLocals {
    pub(super) heap_checkpoint: I64Local,
    pub(super) source_pointer: I64Local,
    pub(super) source_length: I64Local,
    pub(super) units: I64Local,
    pub(super) unit_count: I64Local,
    pub(super) unit_capacity: I64Local,
    pub(super) cursor: I64Local,
    pub(super) flags: I64Local,
    pub(super) nodes: I64Local,
    pub(super) node_count: I64Local,
    pub(super) node_capacity: I64Local,
    pub(super) capture_count: I64Local,
    pub(super) name_rows: I64Local,
    pub(super) name_count: I64Local,
    pub(super) name_bytes: I64Local,
    pub(super) name_byte_length: I64Local,
    pub(super) unique_name_count: I64Local,
    pub(super) tasks: I64Local,
    pub(super) task_capacity: I64Local,
    pub(super) descriptor: I64Local,
    pub(super) instructions: I64Local,
    pub(super) instruction_count: I64Local,
    pub(super) ranges: I64Local,
    pub(super) range_count: I64Local,
    pub(super) split_count: I64Local,
    pub(super) repeatable_split_count: I64Local,
    pub(super) repeat_slot_count: I64Local,
    pub(super) repeat_state_byte_length: I64Local,
    pub(super) repeat_rows: I64Local,
    pub(super) class_bitmap: I64Local,
    pub(super) folded_bitmap: I64Local,
    pub(super) property_bitmap: I64Local,
    pub(super) class_set_frames: I64Local,
    pub(super) graph_visited: I64Local,
    pub(super) graph_work: I64Local,
    pub(super) graph_colors: I64Local,
    pub(super) graph_edges: I64Local,
}

fn character_mode_word(mode: RegExpUnicodeMode) -> i64 {
    match mode {
        RegExpUnicodeMode::Legacy => 0,
        RegExpUnicodeMode::Unicode => 1,
        RegExpUnicodeMode::UnicodeSets => 2,
    }
}

/// Minted only by the emitted flag-validation factory after every abrupt
/// syntax/capability route has returned. Its live mode and folding locals own
/// the actual workspace domain; callers cannot choose them independently.
#[must_use]
pub(super) struct CompilerCharacterMode {
    mode: I64Local,
    fold_table: I64Local,
    fold_count: I64Local,
}

impl CompilerCharacterMode {
    pub(super) fn emit_is_unicode(&self, function: &mut Function) {
        self.mode.load(function);
        function.instruction(&Instruction::I64Const(character_mode_word(
            RegExpUnicodeMode::Legacy,
        )));
        function.instruction(&Instruction::I64Ne);
    }

    /// Unicode grammar always enables named references, including a pattern
    /// with no declarations. Annex B Legacy grammar uses the complete census.
    pub(super) fn emit_uses_named_capture_grammar(
        &self,
        named_captures: I64Local,
        function: &mut Function,
    ) {
        self.emit_is_unicode(function);
        named_captures.load(function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    pub(super) fn emit_is_unicode_sets(&self, function: &mut Function) {
        self.mode.load(function);
        function.instruction(&Instruction::I64Const(character_mode_word(
            RegExpUnicodeMode::UnicodeSets,
        )));
        function.instruction(&Instruction::I64Eq);
    }

    pub(super) fn emit_cardinality(&self, function: &mut Function) {
        self.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(0x110000));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_bitmap_bytes(&self, function: &mut Function) {
        self.emit_cardinality(function);
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64ShrU);
    }

    pub(super) fn emit_require_character(
        &self,
        emitter: &FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        character: I64Local,
        function: &mut Function,
    ) {
        character.load(function);
        self.emit_cardinality(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
    }

    pub(super) const fn fold_table(&self) -> I64Local {
        self.fold_table
    }
    pub(super) const fn fold_count(&self) -> I64Local {
        self.fold_count
    }
}

/// Owns the workspace cursor while regions are being laid out. Every region
/// reservation retains the eight-byte descriptor/word alignment; callers get
/// the end local back only after padding and the complete capacity are checked.
/// A new byte region cannot accidentally publish an unaligned successor region.
pub(super) struct CompilerWorkspace {
    end: I64Local,
}

/// Only a completed aligned, addressable region can extend the checkpoint-owned
/// compiler heap. The commit emitter cannot consume an unchecked end local.
pub(super) struct CheckedCompilerWorkspaceEnd {
    end: I64Local,
}

impl CompilerWorkspace {
    fn align_end(&self, function: &mut Function) {
        self.end.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(-8));
        function.instruction(&Instruction::I64And);
        self.end.store(function);
    }

    pub(super) fn reserve_elements(
        &mut self,
        pointer: I64Local,
        count: I64Local,
        stride: u64,
        function: &mut Function,
    ) {
        self.end.load(function);
        pointer.store(function);
        pointer.load(function);
        count.load(function);
        function.instruction(&Instruction::I64Const(stride as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        self.end.store(function);
        self.align_end(function);
    }

    pub(super) fn reserve_bytes(
        &mut self,
        pointer: I64Local,
        length: u64,
        function: &mut Function,
    ) {
        self.end.load(function);
        pointer.store(function);
        pointer.load(function);
        function.instruction(&Instruction::I64Const(length as i64));
        function.instruction(&Instruction::I64Add);
        self.end.store(function);
        self.align_end(function);
    }

    pub(super) fn reserve_character_bitmap(
        &mut self,
        pointer: I64Local,
        character_mode: &CompilerCharacterMode,
        function: &mut Function,
    ) {
        self.end.load(function);
        pointer.store(function);
        pointer.load(function);
        character_mode.emit_bitmap_bytes(function);
        function.instruction(&Instruction::I64Add);
        self.end.store(function);
        self.align_end(function);
    }

    /// Named output needs a 32-byte header, at most one 24-byte unique-name
    /// record and one 8-byte candidate per source unit, plus up to 4 bytes per
    /// decoded scalar. Products and padding fit u64 because the input is u32
    /// bytes and every compiler workspace capacity is linear in that bound.
    pub(super) fn finish(
        self,
        emitter: &FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) -> CheckedCompilerWorkspaceEnd {
        self.end.load(function);
        compiler.unit_capacity.load(function);
        function.instruction(&Instruction::I64Const(36));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Add);
        // Publication owns one 48-byte row per source node, at most two
        // synthetic digits per node and twice the source decimal digits.
        compiler.node_capacity.load(function);
        function.instruction(&Instruction::I64Const(50));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        compiler.unit_capacity.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Add);
        self.end.store(function);
        self.finish_region(emitter, compiler, function)
    }

    pub(super) fn finish_region(
        self,
        emitter: &FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        function: &mut Function,
    ) -> CheckedCompilerWorkspaceEnd {
        self.align_end(function);
        self.end.load(function);
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::AddressSpace),
            function,
        );
        function.instruction(&Instruction::End);
        CheckedCompilerWorkspaceEnd { end: self.end }
    }
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
    InvalidGroupName,
    DuplicateGroupName,
    UnknownGroupName,
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
}
#[derive(Clone, Copy)]
pub(super) enum CompileFailure {
    Syntax(CompileSyntax),
    Resource(CompileResource),
    Corrupt,
}
impl CompileFailure {
    pub(super) const fn status(self) -> RegExpCompilerStatus {
        match self {
            Self::Syntax(_) => RegExpCompilerStatus::SyntaxError,
            Self::Resource(_) => RegExpCompilerStatus::ResourceExhausted,
            Self::Corrupt => RegExpCompilerStatus::CorruptProgram,
        }
    }
    pub(super) const fn detail(self) -> i64 {
        match self {
            Self::Syntax(reason) => reason as i64,
            Self::Resource(reason) => reason as i64,
            Self::Corrupt => 0,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn begin_regexp_workspace(
        &self,
        compiler: &CompilerLocals,
        end: I64Local,
        function: &mut Function,
    ) -> CompilerWorkspace {
        compiler.heap_checkpoint.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        end.store(function);
        CompilerWorkspace { end }
    }

    /// Class-depth bitmaps grow the same private region after UTF16 decoding.
    /// No JavaScript call can observe or allocate into this compiler checkpoint.
    pub(super) fn extend_regexp_workspace(
        &self,
        compiler: &CompilerLocals,
        end: I64Local,
        function: &mut Function,
    ) -> CompilerWorkspace {
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        end.store(function);
        end.load(function);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        CompilerWorkspace { end }
    }

    pub(super) fn emit_regexp_commit_workspace(
        &mut self,
        compiler: &CompilerLocals,
        region: CheckedCompilerWorkspaceEnd,
        function: &mut Function,
    ) {
        let start = self.runtime_schema().reserve_i64_local(function);
        let available = self.runtime_schema().reserve_i64_local(function);
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        start.store(function);
        region.end.load(function);
        start.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::MemorySize(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Const(65_536));
        function.instruction(&Instruction::I64Mul);
        available.store(function);
        region.end.load(function);
        available.load(function);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        region.end.load(function);
        available.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(65_535));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryGrow(0));
        function.instruction(&Instruction::I32Const(-1));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::MemoryGrowth),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        region.end.load(function);
        function.instruction(&Instruction::GlobalSet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        start.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        region.end.load(function);
        start.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        self.runtime_schema().release_i64_local(available, function);
        self.runtime_schema().release_i64_local(start, function);
    }

    pub(super) fn emit_regexp_compile_failure(
        &self,
        compiler: &CompilerLocals,
        failure: CompileFailure,
        function: &mut Function,
    ) {
        self.emit_regexp_compile_release_tail(compiler.heap_checkpoint, function);
        function.instruction(&Instruction::RefNull(
            self.runtime_schema()
                .reference_type::<RegExpProgram>(GcNullability::Nullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Const(failure.status().abi_word() as i32));
        compiler.cursor.load(function);
        function.instruction(&Instruction::I64Const(failure.detail()));
        function.instruction(&Instruction::Return);
    }

    /// Parser/backtracking bytes have no JavaScript roots. The completed GC
    /// program survives consuming rollback of this entire private byte region.
    pub(super) fn emit_regexp_compile_release_tail(
        &self,
        retained_end: I64Local,
        function: &mut Function,
    ) {
        retained_end.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalGet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
        retained_end.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryFill(0));
        retained_end.load(function);
        function.instruction(&Instruction::GlobalSet(PRIVATE_BYTE_CURSOR_GLOBAL_INDEX));
    }

    pub(super) fn emit_regexp_node_address(
        &self,
        compiler: &CompilerLocals,
        node: I64Local,
        address: I64Local,
        function: &mut Function,
    ) {
        compiler.nodes.load(function);
        node.load(function);
        function.instruction(&Instruction::I64Const(NODE_BYTES as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        address.store(function);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn reserve_regexp_compiler_locals(
        &mut self,
        function: &mut Function,
    ) -> CompilerLocals {
        CompilerLocals {
            heap_checkpoint: self.runtime_schema().reserve_i64_local(function),
            source_pointer: self.runtime_schema().reserve_i64_local(function),
            source_length: self.runtime_schema().reserve_i64_local(function),
            units: self.runtime_schema().reserve_i64_local(function),
            unit_count: self.runtime_schema().reserve_i64_local(function),
            unit_capacity: self.runtime_schema().reserve_i64_local(function),
            cursor: self.runtime_schema().reserve_i64_local(function),
            flags: self.runtime_schema().reserve_i64_local(function),
            nodes: self.runtime_schema().reserve_i64_local(function),
            node_count: self.runtime_schema().reserve_i64_local(function),
            node_capacity: self.runtime_schema().reserve_i64_local(function),
            capture_count: self.runtime_schema().reserve_i64_local(function),
            name_rows: self.runtime_schema().reserve_i64_local(function),
            name_count: self.runtime_schema().reserve_i64_local(function),
            name_bytes: self.runtime_schema().reserve_i64_local(function),
            name_byte_length: self.runtime_schema().reserve_i64_local(function),
            unique_name_count: self.runtime_schema().reserve_i64_local(function),
            tasks: self.runtime_schema().reserve_i64_local(function),
            task_capacity: self.runtime_schema().reserve_i64_local(function),
            descriptor: self.runtime_schema().reserve_i64_local(function),
            instructions: self.runtime_schema().reserve_i64_local(function),
            instruction_count: self.runtime_schema().reserve_i64_local(function),
            ranges: self.runtime_schema().reserve_i64_local(function),
            range_count: self.runtime_schema().reserve_i64_local(function),
            split_count: self.runtime_schema().reserve_i64_local(function),
            repeatable_split_count: self.runtime_schema().reserve_i64_local(function),
            repeat_slot_count: self.runtime_schema().reserve_i64_local(function),
            repeat_state_byte_length: self.runtime_schema().reserve_i64_local(function),
            repeat_rows: self.runtime_schema().reserve_i64_local(function),
            class_bitmap: self.runtime_schema().reserve_i64_local(function),
            folded_bitmap: self.runtime_schema().reserve_i64_local(function),
            property_bitmap: self.runtime_schema().reserve_i64_local(function),
            class_set_frames: self.runtime_schema().reserve_i64_local(function),
            graph_visited: self.runtime_schema().reserve_i64_local(function),
            graph_work: self.runtime_schema().reserve_i64_local(function),
            graph_colors: self.runtime_schema().reserve_i64_local(function),
            graph_edges: self.runtime_schema().reserve_i64_local(function),
        }
    }
    pub(super) fn release_regexp_compiler_locals(
        &mut self,
        compiler: CompilerLocals,
        function: &mut Function,
    ) {
        for local in [
            compiler.graph_edges,
            compiler.graph_colors,
            compiler.graph_work,
            compiler.graph_visited,
            compiler.class_set_frames,
            compiler.property_bitmap,
            compiler.folded_bitmap,
            compiler.class_bitmap,
            compiler.repeat_slot_count,
            compiler.repeat_state_byte_length,
            compiler.repeat_rows,
            compiler.repeatable_split_count,
            compiler.split_count,
            compiler.range_count,
            compiler.ranges,
            compiler.instruction_count,
            compiler.instructions,
            compiler.descriptor,
            compiler.task_capacity,
            compiler.tasks,
            compiler.unique_name_count,
            compiler.name_byte_length,
            compiler.name_bytes,
            compiler.name_count,
            compiler.name_rows,
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
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_compile_flags(
        &mut self,
        compiler: &CompilerLocals,
        flags: &GcLocal<StringValue>,
        function: &mut Function,
    ) -> CompilerCharacterMode {
        // These locals outlive all flag-parser temporaries and are released
        // only after the parser has consumed their domain and folding policy.
        let mode = self.runtime_schema().reserve_i64_local(function);
        let fold_table = self.runtime_schema().reserve_i64_local(function);
        let fold_count = self.runtime_schema().reserve_i64_local(function);
        let schema = self.runtime_schema();
        let units = schema.reserve_gc_local(function).initialize(
            schema
                .field(StringValueSchema::CODE_UNITS)
                .read(flags, schema, function)
                .reference(),
            function,
        );
        let unit_index = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        let length = self.runtime_schema().reserve_i64_local(function);
        let index = self.runtime_schema().reserve_i64_local(function);
        let byte = self.runtime_schema().reserve_i64_local(function);
        let bit = self.runtime_schema().reserve_i64_local(function);
        let seen = self.runtime_schema().reserve_i64_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        function.instruction(&Instruction::I64ExtendI32U);
        length.store(function);
        for local in [index, seen, compiler.flags] {
            function.instruction(&Instruction::I64Const(0));
            local.store(function);
        }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        index.load(function);
        function.instruction(&Instruction::I32WrapI64);
        unit_index.store(function);
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, unit_index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        byte.store(function);
        function.instruction(&Instruction::I64Const(0));
        bit.store(function);
        for (position, flag) in b"dgimsuvy".iter().copied().enumerate() {
            byte.load(function);
            function.instruction(&Instruction::I64Const(flag as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(1 << position));
            bit.store(function);
            function.instruction(&Instruction::End);
        }
        bit.load(function);
        function.instruction(&Instruction::I64Eqz);
        bit.load(function);
        seen.load(function);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidFlags),
            function,
        );
        function.instruction(&Instruction::End);
        bit.load(function);
        seen.load(function);
        function.instruction(&Instruction::I64Or);
        seen.store(function);
        self.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        seen.load(function);
        function.instruction(&Instruction::I64Const(0x60));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0x60));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidFlags),
            function,
        );
        function.instruction(&Instruction::End);
        seen.load(function);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(
            (FLAG_IGNORE_CASE | FLAG_MULTILINE | FLAG_DOT_ALL) as i64,
        ));
        function.instruction(&Instruction::I64And);
        compiler.flags.store(function);
        function.instruction(&Instruction::I64Const(character_mode_word(
            RegExpUnicodeMode::Legacy,
        )));
        mode.store(function);
        for (mask, kind) in [
            (0x20, RegExpUnicodeMode::Unicode),
            (0x40, RegExpUnicodeMode::UnicodeSets),
        ] {
            seen.load(function);
            function.instruction(&Instruction::I64Const(mask));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(character_mode_word(kind)));
            mode.store(function);
            function.instruction(&Instruction::End);
        }
        let character_mode = CompilerCharacterMode {
            mode,
            fold_table,
            fold_count,
        };
        character_mode.emit_is_unicode(function);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (index, folding) in [RegExpCaseFolding::Unicode, RegExpCaseFolding::Legacy]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                function.instruction(&Instruction::Else);
            }
            let table = self
                .strings
                .regexp_case_folding_table(folding)
                .expect("runtime RegExp compiler requires both character-mode folding tables");
            function.instruction(&Instruction::I64Const(table.ptr as i64));
            fold_table.store(function);
            function.instruction(&Instruction::I64Const(table.count as i64));
            fold_count.store(function);
        }
        function.instruction(&Instruction::End);
        schema.release_i32_local(unit, function);
        schema.release_i32_local(unit_index, function);
        units.clear(function);
        for local in [seen, bit, byte, index, length] {
            self.runtime_schema().release_i64_local(local, function);
        }
        character_mode
    }

    pub(super) fn release_regexp_compiler_character_mode(
        &mut self,
        mode: CompilerCharacterMode,
        function: &mut Function,
    ) {
        self.runtime_schema()
            .release_i64_local(mode.fold_count, function);
        self.runtime_schema()
            .release_i64_local(mode.fold_table, function);
        self.runtime_schema().release_i64_local(mode.mode, function);
    }
}
