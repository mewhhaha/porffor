const SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-from-async-source-mode-domain.md");
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

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

fn semantic_bodies() -> String {
    [
        (
            "    pub(crate) fn emit_array_from_async(\n",
            "    fn emit_array_from_async_array_like_start(\n",
        ),
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
            "    fn emit_array_from_async_close_or_reject_callback_current_throw(\n",
            "    fn emit_array_from_async_begin_close_current_throw(\n",
        ),
        (
            "    fn emit_array_from_async_begin_close_current_throw(\n",
            "    fn emit_array_from_async_reject_saved_error_on_current_throw(\n",
        ),
        (
            "    fn emit_array_from_async_finish_callback(\n",
            "    fn emit_array_from_async_set_length(\n",
        ),
    ]
    .into_iter()
    .map(|(start, end)| bounded(SOURCE, start, end))
    .collect()
}

#[test]
fn source_mode_is_one_private_capability_free_wire_domain() {
    let declaration = without_whitespace(bounded(
        SOURCE,
        "enum ArrayFromAsyncSourceMode {",
        "/// The two observable properties of an iterator-result object.",
    ));
    assert_eq!(
        declaration,
        concat!(
            "enumArrayFromAsyncSourceMode{ArrayLike,AsyncIterator,SyncIterator,}",
            "implArrayFromAsyncSourceMode{constfncode(&self)->u64{matchself{",
            "Self::ArrayLike=>0,Self::AsyncIterator=>1,Self::SyncIterator=>2,}}}"
        )
    );
    assert_eq!(SOURCE.matches("ArrayFromAsyncSourceMode").count(), 13);
    assert_eq!(
        SOURCE
            .matches("ArrayFromAsyncSourceMode::ArrayLike")
            .count(),
        5
    );
    assert_eq!(
        SOURCE
            .matches("ArrayFromAsyncSourceMode::AsyncIterator")
            .count(),
        4
    );
    assert_eq!(
        SOURCE
            .matches("ArrayFromAsyncSourceMode::SyncIterator")
            .count(),
        2
    );
    assert_eq!(SOURCE.matches("ArrayFromAsyncSourceMode::").count(), 11);
    assert!(!SOURCE.contains("ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE"));
    assert!(!SOURCE.contains("ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR"));
    assert!(!SOURCE.contains("ARRAY_FROM_ASYNC_MODE_SYNC_ITERATOR"));

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
        assert!(!SOURCE.contains(&format!("impl {capability} for ArrayFromAsyncSourceMode")));
    }
    assert!(!SOURCE.contains("ArrayFromAsyncSourceMode::clone"));
    assert!(!SOURCE.contains("pub enum ArrayFromAsyncSourceMode"));
    assert!(!SOURCE.contains("pub(crate) enum ArrayFromAsyncSourceMode"));
}

#[test]
fn three_mode_producers_preserve_the_runtime_wire_handoff() {
    let entry = bounded(
        SOURCE,
        "    pub(crate) fn emit_array_from_async(\n",
        "    fn emit_array_from_async_array_like_start(\n",
    );
    assert_eq!(
        entry
            .matches("ArrayFromAsyncSourceMode::AsyncIterator.code()")
            .count(),
        1
    );
    assert_eq!(
        entry
            .matches("ArrayFromAsyncSourceMode::SyncIterator.code()")
            .count(),
        1
    );
    assert!(
        entry.find("ArrayFromAsyncSourceMode::AsyncIterator.code()")
            < entry.find("ArrayFromAsyncSourceMode::SyncIterator.code()")
    );

    let array_like = bounded(
        SOURCE,
        "    fn emit_array_from_async_array_like_start(\n",
        "    fn emit_array_from_async_iterable_start(\n",
    );
    assert_eq!(
        array_like
            .matches("ArrayFromAsyncSourceMode::ArrayLike.code()")
            .count(),
        1
    );
    assert!(array_like.contains(concat!(
        "            ARRAY_FROM_ASYNC_MODE_OFFSET,\n",
        "            ArrayFromAsyncSourceMode::ArrayLike.code(),"
    )));

    let iterable = bounded(
        SOURCE,
        "    fn emit_array_from_async_iterable_start(\n",
        "    pub(crate) fn emit_array_from_async_fulfilled(\n",
    );
    assert_eq!(
        iterable
            .matches("(ARRAY_FROM_ASYNC_MODE_OFFSET, iterator_mode_local)")
            .count(),
        1
    );
    assert_eq!(SOURCE.matches("ARRAY_FROM_ASYNC_MODE_OFFSET").count(), 9);
}

