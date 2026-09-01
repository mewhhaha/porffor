use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-from-async-stage-domain.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let end_offset = source[start_offset..]
        .find(end)
        .map(|offset| start_offset + offset)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"));
    &source[start_offset..end_offset]
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn stage_owner_bodies() -> String {
    [
        (
            "    fn emit_array_from_async_array_like_start(\n",
            "    fn emit_array_from_async_iterable_start(\n",
        ),
        (
            "    fn emit_array_from_async_iterable_start(\n",
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
        ),
        (
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
            "    pub(crate) fn emit_array_from_async_rejected(\n",
        ),
        (
            "    pub(crate) fn emit_array_from_async_rejected(\n",
            "    fn emit_array_from_async_schedule_await(\n",
        ),
        (
            "    fn emit_array_from_async_schedule_iterator_step_callback(\n",
            "    fn emit_array_from_async_close_or_reject_callback_current_throw(\n",
        ),
        (
            "    fn emit_array_from_async_begin_close_current_throw(\n",
            "    fn emit_array_from_async_reject_saved_error_on_current_throw(\n",
        ),
    ]
    .into_iter()
    .map(|(start, end)| bounded(SOURCE, start, end))
    .collect()
}

#[test]
fn array_from_async_stage_is_one_private_capability_free_domain() {
    let declaration = without_whitespace(bounded(
        SOURCE,
        "const ARRAY_FROM_ASYNC_SAVED_ERROR_TAG_OFFSET: u64 = 168;",
        "enum ArrayFromAsyncSourceMode {",
    ));
    assert_eq!(
        declaration,
        concat!(
            "constARRAY_FROM_ASYNC_SAVED_ERROR_TAG_OFFSET:u64=168;",
            "enumArrayFromAsyncStage{InputValue,MappedValue,AsyncIteratorResult,",
            "SyncIteratorDoneValue,AsyncCloseResult,SyncCloseValue,}",
            "implArrayFromAsyncStage{constfncode(&self)->u64{matchself{",
            "Self::InputValue=>0,Self::MappedValue=>1,Self::AsyncIteratorResult=>2,",
            "Self::SyncIteratorDoneValue=>3,Self::AsyncCloseResult=>4,",
            "Self::SyncCloseValue=>5,}}}"
        )
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "ArrayFromAsyncStage"),
        24
    );
    for (variant, count) in [
        ("InputValue", 8),
        ("MappedValue", 2),
        ("AsyncIteratorResult", 3),
        ("SyncIteratorDoneValue", 3),
        ("AsyncCloseResult", 3),
        ("SyncCloseValue", 3),
    ] {
        assert_eq!(
            SOURCE
                .matches(&format!("ArrayFromAsyncStage::{variant}.code()"))
                .count(),
            count,
            "{variant} projections"
        );
    }
    assert_eq!(SOURCE.matches("ArrayFromAsyncStage::").count(), 22);
    assert_eq!(SOURCE.matches("ARRAY_FROM_ASYNC_STAGE_OFFSET").count(), 15);
    for old_name in [
        "ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE",
        "ARRAY_FROM_ASYNC_STAGE_MAPPED_VALUE",
        "ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT",
        "ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE",
        "ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT",
        "ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE",
    ] {
        assert!(!SOURCE.contains(old_name), "found `{old_name}`");
    }
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "Hash",
        "PartialOrd",
        "Ord",
    ] {
        assert!(!SOURCE.contains(&format!("impl {capability} for ArrayFromAsyncStage")));
    }
    assert!(!SOURCE.contains("ArrayFromAsyncStage::clone"));
    assert!(!SOURCE.contains("pub enum ArrayFromAsyncStage"));
    assert!(!SOURCE.contains("pub(crate) enum ArrayFromAsyncStage"));
}

