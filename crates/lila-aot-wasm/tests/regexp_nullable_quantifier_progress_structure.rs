const IR_SOURCE: &str = include_str!("../../lila-ir/src/regexp.rs");
const IR_PUBLIC_SOURCE: &str = include_str!("../../lila-ir/src/lib.rs");
const MATCHER_SOURCE: &str = include_str!("../src/builtins/regexp.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_regexp_nullable_quantifier_progress.js");
const CLI_TEST_SOURCE: &str = include_str!("../../lila-cli/tests/cli/regexp.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const EXACT_TEST262: &str =
    include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/nullable-quantifier.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/regexp-nullable-quantifier-progress.md");
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/19-regexp.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: {marker}"));
        cursor += offset + marker.len();
    }
}

#[test]
fn quantifier_lowering_owns_a_closed_optional_progress_lifecycle() {
    for domain in [
        "enum QuantifierOptionalIterations {\n    Finite(usize),\n    Unbounded,\n}",
        "enum QuantifierPreference {\n    Greedy,\n    Lazy,\n}",
        "enum OptionalAtomProgress {\n    MustAdvance,\n    MayRemainAtSameIndex,\n}",
        "enum NullableQuantifierContinuation {\n    NextInstruction,\n    Repeat,\n}",
    ] {
        assert!(
            IR_SOURCE.contains(domain),
            "missing closed domain: {domain}"
        );
    }

    let producer = bounded(
        IR_SOURCE,
        "    fn quantified(\n",
        "    fn atom(&mut self, atom: &ParsedAtom)",
    );
    positions_in_order(
        producer,
        &[
            "for _ in 0..quantifier.required_iterations",
            "match quantifier.optional_iterations",
            "QuantifierOptionalIterations::Finite(count)",
            "OptionalAtomProgress::MayRemainAtSameIndex",
            "self.nullable_finite(atom, quantifier.preference, count)?",
            "QuantifierOptionalIterations::Unbounded",
            "self.nullable_star(atom, quantifier.preference)?",
            "fn nullable_finite(",
            "let mut fallbacks = Vec::with_capacity(count);",
            "self.begin_nullable_optional(preference)?",
            "self.complete_nullable_optional(",
            "let after = self.instructions.len();",
            "self.finish_nullable_optional(fallback, after);",
            "fn nullable_star(",
            "NullableQuantifierContinuation::Repeat",
        ],
    );

    let reverse = bounded(
        IR_SOURCE,
        "    fn reverse_quantified(\n",
        "    fn reverse_atom(&mut self, atom: &ParsedAtom)",
    );
    for marker in [
        "QuantifierOptionalIterations::Finite(count)",
        "QuantifierOptionalIterations::Unbounded",
        "OptionalAtomProgress::MustAdvance",
        "OptionalAtomProgress::MayRemainAtSameIndex",
        "self.reverse_nullable_finite(atom, quantifier.preference, count)?",
        "self.reverse_nullable_star(atom, quantifier.preference)?",
        "self.begin_nullable_optional(preference)?",
        "self.complete_nullable_optional(",
        "self.finish_nullable_optional(fallback, after);",
    ] {
        assert!(reverse.contains(marker), "reverse lowering lost {marker}");
    }

    assert!(!IR_SOURCE.contains(
        "unbounded quantifier over a nullable atom is unsupported by this matcher-program grammar"
    ));
}

#[test]
fn pending_types_force_split_check_and_fallback_registration_order() {
    assert!(IR_SOURCE.contains(
        "#[must_use = \"a nullable optional quantifier attempt must emit its paired progress check\"]\nstruct PendingNullableQuantifierProgress"
    ));
    assert!(IR_SOURCE.contains(
        "#[must_use = \"a completed nullable quantifier attempt must receive its quantifier fallback\"]\nstruct PendingNullableQuantifierFallback"
    ));
    let pending_types = bounded(
        IR_SOURCE,
        "struct PendingNullableQuantifierProgress {",
        "struct ProgramLowerer<'a> {",
    );
    assert!(!pending_types.contains("#[derive"));
    assert!(!pending_types.contains("impl Clone"));
    assert!(!pending_types.contains("impl Copy"));

    let lifecycle = bounded(
        IR_SOURCE,
        "    fn begin_nullable_optional(\n",
        "    fn atom(&mut self, atom: &ParsedAtom)",
    );
    positions_in_order(
        lifecycle,
        &[
            "RegExpInstruction::progress_split(0, 0, preference)",
            "PendingNullableQuantifierProgress {",
            "fn complete_nullable_optional(",
            "pending: PendingNullableQuantifierProgress",
            "RegExpInstruction::progress_check(",
            "PendingNullableQuantifierFallback {",
            "fn finish_nullable_optional(",
            "pending: PendingNullableQuantifierFallback",
            "RegExpInstruction::progress_split(pending.attempt_pc, fallback_pc, pending.preference)",
        ],
    );

    let builders = bounded(
        IR_SOURCE,
        "    fn progress_split(\n",
        "    pub const fn jump(target_pc: usize)",
    );
    for marker in [
        "opcode: REGEXP_OPCODE_PROGRESS_SPLIT",
        "operand0: attempt_pc as u64",
        "operand1: ((fallback_pc as u64) << 1) | preference.word()",
        "opcode: REGEXP_OPCODE_PROGRESS_CHECK",
        "operand0: progress_split_pc as u64",
        "operand1: continuation_pc as u64",
    ] {
        assert!(
            builders.contains(marker),
            "fixed-width builder lost {marker}"
        );
    }
    for opcode in [
        "REGEXP_OPCODE_PROGRESS_SPLIT",
        "REGEXP_OPCODE_PROGRESS_CHECK",
    ] {
        assert!(IR_PUBLIC_SOURCE.contains(opcode));
    }
}