#[test]
fn eight_mode_projections_preserve_the_callback_algorithms() {
    let consumers = [
        (
            "    fn emit_array_from_async_iterable_start(\n",
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
            (0, 1, 0),
        ),
        (
            "    pub(crate) fn emit_array_from_async_fulfilled(\n",
            "    pub(crate) fn emit_array_from_async_rejected(\n",
            (1, 0, 0),
        ),
        (
            "    pub(crate) fn emit_array_from_async_rejected(\n",
            "    fn emit_array_from_async_schedule_await(\n",
            (1, 0, 1),
        ),
        (
            "    fn emit_array_from_async_schedule_iterator_step_callback(\n",
            "    fn emit_array_from_async_close_or_reject_callback_current_throw(\n",
            (0, 1, 0),
        ),
        (
            "    fn emit_array_from_async_close_or_reject_callback_current_throw(\n",
            "    fn emit_array_from_async_begin_close_current_throw(\n",
            (1, 0, 0),
        ),
        (
            "    fn emit_array_from_async_begin_close_current_throw(\n",
            "    fn emit_array_from_async_reject_saved_error_on_current_throw(\n",
            (0, 1, 0),
        ),
        (
            "    fn emit_array_from_async_finish_callback(\n",
            "    fn emit_array_from_async_set_length(\n",
            (1, 0, 0),
        ),
    ];
    for (start, end, (array_like, async_iterator, sync_iterator)) in consumers {
        let body = bounded(SOURCE, start, end);
        assert_eq!(
            body.matches("ArrayFromAsyncSourceMode::ArrayLike.code()")
                .count(),
            array_like,
            "ArrayLike projections in `{start}`"
        );
        assert_eq!(
            body.matches("ArrayFromAsyncSourceMode::AsyncIterator.code()")
                .count(),
            async_iterator,
            "AsyncIterator projections in `{start}`"
        );
        assert_eq!(
            body.matches("ArrayFromAsyncSourceMode::SyncIterator.code()")
                .count(),
            sync_iterator,
            "SyncIterator projections in `{start}`"
        );
    }

    let old_semantics = without_whitespace(&semantic_bodies())
        .replace(
            "ArrayFromAsyncSourceMode::ArrayLike.code()",
            "ARRAY_FROM_ASYNC_MODE_ARRAY_LIKE",
        )
        .replace(
            "ArrayFromAsyncSourceMode::AsyncIterator.code()",
            "ARRAY_FROM_ASYNC_MODE_ASYNC_ITERATOR",
        )
        .replace(
            "ArrayFromAsyncSourceMode::SyncIterator.code()",
            "ARRAY_FROM_ASYNC_MODE_SYNC_ITERATOR",
        )
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
        (51_969, 0x18bf_07d7_1957_d97f),
        "removing only the typed projection vocabulary must recover the frozen algorithms"
    );
}

#[test]
fn contract_and_t16_record_the_closed_source_mode_domain() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "capability-free `ArrayFromAsyncSourceMode`",
        "three semantic producers",
        "eight comparisons",
        "runtime heap wire value",
        "51,969",
        "0x18bf07d71957d97f",
    ] {
        assert!(
            contract_words.contains(marker) || task_words.contains(marker),
            "missing contract/task marker: {marker}"
        );
    }
    for text in [&contract_words, &task_words] {
        assert!(text.contains("Batch AL"));
        assert!(text.contains("cargo xc"));
        assert!(text.contains("4/4"));
        assert!(text.contains("2/2"));
        assert!(text.contains("6/6"));
        assert!(text.contains("Wasm-AOT"));
    }
}