#[test]
fn thirteen_stage_producers_preserve_the_heap_and_local_handoffs() {
    assert_eq!(
        SOURCE
            .matches(concat!(
                "ARRAY_FROM_ASYNC_STAGE_OFFSET,\n",
                "            ArrayFromAsyncStage::"
            ))
            .count(),
        12,
        "twelve state-cell stage producers"
    );
    assert_eq!(
        SOURCE
            .matches(concat!(
                "Instruction::I64Const(\n",
                "            ArrayFromAsyncStage::InputValue.code() as i64,\n",
                "        ));\n",
                "        function.instruction(&Instruction::LocalSet(stage_local));"
            ))
            .count(),
        1,
        "one paired local stage producer"
    );

    let owner_counts = [
        (
            "    fn emit_array_from_async_array_like_start(\n",
            "    fn emit_array_from_async_iterable_start(\n",
            1,
        ),
        (
            "    fn emit_array_from_async_iterable_start(\n",
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
            3,
        ),
        (
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
            "    pub(crate) fn emit_array_from_async_rejected(\n",
            9,
        ),
        (
            "    pub(crate) fn emit_array_from_async_rejected(\n",
            "    fn emit_array_from_async_schedule_await(\n",
            4,
        ),
        (
            "    fn emit_array_from_async_schedule_iterator_step_callback(\n",
            "    fn emit_array_from_async_close_or_reject_callback_current_throw(\n",
            3,
        ),
        (
            "    fn emit_array_from_async_begin_close_current_throw(\n",
            "    fn emit_array_from_async_reject_saved_error_on_current_throw(\n",
            2,
        ),
    ];
    for (start, end, expected) in owner_counts {
        assert_eq!(
            bounded(SOURCE, start, end)
                .matches("ArrayFromAsyncStage::")
                .count(),
            expected,
            "stage routes in `{start}`"
        );
    }
}

#[test]
fn nine_stage_comparisons_and_all_algorithms_recover_the_frozen_source() {
    assert_eq!(
        SOURCE
            .matches(concat!(
                "Instruction::LocalGet(stage_local));\n",
                "        function.instruction(&Instruction::I64Const(\n",
                "            ArrayFromAsyncStage::"
            ))
            .count(),
        9,
        "all stage comparisons must read the shared stage local"
    );

    let old_semantics = without_whitespace(&stage_owner_bodies())
        .replace(
            "ArrayFromAsyncStage::InputValue.code()",
            "ARRAY_FROM_ASYNC_STAGE_INPUT_VALUE",
        )
        .replace(
            "ArrayFromAsyncStage::MappedValue.code()",
            "ARRAY_FROM_ASYNC_STAGE_MAPPED_VALUE",
        )
        .replace(
            "ArrayFromAsyncStage::AsyncIteratorResult.code()",
            "ARRAY_FROM_ASYNC_STAGE_ASYNC_ITERATOR_RESULT",
        )
        .replace(
            "ArrayFromAsyncStage::SyncIteratorDoneValue.code()",
            "ARRAY_FROM_ASYNC_STAGE_SYNC_ITERATOR_DONE_VALUE",
        )
        .replace(
            "ArrayFromAsyncStage::AsyncCloseResult.code()",
            "ARRAY_FROM_ASYNC_STAGE_ASYNC_CLOSE_RESULT",
        )
        .replace(
            "ArrayFromAsyncStage::SyncCloseValue.code()",
            "ARRAY_FROM_ASYNC_STAGE_SYNC_CLOSE_VALUE",
        );
    assert_eq!(
        (old_semantics.len(), fnv1a(&old_semantics)),
        (41_030, 0xd722_936e_3495_17a9),
        "erasing only the typed stage vocabulary must recover the frozen algorithms"
    );
}

#[test]
fn contract_and_t16_record_the_closed_stage_domain() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "capability-free `ArrayFromAsyncStage`",
        "thirteen stage producers",
        "nine comparisons",
        "41,030",
        "0xd722936e349517a9",
    ] {
        assert!(
            contract_words.contains(marker) || task_words.contains(marker),
            "missing contract/task marker: {marker}"
        );
    }
    for text in [&contract_words, &task_words] {
        assert!(text.contains("Batch AM"));
        assert!(text.contains("cargo xc"));
        assert!(text.contains("4/4"));
        assert!(text.contains("3/3"));
        assert!(text.contains("6/6"));
        assert!(text.contains("Wasm-AOT"));
    }
}