#[test]
fn matcher_frames_preserve_ordered_backtracking_and_exact_progress_identity() {
    let frame_kind = bounded(
        MATCHER_SOURCE,
        "enum RegExpChoiceFrameKind {",
        "impl RegExpChoiceFrameKind {",
    );
    let variants = frame_kind
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Ordinary,",
            "GreedyProgress,",
            "LazyProgressChoice,",
            "LazyProgressAttempt,",
        ]
    );
    let frame_words = bounded(
        MATCHER_SOURCE,
        "impl RegExpChoiceFrameKind {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    for variant in variants.iter().map(|variant| variant.trim_end_matches(',')) {
        assert!(frame_words.contains(&format!("Self::{variant} =>")));
    }
    assert!(!frame_words.contains("_ =>"));

    let dispatch = bounded(
        MATCHER_SOURCE,
        "        // `Split` records the fallback before taking the primary arm.",
        "        function.instruction(&Instruction::LocalGet(reverse_mode));",
    );
    positions_in_order(
        dispatch,
        &[
            "REGEXP_OPCODE_PROGRESS_SPLIT as i64",
            "REGEXP_CHOICE_ORIGIN_MASK",
            "RegExpChoiceFrameKind::LazyProgressChoice.word()",
            "RegExpChoiceFrameKind::GreedyProgress.word()",
            "self.emit_regexp_push_choice_frame(",
            "REGEXP_OPCODE_PROGRESS_CHECK as i64",
            "LocalSet(progress_frame_depth)",
            "RegExpChoiceFrameKind::GreedyProgress.word()",
            "RegExpChoiceFrameKind::LazyProgressAttempt.word()",
            "REGEXP_CHOICE_ORIGIN_SHIFT",
            "LocalGet(operand0)",
            "LocalSet(progress_frame_found)",
            "LocalGet(match_utf16)",
            "I64Eq",
            "self.emit_regexp_backtrack_or_fail(",
            "LocalGet(operand1)",
            "LocalSet(pc)",
        ],
    );
    assert_eq!(
        dispatch
            .matches("self.emit_regexp_push_choice_frame(")
            .count(),
        2,
        "ordinary Split and progress Split must share the one frame writer"
    );

    let progress_split = bounded(
        dispatch,
        "        // A nullable optional attempt carries its pre-attempt cursor and",
        "        function.instruction(&Instruction::LocalGet(opcode));\n        function.instruction(&Instruction::I64Const(REGEXP_OPCODE_JUMP as i64));",
    );
    assert_eq!(
        progress_split
            .matches("If(BlockType::Result(ValType::I64))")
            .count(),
        3,
        "fallback, frame kind and selected PC are explicit preference-dependent choices"
    );
    positions_in_order(
        progress_split,
        &[
            "If(BlockType::Result(ValType::I64))",
            "LocalGet(operand0)",
            "Else",
            "LocalGet(operand1)",
            "I64ShrU",
            "RegExpChoiceFrameKind::LazyProgressChoice.word()",
            "RegExpChoiceFrameKind::GreedyProgress.word()",
            "self.emit_regexp_push_choice_frame(",
            "If(BlockType::Result(ValType::I64))",
            "LocalGet(operand1)",
            "I64ShrU",
            "Else",
            "LocalGet(operand0)",
            "LocalSet(pc)",
        ],
    );

    let push = bounded(
        MATCHER_SOURCE,
        "    fn emit_regexp_push_choice_frame(\n",
        "    /// On an atom failure, restore the latest ordered fallback.",
    );
    for marker in [
        "for (offset, local) in [(0, header), (8, byte), (16, utf16), (24, on_low_surrogate)]",
        "Every ordered choice owns the full capture state",
        "LocalGet(capture_count)",
        "I64Load(Self::memarg8(offset))",
        "I64Store(Self::memarg8(0))",
    ] {
        assert!(push.contains(marker), "choice snapshot lost {marker}");
    }

    let backtrack = bounded(
        MATCHER_SOURCE,
        "    fn emit_regexp_backtrack_or_fail(\n",
        "    fn emit_regexp_ascii_class_contains(\n",
    );
    positions_in_order(
        backtrack,
        &[
            "RegExpChoiceFrameKind::LazyProgressAttempt.word()",
            "RegExpChoiceFrameKind::LazyProgressChoice.word()",
            "RegExpChoiceFrameKind::LazyProgressAttempt.word()",
            "LocalSet(pc)",
            "LocalSet(byte)",
            "LocalSet(utf16)",
        ],
    );
}

