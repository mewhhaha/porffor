//! Canonical finite string keys in the compiler's checked private workspace.
use super::*;

mod prepare;

#[derive(Clone, Copy)]
#[repr(u64)]
enum FiniteKeyWord {
    Next = 0,
    CodePointsPointer = 8,
    CodePointLength = 16,
}
impl FiniteKeyWord {
    const BYTES: u64 = 24;
}

/// This set contains canonical empty or multi-code-point keys, ordered
/// lexicographically and unique. Singleton keys belong to the operand bitmap.
/// Only decoded q characters and validated property rows can insert members.
pub(super) struct FiniteStringSet {
    head: I64Local,
}

/// Source characters live in private workspace cells, never parser AST nodes.
/// Their checked count supplies the completed alternative's contiguous key.
pub(super) struct DecodedQString {
    first: I64Local,
    last: I64Local,
    count: I64Local,
}

struct CanonicalFiniteString {
    points: I64Local,
    count: I64Local,
}

impl FiniteStringSet {
    pub(super) fn reserve(emitter: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        let head = emitter.runtime_schema().reserve_i64_local(function);
        set(function, head, 0);
        Self { head }
    }
    pub(super) fn head_local(&self) -> I64Local {
        self.head
    }
    pub(super) fn emit_clear(&self, function: &mut Function) {
        set(function, self.head, 0);
    }
    pub(super) fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        emitter
            .runtime_schema()
            .release_i64_local(self.head, function);
    }

    pub(super) fn emit_insert_decoded(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        decoded: &DecodedQString,
        function: &mut Function,
    ) {
        let key = CanonicalFiniteString {
            points: emitter.runtime_schema().reserve_i64_local(function),
            count: emitter.runtime_schema().reserve_i64_local(function),
        };
        copy(function, key.count, decoded.count);
        set(function, key.points, 0);
        eq(function, key.count, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_finite_workspace_elements(compiler, key.points, key.count, 8, function);
        let cursor = emitter.runtime_schema().reserve_i64_local(function);
        let index = emitter.runtime_schema().reserve_i64_local(function);
        let character = emitter.runtime_schema().reserve_i64_local(function);
        copy(function, cursor, decoded.first);
        set(function, index, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        key.count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        eq(function, cursor, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        emitter.emit_regexp_compile_failure(compiler, CompileFailure::Corrupt, function);
        function.instruction(&Instruction::End);
        load(function, cursor, 8, character);
        emit_indexed_point_address(function, key.points, index);
        character.load(function);
        function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(0)));
        load(function, cursor, 0, cursor);
        emitter.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [character, index, cursor] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
        function.instruction(&Instruction::End);
        self.emit_insert_canonical(emitter, compiler, mode, &key, function);
        emitter
            .runtime_schema()
            .release_i64_local(key.count, function);
        emitter
            .runtime_schema()
            .release_i64_local(key.points, function);
    }

    pub(super) fn emit_insert_image(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        points: I64Local,
        count: I64Local,
        function: &mut Function,
    ) {
        let key = CanonicalFiniteString {
            points: emitter.runtime_schema().reserve_i64_local(function),
            count: emitter.runtime_schema().reserve_i64_local(function),
        };
        copy(function, key.count, count);
        emitter.emit_regexp_finite_workspace_elements(compiler, key.points, key.count, 8, function);
        let index = emitter.runtime_schema().reserve_i64_local(function);
        let character = emitter.runtime_schema().reserve_i64_local(function);
        set(function, index, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        key.count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        points.load(function);
        index.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load32U(FunctionBuilder::memarg32(0)));
        character.store(function);
        mode.emit_require_character(emitter, compiler, character, function);
        emitter.emit_regexp_finite_canonical_character(mode, modifiers, character, function);
        emit_indexed_point_address(function, key.points, index);
        character.load(function);
        function.instruction(&Instruction::I64Store(FunctionBuilder::memarg64(0)));
        emitter.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        emitter
            .runtime_schema()
            .release_i64_local(character, function);
        emitter.runtime_schema().release_i64_local(index, function);
        self.emit_insert_canonical(emitter, compiler, mode, &key, function);
        emitter
            .runtime_schema()
            .release_i64_local(key.count, function);
        emitter
            .runtime_schema()
            .release_i64_local(key.points, function);
    }

    fn emit_insert_canonical(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        key: &CanonicalFiniteString,
        function: &mut Function,
    ) {
        eq(function, key.count, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        let character = emitter.runtime_schema().reserve_i64_local(function);
        load(function, key.points, 0, character);
        emitter.emit_regexp_parser_bitmap_range(
            compiler,
            mode,
            compiler.class_bitmap,
            character,
            character,
            function,
        );
        emitter
            .runtime_schema()
            .release_i64_local(character, function);
        function.instruction(&Instruction::Else);
        self.emit_insert_key(emitter, compiler, key, function);
        function.instruction(&Instruction::End);
    }

    fn emit_insert_key(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        key: &CanonicalFiniteString,
        function: &mut Function,
    ) {
        let cursor = emitter.runtime_schema().reserve_i64_local(function);
        let previous = emitter.runtime_schema().reserve_i64_local(function);
        let other_points = emitter.runtime_schema().reserve_i64_local(function);
        let other_count = emitter.runtime_schema().reserve_i64_local(function);
        let comparison = emitter.runtime_schema().reserve_i64_local(function);
        let node = emitter.runtime_schema().reserve_i64_local(function);
        let one = emitter.runtime_schema().reserve_i64_local(function);
        copy(function, cursor, self.head);
        set(function, previous, 0);
        set(function, comparison, u64::MAX);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, cursor, 0);
        function.instruction(&Instruction::BrIf(1));
        load(
            function,
            cursor,
            FiniteKeyWord::CodePointsPointer as u64,
            other_points,
        );
        load(
            function,
            cursor,
            FiniteKeyWord::CodePointLength as u64,
            other_count,
        );
        emitter.emit_regexp_finite_compare_keys(
            key.points,
            key.count,
            other_points,
            other_count,
            comparison,
            function,
        );
        comparison.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LeS);
        function.instruction(&Instruction::BrIf(1));
        copy(function, previous, cursor);
        load(function, cursor, FiniteKeyWord::Next as u64, cursor);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, cursor, 0);
        eq(function, comparison, 0);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, one, 1);
        emitter.emit_regexp_finite_workspace_elements(
            compiler,
            node,
            one,
            FiniteKeyWord::BYTES,
            function,
        );
        store(function, node, FiniteKeyWord::Next as u64, cursor);
        store(
            function,
            node,
            FiniteKeyWord::CodePointsPointer as u64,
            key.points,
        );
        store(
            function,
            node,
            FiniteKeyWord::CodePointLength as u64,
            key.count,
        );
        eq(function, previous, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, self.head, node);
        function.instruction(&Instruction::Else);
        store(function, previous, FiniteKeyWord::Next as u64, node);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [
            one,
            node,
            comparison,
            other_count,
            other_points,
            previous,
            cursor,
        ] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }

    /// Union copies only key records, retaining immutable canonical point
    /// regions. Intersection/subtraction unlink this accumulator's own records;
    /// no nested operand or finished atom shares their mutable links.
    pub(super) fn emit_combine(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        other: &FiniteStringSet,
        operation: super::unicode_sets::ClassSetMode,
        function: &mut Function,
    ) {
        match operation {
            super::unicode_sets::ClassSetMode::Undecided
            | super::unicode_sets::ClassSetMode::Union => {
                let cursor = emitter.runtime_schema().reserve_i64_local(function);
                let key = CanonicalFiniteString {
                    points: emitter.runtime_schema().reserve_i64_local(function),
                    count: emitter.runtime_schema().reserve_i64_local(function),
                };
                copy(function, cursor, other.head);
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                eq(function, cursor, 0);
                function.instruction(&Instruction::BrIf(1));
                load(
                    function,
                    cursor,
                    FiniteKeyWord::CodePointsPointer as u64,
                    key.points,
                );
                load(
                    function,
                    cursor,
                    FiniteKeyWord::CodePointLength as u64,
                    key.count,
                );
                self.emit_insert_key(emitter, compiler, &key, function);
                load(function, cursor, FiniteKeyWord::Next as u64, cursor);
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                for local in [key.count, key.points, cursor] {
                    emitter.runtime_schema().release_i64_local(local, function);
                }
            }
            super::unicode_sets::ClassSetMode::Intersection
            | super::unicode_sets::ClassSetMode::Subtraction => {
                let cursor = emitter.runtime_schema().reserve_i64_local(function);
                let previous = emitter.runtime_schema().reserve_i64_local(function);
                let next = emitter.runtime_schema().reserve_i64_local(function);
                let points = emitter.runtime_schema().reserve_i64_local(function);
                let count = emitter.runtime_schema().reserve_i64_local(function);
                let member = emitter.runtime_schema().reserve_i64_local(function);
                copy(function, cursor, self.head);
                set(function, previous, 0);
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                eq(function, cursor, 0);
                function.instruction(&Instruction::BrIf(1));
                load(function, cursor, FiniteKeyWord::Next as u64, next);
                load(
                    function,
                    cursor,
                    FiniteKeyWord::CodePointsPointer as u64,
                    points,
                );
                load(
                    function,
                    cursor,
                    FiniteKeyWord::CodePointLength as u64,
                    count,
                );
                other.emit_contains(emitter, points, count, member, function);
                let retain = match operation {
                    super::unicode_sets::ClassSetMode::Intersection => 1,
                    super::unicode_sets::ClassSetMode::Subtraction => 0,
                    super::unicode_sets::ClassSetMode::Undecided
                    | super::unicode_sets::ClassSetMode::Union => {
                        unreachable!("filtered operation")
                    }
                };
                eq(function, member, retain);
                function.instruction(&Instruction::If(BlockType::Empty));
                copy(function, previous, cursor);
                function.instruction(&Instruction::Else);
                eq(function, previous, 0);
                function.instruction(&Instruction::If(BlockType::Empty));
                copy(function, self.head, next);
                function.instruction(&Instruction::Else);
                store(function, previous, FiniteKeyWord::Next as u64, next);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                copy(function, cursor, next);
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                for local in [member, count, points, next, previous, cursor] {
                    emitter.runtime_schema().release_i64_local(local, function);
                }
            }
        }
    }

    fn emit_contains(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        points: I64Local,
        count: I64Local,
        member: I64Local,
        function: &mut Function,
    ) {
        let cursor = emitter.runtime_schema().reserve_i64_local(function);
        let other_points = emitter.runtime_schema().reserve_i64_local(function);
        let other_count = emitter.runtime_schema().reserve_i64_local(function);
        let comparison = emitter.runtime_schema().reserve_i64_local(function);
        set(function, member, 0);
        copy(function, cursor, self.head);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        eq(function, cursor, 0);
        function.instruction(&Instruction::BrIf(1));
        load(
            function,
            cursor,
            FiniteKeyWord::CodePointsPointer as u64,
            other_points,
        );
        load(
            function,
            cursor,
            FiniteKeyWord::CodePointLength as u64,
            other_count,
        );
        emitter.emit_regexp_finite_compare_keys(
            points,
            count,
            other_points,
            other_count,
            comparison,
            function,
        );
        eq(function, comparison, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        set(function, member, 1);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        comparison.load(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::BrIf(1));
        load(function, cursor, FiniteKeyWord::Next as u64, cursor);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [comparison, other_count, other_points, cursor] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl DecodedQString {
    pub(super) fn reserve(emitter: &mut FunctionBuilder<'_>, function: &mut Function) -> Self {
        let result = Self {
            first: emitter.runtime_schema().reserve_i64_local(function),
            last: emitter.runtime_schema().reserve_i64_local(function),
            count: emitter.runtime_schema().reserve_i64_local(function),
        };
        result.emit_clear(function);
        result
    }
    pub(super) fn emit_clear(&self, function: &mut Function) {
        for local in [self.first, self.last, self.count] {
            set(function, local, 0);
        }
    }
    pub(super) fn count_local(&self) -> I64Local {
        self.count
    }
    pub(super) fn emit_append(
        &self,
        emitter: &mut FunctionBuilder<'_>,
        compiler: &CompilerLocals,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        character: I64Local,
        function: &mut Function,
    ) {
        let node = emitter.runtime_schema().reserve_i64_local(function);
        let one = emitter.runtime_schema().reserve_i64_local(function);
        mode.emit_require_character(emitter, compiler, character, function);
        emitter.emit_regexp_finite_canonical_character(mode, modifiers, character, function);
        set(function, one, 1);
        emitter.emit_regexp_finite_workspace_elements(compiler, node, one, 16, function);
        store(function, node, 8, character);
        eq(function, self.last, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        copy(function, self.first, node);
        function.instruction(&Instruction::Else);
        store(function, self.last, 0, node);
        function.instruction(&Instruction::End);
        copy(function, self.last, node);
        emitter.emit_regexp_scratch_increment(self.count, 1, function);
        emitter.runtime_schema().release_i64_local(one, function);
        emitter.runtime_schema().release_i64_local(node, function);
    }
    pub(super) fn release(self, emitter: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in [self.count, self.last, self.first] {
            emitter.runtime_schema().release_i64_local(local, function);
        }
    }
}

fn emit_indexed_point_address(function: &mut Function, points: I64Local, index: I64Local) {
    points.load(function);
    index.load(function);
    function.instruction(&Instruction::I64Const(8));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I32WrapI64);
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_regexp_finite_workspace_elements(
        &mut self,
        compiler: &CompilerLocals,
        pointer: I64Local,
        count: I64Local,
        stride: u64,
        function: &mut Function,
    ) {
        count.load(function);
        function.instruction(&Instruction::I64Const((u32::MAX as u64 / stride) as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::AddressSpace),
            function,
        );
        function.instruction(&Instruction::End);
        let end = self.runtime_schema().reserve_i64_local(function);
        let mut workspace = self.extend_regexp_workspace(compiler, end, function);
        workspace.reserve_elements(pointer, count, stride, function);
        let region = workspace.finish_region(self, compiler, function);
        self.emit_regexp_commit_workspace(compiler, region, function);
        self.runtime_schema().release_i64_local(end, function);
    }

    fn emit_regexp_finite_canonical_character(
        &mut self,
        mode: &CompilerCharacterMode,
        modifiers: &ModifierLocals,
        character: I64Local,
        function: &mut Function,
    ) {
        modifiers.ignore_case.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let low = self.runtime_schema().reserve_i64_local(function);
        let high = self.runtime_schema().reserve_i64_local(function);
        let middle = self.runtime_schema().reserve_i64_local(function);
        self.emit_regexp_canonicalize_character(
            mode.fold_table(),
            mode.fold_count(),
            character,
            low,
            high,
            middle,
            function,
        );
        for local in [middle, high, low] {
            self.runtime_schema().release_i64_local(local, function);
        }
        function.instruction(&Instruction::End);
    }

    fn emit_regexp_finite_compare_keys(
        &mut self,
        left: I64Local,
        left_count: I64Local,
        right: I64Local,
        right_count: I64Local,
        comparison: I64Local,
        function: &mut Function,
    ) {
        let index = self.runtime_schema().reserve_i64_local(function);
        let left_point = self.runtime_schema().reserve_i64_local(function);
        let right_point = self.runtime_schema().reserve_i64_local(function);
        set(function, index, 0);
        set(function, comparison, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        left_count.load(function);
        function.instruction(&Instruction::I64GeU);
        index.load(function);
        right_count.load(function);
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::BrIf(1));
        emit_indexed_point_address(function, left, index);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(0)));
        left_point.store(function);
        emit_indexed_point_address(function, right, index);
        function.instruction(&Instruction::I64Load(FunctionBuilder::memarg64(0)));
        right_point.store(function);
        left_point.load(function);
        right_point.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        left_point.load(function);
        right_point.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        comparison.store(function);
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_increment(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, comparison, 0);
        function.instruction(&Instruction::If(BlockType::Empty));
        left_count.load(function);
        right_count.load(function);
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        left_count.load(function);
        right_count.load(function);
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::End);
        comparison.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [right_point, left_point, index] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
