const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_regexp_unicode_sets_class_strings.js");
const CLI_TEST_SOURCE: &str = include_str!("../../lila-cli/tests/cli/regexp.rs");
const IR_SOURCE: &str = include_str!("../../lila-ir/src/regexp.rs");
const MATCHER_SOURCE: &str = include_str!("../src/builtins/regexp.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const SHORTCUT_ALLOWLIST: &str = include_str!("../../../test262/backlog/shortcut-allowlist.tsv");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/19-regexp.md");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/regexp-unicode-set-finite-string-algebra.md"
);

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

const EXACT_TEST262: [(&str, &str); 27] = [
    (
        "character-class-escape-union-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-escape-union-string-literal.js"),
    ),
    (
        "character-class-union-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-union-string-literal.js"),
    ),
    (
        "character-property-escape-union-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-property-escape-union-string-literal.js"),
    ),
    (
        "character-union-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-union-string-literal.js"),
    ),
    (
        "string-literal-union-character-class-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-character-class-escape.js"),
    ),
    (
        "string-literal-union-character-class.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-character-class.js"),
    ),
    (
        "string-literal-union-character-property-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-character-property-escape.js"),
    ),
    (
        "string-literal-union-character.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-character.js"),
    ),
    (
        "string-literal-union-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-union-string-literal.js"),
    ),
    (
        "character-class-escape-intersection-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-escape-intersection-string-literal.js"),
    ),
    (
        "character-class-intersection-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-intersection-string-literal.js"),
    ),
    (
        "character-property-escape-intersection-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-property-escape-intersection-string-literal.js"),
    ),
    (
        "character-intersection-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-intersection-string-literal.js"),
    ),
    (
        "string-literal-intersection-character-class-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-intersection-character-class-escape.js"),
    ),
    (
        "string-literal-intersection-character-class.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-intersection-character-class.js"),
    ),
    (
        "string-literal-intersection-character-property-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-intersection-character-property-escape.js"),
    ),
    (
        "string-literal-intersection-character.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-intersection-character.js"),
    ),
    (
        "string-literal-intersection-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-intersection-string-literal.js"),
    ),
    (
        "character-class-escape-difference-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-escape-difference-string-literal.js"),
    ),
    (
        "character-class-difference-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-class-difference-string-literal.js"),
    ),
    (
        "character-property-escape-difference-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-property-escape-difference-string-literal.js"),
    ),
    (
        "character-difference-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/character-difference-string-literal.js"),
    ),
    (
        "string-literal-difference-character-class-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-difference-character-class-escape.js"),
    ),
    (
        "string-literal-difference-character-class.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-difference-character-class.js"),
    ),
    (
        "string-literal-difference-character-property-escape.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-difference-character-property-escape.js"),
    ),
    (
        "string-literal-difference-character.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-difference-character.js"),
    ),
    (
        "string-literal-difference-string-literal.js",
        include_str!("../../../test262/vendor/test262/test/built-ins/RegExp/unicodeSets/generated/string-literal-difference-string-literal.js"),
    ),
];

#[test]
fn finite_class_set_is_the_only_canonical_string_algebra() {
    for domain in [
        "struct FiniteClassSet {\n    ranges: Vec<(u32, u32)>,\n    strings: BTreeSet<Vec<u32>>,\n}",
        "struct FiniteClassSetAtom {\n    multi_code_point_strings: Vec<Vec<RegExpInstruction>>,\n    singleton: RegExpInstruction,\n    contains_empty: bool,\n}",
        "enum RequiresUnicodeSetSemantics {\n    PropertyOfStrings(RequiresUnicodePropertyOfStrings),\n    StringCaseFolding(RequiresUnicodeSetStringCaseFolding),\n}",
    ] {
        assert!(IR_SOURCE.contains(domain), "missing closed domain: {domain}");
    }
    assert!(!IR_SOURCE.contains("RequiresClassStringSemantics"));

    let algebra = bounded(
        IR_SOURCE,
        "impl FiniteClassSet {",
        "\nenum ClassSetSemantics {",
    );
    positions_in_order(
        algebra,
        &[
            "fn class_strings(alternatives: BTreeSet<Vec<u32>>) -> Self",
            "[code_point] => ranges.push((*code_point, *code_point))",
            "[] | [_, _, ..]",
            "strings.insert(alternative);",
            "ranges: normalize_ranges(ranges)",
            "fn union(self, right: Self) -> Self",
            "strings.extend(right.strings);",
            "fn intersection(self, right: Self) -> Self",
            "self.strings.intersection(&right.strings)",
            "fn subtraction(self, right: Self) -> Self",
            "self.strings.difference(&right.strings)",
        ],
    );

    let atom = bounded(
        IR_SOURCE,
        "impl FiniteClassSetAtom {",
        "/// Parses a `v`-mode `ClassSetExpression`",
    );
    positions_in_order(
        atom,
        &[
            "let FiniteClassSet { ranges, strings }",
            "let singleton = finish_range_set",
            "let contains_empty = strings.iter().any(Vec::is_empty);",
            ".filter(|string| !string.is_empty())",
            ".map(RegExpInstruction::literal_code_point)",
            "multi_code_point_strings.sort_by_key(|string| std::cmp::Reverse(string.len()));",
            "multi_code_point_strings,",
            "singleton,",
            "contains_empty,",
        ],
    );
    assert!(IR_SOURCE.contains("ParsedAtom::FiniteClassSet(atom) => atom.contains_empty,"));
}

