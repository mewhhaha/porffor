use super::*;
use lila_ir::RegExpIdentifierPosition;

/// Balanced emitted membership checks use the static parser's exact pinned
/// Unicode authority; no independently maintained runtime identifier table.
fn emit_identifier_membership(
    code_point: I64Local,
    ranges: &[(u32, u32)],
    function: &mut Function,
) {
    let Some((&(first, last), right)) = ranges.split_first() else {
        function.instruction(&Instruction::I32Const(0));
        return;
    };
    if right.is_empty() {
        between(function, code_point, u64::from(first), u64::from(last));
        return;
    }
    let middle = ranges.len() / 2;
    let (first, last) = ranges[middle];
    code_point.load(function);
    function.instruction(&Instruction::I64Const(i64::from(first)));
    function.instruction(&Instruction::I64LtU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
    emit_identifier_membership(code_point, &ranges[..middle], function);
    function.instruction(&Instruction::Else);
    code_point.load(function);
    function.instruction(&Instruction::I64Const(i64::from(last)));
    function.instruction(&Instruction::I64LeU);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
    function.instruction(&Instruction::I32Const(1));
    function.instruction(&Instruction::Else);
    emit_identifier_membership(code_point, &ranges[middle + 1..], function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
}

impl FunctionBuilder<'_> {
    fn regexp_name_fail_if(&self, compiler: &CompilerLocals, function: &mut Function) {
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Syntax(CompileSyntax::InvalidGroupName),
            function,
        );
        function.instruction(&Instruction::End);
    }

    fn emit_regexp_name_hex_digit(&self, unit: I64Local, digit: I64Local, function: &mut Function) {
        set(function, digit, u64::MAX);
        for &(first, last, initial) in REGEXP_HEX_DIGIT_RANGES {
            between(function, unit, u64::from(first), u64::from(last));
            function.instruction(&Instruction::If(BlockType::Empty));
            unit.load(function);
            function.instruction(&Instruction::I64Const(i64::from(first - initial)));
            function.instruction(&Instruction::I64Sub);
            digit.store(function);
            function.instruction(&Instruction::End);
        }
    }

    fn emit_regexp_name_fixed_hex(
        &mut self,
        compiler: &CompilerLocals,
        value: I64Local,
        unit: I64Local,
        digit: I64Local,
        function: &mut Function,
    ) {
        set(function, value, 0);
        for _ in 0..4 {
            peek(compiler, function, compiler.cursor, 0, unit);
            self.emit_regexp_name_hex_digit(unit, digit, function);
            eq(function, digit, u64::MAX);
            self.regexp_name_fail_if(compiler, function);
            value.load(function);
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Mul);
            digit.load(function);
            function.instruction(&Instruction::I64Add);
            value.store(function);
            self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        }
    }

    /// Cursor starts after `<` and finishes at `>`. All parsed spellings are
    /// decoded to one canonical UTF-8 payload in the checked compiler workspace.
    /// A fixed lead-surrogate escape requires a following fixed trail escape;
    /// a direct UTF-16 pair is combined, and braced surrogate escapes are invalid.
    pub(super) fn emit_regexp_parser_name(
        &mut self,
        compiler: &CompilerLocals,
        payload: I64Local,
        function: &mut Function,
    ) {
        let start = self.runtime_schema().reserve_i64_local(function);
        let destination = self.runtime_schema().reserve_i64_local(function);
        let code_point = self.runtime_schema().reserve_i64_local(function);
        let escaped = self.runtime_schema().reserve_i64_local(function);
        let fixed = self.runtime_schema().reserve_i64_local(function);
        let next = self.runtime_schema().reserve_i64_local(function);
        let digit = self.runtime_schema().reserve_i64_local(function);
        let digits = self.runtime_schema().reserve_i64_local(function);
        let unit = self.runtime_schema().reserve_i64_local(function);
        let temp = self.runtime_schema().reserve_i64_local(function);
        compiler.name_bytes.load(function);
        compiler.name_byte_length.load(function);
        function.instruction(&Instruction::I64Add);
        start.store(function);
        start.load(function);
        destination.store(function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, code_point);
        eq(function, code_point, b'>' as u64);
        function.instruction(&Instruction::BrIf(1));
        eq(function, code_point, u64::MAX);
        self.regexp_name_fail_if(compiler, function);
        eq(function, code_point, b'\\' as u64);
        function.instruction(&Instruction::I64ExtendI32U);
        escaped.store(function);
        set(function, fixed, 0);
        eq(function, escaped, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, next);
        eq(function, next, b'u' as u64);
        function.instruction(&Instruction::I32Eqz);
        self.regexp_name_fail_if(compiler, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        peek(compiler, function, compiler.cursor, 0, next);
        eq(function, next, b'{' as u64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        set(function, code_point, 0);
        set(function, digits, 0);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, next);
        eq(function, next, b'}' as u64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_regexp_name_hex_digit(next, digit, function);
        eq(function, digit, u64::MAX);
        self.regexp_name_fail_if(compiler, function);
        code_point.load(function);
        function.instruction(&Instruction::I64Const(16));
        function.instruction(&Instruction::I64Mul);
        digit.load(function);
        function.instruction(&Instruction::I64Add);
        code_point.store(function);
        code_point.load(function);
        function.instruction(&Instruction::I64Const(0x10ffff));
        function.instruction(&Instruction::I64GtU);
        self.regexp_name_fail_if(compiler, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        self.emit_regexp_scratch_increment(digits, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        eq(function, digits, 0);
        self.regexp_name_fail_if(compiler, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::Else);
        set(function, fixed, 1);
        self.emit_regexp_name_fixed_hex(compiler, code_point, unit, digit, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);

        between(function, code_point, 0xd800, 0xdbff);
        function.instruction(&Instruction::If(BlockType::Empty));
        eq(function, fixed, 1);
        function.instruction(&Instruction::If(BlockType::Empty));
        peek(compiler, function, compiler.cursor, 0, next);
        eq(function, next, b'\\' as u64);
        function.instruction(&Instruction::I32Eqz);
        self.regexp_name_fail_if(compiler, function);
        peek(compiler, function, compiler.cursor, 1, next);
        eq(function, next, b'u' as u64);
        function.instruction(&Instruction::I32Eqz);
        self.regexp_name_fail_if(compiler, function);
        self.emit_regexp_scratch_increment(compiler.cursor, 2, function);
        self.emit_regexp_name_fixed_hex(compiler, next, unit, digit, function);
        function.instruction(&Instruction::Else);
        eq(function, escaped, 1);
        self.regexp_name_fail_if(compiler, function);
        peek(compiler, function, compiler.cursor, 0, next);
        self.emit_regexp_scratch_increment(compiler.cursor, 1, function);
        function.instruction(&Instruction::End);
        between(function, next, 0xdc00, 0xdfff);
        function.instruction(&Instruction::I32Eqz);
        self.regexp_name_fail_if(compiler, function);
        code_point.load(function);
        function.instruction(&Instruction::I64Const(0xd800));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        next.load(function);
        function.instruction(&Instruction::I64Const(0xdc00));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Add);
        code_point.store(function);
        function.instruction(&Instruction::End);
        between(function, code_point, 0xd800, 0xdfff);
        self.regexp_name_fail_if(compiler, function);
        destination.load(function);
        start.load(function);
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        emit_identifier_membership(
            code_point,
            &RegExpIdentifierPosition::Start.ranges(),
            function,
        );
        function.instruction(&Instruction::Else);
        emit_identifier_membership(
            code_point,
            &RegExpIdentifierPosition::Continue.ranges(),
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Eqz);
        self.regexp_name_fail_if(compiler, function);
        destination.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        compiler.name_bytes.load(function);
        compiler.unit_capacity.load(function);
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_regexp_compile_failure(
            compiler,
            CompileFailure::Resource(CompileResource::AddressSpace),
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_regexp_scratch_store_codepoint(destination, code_point, temp, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        destination.load(function);
        start.load(function);
        function.instruction(&Instruction::I64Eq);
        self.regexp_name_fail_if(compiler, function);
        start.load(function);
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64Shl);
        destination.load(function);
        start.load(function);
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Or);
        payload.store(function);
        destination.load(function);
        compiler.name_bytes.load(function);
        function.instruction(&Instruction::I64Sub);
        compiler.name_byte_length.store(function);
        for local in [
            temp,
            unit,
            digits,
            digit,
            next,
            fixed,
            escaped,
            code_point,
            destination,
            start,
        ] {
            self.runtime_schema().release_i64_local(local, function);
        }
    }
}