#[test]
fn static_data_validation_counts_progress_choices_and_terminates_checks() {
    let queue = bounded(
        DATA_SOURCE,
        "    fn queue_regexp_program(&mut self, program: &RegExpProgram) {",
        "    fn queue_runtime_regexp_programs(&mut self) {",
    );
    assert!(queue.contains("REGEXP_OPCODE_SPLIT | REGEXP_OPCODE_PROGRESS_SPLIT"));
    assert!(queue.contains("repeatable_split_count(program)"));

    let repeatable = bounded(
        DATA_SOURCE,
        "fn repeatable_split_count(program: &RegExpProgram) -> u32 {",
        "fn has_non_consuming_cycle(program: &RegExpProgram) -> bool {",
    );
    for marker in [
        "REGEXP_OPCODE_PROGRESS_SPLIT => [",
        "valid(instruction.operand1 >> 1)",
        "REGEXP_OPCODE_PROGRESS_CHECK => valid(instruction.operand1)",
        "REGEXP_OPCODE_SPLIT | REGEXP_OPCODE_PROGRESS_SPLIT",
    ] {
        assert!(repeatable.contains(marker), "accounting lost {marker}");
    }

    let cycle = bounded(
        DATA_SOURCE,
        "fn has_non_consuming_cycle(program: &RegExpProgram) -> bool {",
        "#[cfg(test)]\nmod runtime_error_message_pool_tests {",
    );
    for marker in [
        "instructions[pc].opcode == REGEXP_OPCODE_PROGRESS_CHECK",
        "REGEXP_OPCODE_PROGRESS_SPLIT => [",
        "valid_target(instruction.operand1 >> 1)",
    ] {
        assert!(cycle.contains(marker), "cycle validation lost {marker}");
    }
}

#[test]
fn exact_inventory_fixture_and_verified_status_remain_bounded() {
    for marker in [
        "esid: sec-runtime-semantics-repeatmatcher-abstract-operation",
        "let regex = /(a?b??)*/;",
        "assert.sameValue(match[0], expected",
    ] {
        assert!(EXACT_TEST262.contains(marker));
    }
    assert!(!EXACT_TEST262.contains("flags:"));
    assert!(!TEST262_RUNNER_SOURCE.contains("built-ins/RegExp/nullable-quantifier.js"));
    assert!(!KNOWN_FAILURES.contains("nullable-quantifier.js"));

    for marker in [
        "exec(/(a?b??)*/, \"ab\", \"ab\", [\"b\"]",
        "exec(/(a?b??)*b/, \"ab\", \"ab\", [\"a\"]",
        "exec(/(a?)*a/, \"aa\", \"aa\"",
        "exec(/(a?)*?a/, \"aa\", \"a\"",
        "exec(/(a?){2,}b/, \"b\", \"b\"",
        "exec(/^(a?){2,4}b$/",
        "exec(/((a?)*)*b/",
        "exec(/(?<=(a?)*)b/",
        "match(/(a?)*/g)",
        "match(/(a?)*?/g)",
    ] {
        assert!(FIXTURE.contains(marker), "fixture lost {marker}");
    }
    assert!(CLI_TEST_SOURCE
        .contains("fn run_wasm_backend_rejects_empty_optional_nullable_quantifier_iterations()"));
    assert!(CLI_TEST_SOURCE.contains("wasm_regexp_nullable_quantifier_progress.js"));

    for source in [README, TASK] {
        for marker in [
            "44247b836b",
            "built-ins/RegExp/nullable-quantifier.js",
            "0/2",
            "Runtime/NotImplemented",
            "workspace/all-target `cargo check`",
            "`cargo xc`",
            "`1/1` in `8.37s`",
            "`5/5` in `22.36s`",
            "`1/1` in `22.83s`",
            "`27.19s`",
            "passes `2/2` with zero unsupported",
            "full-suite claim is made",
        ] {
            assert!(source.contains(marker), "status lost {marker}");
        }
    }
    for marker in [
        "REGEXP_OPCODE_PROGRESS_SPLIT",
        "REGEXP_OPCODE_PROGRESS_CHECK",
        "PendingNullableQuantifierProgress",
        "Forward and reverse matching use the same paired representation",
        "does not claim all RegExp Test262 coverage",
        "--snapshot-name regexp-nullable-quantifier",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost {marker}");
    }
}