#[test]
fn one_lowerer_emits_longest_singleton_empty_priority_in_both_directions() {
    assert!(IR_SOURCE.contains("enum FiniteClassSetDirection {\n    Forward,\n    Reverse,\n}"));
    assert!(IR_SOURCE.contains(
        "ParsedAtom::FiniteClassSet(atom) => {\n                self.finite_class_set_atom(atom, FiniteClassSetDirection::Forward)"
    ));
    assert!(IR_SOURCE.contains(
        "ParsedAtom::FiniteClassSet(atom) => {\n                self.finite_class_set_atom(atom, FiniteClassSetDirection::Reverse)"
    ));

    let producer = bounded(
        IR_SOURCE,
        "    fn finite_class_set_atom(\n",
        "    fn reverse_alternatives(\n",
    );
    positions_in_order(
        producer,
        &[
            "atom.multi_code_point_strings.len() + 1 + usize::from(atom.contains_empty)",
            "for index in 0..alternative_count",
            "self.push(RegExpInstruction::split(0, 0))?;",
            "self.finite_class_set_alternative(atom, index, direction)?;",
            "self.push(RegExpInstruction::jump(0))?;",
            "self.instructions[split] = RegExpInstruction::split(primary, fallback);",
            "fn finite_class_set_alternative(",
            "atom.multi_code_point_strings.get(index)",
            "FiniteClassSetDirection::Forward",
            "for instruction in string",
            "FiniteClassSetDirection::Reverse",
            "for instruction in string.iter().rev()",
            "index == atom.multi_code_point_strings.len()",
            "self.push(atom.singleton)",
            "debug_assert!(atom.contains_empty);",
        ],
    );
    assert!(!producer.contains("progress_split"));
    assert!(IR_SOURCE.contains("fn unicode_sets_finite_string_algebra()"));
}

#[test]
fn existing_aot_choices_and_shared_range_matcher_cover_both_directions() {
    let split = bounded(
        MATCHER_SOURCE,
        "// `Split` records the fallback before taking the primary arm.",
        "// A nullable optional attempt carries its pre-attempt cursor",
    );
    positions_in_order(
        split,
        &[
            "REGEXP_OPCODE_SPLIT as i64",
            "LocalGet(operand1)",
            "LocalSet(choice_header)",
            "self.emit_regexp_push_choice_frame(",
            "LocalGet(operand0)",
            "LocalSet(pc)",
        ],
    );

    let frame = bounded(
        MATCHER_SOURCE,
        "    fn emit_regexp_push_choice_frame(\n",
        "    /// On an atom failure, restore the latest ordered fallback.",
    );
    for marker in [
        "for (offset, local) in [(0, header), (8, byte), (16, utf16), (24, on_low_surrogate)]",
        "Every ordered choice owns the full capture state.",
        "LocalGet(capture_count)",
        "I64Load(Self::memarg8(offset))",
        "I64Store(Self::memarg8(0))",
    ] {
        assert!(frame.contains(marker), "choice frame lost {marker}");
    }

    let accounting = bounded(
        DATA_SOURCE,
        "        let split_count = program\n",
        "        self.pending_regexp_programs.push((",
    );
    assert!(accounting.contains("REGEXP_OPCODE_SPLIT | REGEXP_OPCODE_PROGRESS_SPLIT"));

    assert_eq!(
        MATCHER_SOURCE
            .matches("self.emit_regexp_unicode_property_mismatch(")
            .count(),
        2,
        "forward and reverse matching must share one canonical range test"
    );
    let reverse = bounded(
        MATCHER_SOURCE,
        "        function.instruction(&Instruction::LocalGet(reverse_mode));\n",
        "        // Dot, an explicit UTF-16/code-point literal, and a negative ASCII",
    );
    for marker in [
        "REGEXP_OPCODE_LITERAL_CODE_POINT as i64",
        "REGEXP_OPCODE_UNICODE_PROPERTY as i64",
        "Reverse matching uses the same canonical range slice",
        "self.emit_regexp_unicode_property_mismatch(",
        "LocalGet(utf16_advance)",
        "I64Sub",
    ] {
        assert!(reverse.contains(marker), "reverse matcher lost {marker}");
    }
}

