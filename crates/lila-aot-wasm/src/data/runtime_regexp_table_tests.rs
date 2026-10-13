use super::*;

#[test]
fn candidate_rows_preserve_gc_string_ordinals_and_all_closed_outcomes() {
    let mut pool = StringPool::default();
    let source = "Ā\0𝄞";
    let flags = "u";
    pool.intern_string(source);
    pool.intern_string(flags);
    let program = RegExpProgram::compile(source, flags).unwrap();
    pool.queue_regexp_program(&program);
    pool.append_regexp_programs();
    let reference = pool.regexp_programs[&RegExpProgramStaticKey::from_program(&program)];
    pool.runtime_regexp_programs = vec![
        (
            source.into(),
            flags.into(),
            RuntimeRegExpEntry::Program(reference),
        ),
        (source.into(), flags.into(), RuntimeRegExpEntry::Rejected),
        (source.into(), flags.into(), RuntimeRegExpEntry::Unsupported),
    ];
    pool.append_runtime_regexp_program_table();
    let table = pool.runtime_regexp_table().expect("nonempty checked table");
    assert_eq!(table.count(), 3, "no outcome may disappear from the table");
    assert_eq!(table.address % 8, 0);
    let offset = (table.address - STATIC_DATA_OFFSET) as usize;
    let rows = pool.bytes[offset..]
        .chunks_exact(RUNTIME_REGEXP_RECORD_SIZE as usize)
        .map(|row| {
            row.chunks_exact(8)
                .map(|word| u64::from_le_bytes(word.try_into().unwrap()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    for (row, kind) in rows.iter().zip(RuntimeRegExpEntryKind::ALL) {
        assert_eq!(
            row[RUNTIME_REGEXP_RECORD_SOURCE_WORD],
            u64::from(pool.pooled_string_index(source).unwrap().ordinal())
        );
        assert_eq!(
            row[RUNTIME_REGEXP_RECORD_FLAGS_WORD],
            u64::from(pool.pooled_string_index(flags).unwrap().ordinal())
        );
        assert_eq!(row[RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD], kind.word());
    }
    assert_eq!(
        rows[0][RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD],
        reference.payload()
    );
    assert_eq!(rows[1][RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD], 0);
    assert_eq!(rows[2][RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD], 0);
    let descriptor = rows[0][RUNTIME_REGEXP_RECORD_PROGRAM_PAYLOAD_WORD];
    let start = (descriptor >> 32) as usize - STATIC_DATA_OFFSET as usize;
    let length = descriptor as u32 as usize;
    assert_eq!(
        &pool.bytes[start..start + length],
        ValidatedRegExpProgram::from_program(&program)
            .unwrap()
            .bytes()
    );
}
