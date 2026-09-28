use super::*;

fn append_byte(
    builder: &FunctionBuilder<'_>,
    dst_pos_local: u32,
    byte_local: u32,
    function: &mut Function,
) {
    builder.emit_store_byte_local(dst_pos_local, byte_local, function);
    builder.emit_increment_local(dst_pos_local, 1, function);
}

fn append_ascii(
    builder: &FunctionBuilder<'_>,
    dst_pos_local: u32,
    bytes: &[u8],
    function: &mut Function,
) {
    for byte in bytes {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(i32::from(*byte)));
        function.instruction(&Instruction::I32Store8(FunctionBuilder::memarg8(0)));
        builder.emit_increment_local(dst_pos_local, 1, function);
    }
}

/// Emit EscapeRegExpPattern on the original, unescaped pattern. Strings are
/// canonical UTF-8/WTF-8, so copying ordinary bytes also preserves astral
/// scalars and lone UTF-16 surrogates. Only ASCII regexp syntax and the three
/// byte encodings of U+2028/U+2029 need to be recognized.
pub(super) fn emit_escape_regexp_pattern(
    builder: &mut FunctionBuilder<'_>,
    source_payload_local: u32,
    flags_payload_local: u32,
    function: &mut Function,
) -> Result<(), EmitError> {
    let src_offset_local = builder.reserve_temp_local();
    let src_len_local = builder.reserve_temp_local();
    let src_index_local = builder.reserve_temp_local();
    let byte_local = builder.reserve_temp_local();
    let next_byte_local = builder.reserve_temp_local();
    let separator_local = builder.reserve_temp_local();
    let escaped_local = builder.reserve_temp_local();
    let class_depth_local = builder.reserve_temp_local();
    let unicode_sets_local = builder.reserve_temp_local();
    let alloc_len_local = builder.reserve_temp_local();
    let dst_offset_local = builder.reserve_temp_local();
    let dst_pos_local = builder.reserve_temp_local();
    let dst_len_local = builder.reserve_temp_local();

    builder.emit_unpack_string_payload(
        source_payload_local,
        src_offset_local,
        src_len_local,
        function,
    );
    function.instruction(&Instruction::LocalGet(src_len_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    function.instruction(&Instruction::I64Const(builder.strings.payload("(?:)")));
    function.instruction(&Instruction::Else);

    builder.emit_string_payload_contains_ascii_byte_i32(
        flags_payload_local,
        b'v',
        unicode_sets_local,
        function,
    );
    // A slash or ASCII line terminator needs at most one additional byte;
    // U+2028/U+2029 need six bytes in place of three. Twice the byte length
    // therefore bounds the entire output, including WTF-8 surrogate bytes.
    function.instruction(&Instruction::LocalGet(src_len_local));
    function.instruction(&Instruction::I64Const(2));
    function.instruction(&Instruction::I64Mul);
    function.instruction(&Instruction::I64Const(7));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::I64Const(!7_i64));
    function.instruction(&Instruction::I64And);
    function.instruction(&Instruction::LocalSet(alloc_len_local));
    builder.emit_heap_alloc_from_local(alloc_len_local, function)?;
    function.instruction(&Instruction::LocalSet(dst_offset_local));
    function.instruction(&Instruction::LocalGet(dst_offset_local));
    function.instruction(&Instruction::LocalSet(dst_pos_local));
    for local in [src_index_local, escaped_local, class_depth_local] {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(local));
    }

    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(src_index_local));
    function.instruction(&Instruction::LocalGet(src_len_local));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));
    builder.emit_load_string_byte(src_offset_local, src_index_local, byte_local, function);

    // A backslash immediately before a raw LineTerminator is consumed by the
    // regexp pattern. Replace that last backslash, rather than doubling it.
    for terminator in [b'\n', b'\r'] {
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(i64::from(terminator)));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        builder.emit_increment_local(dst_pos_local, -1, function);
        function.instruction(&Instruction::End);
        let replacement = if terminator == b'\n' { b"\\n" } else { b"\\r" };
        append_ascii(builder, dst_pos_local, replacement, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        builder.emit_increment_local(src_index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
    }

    // The only non-ASCII line terminators have UTF-8 encodings E2 80 A8/A9.
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(separator_local));
    function.instruction(&Instruction::LocalGet(byte_local));
    function.instruction(&Instruction::I64Const(0xE2));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::LocalGet(src_len_local));
    function.instruction(&Instruction::LocalGet(src_index_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::I64Const(3));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::I32And);
    function.instruction(&Instruction::If(BlockType::Empty));
    builder.emit_load_string_byte_at_delta(
        src_offset_local,
        src_index_local,
        1,
        next_byte_local,
        function,
    );
    builder.emit_load_string_byte_at_delta(
        src_offset_local,
        src_index_local,
        2,
        separator_local,
        function,
    );
    function.instruction(&Instruction::LocalGet(next_byte_local));
    function.instruction(&Instruction::I64Const(0x80));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::LocalGet(separator_local));
    function.instruction(&Instruction::I64Const(0xA8));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::LocalGet(separator_local));
    function.instruction(&Instruction::I64Const(0xA9));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::I32Or);
    function.instruction(&Instruction::I32And);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(separator_local));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalGet(separator_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(escaped_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    builder.emit_increment_local(dst_pos_local, -1, function);
    function.instruction(&Instruction::End);
    append_ascii(builder, dst_pos_local, b"\\u202", function);
    function.instruction(&Instruction::LocalGet(separator_local));
    function.instruction(&Instruction::I64Const(0xA8));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    append_ascii(builder, dst_pos_local, b"8", function);
    function.instruction(&Instruction::Else);
    append_ascii(builder, dst_pos_local, b"9", function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(escaped_local));
    builder.emit_increment_local(src_index_local, 3, function);
    function.instruction(&Instruction::Br(1));
    function.instruction(&Instruction::End);

    // Escapes consume the next code point. A raw slash terminates the literal
    // only outside a character class; v-mode classes may contain nested sets.
    function.instruction(&Instruction::LocalGet(escaped_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(byte_local));
    function.instruction(&Instruction::I64Const(i64::from(b'[')));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(class_depth_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::LocalGet(unicode_sets_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::I32Or);
    function.instruction(&Instruction::If(BlockType::Empty));
    builder.emit_increment_local(class_depth_local, 1, function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalGet(byte_local));
    function.instruction(&Instruction::I64Const(i64::from(b']')));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::LocalGet(class_depth_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::I32And);
    function.instruction(&Instruction::If(BlockType::Empty));
    builder.emit_increment_local(class_depth_local, -1, function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::LocalGet(byte_local));
    function.instruction(&Instruction::I64Const(i64::from(b'/')));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::LocalGet(class_depth_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I32And);
    function.instruction(&Instruction::If(BlockType::Empty));
    append_ascii(builder, dst_pos_local, b"\\", function);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    append_byte(builder, dst_pos_local, byte_local, function);
    function.instruction(&Instruction::LocalGet(byte_local));
    function.instruction(&Instruction::I64Const(i64::from(b'\\')));
    function.instruction(&Instruction::I64Eq);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(escaped_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::I64ExtendI32U);
    function.instruction(&Instruction::LocalSet(escaped_local));
    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(escaped_local));
    function.instruction(&Instruction::End);
    builder.emit_increment_local(src_index_local, 1, function);
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(dst_pos_local));
    function.instruction(&Instruction::LocalGet(dst_offset_local));
    function.instruction(&Instruction::I64Sub);
    function.instruction(&Instruction::LocalSet(dst_len_local));
    builder.emit_pack_string_payload(dst_offset_local, dst_len_local, function);
    function.instruction(&Instruction::End);

    for local in [
        dst_len_local,
        dst_pos_local,
        dst_offset_local,
        alloc_len_local,
        unicode_sets_local,
        class_depth_local,
        escaped_local,
        separator_local,
        next_byte_local,
        byte_local,
        src_index_local,
        src_len_local,
        src_offset_local,
    ] {
        builder.release_temp_local(local);
    }
    Ok(())
}