#[test]
fn exact_inventory_is_nine_union_nine_intersection_and_nine_difference_files() {
    assert_eq!(EXACT_TEST262.len(), 27);
    assert_eq!(
        EXACT_TEST262
            .iter()
            .filter(|(path, _)| path.contains("-union-"))
            .count(),
        9
    );
    assert_eq!(
        EXACT_TEST262
            .iter()
            .filter(|(path, _)| path.contains("-intersection-"))
            .count(),
        9
    );
    assert_eq!(
        EXACT_TEST262
            .iter()
            .filter(|(path, _)| path.contains("-difference-"))
            .count(),
        9
    );

    for (path, source) in EXACT_TEST262 {
        assert!(
            path.contains("string-literal"),
            "inventory lost q operand: {path}"
        );
        assert!(
            !path.contains("property-of-strings-escape"),
            "inventory crossed the property-of-strings boundary: {path}"
        );
        for marker in [
            "regexp-v-flag",
            "testExtendedCharacterClass({",
            "\\q{0|2|4|9\\uFE0F\\u20E3}",
        ] {
            assert!(source.contains(marker), "{path} lost {marker}");
        }
        assert!(!source.contains("flags:"), "{path} stopped being unflagged");
    }

    for marker in [
        "That is 27 physical files and 54 strict/non-strict executions.",
        "The six adjacent generated files whose name contains",
        "properties of strings remain a distinct typed capability",
        "Backward/lookbehind lowering preserves the same alternative priority",
        "does not implement Unicode properties of strings",
        "does not add a new Wasm matcher",
        "opcode or data pool",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost {marker}");
    }
}

#[test]
fn exact_inventory_has_no_rewrite_materializer_or_known_failure_mask() {
    for (path, _) in EXACT_TEST262 {
        for source in [TEST262_RUNNER_SOURCE, SHORTCUT_ALLOWLIST, KNOWN_FAILURES] {
            assert!(!source.contains(path), "{path} gained an exact mask");
        }
    }
}

#[test]
fn durable_fixture_exercises_complete_string_members_and_set_algebra() {
    for marker in [
        "longest class string first",
        "shorter class string restores capture",
        "multi-character member is indivisible",
        "first class-string capture",
        "string-left union",
        "string-right union",
        "string intersection",
        "intersection removes multi-code-point member",
        "string subtraction",
        "string-left subtraction retains keycap",
        "string-right subtraction removes scalar",
        "empty class string",
        "global empty progress",
        "lookbehind class-string capture",
    ] {
        assert!(FIXTURE.contains(marker), "fixture lost {marker}");
    }
    assert!(CLI_TEST_SOURCE.contains("fn run_wasm_backend_matches_unicode_sets_class_strings()"));
    assert!(CLI_TEST_SOURCE.contains("wasm_regexp_unicode_sets_class_strings.js"));
}

#[test]
fn verified_status_preserves_baseline_and_exact_scope() {
    for source in [README, TASK] {
        let source = source.split_whitespace().collect::<Vec<_>>().join(" ");
        for marker in [
            "f580b424d",
            "string-literal-union-string-literal.js",
            "string-literal-intersection-string-literal.js",
            "string-literal-difference-string-literal.js",
            "`0/2` sloppy/strict",
            "All six measured",
            "Runtime/NotImplemented",
            "RegExp.prototype.exec unsupported",
            "27-file/54-execution",
            "workspace/all-target",
            "`cargo xc`",
            "`1/1`",
            "`7/7`",
            "`54/54`",
            "zero parser, early-error, lowering, runtime, Wasm-backend",
            "reverse",
            "Unicode properties of strings",
            "`/iv`",
            "no broader UnicodeSets or RegExp completion claim",
        ] {
            assert!(source.contains(marker), "status lost {marker}");
        }
    }
}
