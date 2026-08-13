use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

use icu_properties::props::{GeneralCategory, GeneralCategoryGroup, IdContinue, IdStart, Script};
use icu_properties::script::ScriptWithExtensions;
use icu_properties::{CodePointMapData, CodePointSetData, PropertyParser};

/// The encoded width of every [`RegExpInstruction`] in bytes.
pub const REGEXP_INSTRUCTION_WIDTH: usize = 24;

/// The opcode for a successful match.
pub const REGEXP_OPCODE_ACCEPT: u64 = 0;
/// The opcode for an exact ASCII code-unit match.
pub const REGEXP_OPCODE_LITERAL_ASCII: u64 = 1;
/// The opcode for membership in an ASCII character class.
pub const REGEXP_OPCODE_POSITIVE_ASCII_CLASS: u64 = 2;
/// Branch to `operand0`, retaining `operand1` as the ordered fallback.
pub const REGEXP_OPCODE_SPLIT: u64 = 3;
/// Unconditionally branch to the absolute instruction index in `operand0`.
pub const REGEXP_OPCODE_JUMP: u64 = 4;
/// Record the current input position as the start of a numbered capture.
pub const REGEXP_OPCODE_CAPTURE_START: u64 = 5;
/// Record the current input position as the end of a numbered capture.
pub const REGEXP_OPCODE_CAPTURE_END: u64 = 6;
/// Clear captures in the one-based half-open range `[operand0, operand1)`.
pub const REGEXP_OPCODE_CLEAR_CAPTURE_RANGE: u64 = 7;
/// Match an ECMAScript WhiteSpace or LineTerminator code point (`\\s`).
pub const REGEXP_OPCODE_WHITESPACE: u64 = 8;
/// Match one UTF-16 code unit other than an ECMAScript line terminator.
pub const REGEXP_OPCODE_DOT: u64 = 9;
/// Match one Unicode scalar value (or a lone UTF-16 surrogate in Unicode mode).
pub const REGEXP_OPCODE_LITERAL_CODE_POINT: u64 = 10;
/// Match membership in a code-point range set stored in the program's range
/// pool. `operand0` is the index of the first range entry and `operand1` packs
/// the entry count in bits 1.. with the complement bit in bit 0.
pub const REGEXP_OPCODE_UNICODE_PROPERTY: u64 = 11;
/// Match the capture selected by a named backreference.
pub const REGEXP_OPCODE_NAMED_BACKREFERENCE: u64 = 12;
/// Match outside an ASCII character class.
pub const REGEXP_OPCODE_NEGATIVE_ASCII_CLASS: u64 = 13;
/// Match the numbered capture stored in `operand0`.
pub const REGEXP_OPCODE_NUMBERED_BACKREFERENCE: u64 = 14;
/// Assert that the next ASCII code unit equals `operand0` without consuming it.
pub const REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD: u64 = 15;
/// Assert that the next ASCII code unit differs from `operand0` without consuming it.
pub const REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD: u64 = 16;
/// Assert that the current position is the start of the input or a line.
pub const REGEXP_OPCODE_ASSERT_START: u64 = 17;
/// Assert that the current position is the end of the input or a line.
pub const REGEXP_OPCODE_ASSERT_END: u64 = 18;
/// Match one code point that is not ECMAScript WhiteSpace or a LineTerminator.
pub const REGEXP_OPCODE_NOT_WHITESPACE: u64 = 19;
/// Enter a reverse-matching lookbehind body.
pub const REGEXP_OPCODE_LOOKBEHIND_START: u64 = 20;
/// Complete a lookbehind body. `operand0` identifies its failure sentinel.
pub const REGEXP_OPCODE_LOOKBEHIND_END: u64 = 21;
/// Handle exhaustion of every path through a lookbehind body.
pub const REGEXP_OPCODE_LOOKBEHIND_FAILURE: u64 = 22;

/// The encoded width of one code-point range-pool entry in bytes.
pub const REGEXP_RANGE_ENTRY_WIDTH: usize = 8;

/// A deliberately generous ceiling on the number of pooled code-point ranges.
pub const REGEXP_MAX_RANGE_ENTRIES: usize = 1 << 16;

/// A deliberately small ceiling for expanded flat-atom matcher programs.
///
/// Bounded repetitions are expanded before code generation; rejecting a larger
/// program is preferable to silently truncating it or creating an unbounded
/// scratch requirement in the Wasm matcher.
pub const REGEXP_MAX_INSTRUCTIONS: usize = 4096;

/// A fixed-width instruction in a backend-neutral regular-expression program.
///
/// `PositiveAsciiClass` stores its 128-bit membership bitmap with bits 0 through
/// 63 in `operand0` and bits 64 through 127 in `operand1`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegExpInstruction {
    pub opcode: u64,
    pub operand0: u64,
    pub operand1: u64,
}

impl RegExpInstruction {
    pub const fn accept() -> Self {
        Self {
            opcode: REGEXP_OPCODE_ACCEPT,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn literal_ascii(code_unit: u8) -> Self {
        Self {
            opcode: REGEXP_OPCODE_LITERAL_ASCII,
            operand0: code_unit as u64,
            operand1: 0,
        }
    }

    pub const fn literal_code_point(code_point: u32) -> Self {
        assert!(code_point <= 0x10ffff);
        Self {
            opcode: REGEXP_OPCODE_LITERAL_CODE_POINT,
            operand0: code_point as u64,
            operand1: 0,
        }
    }

    pub const fn code_point_range_set(first_entry: u32, entry_count: u32, negated: bool) -> Self {
        Self {
            opcode: REGEXP_OPCODE_UNICODE_PROPERTY,
            operand0: first_entry as u64,
            operand1: ((entry_count as u64) << 1) | (negated as u64),
        }
    }

    pub const fn positive_ascii_class(bitmap_low: u64, bitmap_high: u64) -> Self {
        Self {
            opcode: REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
            operand0: bitmap_low,
            operand1: bitmap_high,
        }
    }

    pub const fn negative_ascii_class(bitmap_low: u64, bitmap_high: u64) -> Self {
        Self {
            opcode: REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
            operand0: bitmap_low,
            operand1: bitmap_high,
        }
    }

    pub const fn split(primary_pc: usize, fallback_pc: usize) -> Self {
        Self {
            opcode: REGEXP_OPCODE_SPLIT,
            operand0: primary_pc as u64,
            operand1: fallback_pc as u64,
        }
    }

    pub const fn jump(target_pc: usize) -> Self {
        Self {
            opcode: REGEXP_OPCODE_JUMP,
            operand0: target_pc as u64,
            operand1: 0,
        }
    }

    pub const fn capture_start(capture_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_CAPTURE_START,
            operand0: capture_id as u64,
            operand1: 0,
        }
    }

    pub const fn capture_end(capture_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_CAPTURE_END,
            operand0: capture_id as u64,
            operand1: 0,
        }
    }

    pub const fn clear_capture_range(first_capture_id: u32, end_capture_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_CLEAR_CAPTURE_RANGE,
            operand0: first_capture_id as u64,
            operand1: end_capture_id as u64,
        }
    }

    pub const fn whitespace() -> Self {
        Self {
            opcode: REGEXP_OPCODE_WHITESPACE,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn not_whitespace() -> Self {
        Self {
            opcode: REGEXP_OPCODE_NOT_WHITESPACE,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn dot() -> Self {
        Self {
            opcode: REGEXP_OPCODE_DOT,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn named_backreference(name_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_NAMED_BACKREFERENCE,
            operand0: name_id as u64,
            operand1: 0,
        }
    }

    pub const fn numbered_backreference(capture_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
            operand0: capture_id as u64,
            operand1: 0,
        }
    }

    pub const fn nonempty_numbered_backreference(capture_id: u32) -> Self {
        Self {
            opcode: REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
            operand0: capture_id as u64,
            operand1: 1,
        }
    }

    pub const fn positive_ascii_lookahead(code_unit: u8) -> Self {
        Self {
            opcode: REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD,
            operand0: code_unit as u64,
            operand1: 0,
        }
    }

    pub const fn negative_ascii_lookahead(code_unit: u8) -> Self {
        Self {
            opcode: REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD,
            operand0: code_unit as u64,
            operand1: 0,
        }
    }

    pub const fn assert_start() -> Self {
        Self {
            opcode: REGEXP_OPCODE_ASSERT_START,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn assert_end() -> Self {
        Self {
            opcode: REGEXP_OPCODE_ASSERT_END,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn lookbehind_start() -> Self {
        Self {
            opcode: REGEXP_OPCODE_LOOKBEHIND_START,
            operand0: 0,
            operand1: 0,
        }
    }

    pub const fn lookbehind_end(failure_pc: usize, after_pc: usize, negative: bool) -> Self {
        Self {
            opcode: REGEXP_OPCODE_LOOKBEHIND_END,
            operand0: failure_pc as u64,
            operand1: (after_pc as u64) | ((negative as u64) << 63),
        }
    }

    pub const fn lookbehind_failure(after_pc: usize, negative: bool) -> Self {
        Self {
            opcode: REGEXP_OPCODE_LOOKBEHIND_FAILURE,
            operand0: after_pc as u64,
            operand1: negative as u64,
        }
    }

    pub const fn positive_ascii_class_contains(self, code_unit: u8) -> bool {
        if self.opcode != REGEXP_OPCODE_POSITIVE_ASCII_CLASS || code_unit >= 128 {
            return false;
        }

        if code_unit < 64 {
            self.operand0 & (1_u64 << code_unit) != 0
        } else {
            self.operand1 & (1_u64 << (code_unit - 64)) != 0
        }
    }
}

const _: () = assert!(std::mem::size_of::<RegExpInstruction>() == REGEXP_INSTRUCTION_WIDTH);

/// Which of the three mutually exclusive RegExp Unicode grammars applies.
///
/// This mode is carried from flag parsing through atom and character-class
/// parsing. A pair of `unicode` / `unicode_sets` booleans could express an
/// impossible fourth state and previously let `v`-mode class atoms silently
/// inherit `u`-mode escape rules. Keeping the three legal states in one closed
/// domain makes both mistakes compile errors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RegExpUnicodeMode {
    /// Neither `u` nor `v`: legacy and Annex B grammar rules apply.
    #[default]
    Legacy,
    /// The `u` flag: Unicode-mode grammar rules apply.
    Unicode,
    /// The `v` flag: UnicodeSets-mode grammar rules apply.
    UnicodeSets,
}

impl RegExpUnicodeMode {
    /// Whether the restrictions shared by the `u` and `v` modes apply.
    pub const fn is_unicode_mode(self) -> bool {
        match self {
            RegExpUnicodeMode::Legacy => false,
            RegExpUnicodeMode::Unicode | RegExpUnicodeMode::UnicodeSets => true,
        }
    }

    /// Applies the mode-specific `ClassEscape` identity-escape grammar.
    ///
    /// This is deliberately exhaustive: `v` accepts the additional
    /// `ClassSetReservedPunctuator` alternatives that `u` does not.
    fn allows_class_identity_escape(self, escaped: u8) -> bool {
        match self {
            RegExpUnicodeMode::Legacy => true,
            RegExpUnicodeMode::Unicode => is_class_identity_escape(escaped),
            RegExpUnicodeMode::UnicodeSets => {
                is_class_identity_escape(escaped) || is_class_set_reserved_punctuator(escaped)
            }
        }
    }
}

/// RegExp flags that affect matching wrappers rather than match instructions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegExpFlags {
    pub has_indices: bool,
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub dot_all: bool,
    pub sticky: bool,
    pub unicode_mode: RegExpUnicodeMode,
}

/// One source-ordered named capture group and all numbered captures sharing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExpNamedGroup {
    pub name: String,
    pub capture_ids: Vec<u32>,
}

/// A compiled, backend-neutral regular-expression matcher program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExpProgram {
    pub flags: RegExpFlags,
    /// Number of numbered captures in opening-parenthesis order.
    pub capture_count: u32,
    /// Named groups in first-source-occurrence order.
    pub named_groups: Vec<RegExpNamedGroup>,
    pub instructions: Vec<RegExpInstruction>,
    /// Inclusive code-point ranges referenced by range-set instructions. The
    /// encoded blob stores these immediately after the instruction stream.
    pub ranges: Vec<(u32, u32)>,
}

/// An append-only pool of sorted, disjoint inclusive code-point ranges.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RegExpRangePool {
    entries: Vec<(u32, u32)>,
    interned: BTreeMap<Vec<(u32, u32)>, u32>,
}

impl RegExpRangePool {
    /// Interns `ranges`, returning the first entry index and the entry count.
    fn intern(
        &mut self,
        ranges: &[(u32, u32)],
        offset: usize,
    ) -> Result<(u32, u32), RegExpCompileError> {
        if let Some(&first) = self.interned.get(ranges) {
            return Ok((first, ranges.len() as u32));
        }
        if self.entries.len() + ranges.len() > REGEXP_MAX_RANGE_ENTRIES {
            return Err(RegExpCompileError::unsupported_feature(
                offset,
                "regular-expression code-point range pool is too large",
            ));
        }
        let first = self.entries.len() as u32;
        self.entries.extend_from_slice(ranges);
        self.interned.insert(ranges.to_vec(), first);
        Ok((first, ranges.len() as u32))
    }

    fn into_entries(self) -> Vec<(u32, u32)> {
        self.entries
    }
}

/// Normalizes an arbitrary list of inclusive ranges into a sorted, disjoint set.
fn normalize_ranges(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    ranges.retain(|(start, end)| start <= end);
    ranges.sort_unstable();
    let mut normalized: Vec<(u32, u32)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        match normalized.last_mut() {
            Some(last) if start <= last.1.saturating_add(1) => last.1 = last.1.max(end),
            _ => normalized.push((start, end)),
        }
    }
    normalized
}

fn ranges_contain(ranges: &[(u32, u32)], code_point: u32) -> bool {
    ranges
        .binary_search_by(|(start, end)| {
            if code_point < *start {
                std::cmp::Ordering::Greater
            } else if code_point > *end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

fn complement_ranges(ranges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut complement = Vec::with_capacity(ranges.len() + 1);
    let mut next = 0_u32;
    for &(start, end) in ranges {
        if start > next {
            complement.push((next, start - 1));
        }
        next = end.saturating_add(1);
        if end == u32::MAX {
            return complement;
        }
    }
    if next <= 0x10ffff {
        complement.push((next, 0x10ffff));
    }
    complement
}

fn intersect_ranges(left: &[(u32, u32)], right: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        let start = left[i].0.max(right[j].0);
        let end = left[i].1.min(right[j].1);
        if start <= end {
            result.push((start, end));
        }
        if left[i].1 < right[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    result
}

fn subtract_ranges(left: &[(u32, u32)], right: &[(u32, u32)]) -> Vec<(u32, u32)> {
    intersect_ranges(left, &complement_ranges(right))
}

impl RegExpProgram {
    pub fn compile(pattern: &str, flags: &str) -> Result<Self, RegExpCompileError> {
        let flags = parse_flags(flags)?;
        let parsed = parse_pattern(pattern, flags.unicode_mode, flags.ignore_case)?;
        let mut instructions = Vec::with_capacity(pattern.len() + 1);
        let mut lowerer =
            ProgramLowerer::new(&mut instructions, pattern.len(), &parsed.named_groups);
        lowerer.alternatives(&parsed.alternatives)?;
        lowerer.error_offset = pattern.len();
        lowerer.push(RegExpInstruction::accept())?;
        Ok(Self {
            flags,
            capture_count: parsed.capture_count,
            named_groups: parsed.named_groups,
            instructions,
            ranges: parsed.ranges,
        })
    }

    /// Encodes match instructions followed by the code-point range pool. Flags
    /// remain wrapper behavior.
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(
            self.instructions.len() * REGEXP_INSTRUCTION_WIDTH
                + self.ranges.len() * REGEXP_RANGE_ENTRY_WIDTH,
        );
        for instruction in &self.instructions {
            encoded.extend_from_slice(&instruction.opcode.to_le_bytes());
            encoded.extend_from_slice(&instruction.operand0.to_le_bytes());
            encoded.extend_from_slice(&instruction.operand1.to_le_bytes());
        }
        for (start, end) in &self.ranges {
            encoded.extend_from_slice(&start.to_le_bytes());
            encoded.extend_from_slice(&end.to_le_bytes());
        }
        encoded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegExpCompileErrorKind {
    InvalidSyntax,
    UnsupportedFeature,
}

/// The ECMA-262 production or early-error clause an [`RegExpCompileErrorKind::InvalidSyntax`]
/// rejection enforces.
///
/// # Why this exists, and why it is mandatory rather than advisory
///
/// The two verdicts are not symmetric. `InvalidSyntax` is a claim about
/// **ECMAScript** — a conforming engine rejects this pattern, so Lila must throw
/// a `SyntaxError` for it. `UnsupportedFeature` is a claim about **this
/// compiler** — the pattern is legal and Lila cannot build a matcher program for
/// it yet, so the runtime fallback gets its turn. Their costs differ by an order
/// of magnitude: an over-eager `UnsupportedFeature` loses a fast path, while an
/// over-eager `InvalidSyntax` invents a `SyntaxError` for a legal program. Since
/// batch 7 it does so at run time too — `lila-aot-wasm`'s runtime RegExp table
/// maps `InvalidSyntax` to `RuntimeRegExpEntry::Rejected`, which throws.
///
/// Batch 8 found two sites that had made exactly that mistake (`/\//u` and the
/// `v`-mode `ClassSetReservedPunctuator` escapes), so the citation is a
/// **parameter** of [`RegExpCompileError::invalid_syntax`], not a comment: a
/// rejection whose author cannot name the production it enforces does not
/// compile, and `unsupported_feature` — which cites nothing, because it claims
/// nothing about the spec — is the correct constructor for it.
///
/// [`SyntaxRule::ALL`] and the exhaustive `match` in [`SyntaxRule::citation`]
/// are what keep the witness table in this module's tests total: a new variant
/// forces an edit to both, and the table test fails until the new rule has a
/// pinned `(pattern, flags)` witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SyntaxRule {
    /// `(` with no matching `)`.
    UnclosedGroup,
    /// `)` with no matching `(`.
    StrayClosingParenthesis,
    /// The modifier-group flag list `(?ims-ims: … )`.
    ModifierFlags,
    /// A quantifier with no atom in front of it.
    QuantifierWithoutAtom,
    /// A quantifier applied to another quantifier.
    QuantifierAfterQuantifier,
    /// `{n,m}` with `m` below `n`.
    QuantifierBounds,
    /// A `SyntaxCharacter` used unescaped where the grammar requires an escape.
    UnescapedSyntaxCharacter,
    /// `\k` not followed by a `GroupName`.
    NamedBackreferenceSyntax,
    /// `\k<name>` naming no group.
    UnknownGroupName,
    /// Two group specifiers that can match at the same position sharing a name.
    DuplicateGroupName,
    /// The identifier grammar shared by `(?<name>…)` and `\k<name>`.
    RegExpIdentifierName,
    /// The flag string: unknown, duplicated, non-ASCII, or `u` with `v`.
    Flags,
    /// A `\` with nothing, or nothing legal, after it.
    CharacterEscape,
    /// `IdentityEscape` in Unicode mode.
    IdentityEscape,
    /// `ClassEscape` / `ClassSetCharacter` inside a character class.
    ClassEscape,
    /// `\xHH`.
    HexEscapeSequence,
    /// `\uHHHH`.
    UnicodeEscapeSequence,
    /// `\u{…}`.
    CodePointEscape,
    /// `\u{…}` above `U+10FFFF`.
    CodePointEscapeRange,
    /// The shape of `\p{…}` / `\P{…}`.
    UnicodePropertyEscape,
    /// The property name or value inside `\p{…}`.
    UnicodePropertyName,
    /// `[` with no matching `]`.
    UnclosedCharacterClass,
    /// A class range whose bounds are the wrong way round.
    ClassRangeOrder,
    /// A class range bound that is a character class rather than a character.
    ClassRangeBound,
}

impl SyntaxRule {
    /// Every rule, for the witness table in this module's tests.
    ///
    /// Hand-maintained, and deliberately paired with the exhaustive `match` in
    /// [`SyntaxRule::citation`]: adding a variant without extending that match
    /// is a compile error, and `every_syntax_rule_has_a_pinned_witness` fails
    /// for a variant that reaches this array with no witness. The array cannot
    /// be derived, so `all_syntax_rules_are_listed_once` checks it is sorted
    /// and duplicate-free — a copy-paste omission then shows up as a failing
    /// test rather than as a silently unaudited rule.
    pub(crate) const ALL: [SyntaxRule; 24] = [
        SyntaxRule::UnclosedGroup,
        SyntaxRule::StrayClosingParenthesis,
        SyntaxRule::ModifierFlags,
        SyntaxRule::QuantifierWithoutAtom,
        SyntaxRule::QuantifierAfterQuantifier,
        SyntaxRule::QuantifierBounds,
        SyntaxRule::UnescapedSyntaxCharacter,
        SyntaxRule::NamedBackreferenceSyntax,
        SyntaxRule::UnknownGroupName,
        SyntaxRule::DuplicateGroupName,
        SyntaxRule::RegExpIdentifierName,
        SyntaxRule::Flags,
        SyntaxRule::CharacterEscape,
        SyntaxRule::IdentityEscape,
        SyntaxRule::ClassEscape,
        SyntaxRule::HexEscapeSequence,
        SyntaxRule::UnicodeEscapeSequence,
        SyntaxRule::CodePointEscape,
        SyntaxRule::CodePointEscapeRange,
        SyntaxRule::UnicodePropertyEscape,
        SyntaxRule::UnicodePropertyName,
        SyntaxRule::UnclosedCharacterClass,
        SyntaxRule::ClassRangeOrder,
        SyntaxRule::ClassRangeBound,
    ];

    /// The production or early-error clause this rule enforces.
    ///
    /// Exhaustive, with no catch-all arm, per AGENTS.md.
    pub(crate) const fn citation(self) -> &'static str {
        match self {
            SyntaxRule::UnclosedGroup => {
                "22.2.1 Atom :: `(` GroupSpecifier? Disjunction `)`; the closing parenthesis is not optional"
            }
            SyntaxRule::StrayClosingParenthesis => {
                "22.2.1 Disjunction; `)` is a SyntaxCharacter and is not a PatternCharacter"
            }
            SyntaxRule::ModifierFlags => {
                "22.2.1 Atom :: `(` `?` RegularExpressionFlags `-`? RegularExpressionFlags? `:` Disjunction `)`, and its 22.2.1.1 early errors"
            }
            SyntaxRule::QuantifierWithoutAtom => {
                "22.2.1 Term :: Atom Quantifier; a Quantifier requires a preceding Atom"
            }
            SyntaxRule::QuantifierAfterQuantifier => {
                "22.2.1 Term :: Atom Quantifier; a Quantifier is not itself an Atom"
            }
            SyntaxRule::QuantifierBounds => {
                "22.2.1.1: it is a Syntax Error if the MV of the first DecimalDigits of a QuantifierPrefix is larger than the MV of the second"
            }
            SyntaxRule::UnescapedSyntaxCharacter => {
                "22.2.1 Atom :: PatternCharacter :: SourceCharacter but not SyntaxCharacter; Annex B ExtendedPatternCharacter does not apply in UnicodeMode"
            }
            SyntaxRule::NamedBackreferenceSyntax => {
                "22.2.1 AtomEscape[+NamedCaptureGroups] :: `k` GroupName"
            }
            SyntaxRule::UnknownGroupName => {
                "22.2.1.1: it is a Syntax Error if GroupSpecifiersThatMatch(GroupName) is empty"
            }
            SyntaxRule::DuplicateGroupName => {
                "22.2.1.1 Pattern early error: it is a Syntax Error if MightBothParticipate is true for two GroupSpecifiers with the same name"
            }
            SyntaxRule::RegExpIdentifierName => {
                "22.2.1 RegExpIdentifierName :: RegExpIdentifierStart RegExpIdentifierPart*, and its 22.2.1.1 early errors"
            }
            SyntaxRule::Flags => {
                "22.2.3.1 RegExpInitialize: it is a Syntax Error if F contains a code unit outside `dgimsuvy`, repeats one, or contains both `u` and `v`"
            }
            SyntaxRule::CharacterEscape => {
                "22.2.1 AtomEscape :: CharacterEscape and ClassEscape :: CharacterEscape; `\\` must be followed by an escape"
            }
            SyntaxRule::IdentityEscape => {
                "22.2.1 IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/`"
            }
            SyntaxRule::ClassEscape => {
                "22.2.1 ClassEscape[+UnicodeMode] :: `b` | `-` | CharacterClassEscape | CharacterEscape, extended in UnicodeSetsMode by ClassSetCharacter :: `\\` ClassSetReservedPunctuator"
            }
            SyntaxRule::HexEscapeSequence => {
                "22.2.1 CharacterEscape :: HexEscapeSequence :: `x` HexDigit HexDigit"
            }
            SyntaxRule::UnicodeEscapeSequence => {
                "22.2.1 RegExpUnicodeEscapeSequence[+UnicodeMode] :: `u` Hex4Digits"
            }
            SyntaxRule::CodePointEscape => {
                "22.2.1 RegExpUnicodeEscapeSequence[+UnicodeMode] :: `u{` CodePoint `}`"
            }
            SyntaxRule::CodePointEscapeRange => {
                "22.2.1 CodePoint early error: it is a Syntax Error if the MV of HexDigits is greater than 0x10FFFF"
            }
            SyntaxRule::UnicodePropertyEscape => {
                "22.2.1 CharacterClassEscape :: `p{` UnicodePropertyValueExpression `}`"
            }
            SyntaxRule::UnicodePropertyName => {
                "22.2.1.1: it is a Syntax Error if UnicodeMatchProperty / UnicodeMatchPropertyValue does not resolve against tables 69-72"
            }
            SyntaxRule::UnclosedCharacterClass => {
                "22.2.1 CharacterClass :: `[` ClassContents `]`"
            }
            SyntaxRule::ClassRangeOrder => {
                "22.2.1.1 NonemptyClassRanges early error: it is a Syntax Error if the CharacterValue of the first ClassAtom is strictly greater than that of the second"
            }
            SyntaxRule::ClassRangeBound => {
                "22.2.1.1: it is a Syntax Error if IsCharacterClass of either ClassAtom of a range is true"
            }
        }
    }
}

/// The message every `&str`-boundary decode failure inside this parser reports.
///
/// [`RegExpProgram::compile`] takes `&str`, so the byte slice these parsers walk
/// is always well-formed UTF-8 and `std::str::from_utf8` can only fail if the
/// cursor reached a position that is not a character boundary. That is a defect
/// in *this compiler*, never in the pattern, so none of those sites can cite an
/// ECMA-262 production and none of them may answer `InvalidSyntax`. They were
/// `InvalidSyntax` until batch 8; `UnsupportedFeature` is the honest verdict of
/// the two this error type has. Making the state unrepresentable — decoding from
/// the `&str` rather than re-decoding a `&[u8]` — is filed as follow-up in
/// `target/lane-notes/re-verdict-b8-integration.md`.
const NON_BOUNDARY_SOURCE: &str =
    "regular-expression source could not be decoded at a character boundary";

/// A compile failure with the byte offset in the pattern or flag string supplied
/// to [`RegExpProgram::compile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExpCompileError {
    pub kind: RegExpCompileErrorKind,
    pub offset: usize,
    pub message: String,
    /// The rule an `InvalidSyntax` rejection enforces.
    ///
    /// `None` exactly when `kind` is `UnsupportedFeature`, which is a claim
    /// about this compiler and cites nothing. Carried on the error rather than
    /// only passed to the constructor so that a test can pin *which* site
    /// answered, not merely that some site did.
    pub(crate) rule: Option<SyntaxRule>,
}

impl RegExpCompileError {
    /// A claim that a conforming engine rejects this pattern.
    ///
    /// `rule` is mandatory. See [`SyntaxRule`] for why.
    fn invalid_syntax(rule: SyntaxRule, offset: usize, message: impl Into<String>) -> Self {
        Self {
            kind: RegExpCompileErrorKind::InvalidSyntax,
            offset,
            message: message.into(),
            rule: Some(rule),
        }
    }

    /// A claim about this compiler only: the pattern is legal and Lila cannot
    /// build a matcher program for it. Cites nothing, by construction.
    fn unsupported_feature(offset: usize, message: impl Into<String>) -> Self {
        Self {
            kind: RegExpCompileErrorKind::UnsupportedFeature,
            offset,
            message: message.into(),
            rule: None,
        }
    }
}

impl fmt::Display for RegExpCompileError {
    /// An `InvalidSyntax` rejection prints the rule it enforces.
    ///
    /// Not decoration, and not only for the reader: this is what puts
    /// [`SyntaxRule::citation`] on the product path. `lowering.rs` formats this
    /// message into the `SyntaxError` a rejected pattern throws, so the thrown
    /// error now names the production — and a citation that no code path reads
    /// would be exactly the "compiles clean, no call site" shape AGENTS.md
    /// warns about, which is how an unaudited claim survives.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)?;
        match self.rule {
            Some(rule) => write!(formatter, " ({})", rule.citation()),
            None => Ok(()),
        }
    }
}

/// [`SyntaxRule::ALL`] must list every variant exactly once. The length is the
/// half of that a `const` can check; `all_syntax_rules_are_listed_once` checks
/// ordering and uniqueness, and `every_syntax_rule_has_a_pinned_witness` checks
/// that each one is demonstrable.
const _: () = assert!(SyntaxRule::ALL.len() == 24);

impl Error for RegExpCompileError {}

struct ParsedPattern {
    alternatives: Vec<Vec<ParsedTerm>>,
    capture_count: u32,
    named_groups: Vec<RegExpNamedGroup>,
    ranges: Vec<(u32, u32)>,
}

enum ParsedTerm {
    Quantified {
        atom: ParsedAtom,
        quantifier: Quantifier,
        quantifier_offset: usize,
    },
    LegacyUtf16Pair {
        pair: LegacyUtf16Pair,
        trail_quantifier: Quantifier,
        quantifier_offset: usize,
    },
}

mod legacy_utf16_pair {
    use super::RegExpInstruction;

    #[derive(Clone, Copy)]
    pub(super) struct LegacyUtf16Pair {
        lead: u32,
        trail: u32,
    }

    impl LegacyUtf16Pair {
        pub(super) fn from_scalar(scalar: char) -> Option<Self> {
            let supplementary = u32::from(scalar).checked_sub(0x1_0000)?;
            Some(Self {
                lead: 0xD800 + (supplementary >> 10),
                trail: 0xDC00 + (supplementary & 0x3ff),
            })
        }

        pub(super) fn lead_instruction(self) -> RegExpInstruction {
            RegExpInstruction::literal_code_point(self.lead)
        }

        pub(super) fn trail_instruction(self) -> RegExpInstruction {
            RegExpInstruction::literal_code_point(self.trail)
        }
    }
}

use legacy_utf16_pair::LegacyUtf16Pair;

enum ParsedTermAtom {
    Ordinary(ParsedAtom),
    LegacyUtf16Pair(LegacyUtf16Pair),
}

enum ParsedAtom {
    Instruction(RegExpInstruction),
    Capture {
        id: u32,
        body: Vec<Vec<ParsedTerm>>,
        subtree_end: u32,
    },
    NonCapture {
        body: Vec<Vec<ParsedTerm>>,
        subtree_start: u32,
        subtree_end: u32,
    },
    NamedBackreference {
        name: String,
        offset: usize,
    },
    NumberedBackreference {
        capture_id: u32,
        nullable: bool,
    },
    Lookbehind {
        negative: bool,
        body: Vec<Vec<ParsedTerm>>,
    },
}

struct NamedCapture {
    name: String,
    id: u32,
    offset: usize,
    path: Vec<(u32, usize)>,
}

fn parse_pattern(
    pattern: &str,
    unicode_mode: RegExpUnicodeMode,
    ignore_case: bool,
) -> Result<ParsedPattern, RegExpCompileError> {
    if pattern.is_empty() {
        return Ok(ParsedPattern {
            alternatives: vec![Vec::new()],
            capture_count: 0,
            named_groups: Vec::new(),
            ranges: Vec::new(),
        });
    }

    let (total_capture_count, has_named_capture_syntax) = regexp_capture_syntax(pattern.as_bytes());
    let mut parser = PatternParser {
        bytes: pattern.as_bytes(),
        offset: 0,
        capture_count: 0,
        unicode_mode,
        modifiers: Modifiers {
            ignore_case,
            multiline: None,
            dot_all: None,
        },
        ranges: RegExpRangePool::default(),
        choice_count: 0,
        choice_path: Vec::new(),
        named_captures: Vec::new(),
        capture_nullability: BTreeMap::new(),
        total_capture_count,
        has_named_capture_syntax,
    };
    let alternatives = parser.alternatives(None)?;
    let named_groups = named_groups(&parser.named_captures)?;
    Ok(ParsedPattern {
        alternatives,
        capture_count: parser.capture_count,
        named_groups,
        ranges: parser.ranges.into_entries(),
    })
}

/// Matching state that RegExp modifier groups (`(?i-s:…)`) can override for the
/// enclosed pattern only.
#[derive(Debug, Clone, Copy)]
struct Modifiers {
    ignore_case: bool,
    /// `None` defers to the runtime `m` flag; `Some` forces the local value.
    multiline: Option<bool>,
    /// `None` defers to the runtime `s` flag; `Some` forces the local value.
    dot_all: Option<bool>,
}

struct PatternParser<'a> {
    bytes: &'a [u8],
    offset: usize,
    capture_count: u32,
    unicode_mode: RegExpUnicodeMode,
    modifiers: Modifiers,
    ranges: RegExpRangePool,
    choice_count: u32,
    choice_path: Vec<(u32, usize)>,
    named_captures: Vec<NamedCapture>,
    capture_nullability: BTreeMap<u32, bool>,
    total_capture_count: u32,
    has_named_capture_syntax: bool,
}

impl PatternParser<'_> {
    fn alternatives(
        &mut self,
        opening: Option<usize>,
    ) -> Result<Vec<Vec<ParsedTerm>>, RegExpCompileError> {
        let choice_id = self.choice_count;
        self.choice_count += 1;
        self.choice_path.push((choice_id, 0));
        let result = self.alternatives_inner(opening, &mut vec![Vec::new()]);
        self.choice_path.pop();
        result
    }

    fn alternatives_inner(
        &mut self,
        opening: Option<usize>,
        alternatives: &mut Vec<Vec<ParsedTerm>>,
    ) -> Result<Vec<Vec<ParsedTerm>>, RegExpCompileError> {
        loop {
            match self.bytes.get(self.offset).copied() {
                None => {
                    if let Some(opening) = opening {
                        return Err(RegExpCompileError::invalid_syntax(
                            SyntaxRule::UnclosedGroup,
                            opening,
                            "regular-expression capturing group is unclosed",
                        ));
                    }
                    break;
                }
                Some(b')') => {
                    if opening.is_none() {
                        return Err(RegExpCompileError::invalid_syntax(
                            SyntaxRule::StrayClosingParenthesis,
                            self.offset,
                            "regular-expression closing parenthesis has no opening parenthesis",
                        ));
                    }
                    self.offset += 1;
                    break;
                }
                Some(b'|') => {
                    self.offset += 1;
                    alternatives.push(Vec::new());
                    self.choice_path.last_mut().unwrap().1 += 1;
                }
                _ => alternatives.last_mut().unwrap().push(self.term()?),
            }
        }
        Ok(std::mem::take(alternatives))
    }

    fn term(&mut self) -> Result<ParsedTerm, RegExpCompileError> {
        let atom_offset = self.offset;
        let atom = if matches!(
            self.bytes.get(self.offset..self.offset + 4),
            Some([b'(', b'?', b'=' | b'!', byte @ 0..=127])
                if *byte != b')' && self.bytes.get(self.offset + 4) == Some(&b')')
        ) {
            let negative = self.bytes[self.offset + 2] == b'!';
            let code_unit = self.bytes[self.offset + 3];
            self.offset += 5;
            ParsedTermAtom::Ordinary(ParsedAtom::Instruction(if negative {
                RegExpInstruction::negative_ascii_lookahead(code_unit)
            } else {
                RegExpInstruction::positive_ascii_lookahead(code_unit)
            }))
        } else if self.bytes[self.offset] == b'(' {
            if self.bytes.get(self.offset + 1) == Some(&b'?') {
                match self.bytes.get(self.offset + 2).copied() {
                    Some(b':') => {
                        self.offset += 3;
                        let subtree_start = self.capture_count + 1;
                        let body = self.alternatives(Some(atom_offset))?;
                        ParsedTermAtom::Ordinary(ParsedAtom::NonCapture {
                            body,
                            subtree_start,
                            subtree_end: self.capture_count + 1,
                        })
                    }
                    Some(b'<') => {
                        if matches!(self.bytes.get(self.offset + 3), Some(b'=') | Some(b'!')) {
                            let negative = self.bytes[self.offset + 3] == b'!';
                            self.offset += 4;
                            let body = self.alternatives(Some(atom_offset))?;
                            if !lookbehind_body_supported(&body) {
                                return Err(RegExpCompileError::unsupported_feature(
                                    atom_offset,
                                    "lookbehind body uses an unsupported matcher atom",
                                ));
                            }
                            ParsedTermAtom::Ordinary(ParsedAtom::Lookbehind { negative, body })
                        } else {
                            let name = self.parse_group_name()?;
                            self.capture_count =
                                self.capture_count.checked_add(1).ok_or_else(|| {
                                    RegExpCompileError::unsupported_feature(
                                        atom_offset,
                                        "regular-expression has too many numbered captures",
                                    )
                                })?;
                            let id = self.capture_count;
                            self.named_captures.push(NamedCapture {
                                name,
                                id,
                                offset: atom_offset,
                                path: self.choice_path.clone(),
                            });
                            let body = self.alternatives(Some(atom_offset))?;
                            self.capture_nullability.insert(
                                id,
                                body.iter()
                                    .any(|sequence| sequence.iter().all(term_nullable)),
                            );
                            ParsedTermAtom::Ordinary(ParsedAtom::Capture {
                                id,
                                body,
                                subtree_end: self.capture_count + 1,
                            })
                        }
                    }
                    Some(b'i' | b'm' | b's' | b'-') => {
                        let modifiers = self.parse_modifier_group_prefix(atom_offset)?;
                        let outer = self.modifiers;
                        self.modifiers = modifiers;
                        let subtree_start = self.capture_count + 1;
                        let body = self.alternatives(Some(atom_offset));
                        self.modifiers = outer;
                        ParsedTermAtom::Ordinary(ParsedAtom::NonCapture {
                            body: body?,
                            subtree_start,
                            subtree_end: self.capture_count + 1,
                        })
                    }
                    _ => {
                        return Err(RegExpCompileError::unsupported_feature(
                            self.offset,
                            "unsupported regular-expression group prefix",
                        ));
                    }
                }
            } else {
                self.capture_count = self.capture_count.checked_add(1).ok_or_else(|| {
                    RegExpCompileError::unsupported_feature(
                        self.offset,
                        "regular-expression has too many numbered captures",
                    )
                })?;
                let id = self.capture_count;
                self.offset += 1;
                let body = self.alternatives(Some(atom_offset))?;
                self.capture_nullability.insert(
                    id,
                    body.iter()
                        .any(|sequence| sequence.iter().all(term_nullable)),
                );
                ParsedTermAtom::Ordinary(ParsedAtom::Capture {
                    id,
                    body,
                    subtree_end: self.capture_count + 1,
                })
            }
        } else {
            let atom = parse_instruction_atom(
                self.bytes,
                &mut self.offset,
                self.unicode_mode,
                self.modifiers,
                &mut self.ranges,
                self.total_capture_count,
                self.has_named_capture_syntax,
            )?;
            match atom {
                ParsedTermAtom::Ordinary(ParsedAtom::NumberedBackreference {
                    capture_id,
                    nullable: _,
                }) => ParsedTermAtom::Ordinary(ParsedAtom::NumberedBackreference {
                    capture_id,
                    nullable: self
                        .capture_nullability
                        .get(&capture_id)
                        .copied()
                        .unwrap_or(true),
                }),
                ParsedTermAtom::Ordinary(ParsedAtom::Instruction(mut instruction)) => {
                    apply_modifiers(&mut instruction, self.modifiers);
                    ParsedTermAtom::Ordinary(ParsedAtom::Instruction(instruction))
                }
                atom => atom,
            }
        };
        let quantifier_offset = self.offset;
        let mut quantifier = parse_postfix_quantifier(self.bytes, &mut self.offset)?;
        if matches!(
            atom,
            ParsedTermAtom::Ordinary(ParsedAtom::Instruction(RegExpInstruction {
                opcode: REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD
                    | REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD
                    | REGEXP_OPCODE_ASSERT_START
                    | REGEXP_OPCODE_ASSERT_END,
                ..
            }))
        ) {
            quantifier = if quantifier.min == 0 {
                Quantifier {
                    min: 0,
                    max: Some(0),
                    lazy: quantifier.lazy,
                }
            } else {
                Quantifier {
                    min: 1,
                    max: Some(1),
                    lazy: quantifier.lazy,
                }
            };
        }
        if quantifier.max.is_none()
            && matches!(&atom, ParsedTermAtom::Ordinary(atom) if atom_nullable(atom))
        {
            return Err(RegExpCompileError::unsupported_feature(
                quantifier_offset,
                "unbounded quantifier over a nullable atom is unsupported by this matcher-program grammar",
            ));
        }
        Ok(match atom {
            ParsedTermAtom::Ordinary(atom) => ParsedTerm::Quantified {
                atom,
                quantifier,
                quantifier_offset,
            },
            ParsedTermAtom::LegacyUtf16Pair(pair) => ParsedTerm::LegacyUtf16Pair {
                pair,
                trail_quantifier: quantifier,
                quantifier_offset,
            },
        })
    }

    /// Parses `(?ims-ims:` and returns the modifier state for the group body.
    ///
    /// `self.offset` is left immediately after the `:`.
    fn parse_modifier_group_prefix(
        &mut self,
        group_offset: usize,
    ) -> Result<Modifiers, RegExpCompileError> {
        let mut cursor = group_offset + 2;
        let mut added = 0_u8;
        let mut removed = 0_u8;
        let mut seen_dash = false;
        loop {
            let Some(&byte) = self.bytes.get(cursor) else {
                // `ModifierFlags`, not `UnclosedGroup`: what ran out here is the
                // modifier prefix of `(?` RegularExpressionFlags `:`, so
                // `UnclosedGroup`'s citation ("Atom :: `(` GroupSpecifier?
                // Disjunction `)`") names a production this pattern does not
                // violate — and since `Display` puts `citation()` on the product
                // path, `/(?i/` would throw a `SyntaxError` naming the wrong
                // rule. The five sibling rejections in this function all cite
                // `ModifierFlags`. `every_syntax_rule_has_a_pinned_witness` is
                // per-RULE, not per-SITE, so it cannot catch a wrong variant at a
                // non-witness site; this is one such site.
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::ModifierFlags,
                    group_offset,
                    "regular-expression modifier group is unclosed",
                ));
            };
            if byte == b':' {
                cursor += 1;
                break;
            }
            if byte == b'-' {
                if seen_dash {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::ModifierFlags,
                        cursor,
                        "regular-expression modifier group has a repeated `-`",
                    ));
                }
                seen_dash = true;
                cursor += 1;
                continue;
            }
            let bit = match byte {
                b'i' => 1 << 0,
                b'm' => 1 << 1,
                b's' => 1 << 2,
                byte => {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::ModifierFlags,
                        cursor,
                        format!(
                            "invalid regular-expression modifier `{}`",
                            byte.escape_ascii()
                        ),
                    ));
                }
            };
            if added & bit != 0 || removed & bit != 0 {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::ModifierFlags,
                    cursor,
                    format!("duplicate regular-expression modifier `{}`", byte as char),
                ));
            }
            if seen_dash {
                removed |= bit;
            } else {
                added |= bit;
            }
            cursor += 1;
        }
        if added == 0 && removed == 0 {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::ModifierFlags,
                group_offset,
                "regular-expression modifier group has no modifiers",
            ));
        }
        if seen_dash && removed == 0 {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::ModifierFlags,
                group_offset,
                "regular-expression modifier group has an empty removal list",
            ));
        }
        self.offset = cursor;
        let mut modifiers = self.modifiers;
        if added & 1 != 0 {
            modifiers.ignore_case = true;
        }
        if removed & 1 != 0 {
            modifiers.ignore_case = false;
        }
        if added & 2 != 0 {
            modifiers.multiline = Some(true);
        }
        if removed & 2 != 0 {
            modifiers.multiline = Some(false);
        }
        if added & 4 != 0 {
            modifiers.dot_all = Some(true);
        }
        if removed & 4 != 0 {
            modifiers.dot_all = Some(false);
        }
        Ok(modifiers)
    }

    fn parse_group_name(&mut self) -> Result<String, RegExpCompileError> {
        let start = self.offset + 3;
        let (name, end) = parse_regexp_identifier_name(self.bytes, start, "named capture group")?;
        self.offset = end;
        Ok(name)
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_instruction_atom(
    bytes: &[u8],
    offset: &mut usize,
    unicode_mode: RegExpUnicodeMode,
    modifiers: Modifiers,
    pool: &mut RegExpRangePool,
    total_capture_count: u32,
    has_named_capture_syntax: bool,
) -> Result<ParsedTermAtom, RegExpCompileError> {
    let atom_offset = *offset;
    let byte = bytes[atom_offset];
    let unicode = unicode_mode.is_unicode_mode();
    if bytes.get(atom_offset..atom_offset + 2) == Some(b"\\k") {
        if !unicode && !has_named_capture_syntax {
            *offset += 2;
            return Ok(ParsedTermAtom::Ordinary(ParsedAtom::Instruction(
                RegExpInstruction::literal_ascii(b'k'),
            )));
        }
        if bytes.get(atom_offset + 2) != Some(&b'<') {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::NamedBackreferenceSyntax,
                atom_offset,
                "malformed named backreference",
            ));
        }
        let (name, end) =
            parse_regexp_identifier_name(bytes, atom_offset + 3, "named backreference")?;
        *offset = end;
        return Ok(ParsedTermAtom::Ordinary(ParsedAtom::NamedBackreference {
            name,
            offset: atom_offset,
        }));
    }
    if byte == b'\\' {
        if let Some(digit @ b'1'..=b'9') = bytes.get(atom_offset + 1).copied() {
            let capture_id = u32::from(digit - b'0');
            if capture_id <= total_capture_count {
                *offset += 2;
                return Ok(ParsedTermAtom::Ordinary(
                    ParsedAtom::NumberedBackreference {
                        capture_id,
                        nullable: true,
                    },
                ));
            }
        }
    }
    if !byte.is_ascii() {
        if unicode {
            let source = std::str::from_utf8(&bytes[atom_offset..]).map_err(|_| {
                RegExpCompileError::unsupported_feature(atom_offset, NON_BOUNDARY_SOURCE)
            })?;
            let ch = source.chars().next().expect("non-empty source");
            *offset += ch.len_utf8();
            return Ok(ParsedTermAtom::Ordinary(ParsedAtom::Instruction(
                RegExpInstruction::literal_code_point(ch as u32),
            )));
        }
        let source = std::str::from_utf8(&bytes[atom_offset..]).map_err(|_| {
            RegExpCompileError::unsupported_feature(atom_offset, NON_BOUNDARY_SOURCE)
        })?;
        let ch = source.chars().next().expect("non-empty source");
        *offset += ch.len_utf8();
        let code_point = ch as u32;
        if code_point <= 0xffff {
            return Ok(ParsedTermAtom::Ordinary(ParsedAtom::Instruction(
                RegExpInstruction::literal_code_point(code_point),
            )));
        }
        let pair = LegacyUtf16Pair::from_scalar(ch)
            .expect("an astral Unicode scalar has one UTF-16 surrogate pair");
        return Ok(ParsedTermAtom::LegacyUtf16Pair(pair));
    }
    let instruction = match byte {
        b'^' => {
            *offset += 1;
            RegExpInstruction::assert_start()
        }
        b'$' => {
            *offset += 1;
            RegExpInstruction::assert_end()
        }
        b'{' => {
            // A complete braced quantifier cannot appear without a preceding
            // atom. Incomplete/non-decimal forms are Annex B literal braces.
            let mut probe = atom_offset;
            if parse_braced_quantifier(bytes, &mut probe)?.is_some() {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::QuantifierWithoutAtom,
                    atom_offset,
                    "regular-expression quantifier has no preceding atom",
                ));
            }
            if unicode {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::UnescapedSyntaxCharacter,
                    atom_offset,
                    "unescaped regular-expression opening brace is invalid in Unicode mode",
                ));
            }
            *offset += 1;
            RegExpInstruction::literal_ascii(byte)
        }
        b'}' => {
            *offset += 1;
            RegExpInstruction::literal_ascii(byte)
        }
        b']' => {
            *offset += 1;
            RegExpInstruction::literal_ascii(byte)
        }
        b'[' => match unicode_mode {
            RegExpUnicodeMode::Legacy => parse_class(bytes, offset, unicode_mode, modifiers, pool)?,
            RegExpUnicodeMode::Unicode => {
                if let Some(instruction) = parse_single_unicode_class(bytes, offset)? {
                    instruction
                } else {
                    parse_class(bytes, offset, unicode_mode, modifiers, pool)?
                }
            }
            RegExpUnicodeMode::UnicodeSets => {
                parse_unicode_sets_class(bytes, offset, modifiers, pool)?
            }
        },
        b'\\' => parse_escaped_atom(bytes, offset, unicode_mode, modifiers, pool)?,
        b'.' => {
            *offset += 1;
            RegExpInstruction::dot()
        }
        b'*' | b'+' | b'?' => {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::QuantifierWithoutAtom,
                atom_offset,
                "regular-expression quantifier has no preceding atom",
            ));
        }
        byte if is_syntax_character(byte) => {
            return Err(RegExpCompileError::unsupported_feature(
                atom_offset,
                format!(
                    "unsupported regular-expression metacharacter `{}`",
                    byte as char
                ),
            ));
        }
        byte => {
            *offset += 1;
            RegExpInstruction::literal_ascii(byte)
        }
    };
    Ok(ParsedTermAtom::Ordinary(ParsedAtom::Instruction(
        instruction,
    )))
}

fn regexp_capture_syntax(bytes: &[u8]) -> (u32, bool) {
    let mut capture_count = 0_u32;
    let mut has_named_capture = false;
    let mut offset = 0;
    let mut in_class = false;
    while let Some(&byte) = bytes.get(offset) {
        if byte == b'\\' {
            offset += 2;
            continue;
        }
        if byte == b'[' {
            in_class = true;
            offset += 1;
            continue;
        }
        if byte == b']' && in_class {
            in_class = false;
            offset += 1;
            continue;
        }
        if byte != b'(' || in_class {
            offset += 1;
            continue;
        }
        match bytes.get(offset + 1..offset + 3) {
            Some(b"?<") if matches!(bytes.get(offset + 3), Some(b'=') | Some(b'!')) => {}
            Some(b"?<") if !matches!(bytes.get(offset + 3), Some(b'=') | Some(b'!')) => {
                capture_count += 1;
                has_named_capture = true;
            }
            Some(b"?:" | b"?=" | b"?!") => {}
            _ => capture_count += 1,
        }
        offset += 1;
    }
    (capture_count, has_named_capture)
}

fn atom_nullable(atom: &ParsedAtom) -> bool {
    match atom {
        ParsedAtom::Instruction(instruction) => matches!(
            instruction.opcode,
            REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD
                | REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD
                | REGEXP_OPCODE_ASSERT_START
                | REGEXP_OPCODE_ASSERT_END
        ),
        ParsedAtom::Capture { body, .. } | ParsedAtom::NonCapture { body, .. } => body
            .iter()
            .any(|sequence| sequence.iter().all(|term| term_nullable(term))),
        ParsedAtom::NamedBackreference { .. } => true,
        ParsedAtom::NumberedBackreference { nullable, .. } => *nullable,
        ParsedAtom::Lookbehind { .. } => true,
    }
}

fn lookbehind_body_supported(alternatives: &[Vec<ParsedTerm>]) -> bool {
    alternatives.iter().flatten().all(|term| match term {
        ParsedTerm::Quantified { atom, .. } => match atom {
            ParsedAtom::Instruction(instruction) => matches!(
                instruction.opcode,
                REGEXP_OPCODE_LITERAL_ASCII
                    | REGEXP_OPCODE_POSITIVE_ASCII_CLASS
                    | REGEXP_OPCODE_NEGATIVE_ASCII_CLASS
                    | REGEXP_OPCODE_DOT
            ),
            ParsedAtom::Capture { body, .. } | ParsedAtom::NonCapture { body, .. } => {
                lookbehind_body_supported(body)
            }
            ParsedAtom::NamedBackreference { .. }
            | ParsedAtom::NumberedBackreference { .. }
            | ParsedAtom::Lookbehind { .. } => false,
        },
        ParsedTerm::LegacyUtf16Pair { .. } => false,
    })
}
fn term_nullable(term: &ParsedTerm) -> bool {
    match term {
        ParsedTerm::Quantified {
            atom, quantifier, ..
        } => quantifier.min == 0 || atom_nullable(atom),
        ParsedTerm::LegacyUtf16Pair { .. } => false,
    }
}

fn named_groups(captures: &[NamedCapture]) -> Result<Vec<RegExpNamedGroup>, RegExpCompileError> {
    let mut groups = Vec::<RegExpNamedGroup>::new();
    let mut first = Vec::<&NamedCapture>::new();
    for capture in captures {
        if let Some((index, prior)) = first
            .iter()
            .enumerate()
            .find(|(_, prior)| prior.name == capture.name)
        {
            if !duplicate_names_diverge(prior, capture) {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::DuplicateGroupName,
                    capture.offset,
                    format!("duplicate named capture group `{}`", capture.name),
                ));
            }
            // Every prior occurrence must be on a distinct arm of a shared choice.
            if groups[index].capture_ids.len() > 1
                && captures
                    .iter()
                    .filter(|other| other.name == capture.name)
                    .take_while(|other| other.id != capture.id)
                    .any(|other| !duplicate_names_diverge(other, capture))
            {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::DuplicateGroupName,
                    capture.offset,
                    format!("duplicate named capture group `{}`", capture.name),
                ));
            }
            groups[index].capture_ids.push(capture.id);
        } else {
            first.push(capture);
            groups.push(RegExpNamedGroup {
                name: capture.name.clone(),
                capture_ids: vec![capture.id],
            });
        }
    }
    Ok(groups)
}

fn duplicate_names_diverge(left: &NamedCapture, right: &NamedCapture) -> bool {
    left.path.iter().any(|(choice, arm)| {
        right
            .path
            .iter()
            .any(|(other_choice, other_arm)| choice == other_choice && arm != other_arm)
    })
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    let byte = byte.to_ascii_lowercase();
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a' + 10)),
        _ => None,
    }
}

/// Which `RegExpIdentifierName` grammar position is being classified.
///
/// Keeping the two Unicode property domains closed prevents a new call site
/// from selecting `ID_Start` or `ID_Continue` through a stringly regular
/// expression. The pinned ICU property tables are the semantic data source;
/// the separate `regress` dependency remains only in a shape-limited static
/// generator fold outside the RegExp parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegExpIdentifierPosition {
    Start,
    Continue,
}

impl RegExpIdentifierPosition {
    fn after(prefix: &str) -> Self {
        if prefix.is_empty() {
            Self::Start
        } else {
            Self::Continue
        }
    }

    fn accepts(self, code_point: char) -> bool {
        match self {
            Self::Start => {
                matches!(code_point, '$' | '_')
                    || CodePointSetData::new::<IdStart>().contains(code_point)
            }
            Self::Continue => {
                matches!(code_point, '$' | '_' | '\u{200C}' | '\u{200D}')
                    || CodePointSetData::new::<IdContinue>().contains(code_point)
            }
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Continue => "continuation",
        }
    }
}

fn parse_regexp_identifier_name(
    bytes: &[u8],
    start: usize,
    description: &'static str,
) -> Result<(String, usize), RegExpCompileError> {
    let mut name = String::new();
    let mut cursor = start;

    loop {
        let Some(&byte) = bytes.get(cursor) else {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::RegExpIdentifierName,
                cursor,
                format!("{description} identifier is unclosed"),
            ));
        };
        if byte == b'>' {
            if name.is_empty() {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::RegExpIdentifierName,
                    cursor,
                    format!("{description} identifier is empty"),
                ));
            }
            return Ok((name, cursor + 1));
        }

        let code_point_offset = cursor;
        let code_point = if byte == b'\\' {
            if bytes.get(cursor + 1) != Some(&b'u') {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::RegExpIdentifierName,
                    cursor,
                    format!("{description} contains a non-Unicode identifier escape"),
                ));
            }

            if bytes.get(cursor + 2) == Some(&b'{') {
                let digits_start = cursor + 3;
                let mut value = 0_u32;
                cursor = digits_start;
                if bytes.get(cursor) == Some(&b'}') {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::RegExpIdentifierName,
                        cursor,
                        format!("{description} contains an empty Unicode identifier escape"),
                    ));
                }
                loop {
                    let Some(&digit) = bytes.get(cursor) else {
                        return Err(RegExpCompileError::invalid_syntax(
                            SyntaxRule::RegExpIdentifierName,
                            cursor,
                            format!("{description} contains an unclosed Unicode identifier escape"),
                        ));
                    };
                    if digit == b'}' {
                        break;
                    }
                    let Some(digit) = ascii_hex_value(digit) else {
                        return Err(RegExpCompileError::invalid_syntax(
                            SyntaxRule::RegExpIdentifierName,
                            cursor,
                            format!("{description} contains a malformed Unicode identifier escape"),
                        ));
                    };
                    value = value
                        .checked_mul(16)
                        .and_then(|value| value.checked_add(digit))
                        .ok_or_else(|| {
                            RegExpCompileError::invalid_syntax(
                                SyntaxRule::RegExpIdentifierName,
                                code_point_offset,
                                format!("{description} Unicode identifier escape is out of range"),
                            )
                        })?;
                    cursor += 1;
                }
                cursor += 1;
                char::from_u32(value).ok_or_else(|| {
                    RegExpCompileError::invalid_syntax(
                        SyntaxRule::RegExpIdentifierName,
                        code_point_offset,
                        format!("{description} Unicode identifier escape is not a scalar value"),
                    )
                })?
            } else {
                let digits = bytes.get(cursor + 2..cursor + 6).ok_or_else(|| {
                    RegExpCompileError::invalid_syntax(
                        SyntaxRule::RegExpIdentifierName,
                        bytes.len(),
                        format!("{description} contains an incomplete Unicode identifier escape"),
                    )
                })?;
                let invalid_digit = digits.iter().position(|digit| !digit.is_ascii_hexdigit());
                if let Some(invalid_digit) = invalid_digit {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::RegExpIdentifierName,
                        cursor + 2 + invalid_digit,
                        format!("{description} contains a malformed Unicode identifier escape"),
                    ));
                }
                let high = digits.iter().fold(0_u16, |value, digit| {
                    (value << 4) | ascii_hex_value(*digit).expect("hex digit") as u16
                });
                cursor += 6;

                if (0xD800..=0xDBFF).contains(&high) {
                    let low_digits = bytes.get(cursor + 2..cursor + 6).filter(|digits| {
                        bytes.get(cursor..cursor + 2) == Some(b"\\u")
                            && digits.iter().all(u8::is_ascii_hexdigit)
                    });
                    if let Some(low_digits) = low_digits {
                        let low = low_digits.iter().fold(0_u16, |value, digit| {
                            (value << 4) | ascii_hex_value(*digit).expect("hex digit") as u16
                        });
                        if (0xDC00..=0xDFFF).contains(&low) {
                            cursor += 6;
                            let scalar = 0x1_0000
                                + ((u32::from(high) - 0xD800) << 10)
                                + (u32::from(low) - 0xDC00);
                            char::from_u32(scalar).expect("paired surrogates form a scalar")
                        } else {
                            return Err(RegExpCompileError::invalid_syntax(
                                SyntaxRule::RegExpIdentifierName,
                                code_point_offset,
                                format!("{description} contains an unpaired lead surrogate escape"),
                            ));
                        }
                    } else {
                        return Err(RegExpCompileError::invalid_syntax(
                            SyntaxRule::RegExpIdentifierName,
                            code_point_offset,
                            format!("{description} contains an unpaired lead surrogate escape"),
                        ));
                    }
                } else if (0xDC00..=0xDFFF).contains(&high) {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::RegExpIdentifierName,
                        code_point_offset,
                        format!("{description} contains an unpaired trail surrogate escape"),
                    ));
                } else {
                    char::from_u32(u32::from(high)).expect("non-surrogate u16 is a scalar")
                }
            }
        } else {
            let source = std::str::from_utf8(&bytes[cursor..]).map_err(|_| {
                RegExpCompileError::unsupported_feature(cursor, NON_BOUNDARY_SOURCE)
            })?;
            let code_point = source.chars().next().expect("source is non-empty");
            cursor += code_point.len_utf8();
            code_point
        };

        let position = RegExpIdentifierPosition::after(&name);
        if !position.accepts(code_point) {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::RegExpIdentifierName,
                code_point_offset,
                format!(
                    "{description} code point U+{:04X} is not valid in identifier {position}",
                    u32::from(code_point),
                    position = position.description(),
                ),
            ));
        }
        name.push(code_point);
    }
}

/// One of the eight flag letters 22.2.3.1 accepts. Closed domain.
///
/// Introduced to delete a dead arm rather than to abstract anything.
/// [`parse_flags`] used to narrow `byte` to these eight in one `match` and then
/// re-`match` the same byte, and that second match carried a
/// `byte if first_unsupported.is_none()` arm plus a trailing
/// `if let Some(..) = first_unsupported { unsupported_feature(..) }`. Neither
/// could ever run: every byte outside the eight has already returned
/// `InvalidSyntax` from the first match. It compiled, it formatted cleanly and
/// it produced no dead-code warning — exactly the pattern AGENTS.md calls out
/// ("if something is unreachable from the product path, that should fail to
/// build, not merely fail to run"). Parsing the byte into this enum once makes
/// the second match exhaustive over a closed set with no catch-all, so the
/// unreachable state stops being expressible instead of being commented as
/// impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlagLetter {
    HasIndices,
    Global,
    IgnoreCase,
    Multiline,
    DotAll,
    Unicode,
    UnicodeSets,
    Sticky,
}

impl FlagLetter {
    /// `None` for any code unit outside `dgimsuvy`, which 22.2.3.1 makes a
    /// `SyntaxError`.
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'd' => Some(FlagLetter::HasIndices),
            b'g' => Some(FlagLetter::Global),
            b'i' => Some(FlagLetter::IgnoreCase),
            b'm' => Some(FlagLetter::Multiline),
            b's' => Some(FlagLetter::DotAll),
            b'u' => Some(FlagLetter::Unicode),
            b'v' => Some(FlagLetter::UnicodeSets),
            b'y' => Some(FlagLetter::Sticky),
            _ => None,
        }
    }

    /// The bit this letter occupies in the seen-flags set.
    fn bit(self) -> u8 {
        match self {
            FlagLetter::HasIndices => 1 << 0,
            FlagLetter::Global => 1 << 1,
            FlagLetter::IgnoreCase => 1 << 2,
            FlagLetter::Multiline => 1 << 3,
            FlagLetter::DotAll => 1 << 4,
            FlagLetter::Unicode => 1 << 5,
            FlagLetter::UnicodeSets => 1 << 6,
            FlagLetter::Sticky => 1 << 7,
        }
    }

    /// Records this letter on `flags`. Exhaustive, no catch-all.
    fn apply(self, flags: &mut RegExpFlags) {
        match self {
            FlagLetter::HasIndices => flags.has_indices = true,
            FlagLetter::Global => flags.global = true,
            FlagLetter::IgnoreCase => flags.ignore_case = true,
            FlagLetter::Multiline => flags.multiline = true,
            FlagLetter::DotAll => flags.dot_all = true,
            FlagLetter::Unicode => flags.unicode_mode = RegExpUnicodeMode::Unicode,
            FlagLetter::UnicodeSets => flags.unicode_mode = RegExpUnicodeMode::UnicodeSets,
            FlagLetter::Sticky => flags.sticky = true,
        }
    }
}

fn parse_flags(flags: &str) -> Result<RegExpFlags, RegExpCompileError> {
    let mut parsed = RegExpFlags::default();
    let mut seen_flags = 0_u8;
    for (offset, byte) in flags.bytes().enumerate() {
        if !byte.is_ascii() {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::Flags,
                offset,
                "regular-expression flags must be ASCII",
            ));
        }

        let Some(letter) = FlagLetter::from_byte(byte) else {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::Flags,
                offset,
                format!("unknown regular-expression flag `{}`", byte as char),
            ));
        };
        if seen_flags & letter.bit() != 0 {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::Flags,
                offset,
                format!("duplicate regular-expression flag `{}`", byte as char),
            ));
        }
        if (letter == FlagLetter::Unicode && seen_flags & FlagLetter::UnicodeSets.bit() != 0)
            || (letter == FlagLetter::UnicodeSets && seen_flags & FlagLetter::Unicode.bit() != 0)
        {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::Flags,
                offset,
                "regular-expression flags `u` and `v` are mutually exclusive",
            ));
        }
        seen_flags |= letter.bit();
        letter.apply(&mut parsed);
    }

    Ok(parsed)
}

/// Rewrites one freshly parsed instruction for the enclosing modifier state.
///
/// Case-insensitivity is folded into ASCII literals and classes here so that
/// RegExp modifier groups (`(?i:…)`, `(?-i:…)`) scope correctly. Multiline and
/// dotAll cannot be folded into an instruction, so they are recorded in
/// `operand0`: `0` defers to the runtime flag, `1` forces the mode on and `2`
/// forces it off.
fn apply_modifiers(instruction: &mut RegExpInstruction, modifiers: Modifiers) {
    match instruction.opcode {
        REGEXP_OPCODE_DOT => {
            instruction.operand0 = match modifiers.dot_all {
                None => 0,
                Some(true) => 1,
                Some(false) => 2,
            };
            return;
        }
        REGEXP_OPCODE_ASSERT_START | REGEXP_OPCODE_ASSERT_END => {
            instruction.operand0 = match modifiers.multiline {
                None => 0,
                Some(true) => 1,
                Some(false) => 2,
            };
            return;
        }
        _ => {}
    }
    if !modifiers.ignore_case {
        return;
    }
    apply_ascii_ignore_case(std::slice::from_mut(instruction));
}

fn apply_ascii_ignore_case(instructions: &mut [RegExpInstruction]) {
    for instruction in instructions {
        if instruction.opcode == REGEXP_OPCODE_LITERAL_ASCII {
            let member = instruction.operand0 as u8;
            if member.is_ascii_alphabetic() {
                let mut bitmap_low = 0;
                let mut bitmap_high = 0;
                add_ascii_member(
                    &mut bitmap_low,
                    &mut bitmap_high,
                    member.to_ascii_lowercase(),
                );
                add_ascii_member(
                    &mut bitmap_low,
                    &mut bitmap_high,
                    member.to_ascii_uppercase(),
                );
                *instruction = RegExpInstruction::positive_ascii_class(bitmap_low, bitmap_high);
            }
            continue;
        }
        if !matches!(
            instruction.opcode,
            REGEXP_OPCODE_POSITIVE_ASCII_CLASS | REGEXP_OPCODE_NEGATIVE_ASCII_CLASS
        ) {
            continue;
        }
        for member in b'A'..=b'Z' {
            let lowercase = member.to_ascii_lowercase();
            let contains_uppercase = instruction.operand1 & (1_u64 << (member - 64)) != 0;
            let contains_lowercase = instruction.operand1 & (1_u64 << (lowercase - 64)) != 0;
            if contains_uppercase || contains_lowercase {
                instruction.operand1 |= (1_u64 << (member - 64)) | (1_u64 << (lowercase - 64));
            }
        }
    }
}

fn parse_escaped_atom(
    bytes: &[u8],
    offset: &mut usize,
    unicode_mode: RegExpUnicodeMode,
    modifiers: Modifiers,
    pool: &mut RegExpRangePool,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let escape_offset = *offset;
    let unicode = unicode_mode.is_unicode_mode();
    let Some(&escaped) = bytes.get(escape_offset + 1) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::CharacterEscape,
            escape_offset,
            "regular-expression escape is missing its escaped character",
        ));
    };
    if !escaped.is_ascii() {
        return Err(RegExpCompileError::unsupported_feature(
            escape_offset + 1,
            "non-ASCII regular-expression source is unsupported by this matcher-program grammar",
        ));
    }
    if unicode && matches!(escaped, b'p' | b'P') {
        return parse_unicode_property_escape(bytes, offset, modifiers, pool);
    }
    if escaped == b'u' {
        // DEFECT 3, found while picking a witness for `SyntaxRule::CodePointEscape`
        // and the same fingerprint as DEFECT 1: `RegExpUnicodeEscapeSequence`
        // has two alternatives in Unicode mode,
        // ``u` Hex4Digits`` and ``u{` CodePoint `}``, and only the class parser
        // implemented the second (`parse_class_atom`'s `b'u'` arm, just below).
        // So `/[\u{41}]/u` compiled while `/\u{41}/u` — and therefore every
        // astral pattern written the ordinary way, `/\u{1F600}/u` — was refused
        // as a SyntaxError. `parse_unicode_escape` sees the four bytes `{41}`,
        // finds them not all hex digits, and in Unicode mode there is no
        // fallback to the Annex B identity escape.
        if unicode && bytes.get(escape_offset + 2) == Some(&b'{') {
            let (code_point, end) = parse_braced_code_point_escape(bytes, escape_offset)?;
            *offset = end;
            return Ok(if code_point <= 0x7f {
                RegExpInstruction::literal_ascii(code_point as u8)
            } else {
                RegExpInstruction::literal_code_point(code_point)
            });
        }
        let (code_unit, consumed) = match parse_unicode_escape(bytes, escape_offset) {
            Ok(parsed) => parsed,
            Err(_) if !unicode => {
                *offset += 2;
                return Ok(RegExpInstruction::literal_ascii(b'u'));
            }
            Err(error) => return Err(error),
        };
        *offset = consumed;
        if unicode && (0xD800..=0xDBFF).contains(&code_unit) {
            if bytes.get(consumed..consumed + 2) == Some(b"\\u") {
                if let Ok((low, end)) = parse_unicode_escape(bytes, consumed) {
                    if (0xDC00..=0xDFFF).contains(&low) {
                        let scalar = 0x1_0000
                            + (((code_unit as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
                        *offset = end;
                        return Ok(RegExpInstruction::literal_code_point(scalar));
                    }
                }
            }
        }
        return Ok(if code_unit <= 0x7f {
            RegExpInstruction::literal_ascii(code_unit as u8)
        } else {
            RegExpInstruction::literal_code_point(code_unit as u32)
        });
    }
    if escaped == b'x' {
        let digits = bytes.get(escape_offset + 2..escape_offset + 4);
        if !digits.is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)) {
            if !unicode {
                *offset += 2;
                return Ok(RegExpInstruction::literal_ascii(b'x'));
            }
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::HexEscapeSequence,
                escape_offset,
                "malformed hexadecimal escape",
            ));
        }
        let digits = digits.unwrap();
        let value = digits.iter().fold(0_u8, |value, digit| {
            (value << 4) | ascii_hex_value(*digit).unwrap() as u8
        });
        *offset += 4;
        return Ok(if value.is_ascii() {
            RegExpInstruction::literal_ascii(value)
        } else {
            RegExpInstruction::literal_code_point(u32::from(value))
        });
    }
    if escaped == b'c'
        && matches!(
            bytes.get(escape_offset + 2),
            Some(b'a'..=b'z') | Some(b'A'..=b'Z')
        )
    {
        let control = bytes[escape_offset + 2].to_ascii_uppercase() % 32;
        *offset += 3;
        return Ok(RegExpInstruction::literal_ascii(control));
    }
    if matches!(escaped, b'n' | b'r' | b't' | b'v' | b'f') {
        let value = match escaped {
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 0x0b,
            b'f' => 0x0c,
            _ => unreachable!(),
        };
        *offset += 2;
        return Ok(RegExpInstruction::literal_ascii(value));
    }
    if !unicode && matches!(escaped, b'0'..=b'7') {
        let (value, end) = parse_legacy_octal_escape(bytes, escape_offset);
        *offset = end;
        return Ok(if value.is_ascii() {
            RegExpInstruction::literal_ascii(value)
        } else {
            RegExpInstruction::literal_code_point(u32::from(value))
        });
    }
    if matches!(escaped, b'd' | b'D') {
        let mut bitmap_low = 0;
        let mut bitmap_high = 0;
        add_ascii_range(&mut bitmap_low, &mut bitmap_high, b'0', b'9');
        *offset += 2;
        return Ok(if escaped == b'd' {
            RegExpInstruction::positive_ascii_class(bitmap_low, bitmap_high)
        } else {
            RegExpInstruction::negative_ascii_class(bitmap_low, bitmap_high)
        });
    }
    if matches!(escaped, b'w' | b'W') {
        let mut bitmap_low = 0;
        let mut bitmap_high = 0;
        add_ascii_range(&mut bitmap_low, &mut bitmap_high, b'A', b'Z');
        add_ascii_range(&mut bitmap_low, &mut bitmap_high, b'a', b'z');
        add_ascii_range(&mut bitmap_low, &mut bitmap_high, b'0', b'9');
        add_ascii_member(&mut bitmap_low, &mut bitmap_high, b'_');
        *offset += 2;
        return Ok(if escaped == b'w' {
            RegExpInstruction::positive_ascii_class(bitmap_low, bitmap_high)
        } else {
            RegExpInstruction::negative_ascii_class(bitmap_low, bitmap_high)
        });
    }
    if matches!(escaped, b's' | b'S') {
        *offset += 2;
        return Ok(if escaped == b's' {
            RegExpInstruction::whitespace()
        } else {
            RegExpInstruction::not_whitespace()
        });
    }
    // `IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/``. BOTH alternatives,
    // which is the whole of batch 8's DEFECT 1: this tested SyntaxCharacter
    // alone (the predicate then called `is_regex_metacharacter`), so `/\//u` and
    // `/https?:\/\//u` — ordinary patterns, not corner cases — were rejected as
    // SyntaxErrors. The class path next door had the rule right
    // (`is_class_identity_escape` has always carried `/`), so `/[\/]/u` compiled
    // while `/\//u` did not. That divergence is the fingerprint of one predicate
    // serving two productions.
    if !is_unicode_identity_escape(escaped) {
        if unicode {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::IdentityEscape,
                escape_offset,
                format!(
                    "invalid regular-expression identity escape `\\{}`",
                    escaped as char
                ),
            ));
        }
        *offset += 2;
        return Ok(RegExpInstruction::literal_ascii(escaped));
    }

    *offset += 2;
    Ok(RegExpInstruction::literal_ascii(escaped))
}

/// Parses `\p{…}` / `\P{…}` and yields the matching code-point ranges.
///
/// `\P` is complemented here rather than through the instruction's negation
/// bit, because case closure applies after every set operation in
/// 22.2.2.7.1 CharacterSetMatcher.
fn parse_unicode_property_ranges(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<Vec<(u32, u32)>, RegExpCompileError> {
    let escape_offset = *offset;
    let complement = bytes[escape_offset + 1] == b'P';
    if bytes.get(escape_offset + 2) != Some(&b'{') {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodePropertyEscape,
            escape_offset,
            "malformed Unicode property escape",
        ));
    }
    let value_start = escape_offset + 3;
    let Some(relative_end) = bytes[value_start..].iter().position(|byte| *byte == b'}') else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodePropertyEscape,
            escape_offset,
            "malformed Unicode property escape",
        ));
    };
    let value_end = value_start + relative_end;
    let value = std::str::from_utf8(&bytes[value_start..value_end])
        .map_err(|_| RegExpCompileError::unsupported_feature(escape_offset, NON_BOUNDARY_SOURCE))?;
    if value.is_empty() {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodePropertyEscape,
            escape_offset,
            "malformed Unicode property escape",
        ));
    }

    let Some(ranges) = unicode_property_ranges(value) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodePropertyName,
            escape_offset,
            format!("unknown Unicode property escape `{value}`"),
        ));
    };
    *offset = value_end + 1;
    let ranges = normalize_ranges(ranges);
    Ok(if complement {
        complement_ranges(&ranges)
    } else {
        ranges
    })
}

fn parse_unicode_property_escape(
    bytes: &[u8],
    offset: &mut usize,
    modifiers: Modifiers,
    pool: &mut RegExpRangePool,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let escape_offset = *offset;
    let ranges = parse_unicode_property_ranges(bytes, offset)?;
    finish_range_set(ranges, false, modifiers.ignore_case, pool, escape_offset)
}

/// Resolves an ECMA-262 `UnicodePropertyValueExpression` to code-point ranges.
fn unicode_property_ranges(value: &str) -> Option<Vec<(u32, u32)>> {
    let collect = |ranges: &mut dyn Iterator<Item = std::ops::RangeInclusive<u32>>| {
        ranges
            .map(|range| (*range.start(), *range.end()))
            .collect::<Vec<_>>()
    };
    match value.split_once('=') {
        Some((name, property_value)) => match name {
            "General_Category" | "gc" => general_category_ranges(property_value),
            "Script" | "sc" => script_ranges(property_value, false),
            "Script_Extensions" | "scx" => script_ranges(property_value, true),
            _ => None,
        },
        None => {
            match value {
                "Any" => return Some(vec![(0, 0x10ffff)]),
                "ASCII" => return Some(vec![(0, 0x7f)]),
                "Assigned" => {
                    let unassigned = general_category_ranges("Unassigned")?;
                    return Some(complement_ranges(&normalize_ranges(unassigned)));
                }
                _ => {}
            }
            if let Some(set) = CodePointSetData::new_for_ecma262(value.as_bytes()) {
                return Some(collect(&mut set.iter_ranges()));
            }
            general_category_ranges(value)
        }
    }
}

fn general_category_ranges(value: &str) -> Option<Vec<(u32, u32)>> {
    let group = PropertyParser::<GeneralCategoryGroup>::new().get_strict(value)?;
    Some(
        CodePointMapData::<GeneralCategory>::new()
            .iter_ranges_for_group(group)
            .map(|range| (*range.start(), *range.end()))
            .collect(),
    )
}

fn script_ranges(value: &str, extensions: bool) -> Option<Vec<(u32, u32)>> {
    let script = PropertyParser::<Script>::new().get_strict(value)?;
    if extensions {
        Some(
            ScriptWithExtensions::new()
                .get_script_extensions_ranges(script)
                .map(|range| (*range.start(), *range.end()))
                .collect(),
        )
    } else {
        Some(
            CodePointMapData::<Script>::new()
                .iter_ranges_for_value(script)
                .map(|range| (*range.start(), *range.end()))
                .collect(),
        )
    }
}

/// Groups every code point by its simple case-folding key, keeping only the
/// classes that contain more than one member.
fn case_fold_classes() -> &'static [Vec<u32>] {
    static CLASSES: OnceLock<Vec<Vec<u32>>> = OnceLock::new();
    CLASSES.get_or_init(|| {
        fn simple_lowercase(character: char) -> char {
            let mut mapped = character.to_lowercase();
            let first = mapped.next().expect("lowercase mapping is never empty");
            if mapped.next().is_some() {
                character
            } else {
                first
            }
        }
        fn simple_uppercase(character: char) -> char {
            let mut mapped = character.to_uppercase();
            let first = mapped.next().expect("uppercase mapping is never empty");
            if mapped.next().is_some() {
                character
            } else {
                first
            }
        }

        let mut groups: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for character in (0..=char::MAX as u32).filter_map(char::from_u32) {
            let key = simple_lowercase(simple_uppercase(character));
            if key == character
                && simple_lowercase(character) == character
                && simple_uppercase(character) == character
            {
                continue;
            }
            groups.entry(key as u32).or_default().push(character as u32);
        }
        groups
            .into_values()
            .filter(|members| members.len() > 1)
            .collect()
    })
}

/// Closes `ranges` under simple case folding, matching the effect of
/// canonicalizing both the input and the set members.
fn case_close_ranges(ranges: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut extra = Vec::new();
    for members in case_fold_classes() {
        if members
            .iter()
            .any(|code_point| ranges_contain(ranges, *code_point))
        {
            extra.extend(
                members
                    .iter()
                    .map(|code_point| (*code_point, *code_point))
                    .filter(|(code_point, _)| !ranges_contain(ranges, *code_point)),
            );
        }
    }
    if extra.is_empty() {
        return ranges.to_vec();
    }
    let mut closed = ranges.to_vec();
    closed.append(&mut extra);
    normalize_ranges(closed)
}

fn parse_unicode_escape(bytes: &[u8], start: usize) -> Result<(u16, usize), RegExpCompileError> {
    if bytes.get(start..start + 2) != Some(b"\\u") {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodeEscapeSequence,
            start,
            "malformed Unicode escape",
        ));
    }
    let digits = bytes.get(start + 2..start + 6).ok_or_else(|| {
        RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodeEscapeSequence,
            start,
            "malformed Unicode escape",
        )
    })?;
    if digits.len() != 4 || !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnicodeEscapeSequence,
            start,
            "malformed Unicode escape",
        ));
    }
    let value = digits.iter().fold(0_u16, |value, digit| {
        let digit = digit.to_ascii_lowercase();
        let digit = match digit {
            b'0'..=b'9' => digit - b'0',
            b'a'..=b'f' => digit - b'a' + 10,
            _ => unreachable!(),
        };
        (value << 4) | digit as u16
    });
    Ok((value, start + 6))
}

/// ECMAScript `WhiteSpace` plus `LineTerminator`, i.e. the `\s` class.
const REGEXP_WHITESPACE_RANGES: &[(u32, u32)] = &[
    (0x0009, 0x000d),
    (0x0020, 0x0020),
    (0x00a0, 0x00a0),
    (0x1680, 0x1680),
    (0x2000, 0x200a),
    (0x2028, 0x2029),
    (0x202f, 0x202f),
    (0x205f, 0x205f),
    (0x3000, 0x3000),
    (0xfeff, 0xfeff),
];

const REGEXP_DIGIT_RANGES: &[(u32, u32)] = &[(0x30, 0x39)];

const REGEXP_WORD_RANGES: &[(u32, u32)] = &[(0x30, 0x39), (0x41, 0x5a), (0x5f, 0x5f), (0x61, 0x7a)];

/// One member of a character class in the general code-point representation.
enum ClassAtom {
    CodePoint(u32),
    Ranges(Vec<(u32, u32)>),
}

impl ClassAtom {
    fn into_ranges(self) -> Vec<(u32, u32)> {
        match self {
            Self::CodePoint(code_point) => vec![(code_point, code_point)],
            Self::Ranges(ranges) => ranges,
        }
    }
}

/// Returns whether the class starting at `offset` holds anything the ASCII
/// bitmap representation cannot express.
fn class_needs_code_point_ranges(bytes: &[u8], offset: usize) -> bool {
    let mut cursor = offset + 1;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b']' => return false,
            b'\\' => {
                // `\w`, `\W`, `\D` and `\S` are CharacterClassEscapes that the
                // ASCII bitmap atom parser does not model, and the negated
                // three cover code points outside the bitmap's 0..=0x7f domain
                // anyway. Send any class containing them down the code-point
                // range path, which expands them through `complement_ranges`.
                if matches!(
                    bytes.get(cursor + 1),
                    Some(b'p' | b'P' | b'u' | b'x' | b'w' | b'W' | b'D' | b'S')
                ) {
                    return true;
                }
                cursor += 2;
            }
            byte if !byte.is_ascii() => return true,
            _ => cursor += 1,
        }
    }
    false
}

fn parse_class(
    bytes: &[u8],
    offset: &mut usize,
    mode: RegExpUnicodeMode,
    modifiers: Modifiers,
    pool: &mut RegExpRangePool,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let unicode = mode.is_unicode_mode();
    if !class_needs_code_point_ranges(bytes, *offset) {
        return parse_ascii_class(bytes, offset);
    }
    let class_offset = *offset;
    let mut cursor = class_offset + 1;
    let negated = bytes.get(cursor) == Some(&b'^');
    cursor += usize::from(negated);

    let mut ranges = Vec::new();
    loop {
        let Some(&member) = bytes.get(cursor) else {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::UnclosedCharacterClass,
                class_offset,
                "regular-expression character class is unclosed",
            ));
        };
        if member == b']' {
            break;
        }
        let range_offset = cursor;
        let start = parse_class_atom(bytes, &mut cursor, mode)?;
        if bytes.get(cursor) == Some(&b'-') && bytes.get(cursor + 1) != Some(&b']') {
            cursor += 1;
            if bytes.get(cursor).is_none() {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::UnclosedCharacterClass,
                    class_offset,
                    "regular-expression character class is unclosed",
                ));
            }
            let end = parse_class_atom(bytes, &mut cursor, mode)?;
            match (start, end) {
                (ClassAtom::CodePoint(start), ClassAtom::CodePoint(end)) if end < start => {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::ClassRangeOrder,
                        range_offset,
                        "regular-expression character class range is reversed",
                    ));
                }
                (ClassAtom::CodePoint(start), ClassAtom::CodePoint(end)) => {
                    ranges.push((start, end));
                }
                (start, end) if unicode => {
                    let _ = (start, end);
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::ClassRangeBound,
                        range_offset,
                        "regular-expression character class range bound is a class escape",
                    ));
                }
                (start, end) => {
                    ranges.extend(start.into_ranges());
                    ranges.extend(end.into_ranges());
                    ranges.push((u32::from(b'-'), u32::from(b'-')));
                }
            }
        } else {
            ranges.extend(start.into_ranges());
        }
    }

    *offset = cursor + 1;
    finish_range_set(ranges, negated, modifiers.ignore_case, pool, class_offset)
}

/// Normalizes, case-closes and interns `ranges`, returning a range-set atom.
fn finish_range_set(
    ranges: Vec<(u32, u32)>,
    negated: bool,
    ignore_case: bool,
    pool: &mut RegExpRangePool,
    offset: usize,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let mut ranges = normalize_ranges(ranges);
    if ignore_case {
        ranges = case_close_ranges(&ranges);
    }
    let (first_entry, entry_count) = pool.intern(&ranges, offset)?;
    Ok(RegExpInstruction::code_point_range_set(
        first_entry,
        entry_count,
        negated,
    ))
}

fn parse_class_atom(
    bytes: &[u8],
    cursor: &mut usize,
    mode: RegExpUnicodeMode,
) -> Result<ClassAtom, RegExpCompileError> {
    let unicode = mode.is_unicode_mode();
    let offset = *cursor;
    let Some(&member) = bytes.get(offset) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnclosedCharacterClass,
            offset,
            "regular-expression character class is unclosed",
        ));
    };
    if member != b'\\' {
        let source = std::str::from_utf8(&bytes[offset..])
            .map_err(|_| RegExpCompileError::unsupported_feature(offset, NON_BOUNDARY_SOURCE))?;
        let character = source.chars().next().expect("non-empty class source");
        *cursor += character.len_utf8();
        return Ok(ClassAtom::CodePoint(character as u32));
    }

    let Some(&escaped) = bytes.get(offset + 1) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::CharacterEscape,
            offset,
            "regular-expression escape is missing its escaped character",
        ));
    };
    match escaped {
        b'd' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(REGEXP_DIGIT_RANGES.to_vec()))
        }
        b'D' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(complement_ranges(REGEXP_DIGIT_RANGES)))
        }
        b's' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(REGEXP_WHITESPACE_RANGES.to_vec()))
        }
        b'S' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(complement_ranges(
                REGEXP_WHITESPACE_RANGES,
            )))
        }
        b'w' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(REGEXP_WORD_RANGES.to_vec()))
        }
        b'W' => {
            *cursor += 2;
            Ok(ClassAtom::Ranges(complement_ranges(REGEXP_WORD_RANGES)))
        }
        b'p' | b'P' if unicode => {
            let ranges = parse_unicode_property_ranges(bytes, cursor)?;
            Ok(ClassAtom::Ranges(ranges))
        }
        b'b' => {
            *cursor += 2;
            Ok(ClassAtom::CodePoint(0x08))
        }
        b'n' | b'r' | b't' | b'v' | b'f' => {
            let value = match escaped {
                b'n' => 0x0a,
                b'r' => 0x0d,
                b't' => 0x09,
                b'v' => 0x0b,
                _ => 0x0c,
            };
            *cursor += 2;
            Ok(ClassAtom::CodePoint(value))
        }
        b'c' if matches!(bytes.get(offset + 2), Some(b'a'..=b'z') | Some(b'A'..=b'Z')) => {
            let control = u32::from(bytes[offset + 2].to_ascii_uppercase() % 32);
            *cursor += 3;
            Ok(ClassAtom::CodePoint(control))
        }
        b'c' if !unicode && matches!(bytes.get(offset + 2), Some(b'0'..=b'9') | Some(b'_')) => {
            let control = u32::from(bytes[offset + 2] % 32);
            *cursor += 3;
            Ok(ClassAtom::CodePoint(control))
        }
        b'x' => {
            let digits = bytes.get(offset + 2..offset + 4);
            if let Some(digits) = digits.filter(|digits| digits.iter().all(u8::is_ascii_hexdigit)) {
                let value = digits.iter().fold(0_u32, |value, digit| {
                    (value << 4) | ascii_hex_value(*digit).unwrap_or(0)
                });
                *cursor += 4;
                return Ok(ClassAtom::CodePoint(value));
            }
            if unicode {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::HexEscapeSequence,
                    offset,
                    "malformed hexadecimal escape",
                ));
            }
            *cursor += 2;
            Ok(ClassAtom::CodePoint(u32::from(b'x')))
        }
        b'u' => {
            if unicode && bytes.get(offset + 2) == Some(&b'{') {
                let (value, end) = parse_braced_code_point_escape(bytes, offset)?;
                *cursor = end;
                return Ok(ClassAtom::CodePoint(value));
            }
            match parse_unicode_escape(bytes, offset) {
                Ok((code_unit, end)) => {
                    *cursor = end;
                    if unicode && (0xd800..=0xdbff).contains(&code_unit) {
                        if let Ok((low, low_end)) = parse_unicode_escape(bytes, end) {
                            if (0xdc00..=0xdfff).contains(&low) {
                                *cursor = low_end;
                                return Ok(ClassAtom::CodePoint(
                                    0x1_0000
                                        + (((u32::from(code_unit) - 0xd800) << 10)
                                            | (u32::from(low) - 0xdc00)),
                                ));
                            }
                        }
                    }
                    Ok(ClassAtom::CodePoint(u32::from(code_unit)))
                }
                Err(error) if unicode => Err(error),
                Err(_) => {
                    *cursor += 2;
                    Ok(ClassAtom::CodePoint(u32::from(b'u')))
                }
            }
        }
        b'0'..=b'7' if !unicode => {
            let (value, end) = parse_legacy_octal_escape(bytes, offset);
            *cursor = end;
            Ok(ClassAtom::CodePoint(u32::from(value)))
        }
        b'0' => {
            *cursor += 2;
            Ok(ClassAtom::CodePoint(0))
        }
        // DEFECT 2 lived here, in the argument rather than in the predicate:
        // `parse_class_set` passed a literal `true` for what was then a
        // `unicode: bool` parameter, so every `v`-mode class atom was checked
        // against the `u`-mode `ClassEscape` rule and the thirteen additional
        // `ClassSetReservedPunctuator` escapes (`[\&]`, `[\!]`, `[\#]`, `[\%]`,
        // `[\,]`, `[\:]`, `[\;]`, `[\<]`, `[\=]`, `[\>]`, `[\@]`, ``[\`]``,
        // `[\~]`) were rejected as SyntaxErrors. The parameter is a
        // `RegExpUnicodeMode` now, so a literal `true` does not compile and the
        // third mode cannot silently reuse the second's rule again.
        escaped if unicode && !mode.allows_class_identity_escape(escaped) => {
            Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::ClassEscape,
                offset,
                "invalid regular-expression class escape",
            ))
        }
        _ => {
            let source = std::str::from_utf8(&bytes[offset + 1..]).map_err(|_| {
                RegExpCompileError::unsupported_feature(offset, NON_BOUNDARY_SOURCE)
            })?;
            let character = source.chars().next().expect("non-empty escape source");
            *cursor = offset + 1 + character.len_utf8();
            Ok(ClassAtom::CodePoint(character as u32))
        }
    }
}

/// Parses a `v`-mode `ClassSetExpression`, including nested classes, `--`
/// difference and `&&` intersection.
fn parse_unicode_sets_class(
    bytes: &[u8],
    offset: &mut usize,
    modifiers: Modifiers,
    pool: &mut RegExpRangePool,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let class_offset = *offset;
    let mut cursor = class_offset;
    let (ranges, negated) = parse_class_set(bytes, &mut cursor)?;
    *offset = cursor;
    finish_range_set(ranges, negated, modifiers.ignore_case, pool, class_offset)
}

fn parse_class_set(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<(Vec<(u32, u32)>, bool), RegExpCompileError> {
    let class_offset = *cursor;
    debug_assert_eq!(bytes.get(class_offset), Some(&b'['));
    *cursor += 1;
    let negated = bytes.get(*cursor) == Some(&b'^');
    *cursor += usize::from(negated);

    let mut accumulated: Option<Vec<(u32, u32)>> = None;
    let mut operator: Option<&'static str> = None;
    let mut union: Vec<(u32, u32)> = Vec::new();

    loop {
        match bytes.get(*cursor).copied() {
            None => {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::UnclosedCharacterClass,
                    class_offset,
                    "regular-expression character class is unclosed",
                ));
            }
            Some(b']') => {
                *cursor += 1;
                break;
            }
            Some(b'-') if bytes.get(*cursor + 1) == Some(&b'-') => {
                *cursor += 2;
                accumulated = Some(match (accumulated, operator) {
                    (None, _) => normalize_ranges(std::mem::take(&mut union)),
                    (Some(left), Some("--")) => {
                        subtract_ranges(&left, &normalize_ranges(std::mem::take(&mut union)))
                    }
                    (Some(left), _) => {
                        intersect_ranges(&left, &normalize_ranges(std::mem::take(&mut union)))
                    }
                });
                operator = Some("--");
            }
            Some(b'&') if bytes.get(*cursor + 1) == Some(&b'&') => {
                *cursor += 2;
                accumulated = Some(match (accumulated, operator) {
                    (None, _) => normalize_ranges(std::mem::take(&mut union)),
                    (Some(left), Some("--")) => {
                        subtract_ranges(&left, &normalize_ranges(std::mem::take(&mut union)))
                    }
                    (Some(left), _) => {
                        intersect_ranges(&left, &normalize_ranges(std::mem::take(&mut union)))
                    }
                });
                operator = Some("&&");
            }
            Some(b'[') => {
                let (nested, nested_negated) = parse_class_set(bytes, cursor)?;
                let nested = normalize_ranges(nested);
                union.extend(if nested_negated {
                    complement_ranges(&nested)
                } else {
                    nested
                });
            }
            Some(b'\\') if bytes.get(*cursor + 1) == Some(&b'q') => {
                return Err(RegExpCompileError::unsupported_feature(
                    *cursor,
                    "`\\q` string literals are unsupported by this matcher-program grammar",
                ));
            }
            Some(_) => {
                let range_offset = *cursor;
                let start = parse_class_atom(bytes, cursor, RegExpUnicodeMode::UnicodeSets)?;
                if bytes.get(*cursor) == Some(&b'-')
                    && bytes.get(*cursor + 1) != Some(&b']')
                    && bytes.get(*cursor + 1) != Some(&b'-')
                {
                    *cursor += 1;
                    let end = parse_class_atom(bytes, cursor, RegExpUnicodeMode::UnicodeSets)?;
                    match (start, end) {
                        (ClassAtom::CodePoint(start), ClassAtom::CodePoint(end)) if end < start => {
                            return Err(RegExpCompileError::invalid_syntax(
                                SyntaxRule::ClassRangeOrder,
                                range_offset,
                                "regular-expression character class range is reversed",
                            ));
                        }
                        (ClassAtom::CodePoint(start), ClassAtom::CodePoint(end)) => {
                            union.push((start, end));
                        }
                        _ => {
                            return Err(RegExpCompileError::invalid_syntax(
                                SyntaxRule::ClassRangeBound,
                                range_offset,
                                "regular-expression character class range bound is a class escape",
                            ));
                        }
                    }
                } else {
                    union.extend(start.into_ranges());
                }
            }
        }
    }

    let union = normalize_ranges(union);
    let ranges = match (accumulated, operator) {
        (None, _) => union,
        (Some(left), Some("--")) => subtract_ranges(&left, &union),
        (Some(left), _) => intersect_ranges(&left, &union),
    };
    Ok((ranges, negated))
}

/// `ClassEscape[+UnicodeMode]`'s identity-escape set: `SyntaxCharacter`, plus
/// `/` from `IdentityEscape`, plus `-` from `ClassEscape` itself.
///
/// This predicate has always been right, which is the useful part of the
/// evidence: `/[\/]/u` compiled while `/\//u` did not, and that divergence is
/// what identified DEFECT 1 as a missing alternative rather than a design
/// choice.
fn is_class_identity_escape(escaped: u8) -> bool {
    // Delegates rather than re-spelling `SyntaxCharacter | `/``. Written out
    // independently, `IdentityEscape[+UnicodeMode]` lived in two predicates and
    // the next correction to that production would have had to be made twice —
    // getting it right in only one of them is exactly the atom/class divergence
    // that made DEFECT 1 detectable. As delegation, this reads as the spec does:
    // the `u`-mode identity escape, plus `-` from `ClassEscape` itself.
    is_unicode_identity_escape(escaped) || escaped == b'-'
}

/// ``ClassSetReservedPunctuator :: one of & - ! # % , : ; < = > @ ` ~`` (22.2.1).
///
/// Fourteen characters. `-` is already a `u`-mode `ClassEscape` alternative, so
/// thirteen of them are new in `v` mode — and HEAD rejected all thirteen.
fn is_class_set_reserved_punctuator(byte: u8) -> bool {
    matches!(
        byte,
        b'&' | b'-'
            | b'!'
            | b'#'
            | b'%'
            | b','
            | b':'
            | b';'
            | b'<'
            | b'='
            | b'>'
            | b'@'
            | b'`'
            | b'~'
    )
}

fn parse_braced_code_point_escape(
    bytes: &[u8],
    escape_offset: usize,
) -> Result<(u32, usize), RegExpCompileError> {
    let mut cursor = escape_offset + 3;
    let mut value = 0_u32;
    let mut digits = 0;
    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'}' {
            break;
        }
        let Some(digit) = ascii_hex_value(byte) else {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::CodePointEscape,
                escape_offset,
                "malformed Unicode code-point escape",
            ));
        };
        value = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(digit))
            .filter(|value| *value <= 0x10ffff)
            .ok_or_else(|| {
                RegExpCompileError::invalid_syntax(
                    SyntaxRule::CodePointEscapeRange,
                    escape_offset,
                    "Unicode code-point escape is out of range",
                )
            })?;
        digits += 1;
        cursor += 1;
    }
    if digits == 0 || bytes.get(cursor) != Some(&b'}') {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::CodePointEscape,
            escape_offset,
            "malformed Unicode code-point escape",
        ));
    }
    Ok((value, cursor + 1))
}

#[derive(Clone, Copy)]
struct AsciiClassAtom {
    bitmap_low: u64,
    bitmap_high: u64,
    singleton: Option<u8>,
}

fn parse_ascii_class(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let class_offset = *offset;
    let mut cursor = class_offset + 1;
    let Some(&first) = bytes.get(cursor) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnclosedCharacterClass,
            class_offset,
            "regular-expression character class is unclosed",
        ));
    };
    let negated = first == b'^';
    cursor += usize::from(negated);

    let mut bitmap_low = 0;
    let mut bitmap_high = 0;
    loop {
        let Some(&member) = bytes.get(cursor) else {
            return Err(RegExpCompileError::invalid_syntax(
                SyntaxRule::UnclosedCharacterClass,
                class_offset,
                "regular-expression character class is unclosed",
            ));
        };
        if member == b']' {
            break;
        }
        let range_start = parse_ascii_class_atom(bytes, &mut cursor)?;

        if bytes.get(cursor) == Some(&b'-') && bytes.get(cursor + 1) != Some(&b']') {
            let range_offset = cursor;
            cursor += 1;
            if bytes.get(cursor).is_none() {
                return Err(RegExpCompileError::invalid_syntax(
                    SyntaxRule::UnclosedCharacterClass,
                    class_offset,
                    "regular-expression character class is unclosed",
                ));
            }
            let range_end = parse_ascii_class_atom(bytes, &mut cursor)?;
            match (range_start.singleton, range_end.singleton) {
                (Some(start), Some(end)) if end < start => {
                    return Err(RegExpCompileError::invalid_syntax(
                        SyntaxRule::ClassRangeOrder,
                        range_offset,
                        "regular-expression character class range is reversed",
                    ));
                }
                (Some(start), Some(end)) => {
                    add_ascii_range(&mut bitmap_low, &mut bitmap_high, start, end);
                }
                _ => {
                    bitmap_low |= range_start.bitmap_low | range_end.bitmap_low;
                    bitmap_high |= range_start.bitmap_high | range_end.bitmap_high;
                    add_ascii_member(&mut bitmap_low, &mut bitmap_high, b'-');
                }
            }
        } else {
            bitmap_low |= range_start.bitmap_low;
            bitmap_high |= range_start.bitmap_high;
        }
    }

    *offset = cursor + 1;
    Ok(if negated {
        RegExpInstruction::negative_ascii_class(bitmap_low, bitmap_high)
    } else {
        RegExpInstruction::positive_ascii_class(bitmap_low, bitmap_high)
    })
}

fn parse_single_unicode_class(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<Option<RegExpInstruction>, RegExpCompileError> {
    let class_offset = *offset;
    let Some(relative_end) = bytes[class_offset + 1..]
        .iter()
        .position(|byte| *byte == b']')
    else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnclosedCharacterClass,
            class_offset,
            "regular-expression character class is unclosed",
        ));
    };
    let end = class_offset + 1 + relative_end;
    let source = std::str::from_utf8(&bytes[class_offset + 1..end])
        .map_err(|_| RegExpCompileError::unsupported_feature(class_offset, NON_BOUNDARY_SOURCE))?;
    let mut characters = source.chars();
    let Some(character) = characters.next() else {
        return Ok(None);
    };
    if character.is_ascii() || characters.next().is_some() {
        return Ok(None);
    }
    *offset = end + 1;
    Ok(Some(RegExpInstruction::literal_code_point(
        character as u32,
    )))
}

fn parse_ascii_class_atom(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<AsciiClassAtom, RegExpCompileError> {
    let offset = *cursor;
    let Some(&member) = bytes.get(offset) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::UnclosedCharacterClass,
            offset,
            "regular-expression character class is unclosed",
        ));
    };
    if member != b'\\' {
        if !member.is_ascii() {
            return Err(RegExpCompileError::unsupported_feature(
                offset,
                "non-ASCII regular-expression source is unsupported by this matcher-program grammar",
            ));
        }
        *cursor += 1;
        return Ok(singleton_ascii_class_atom(member));
    }

    let Some(&escaped) = bytes.get(offset + 1) else {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::CharacterEscape,
            offset,
            "regular-expression escape is missing its escaped character",
        ));
    };
    match escaped {
        b'd' => {
            *cursor += 2;
            let mut atom = AsciiClassAtom {
                bitmap_low: 0,
                bitmap_high: 0,
                singleton: None,
            };
            add_ascii_range(&mut atom.bitmap_low, &mut atom.bitmap_high, b'0', b'9');
            Ok(atom)
        }
        b's' => {
            *cursor += 2;
            let mut atom = AsciiClassAtom {
                bitmap_low: 0,
                bitmap_high: 0,
                singleton: None,
            };
            add_ascii_range(&mut atom.bitmap_low, &mut atom.bitmap_high, 0x09, 0x0d);
            add_ascii_member(&mut atom.bitmap_low, &mut atom.bitmap_high, 0x20);
            Ok(atom)
        }
        b'c' if matches!(bytes.get(offset + 2), Some(b'0'..=b'9') | Some(b'_')) => {
            let control = bytes[offset + 2] % 32;
            *cursor += 3;
            Ok(singleton_ascii_class_atom(control))
        }
        b'0'..=b'7' => {
            let (value, end) = parse_legacy_octal_escape(bytes, offset);
            *cursor = end;
            Ok(singleton_ascii_class_atom(value))
        }
        b'b' => {
            *cursor += 2;
            Ok(singleton_ascii_class_atom(0x08))
        }
        b'n' | b'r' | b't' | b'v' | b'f' => {
            let value = match escaped {
                b'n' => b'\n',
                b'r' => b'\r',
                b't' => b'\t',
                b'v' => 0x0b,
                b'f' => 0x0c,
                _ => unreachable!(),
            };
            *cursor += 2;
            Ok(singleton_ascii_class_atom(value))
        }
        _ => {
            *cursor += 2;
            Ok(singleton_ascii_class_atom(escaped))
        }
    }
}

fn singleton_ascii_class_atom(member: u8) -> AsciiClassAtom {
    let mut atom = AsciiClassAtom {
        bitmap_low: 0,
        bitmap_high: 0,
        singleton: Some(member),
    };
    add_ascii_member(&mut atom.bitmap_low, &mut atom.bitmap_high, member);
    atom
}

fn parse_legacy_octal_escape(bytes: &[u8], escape_offset: usize) -> (u8, usize) {
    let first = bytes[escape_offset + 1];
    let max_digits = if first <= b'3' { 3 } else { 2 };
    let mut value = 0_u8;
    let mut cursor = escape_offset + 1;
    for _ in 0..max_digits {
        let Some(digit @ b'0'..=b'7') = bytes.get(cursor).copied() else {
            break;
        };
        value = value * 8 + (digit - b'0');
        cursor += 1;
    }
    (value, cursor)
}

#[derive(Clone, Copy)]
struct Quantifier {
    min: usize,
    max: Option<usize>,
    lazy: bool,
}

fn parse_postfix_quantifier(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<Quantifier, RegExpCompileError> {
    let Some(&byte) = bytes.get(*offset) else {
        return Ok(Quantifier {
            min: 1,
            max: Some(1),
            lazy: false,
        });
    };
    let start = *offset;
    let (min, max) = match byte {
        b'?' => {
            *offset += 1;
            (0, Some(1))
        }
        b'*' => {
            *offset += 1;
            (0, None)
        }
        b'+' => {
            *offset += 1;
            (1, None)
        }
        b'{' => match parse_braced_quantifier(bytes, offset)? {
            Some(quantifier) => quantifier,
            None => {
                return Ok(Quantifier {
                    min: 1,
                    max: Some(1),
                    lazy: false,
                });
            }
        },
        _ => {
            return Ok(Quantifier {
                min: 1,
                max: Some(1),
                lazy: false,
            });
        }
    };
    let lazy = if bytes.get(*offset) == Some(&b'?') {
        *offset += 1;
        true
    } else {
        false
    };
    let repeated_brace = if bytes.get(*offset) == Some(&b'{') {
        let mut probe = *offset;
        match parse_braced_quantifier(bytes, &mut probe)? {
            Some(_) => true,
            None => false,
        }
    } else {
        false
    };
    if matches!(bytes.get(*offset), Some(b'?' | b'*' | b'+')) || repeated_brace {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::QuantifierAfterQuantifier,
            *offset,
            "regular-expression quantifier follows another quantifier",
        ));
    }
    debug_assert!(start < *offset);
    Ok(Quantifier { min, max, lazy })
}

fn parse_braced_quantifier(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<Option<(usize, Option<usize>)>, RegExpCompileError> {
    let start = *offset;
    let mut cursor = start + 1;
    let (min, min_overflow) = parse_decimal_checked(bytes, &mut cursor);
    if cursor == start + 1 {
        return Ok(None);
    }
    let (max, max_overflow) = match bytes.get(cursor) {
        Some(b'}') => {
            cursor += 1;
            (Some(min), false)
        }
        Some(b',') => {
            cursor += 1;
            if bytes.get(cursor) == Some(&b'}') {
                cursor += 1;
                (None, false)
            } else {
                let max_start = cursor;
                let (max, overflow) = parse_decimal_checked(bytes, &mut cursor);
                if cursor == max_start || bytes.get(cursor) != Some(&b'}') {
                    return Ok(None);
                }
                cursor += 1;
                (Some(max), overflow)
            }
        }
        _ => return Ok(None),
    };
    if min_overflow || max_overflow {
        return Err(RegExpCompileError::unsupported_feature(
            start,
            "regular-expression quantifier bound is too large",
        ));
    }
    if max.is_some_and(|max| max < min) {
        return Err(RegExpCompileError::invalid_syntax(
            SyntaxRule::QuantifierBounds,
            start,
            "regular-expression quantifier bounds are reversed",
        ));
    }
    *offset = cursor;
    Ok(Some((min, max)))
}

fn parse_decimal_checked(bytes: &[u8], offset: &mut usize) -> (usize, bool) {
    let first = *offset;
    let mut value = 0usize;
    let mut overflow = false;
    while let Some(byte @ b'0'..=b'9') = bytes.get(*offset).copied() {
        if let Some(next) = value
            .checked_mul(10)
            .and_then(|v| v.checked_add((byte - b'0') as usize))
        {
            value = next;
        } else {
            overflow = true;
        }
        *offset += 1;
    }
    (value, overflow || *offset == first)
}

struct ProgramLowerer<'a> {
    instructions: &'a mut Vec<RegExpInstruction>,
    error_offset: usize,
    named_groups: &'a [RegExpNamedGroup],
}

impl<'a> ProgramLowerer<'a> {
    fn new(
        instructions: &'a mut Vec<RegExpInstruction>,
        pattern_len: usize,
        named_groups: &'a [RegExpNamedGroup],
    ) -> Self {
        Self {
            instructions,
            error_offset: pattern_len,
            named_groups,
        }
    }

    fn push(&mut self, instruction: RegExpInstruction) -> Result<(), RegExpCompileError> {
        if self.instructions.len() >= REGEXP_MAX_INSTRUCTIONS {
            return Err(RegExpCompileError::unsupported_feature(
                self.error_offset,
                format!(
                    "regular-expression expands beyond the {REGEXP_MAX_INSTRUCTIONS}-instruction matcher-program limit"
                ),
            ));
        }
        self.instructions.push(instruction);
        Ok(())
    }

    fn alternatives(&mut self, alternatives: &[Vec<ParsedTerm>]) -> Result<(), RegExpCompileError> {
        let mut exits = Vec::new();
        for (index, sequence) in alternatives.iter().enumerate() {
            if index + 1 == alternatives.len() {
                self.sequence(sequence)?;
                break;
            }
            let split = self.instructions.len();
            self.push(RegExpInstruction::split(0, 0))?;
            let primary = self.instructions.len();
            self.sequence(sequence)?;
            let exit = self.instructions.len();
            self.push(RegExpInstruction::jump(0))?;
            let fallback = self.instructions.len();
            self.instructions[split] = RegExpInstruction::split(primary, fallback);
            exits.push(exit);
        }
        let after = self.instructions.len();
        for exit in exits {
            self.instructions[exit] = RegExpInstruction::jump(after);
        }
        Ok(())
    }

    fn sequence(&mut self, terms: &[ParsedTerm]) -> Result<(), RegExpCompileError> {
        for term in terms {
            match term {
                ParsedTerm::Quantified {
                    atom,
                    quantifier,
                    quantifier_offset,
                } => self.quantified(atom, *quantifier, *quantifier_offset)?,
                ParsedTerm::LegacyUtf16Pair {
                    pair,
                    trail_quantifier,
                    quantifier_offset,
                } => {
                    self.error_offset = *quantifier_offset;
                    self.push(pair.lead_instruction())?;
                    self.quantified(
                        &ParsedAtom::Instruction(pair.trail_instruction()),
                        *trail_quantifier,
                        *quantifier_offset,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn quantified(
        &mut self,
        atom: &ParsedAtom,
        quantifier: Quantifier,
        offset: usize,
    ) -> Result<(), RegExpCompileError> {
        self.error_offset = offset;
        for _ in 0..quantifier.min {
            self.atom(atom)?;
        }
        match quantifier.max {
            Some(max) => {
                for _ in quantifier.min..max {
                    self.optional(atom, quantifier.lazy)?;
                }
            }
            None => self.star(atom, quantifier.lazy)?,
        }
        Ok(())
    }

    fn optional(&mut self, atom: &ParsedAtom, lazy: bool) -> Result<(), RegExpCompileError> {
        let split = self.instructions.len();
        self.push(RegExpInstruction::split(0, 0))?;
        let attempt = self.instructions.len();
        self.atom(atom)?;
        let after = self.instructions.len();
        self.instructions[split] = if lazy {
            RegExpInstruction::split(after, attempt)
        } else {
            RegExpInstruction::split(attempt, after)
        };
        Ok(())
    }

    fn star(&mut self, atom: &ParsedAtom, lazy: bool) -> Result<(), RegExpCompileError> {
        let split = self.instructions.len();
        self.push(RegExpInstruction::split(0, 0))?;
        let attempt = self.instructions.len();
        self.atom(atom)?;
        self.push(RegExpInstruction::jump(split))?;
        let after = self.instructions.len();
        self.instructions[split] = if lazy {
            RegExpInstruction::split(after, attempt)
        } else {
            RegExpInstruction::split(attempt, after)
        };
        Ok(())
    }

    fn atom(&mut self, atom: &ParsedAtom) -> Result<(), RegExpCompileError> {
        match atom {
            ParsedAtom::Instruction(instruction) => self.push(*instruction),
            ParsedAtom::Capture {
                id,
                body,
                subtree_end,
            } => {
                self.push(RegExpInstruction::clear_capture_range(*id, *subtree_end))?;
                self.push(RegExpInstruction::capture_start(*id))?;
                self.alternatives(body)?;
                self.push(RegExpInstruction::capture_end(*id))
            }
            ParsedAtom::NonCapture {
                body,
                subtree_start,
                subtree_end,
            } => {
                if subtree_start != subtree_end {
                    self.push(RegExpInstruction::clear_capture_range(
                        *subtree_start,
                        *subtree_end,
                    ))?;
                }
                self.alternatives(body)
            }
            ParsedAtom::NamedBackreference { name, offset } => {
                let name_id = self
                    .named_groups
                    .iter()
                    .position(|group| group.name == *name)
                    .ok_or_else(|| {
                        RegExpCompileError::invalid_syntax(
                            SyntaxRule::UnknownGroupName,
                            *offset,
                            format!("unknown named backreference `{name}`"),
                        )
                    })?;
                self.push(RegExpInstruction::named_backreference(name_id as u32))
            }
            ParsedAtom::NumberedBackreference {
                capture_id,
                nullable,
            } => self.push(if *nullable {
                RegExpInstruction::numbered_backreference(*capture_id)
            } else {
                RegExpInstruction::nonempty_numbered_backreference(*capture_id)
            }),
            ParsedAtom::Lookbehind { negative, body } => {
                self.push(RegExpInstruction::lookbehind_start())?;
                let sentinel = self.instructions.len();
                self.push(RegExpInstruction::split(0, 0))?;
                let body_start = self.instructions.len();
                self.reverse_alternatives(body)?;
                let end = self.instructions.len();
                self.push(RegExpInstruction::lookbehind_end(0, 0, *negative))?;
                let failure = self.instructions.len();
                self.push(RegExpInstruction::lookbehind_failure(0, *negative))?;
                let after = self.instructions.len();
                self.instructions[sentinel] = RegExpInstruction::split(body_start, failure);
                self.instructions[end] =
                    RegExpInstruction::lookbehind_end(failure, after, *negative);
                self.instructions[failure] =
                    RegExpInstruction::lookbehind_failure(after, *negative);
                Ok(())
            }
        }
    }

    fn reverse_alternatives(
        &mut self,
        alternatives: &[Vec<ParsedTerm>],
    ) -> Result<(), RegExpCompileError> {
        let mut exits = Vec::new();
        for (index, sequence) in alternatives.iter().enumerate() {
            if index + 1 == alternatives.len() {
                self.reverse_sequence(sequence)?;
                break;
            }
            let split = self.instructions.len();
            self.push(RegExpInstruction::split(0, 0))?;
            let primary = self.instructions.len();
            self.reverse_sequence(sequence)?;
            let exit = self.instructions.len();
            self.push(RegExpInstruction::jump(0))?;
            let fallback = self.instructions.len();
            self.instructions[split] = RegExpInstruction::split(primary, fallback);
            exits.push(exit);
        }
        let after = self.instructions.len();
        for exit in exits {
            self.instructions[exit] = RegExpInstruction::jump(after);
        }
        Ok(())
    }

    fn reverse_sequence(&mut self, terms: &[ParsedTerm]) -> Result<(), RegExpCompileError> {
        for term in terms.iter().rev() {
            match term {
                ParsedTerm::Quantified {
                    atom,
                    quantifier,
                    quantifier_offset,
                } => self.reverse_quantified(atom, *quantifier, *quantifier_offset)?,
                ParsedTerm::LegacyUtf16Pair {
                    pair,
                    trail_quantifier,
                    quantifier_offset,
                } => {
                    self.reverse_quantified(
                        &ParsedAtom::Instruction(pair.trail_instruction()),
                        *trail_quantifier,
                        *quantifier_offset,
                    )?;
                    self.push(pair.lead_instruction())?;
                }
            }
        }
        Ok(())
    }

    fn reverse_quantified(
        &mut self,
        atom: &ParsedAtom,
        quantifier: Quantifier,
        offset: usize,
    ) -> Result<(), RegExpCompileError> {
        self.error_offset = offset;
        for _ in 0..quantifier.min {
            self.reverse_atom(atom)?;
        }
        match quantifier.max {
            Some(max) => {
                for _ in quantifier.min..max {
                    let split = self.instructions.len();
                    self.push(RegExpInstruction::split(0, 0))?;
                    let attempt = self.instructions.len();
                    self.reverse_atom(atom)?;
                    let after = self.instructions.len();
                    self.instructions[split] = if quantifier.lazy {
                        RegExpInstruction::split(after, attempt)
                    } else {
                        RegExpInstruction::split(attempt, after)
                    };
                }
                Ok(())
            }
            None => {
                let split = self.instructions.len();
                self.push(RegExpInstruction::split(0, 0))?;
                let attempt = self.instructions.len();
                self.reverse_atom(atom)?;
                self.push(RegExpInstruction::jump(split))?;
                let after = self.instructions.len();
                self.instructions[split] = if quantifier.lazy {
                    RegExpInstruction::split(after, attempt)
                } else {
                    RegExpInstruction::split(attempt, after)
                };
                Ok(())
            }
        }
    }

    fn reverse_atom(&mut self, atom: &ParsedAtom) -> Result<(), RegExpCompileError> {
        match atom {
            ParsedAtom::Instruction(instruction) => self.push(*instruction),
            ParsedAtom::Capture {
                id,
                body,
                subtree_end,
            } => {
                self.push(RegExpInstruction::clear_capture_range(*id, *subtree_end))?;
                self.push(RegExpInstruction::capture_end(*id))?;
                self.reverse_alternatives(body)?;
                self.push(RegExpInstruction::capture_start(*id))
            }
            ParsedAtom::NonCapture {
                body,
                subtree_start,
                subtree_end,
            } => {
                if subtree_start != subtree_end {
                    self.push(RegExpInstruction::clear_capture_range(
                        *subtree_start,
                        *subtree_end,
                    ))?;
                }
                self.reverse_alternatives(body)
            }
            _ => Err(RegExpCompileError::unsupported_feature(
                self.error_offset,
                "lookbehind body uses an unsupported matcher atom",
            )),
        }
    }
}

fn add_ascii_range(bitmap_low: &mut u64, bitmap_high: &mut u64, start: u8, end: u8) {
    for member in start..=end {
        add_ascii_member(bitmap_low, bitmap_high, member);
    }
}

fn add_ascii_member(bitmap_low: &mut u64, bitmap_high: &mut u64, member: u8) {
    if member < 64 {
        *bitmap_low |= 1_u64 << member;
    } else {
        *bitmap_high |= 1_u64 << (member - 64);
    }
}

/// `SyntaxCharacter :: one of ^ $ \ . * + ? ( ) [ ] { } |` (22.2.1), and
/// nothing else.
///
/// Named for the production rather than for a vague notion of "characters with
/// special meaning", which is what let DEFECT 1 hide: read as "metacharacter",
/// this looked like a plausible spelling of the Unicode identity-escape rule,
/// and the missing `/` alternative was invisible for as long as the name did
/// not say which production it was. Three call sites, each a *different*
/// grammar rule built on top of this one, and none of them may widen it:
/// [`is_unicode_identity_escape`], [`is_class_identity_escape`], and the
/// unsupported-metacharacter fallthrough in `parse_instruction_atom`. Adding
/// `/` here would have turned a bare `/` in `new RegExp("a/b")` into an
/// `UnsupportedFeature` verdict.
fn is_syntax_character(byte: u8) -> bool {
    matches!(
        byte,
        b'^' | b'$'
            | b'\\'
            | b'.'
            | b'*'
            | b'+'
            | b'?'
            | b'('
            | b')'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'|'
    )
}

/// `IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/`` (22.2.1).
///
/// The `/` alternative exists because a RegExp *literal* must be able to escape
/// its own delimiter, and `\/` is therefore an everyday pattern rather than a
/// corner case. `test262/vendor/test262/test/built-ins/RegExp/unicode_identity_escape.js`
/// asserts both alternatives, in `AtomEscape` and in `ClassEscape`.
fn is_unicode_identity_escape(byte: u8) -> bool {
    is_syntax_character(byte) || byte == b'/'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(pattern: &str) -> RegExpProgram {
        RegExpProgram::compile(pattern, "").expect("pattern should compile")
    }

    #[test]
    fn compiles_literals_and_class_ranges() {
        let program = compile("t[a-b|q-s]");
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(
            program.instructions[0],
            RegExpInstruction::literal_ascii(b't')
        );
        let class = program.instructions[1];
        for member in b"ab|qrs" {
            assert!(class.positive_ascii_class_contains(*member));
        }
        assert!(!class.positive_ascii_class_contains(b'c'));
        assert_eq!(program.instructions[2], RegExpInstruction::accept());
    }

    #[test]
    fn empty_pattern_compiles_to_accept() {
        assert_eq!(compile("").instructions, vec![RegExpInstruction::accept()]);
    }

    #[test]
    fn ascii_ignore_case_expands_literals_and_classes() {
        let program = RegExpProgram::compile("a[B-c]", "i").expect("pattern should compile");
        for (instruction, lowercase, uppercase) in [
            (program.instructions[0], b'a', b'A'),
            (program.instructions[1], b'b', b'B'),
        ] {
            assert!(instruction.positive_ascii_class_contains(lowercase));
            assert!(instruction.positive_ascii_class_contains(uppercase));
        }
    }

    #[test]
    fn unicode_singleton_class_compiles_as_one_scalar() {
        let program = RegExpProgram::compile("[𝌆]", "u").expect("pattern should compile");
        assert_eq!(
            program.instructions,
            vec![
                RegExpInstruction::literal_code_point(0x1d306),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn annex_b_quantified_ascii_lookaheads_collapse_zero_width_repetitions() {
        assert_eq!(
            compile(".(?=Z)*").instructions,
            vec![RegExpInstruction::dot(), RegExpInstruction::accept()]
        );
        assert_eq!(
            compile(".(?=Z)+").instructions,
            vec![
                RegExpInstruction::dot(),
                RegExpInstruction::positive_ascii_lookahead(b'Z'),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            compile("[a-e](?!Z){2,3}").instructions,
            vec![
                compile("[a-e]").instructions[0],
                RegExpInstruction::negative_ascii_lookahead(b'Z'),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn compiles_range_classes_at_ascii_bitmap_boundaries() {
        let program = compile("[a-f]d");
        let class = program.instructions[0];
        assert!(class.positive_ascii_class_contains(b'a'));
        assert!(class.positive_ascii_class_contains(b'f'));
        assert!(!class.positive_ascii_class_contains(b'g'));
        assert_eq!(
            program.instructions[1],
            RegExpInstruction::literal_ascii(b'd')
        );

        let program = compile("[a-z]n");
        let class = program.instructions[0];
        assert!(class.positive_ascii_class_contains(b'a'));
        assert!(class.positive_ascii_class_contains(b'z'));
        assert!(!class.positive_ascii_class_contains(b'A'));
    }

    #[test]
    fn compiles_empty_character_classes() {
        assert_eq!(
            compile("[]").instructions,
            vec![
                RegExpInstruction::positive_ascii_class(0, 0),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            compile("[^]").instructions,
            vec![
                RegExpInstruction::negative_ascii_class(0, 0),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn compiles_singleton_class_members() {
        let program = compile("[Nn]evermore");
        let class = program.instructions[0];
        assert!(class.positive_ascii_class_contains(b'N'));
        assert!(class.positive_ascii_class_contains(b'n'));
        assert!(!class.positive_ascii_class_contains(b'm'));
        assert_eq!(
            program.instructions[1],
            RegExpInstruction::literal_ascii(b'e')
        );
    }

    #[test]
    fn class_hyphens_are_literal_when_not_range_separators() {
        let leading = compile("[-a]").instructions[0];
        assert!(leading.positive_ascii_class_contains(b'-'));
        assert!(leading.positive_ascii_class_contains(b'a'));
        assert!(!leading.positive_ascii_class_contains(b'.'));

        let after_range = compile("[a-b-c]").instructions[0];
        for member in b"ab-c" {
            assert!(after_range.positive_ascii_class_contains(*member));
        }
        assert!(!after_range.positive_ascii_class_contains(b'd'));

        let trailing = compile("[a-]").instructions[0];
        assert!(trailing.positive_ascii_class_contains(b'a'));
        assert!(trailing.positive_ascii_class_contains(b'-'));

        let leading_dash_range = compile("[--a]").instructions[0];
        for member in b'-'..=b'a' {
            assert!(leading_dash_range.positive_ascii_class_contains(member));
        }
        assert!(!leading_dash_range.positive_ascii_class_contains(b'b'));
    }

    #[test]
    fn annex_b_class_escapes_and_set_ranges_compile_to_ascii_bitmaps() {
        let controls = compile(r"[\c0\c1\c8\c9\c_]").instructions[0];
        for member in [0x10, 0x11, 0x18, 0x19, 0x1f] {
            assert!(controls.positive_ascii_class_contains(member));
        }

        let decimal_range = compile(r"[\12-\14]").instructions[0];
        for member in 0x0a..=0x0c {
            assert!(decimal_range.positive_ascii_class_contains(member));
        }

        let union_range = compile(r"[\d-a]").instructions[0];
        for member in b'0'..=b'9' {
            assert!(union_range.positive_ascii_class_contains(member));
        }
        assert!(union_range.positive_ascii_class_contains(b'-'));
        assert!(union_range.positive_ascii_class_contains(b'a'));
    }

    #[test]
    fn negated_ascii_classes_compile_to_negative_class_instructions() {
        let instruction = compile(r"[^\d]").instructions[0];
        assert_eq!(instruction.opcode, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS);
        assert_ne!(instruction.operand0 & (1_u64 << b'0'), 0);
        assert_ne!(instruction.operand0 & (1_u64 << b'9'), 0);
    }

    #[test]
    fn class_membership_rejects_non_ascii_code_units() {
        let class = RegExpInstruction::positive_ascii_class(0, u64::MAX);
        assert!(class.positive_ascii_class_contains(127));
        assert!(!class.positive_ascii_class_contains(128));
        assert!(!class.positive_ascii_class_contains(255));
    }

    #[test]
    fn escaped_syntax_characters_are_literal_atoms() {
        let program = compile(r"\.\^\$\*\+\?\(\)\[\]\{\}\|");
        let literals = program.instructions[..program.instructions.len() - 1]
            .iter()
            .map(|instruction| instruction.operand0 as u8)
            .collect::<Vec<_>>();
        assert_eq!(literals, b".^$*+?()[]{}|");
        assert!(program.instructions[..program.instructions.len() - 1]
            .iter()
            .all(|instruction| instruction.opcode == REGEXP_OPCODE_LITERAL_ASCII));
    }

    #[test]
    fn annex_b_extended_literals_identity_escapes_and_octal_escapes_compile() {
        assert_eq!(
            compile(r"]{}\C\8\9\377").instructions,
            vec![
                RegExpInstruction::literal_ascii(b']'),
                RegExpInstruction::literal_ascii(b'{'),
                RegExpInstruction::literal_ascii(b'}'),
                RegExpInstruction::literal_ascii(b'C'),
                RegExpInstruction::literal_ascii(b'8'),
                RegExpInstruction::literal_ascii(b'9'),
                RegExpInstruction::literal_code_point(0xff),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn numbered_escape_uses_an_existing_capture_before_legacy_octal() {
        let program = compile(r"(.)\1");
        assert!(program
            .instructions
            .contains(&RegExpInstruction::nonempty_numbered_backreference(1)));
        assert!(!program
            .instructions
            .contains(&RegExpInstruction::literal_ascii(1)));
    }

    #[test]
    fn nonempty_capture_backreference_allows_unbounded_quantification() {
        let program = compile(r"^(a+)\1*,\1+$");
        assert!(program
            .instructions
            .contains(&RegExpInstruction::nonempty_numbered_backreference(1)));
    }

    #[test]
    fn annex_b_identity_k_and_forward_numbered_backreferences_use_whole_pattern_syntax() {
        let identity = compile(r"\k<a>");
        let literals = identity.instructions[..identity.instructions.len() - 1]
            .iter()
            .map(|instruction| instruction.operand0 as u8)
            .collect::<Vec<_>>();
        assert_eq!(literals, b"k<a>");

        let forward = compile(r"\1(b)");
        assert_eq!(
            forward.instructions[0],
            RegExpInstruction::numbered_backreference(1)
        );

        for pattern in [
            r"\k<a>(?<=>)a",
            r"(?<=>)\k<a>",
            r"\k<a>(?<!a)a",
            r"(?<!a>)\k<a>",
        ] {
            RegExpProgram::compile(pattern, "").unwrap_or_else(|error| {
                panic!("{pattern} should compile: {error:?}");
            });
        }
    }

    #[test]
    fn digit_escape_compiles_to_exact_ascii_bitmap() {
        let program = compile(r"\d");
        let instruction = program.instructions[0];
        assert_eq!(instruction.opcode, REGEXP_OPCODE_POSITIVE_ASCII_CLASS);
        for member in b'0'..=b'9' {
            assert!(instruction.positive_ascii_class_contains(member));
        }
        for member in [b'/', b':', b'A', 0, 127] {
            assert!(!instruction.positive_ascii_class_contains(member));
        }
        assert_eq!(program.encode().len(), 2 * REGEXP_INSTRUCTION_WIDTH);
        assert_eq!(
            &program.encode()[8..16],
            &instruction.operand0.to_le_bytes()
        );
        assert_eq!(
            &program.encode()[16..24],
            &instruction.operand1.to_le_bytes()
        );
    }

    #[test]
    fn digit_escape_integrates_with_greedy_quantifiers() {
        let digit_class = RegExpInstruction::positive_ascii_class(((1_u64 << 10) - 1) << 48, 0);
        assert_eq!(
            compile(r"\d+").instructions,
            vec![
                digit_class,
                RegExpInstruction::split(2, 4),
                digit_class,
                RegExpInstruction::jump(1),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn captures_emit_boundaries_around_existing_quantified_atom_programs() {
        let digit_class = RegExpInstruction::positive_ascii_class(((1_u64 << 10) - 1) << 48, 0);
        let program = compile(r"(\d+)");
        assert_eq!(program.capture_count, 1);
        assert_eq!(
            program.instructions,
            vec![
                RegExpInstruction::clear_capture_range(1, 2),
                RegExpInstruction::capture_start(1),
                digit_class,
                RegExpInstruction::split(4, 6),
                digit_class,
                RegExpInstruction::jump(3),
                RegExpInstruction::capture_end(1),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn numbers_sequential_captures_and_keeps_surrounding_terms() {
        let program = compile(r"x(a)(\d)y");
        assert_eq!(program.capture_count, 2);
        assert_eq!(
            program.instructions,
            vec![
                RegExpInstruction::literal_ascii(b'x'),
                RegExpInstruction::clear_capture_range(1, 2),
                RegExpInstruction::capture_start(1),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::capture_end(1),
                RegExpInstruction::clear_capture_range(2, 3),
                RegExpInstruction::capture_start(2),
                RegExpInstruction::positive_ascii_class(((1_u64 << 10) - 1) << 48, 0),
                RegExpInstruction::capture_end(2),
                RegExpInstruction::literal_ascii(b'y'),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn supports_empty_groups_and_alternatives() {
        assert_eq!(compile("()").capture_count, 1);
        assert_eq!(compile("(?:)").capture_count, 0);
        assert_eq!(compile("(|)").capture_count, 1);
        assert_eq!(compile("|a").capture_count, 0);
        assert_eq!(compile("a|").capture_count, 0);
        assert_eq!(compile("(?:a)").capture_count, 0);
        assert_eq!(compile("(?<x>a)").named_groups[0].name, "x");
    }

    #[test]
    fn lowers_ordered_alternation_and_nested_capture_ranges() {
        assert_eq!(
            compile("(a|b)").instructions,
            vec![
                RegExpInstruction::clear_capture_range(1, 2),
                RegExpInstruction::capture_start(1),
                RegExpInstruction::split(3, 5),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::jump(6),
                RegExpInstruction::literal_ascii(b'b'),
                RegExpInstruction::capture_end(1),
                RegExpInstruction::accept(),
            ]
        );
        let nested = compile("((a))");
        assert_eq!(nested.capture_count, 2);
        assert_eq!(
            nested.instructions[0],
            RegExpInstruction::clear_capture_range(1, 3)
        );
        assert_eq!(
            nested.instructions[2],
            RegExpInstruction::clear_capture_range(2, 3)
        );
    }

    #[test]
    fn compiles_capture_alternation_quantifier_targets_and_rejects_nullable_loops() {
        let first = compile("((1)|(12))((3)|(23))");
        assert_eq!(first.capture_count, 6);
        assert!(first
            .instructions
            .iter()
            .any(|i| *i == RegExpInstruction::clear_capture_range(1, 4)));
        let star = compile("(aa|aabaac|ba|b|c)*");
        assert_eq!(star.capture_count, 1);
        assert_eq!(
            star.instructions[0],
            RegExpInstruction::split(1, star.instructions.len() - 1)
        );
        let nested_star = compile("(z)((a+)?(b+)?(c))*");
        assert_eq!(nested_star.capture_count, 5);
        assert!(nested_star
            .instructions
            .iter()
            .any(|i| *i == RegExpInstruction::clear_capture_range(2, 6)));
        for pattern in ["(a?)*", "(a?)+", "(a?){1,}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "").expect_err(pattern).kind,
                RegExpCompileErrorKind::UnsupportedFeature
            );
        }
    }

    #[test]
    fn reports_malformed_group_delimiters_as_invalid_syntax() {
        for pattern in ["(a", "a)"] {
            let error = RegExpProgram::compile(pattern, "").expect_err(pattern);
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
    }

    #[test]
    fn non_unicode_property_syntax_is_an_identity_escape() {
        let program = compile(r"\p{Decimal_Number}");
        assert_eq!(
            program.instructions[0],
            RegExpInstruction::literal_ascii(b'p')
        );
        assert_eq!(
            program.instructions.last(),
            Some(&RegExpInstruction::accept())
        );
    }

    #[test]
    fn annex_b_malformed_hex_escape_is_an_identity_escape() {
        assert_eq!(
            compile(r"\x").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'x'),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            RegExpProgram::compile(r"\x", "u").unwrap_err().kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
        for (pattern, expected) in [(r"\xa", b"xa".as_slice()), (r"\ua", b"ua".as_slice())] {
            let program = compile(pattern);
            let literals = program.instructions[..program.instructions.len() - 1]
                .iter()
                .map(|instruction| instruction.operand0 as u8)
                .collect::<Vec<_>>();
            assert_eq!(literals, expected);
        }
        assert_eq!(
            RegExpProgram::compile(r"\u", "u").unwrap_err().kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
    }

    #[test]
    fn word_escapes_compile_to_ascii_class_opcodes() {
        let word_bitmap_low = ((1_u64 << 10) - 1) << 48;
        let word_bitmap_high = ((1_u64 << 26) - 1) << 1 | ((1_u64 << 26) - 1) << 33 | (1_u64 << 31);
        assert_eq!(
            compile(r"\w").instructions,
            vec![
                RegExpInstruction::positive_ascii_class(word_bitmap_low, word_bitmap_high),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            compile(r"\W").instructions,
            vec![
                RegExpInstruction::negative_ascii_class(word_bitmap_low, word_bitmap_high),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn complemented_character_class_escapes_compile() {
        let non_digit = compile(r"\D").instructions[0];
        assert_eq!(non_digit.opcode, REGEXP_OPCODE_NEGATIVE_ASCII_CLASS);
        assert_eq!(
            compile(r"\S").instructions[0],
            RegExpInstruction::not_whitespace()
        );
    }

    #[test]
    fn whitespace_escape_compiles_to_zero_operand_opcode() {
        let program = compile(r"\s");
        assert_eq!(
            program.instructions,
            vec![RegExpInstruction::whitespace(), RegExpInstruction::accept()]
        );
        assert_eq!(program.instructions[0].operand0, 0);
        assert_eq!(program.instructions[0].operand1, 0);
        assert_eq!(program.instructions[0].opcode, REGEXP_OPCODE_WHITESPACE);
    }

    #[test]
    fn line_assertions_compile_to_zero_width_opcodes() {
        let program = compile("^a$");
        assert_eq!(
            program.instructions,
            vec![
                RegExpInstruction::assert_start(),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::assert_end(),
                RegExpInstruction::accept(),
            ]
        );
    }

    #[test]
    fn dot_compiles_to_a_canonical_zero_operand_instruction() {
        let program = compile(".");
        assert_eq!(
            program.instructions,
            vec![RegExpInstruction::dot(), RegExpInstruction::accept()]
        );
        assert_eq!(program.instructions[0].opcode, REGEXP_OPCODE_DOT);
        assert_eq!(program.instructions[0].operand0, 0);
        assert_eq!(program.instructions[0].operand1, 0);
    }

    #[test]
    fn whitespace_escape_integrates_with_a3_t4_pattern() {
        let program = compile(r"([Nn]?ever|([Nn]othing\s{1,}))more");
        assert_eq!(program.capture_count, 2);
        assert!(program
            .instructions
            .iter()
            .any(|instruction| instruction.opcode == REGEXP_OPCODE_WHITESPACE));
    }

    #[test]
    fn escaped_braces_can_still_be_postfix_quantified() {
        assert_eq!(
            compile(r"\{{2}").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'{'),
                RegExpInstruction::literal_ascii(b'{'),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            compile(r"\}{1,2}").instructions.len(),
            4,
            "escaped closing brace remains an atom for postfix quantification"
        );
    }

    #[test]
    fn encodes_instructions_as_deterministic_little_endian_words() {
        let program = compile("[?@]");
        let encoded = program.encode();
        assert_eq!(encoded.len(), 2 * REGEXP_INSTRUCTION_WIDTH);
        assert_eq!(
            &encoded[0..8],
            &REGEXP_OPCODE_POSITIVE_ASCII_CLASS.to_le_bytes()
        );
        let expected_low = 1_u64 << 63;
        assert_eq!(&encoded[8..16], &expected_low.to_le_bytes());
        assert_eq!(&encoded[16..24], &1_u64.to_le_bytes());
        assert_eq!(&encoded[24..32], &REGEXP_OPCODE_ACCEPT.to_le_bytes());
        assert_eq!(&encoded[32..48], &[0; 16]);
    }

    #[test]
    fn quantifiers_encode_ordered_backtracking_programs() {
        assert_eq!(
            compile("a?").instructions,
            vec![
                RegExpInstruction::split(1, 2),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a??").instructions,
            vec![
                RegExpInstruction::split(2, 1),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a+?").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::split(4, 2),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::jump(1),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a{0}").instructions,
            vec![RegExpInstruction::accept()]
        );
        assert_eq!(
            compile("a{1}").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a{2}").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a{2,4}").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::split(3, 4),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::split(5, 6),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a{2,}").instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::split(3, 5),
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::jump(2),
                RegExpInstruction::accept()
            ]
        );
        assert_eq!(
            compile("a{2,4}?").instructions[2],
            RegExpInstruction::split(4, 3)
        );
    }

    #[test]
    fn quantifier_errors_are_precise_and_bounded() {
        for pattern in ["a{4,2}", "a**", "a+*", "*", "+", "?"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "").expect_err(pattern).kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
        assert_eq!(
            RegExpProgram::compile("a{184467440737095516160}", "")
                .expect_err("overflow")
                .kind,
            RegExpCompileErrorKind::UnsupportedFeature
        );
        assert_eq!(
            RegExpProgram::compile("a{4096}", "").expect_err("cap").kind,
            RegExpCompileErrorKind::UnsupportedFeature
        );
    }

    #[test]
    fn incomplete_braces_are_legacy_literals() {
        for pattern in ["a{b}", "a{", "a{,2}", "a{1", "a{1,x}", "a{1,2"] {
            let program = compile(pattern);
            let literals = program.instructions[..program.instructions.len() - 1]
                .iter()
                .map(|instruction| instruction.operand0 as u8)
                .collect::<Vec<_>>();
            assert_eq!(literals, pattern.as_bytes(), "{pattern}");
        }
        for pattern in ["}", "a}", "a{}"] {
            let program = compile(pattern);
            let literals = program.instructions[..program.instructions.len() - 1]
                .iter()
                .map(|instruction| instruction.operand0 as u8)
                .collect::<Vec<_>>();
            assert_eq!(literals, pattern.as_bytes(), "{pattern}");
        }
    }

    #[test]
    fn braced_quantifiers_require_a_preceding_atom() {
        for pattern in ["{1}", "{1,}", "{1,2}", "{4,2}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "").expect_err(pattern).kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
        assert_eq!(
            RegExpProgram::compile("{", "u")
                .expect_err("Unicode brace")
                .kind,
            RegExpCompileErrorKind::InvalidSyntax
        );
    }

    #[test]
    fn incomplete_braces_after_quantifiers_remain_literals() {
        for pattern in ["a{1}{b}", "a?{b}", "a*{", "a+{1,x}"] {
            compile(pattern);
        }
        for pattern in ["a{1}{2}", "a?{1}", "a*{1,}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "").expect_err(pattern).kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
    }

    #[test]
    fn preserves_global_and_sticky_flags_in_either_order() {
        for flags in ["gy", "yg"] {
            let program = RegExpProgram::compile("a", flags).expect("flags should compile");
            assert_eq!(
                program.flags,
                RegExpFlags {
                    has_indices: false,
                    global: true,
                    ignore_case: false,
                    multiline: false,
                    dot_all: false,
                    sticky: true,
                    unicode_mode: RegExpUnicodeMode::Legacy,
                }
            );
            assert_eq!(
                program.instructions,
                vec![
                    RegExpInstruction::literal_ascii(b'a'),
                    RegExpInstruction::accept()
                ]
            );
            assert_eq!(program.capture_count, 0);
        }
    }

    #[test]
    fn unicode_literals_and_escaped_surrogates_are_code_points() {
        let paired = RegExpProgram::compile(r"\uD842\uDFB7", "u").unwrap();
        assert_eq!(
            paired.instructions[0],
            RegExpInstruction::literal_code_point(0x20BB7)
        );
        let lone = RegExpProgram::compile(r"\uDFFF", "u").unwrap();
        assert_eq!(
            lone.instructions[0],
            RegExpInstruction::literal_code_point(0xDFFF)
        );
        let direct = RegExpProgram::compile("𠮷", "u").unwrap();
        assert_eq!(
            direct.instructions[0],
            RegExpInstruction::literal_code_point(0x20BB7)
        );
    }

    #[test]
    fn malformed_unicode_escape_is_invalid_syntax() {
        for pattern in [r"\u12", r"\u12G4"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "u").unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax
            );
        }
    }

    #[test]
    fn distinguishes_unicode_and_unicode_sets_flags() {
        let unicode = RegExpProgram::compile("𠮷", "u").unwrap();
        assert_eq!(unicode.flags.unicode_mode, RegExpUnicodeMode::Unicode);
        let unicode_sets = RegExpProgram::compile("𠮷", "v").unwrap();
        assert_eq!(
            unicode_sets.flags.unicode_mode,
            RegExpUnicodeMode::UnicodeSets
        );
        assert_eq!(unicode.instructions, unicode_sets.instructions);
    }

    #[test]
    fn compiles_unicode_properties_into_the_range_pool() {
        for flags in ["u", "v"] {
            let program = RegExpProgram::compile(r"\p{ASCII}\P{ASCII}", flags).unwrap();
            assert_eq!(
                program.instructions,
                vec![
                    RegExpInstruction::code_point_range_set(0, 1, false),
                    RegExpInstruction::code_point_range_set(1, 1, false),
                    RegExpInstruction::accept(),
                ]
            );
            assert_eq!(program.ranges, vec![(0, 0x7f), (0x80, 0x10ffff)]);
            let encoded = program.encode();
            assert_eq!(&encoded[..8], &REGEXP_OPCODE_UNICODE_PROPERTY.to_le_bytes());
            assert_eq!(
                encoded.len(),
                program.instructions.len() * REGEXP_INSTRUCTION_WIDTH
                    + program.ranges.len() * REGEXP_RANGE_ENTRY_WIDTH
            );
            assert_eq!(
                &encoded[encoded.len() - 8..],
                &[0x80, 0, 0, 0, 0xff, 0xff, 0x10, 0]
            );
        }
    }

    #[test]
    fn resolves_general_category_and_script_property_escapes() {
        let letters = RegExpProgram::compile(r"\p{L}", "u").unwrap();
        assert!(ranges_contain(&letters.ranges, u32::from(b'a')));
        assert!(ranges_contain(&letters.ranges, 0x00e9));
        assert!(!ranges_contain(&letters.ranges, u32::from(b'0')));

        let han = RegExpProgram::compile(r"\p{Script=Han}", "u").unwrap();
        assert!(ranges_contain(&han.ranges, 0x4e00));
        assert!(!ranges_contain(&han.ranges, u32::from(b'a')));

        let not_han = RegExpProgram::compile(r"\P{Script=Han}", "u").unwrap();
        assert!(!ranges_contain(&not_han.ranges, 0x4e00));
        assert!(ranges_contain(&not_han.ranges, u32::from(b'a')));
    }

    #[test]
    fn rejects_unknown_and_malformed_unicode_properties() {
        for pattern in [r"\p{NotAProperty}", r"\p{script=Han}", r"\p{Foo=Bar}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "u").unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
        for pattern in [r"\p", r"\pASCII", r"\p{ASCII", r"\p{}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "v").unwrap_err().kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
        }
        assert!(RegExpProgram::compile(r"\p{ASCII}", "").is_ok());
    }

    #[test]
    fn compiles_unicode_sets_set_operations() {
        let intersection = RegExpProgram::compile(r"[\p{ASCII}&&\p{L}]", "v").unwrap();
        assert!(ranges_contain(&intersection.ranges, u32::from(b'a')));
        assert!(!ranges_contain(&intersection.ranges, 0x00e9));
        assert!(!ranges_contain(&intersection.ranges, u32::from(b'0')));

        let difference = RegExpProgram::compile(r"[[a-f]--[c-d]]", "v").unwrap();
        assert_eq!(difference.ranges, vec![(0x61, 0x62), (0x65, 0x66)]);
    }

    #[test]
    fn modifier_groups_scope_case_insensitivity() {
        let program = RegExpProgram::compile("(?i:a)b", "").unwrap();
        assert_eq!(
            program.instructions.last(),
            Some(&RegExpInstruction::accept())
        );
        assert!(program.instructions[0].positive_ascii_class_contains(b'A'));
        assert!(program.instructions[0].positive_ascii_class_contains(b'a'));
        assert_eq!(
            program.instructions[1],
            RegExpInstruction::literal_ascii(b'b')
        );

        let disabled = RegExpProgram::compile("(?-i:a)b", "i").unwrap();
        assert_eq!(
            disabled.instructions[0],
            RegExpInstruction::literal_ascii(b'a')
        );
        assert!(disabled.instructions[1].positive_ascii_class_contains(b'B'));
    }

    #[test]
    fn direct_non_unicode_source_quantifies_only_its_utf16_trail_unit() {
        let lead = RegExpInstruction::literal_code_point(0xD842);
        let trail = RegExpInstruction::literal_code_point(0xDFB7);
        assert_eq!(
            RegExpProgram::compile("é𠮷", "").unwrap().instructions,
            vec![
                RegExpInstruction::literal_code_point(0xE9),
                lead,
                trail,
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            RegExpProgram::compile("𠮷?", "").unwrap().instructions,
            vec![
                lead,
                RegExpInstruction::split(2, 3),
                trail,
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            RegExpProgram::compile("𠮷??", "").unwrap().instructions,
            vec![
                lead,
                RegExpInstruction::split(3, 2),
                trail,
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            RegExpProgram::compile("𠮷{0}", "").unwrap().instructions,
            vec![lead, RegExpInstruction::accept()]
        );
        assert_eq!(
            RegExpProgram::compile("𠮷{2}", "").unwrap().instructions,
            vec![lead, trail, trail, RegExpInstruction::accept()]
        );
        assert_eq!(
            RegExpProgram::compile("𠮷?", "u").unwrap().instructions,
            vec![
                RegExpInstruction::split(1, 2),
                RegExpInstruction::literal_code_point(0x20BB7),
                RegExpInstruction::accept(),
            ],
            "Unicode mode quantifies the whole scalar"
        );
    }

    #[test]
    fn reports_invalid_syntax_and_unsupported_features_with_offsets() {
        let invalid_cases = [("[a", 0), ("[z-a]", 2), ("\\", 0)];
        for (pattern, offset) in invalid_cases {
            let error = RegExpProgram::compile(pattern, "").expect_err("pattern should be invalid");
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
            assert_eq!(error.offset, offset, "{pattern}");
        }

        let error = RegExpProgram::compile("(a?)*", "")
            .expect_err("unbounded repetition of a nullable atom should be unsupported");
        assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
        assert_eq!(error.offset, 4);
    }

    /// No "unsupported" category any more: the `first_unsupported` arm this test
    /// was named for was deleted with the rest of the flag-verdict audit, and
    /// `parse_flags` can now only answer `InvalidSyntax`. The test stayed green
    /// through that deletion because it never asserted the third category — which
    /// is why the *name* was the only thing left claiming it exists.
    #[test]
    fn distinguishes_duplicate_and_unknown_flags() {
        for flags in ["gg", "yy", "ii", "igi"] {
            let error = RegExpProgram::compile("a", flags).expect_err("flags should be invalid");
            assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
        }

        let error = RegExpProgram::compile("a", "z").expect_err("flag should be unknown");
        assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
        assert_eq!(error.offset, 0);

        for flags in ["uv", "vu"] {
            let error = RegExpProgram::compile("a", flags)
                .expect_err("unicode modes should be mutually exclusive");
            assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
            assert_eq!(error.offset, 1);
        }

        let flags = RegExpProgram::compile("a", "imsv").unwrap().flags;
        assert!(flags.ignore_case);
        assert!(flags.multiline);
        assert!(flags.dot_all);
        assert_eq!(flags.unicode_mode, RegExpUnicodeMode::UnicodeSets);
    }

    #[test]
    fn named_groups_preserve_source_order_and_legal_duplicate_mappings() {
        let program =
            RegExpProgram::compile("(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))", "d").unwrap();
        assert!(program.flags.has_indices);
        assert_eq!(program.capture_count, 5);
        assert_eq!(
            program.named_groups,
            vec![
                RegExpNamedGroup {
                    name: "x".into(),
                    capture_ids: vec![1, 3]
                },
                RegExpNamedGroup {
                    name: "y".into(),
                    capture_ids: vec![2]
                },
                RegExpNamedGroup {
                    name: "z".into(),
                    capture_ids: vec![4, 5]
                },
            ]
        );
    }

    #[test]
    fn named_backreferences_resolve_forward_and_are_nullable() {
        let forward = compile(r"\k<x>(?<x>a)");
        assert_eq!(
            forward.instructions[0],
            RegExpInstruction::named_backreference(0)
        );
        let repeated = compile(r"(?:(?:(?<x>a)|(?<x>b)|c)\k<x>){2}");
        assert_eq!(repeated.capture_count, 2);
        assert_eq!(repeated.named_groups[0].capture_ids, vec![1, 2]);
        assert!(repeated
            .instructions
            .iter()
            .any(
                |instruction| instruction.opcode == REGEXP_OPCODE_NAMED_BACKREFERENCE
                    && instruction.operand0 == 0
                    && instruction.operand1 == 0
            ));
        assert_eq!(
            repeated.instructions[0],
            RegExpInstruction::clear_capture_range(1, 3)
        );
        assert_eq!(
            repeated
                .instructions
                .iter()
                .filter(|instruction| {
                    **instruction == RegExpInstruction::clear_capture_range(1, 3)
                })
                .count(),
            4
        );
    }

    #[test]
    fn named_group_identifiers_use_unicode_id_properties_and_canonical_names() {
        let program = RegExpProgram::compile(
            r"(?<π>a)(?<ಠ_ಠ>b)(?<ͺ>c)(?<$𐒤>d)(?<a\uD801\uDCA4>e)(?<_\u200C>f)(?<_\u200D>g)\k<\u03C0>\k<ಠ_ಠ>\k<ͺ>\k<$\u{104A4}>\k<a𐒤>\k<_\u200C>\k<_\u200D>",
            "du",
        )
        .unwrap();

        assert_eq!(
            program
                .named_groups
                .iter()
                .map(|group| group.name.as_str())
                .collect::<Vec<_>>(),
            vec!["π", "ಠ_ಠ", "ͺ", "$𐒤", "a𐒤", "_\u{200C}", "_\u{200D}"]
        );
        assert_eq!(
            program
                .instructions
                .iter()
                .filter(|instruction| instruction.opcode == REGEXP_OPCODE_NAMED_BACKREFERENCE)
                .count(),
            7
        );
    }

    #[test]
    fn named_group_identifier_escapes_are_canonical_across_groups_and_references() {
        let non_unicode = RegExpProgram::compile(r"(?<\u{03C0}>a)\k<π>", "").unwrap();
        assert_eq!(non_unicode.named_groups[0].name, "π");
        assert_eq!(
            non_unicode.instructions[4],
            RegExpInstruction::named_backreference(0)
        );

        let unicode_sets = RegExpProgram::compile(r"(?<\u03C0>a)\k<\u{03C0}>", "v").unwrap();
        assert_eq!(unicode_sets.named_groups[0].name, "π");

        let duplicate = RegExpProgram::compile(r"(?:(?<π>a)|(?<\u03C0>b))", "u").unwrap();
        assert_eq!(duplicate.named_groups[0].name, "π");
        assert_eq!(duplicate.named_groups[0].capture_ids, vec![1, 2]);

        let same_path = RegExpProgram::compile(r"(?<π>a)(?<\u{03C0}>b)", "u")
            .expect_err("decoded duplicate name should be rejected");
        assert_eq!(same_path.kind, RegExpCompileErrorKind::InvalidSyntax);
    }

    #[test]
    fn invalid_named_group_identifier_code_points_and_escapes_are_syntax_errors() {
        let invalid = [
            ("(?<1>a)", 3),
            (r"(?<\u0031>a)", 3),
            (r"(?<\u{31}>a)", 3),
            ("(?<❤>a)", 3),
            (r"(?<\u200C>a)", 3),
            (r"(?<\u200D>a)", 3),
            (r"(?<\x41>a)", 3),
            (r"(?<\u12G4>a)", 7),
            (r"(?<\u{}>a)", 6),
            (r"(?<\u{D800}>a)", 3),
            (r"(?<\u{110000}>a)", 3),
            (r"(?<\uD801>a)", 3),
            (r"(?<\uDCA4>a)", 3),
            (r"(?<a\uD801\u0041>a)", 4),
            (r"(?<x>a)\k<\uD801>", 10),
        ];
        for (pattern, offset) in invalid {
            let error = RegExpProgram::compile(pattern, "")
                .expect_err("invalid identifier should be rejected");
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "{pattern}"
            );
            assert_eq!(error.offset, offset, "{pattern}: {error}");
        }
    }

    #[test]
    fn lowers_supported_lookbehind_to_reverse_matcher_instructions() {
        let program = RegExpProgram::compile(r"(?<=\w+)f", "").unwrap();
        assert_eq!(
            program.instructions[0],
            RegExpInstruction::lookbehind_start()
        );
        assert_eq!(program.instructions[1], RegExpInstruction::split(2, 7));
        assert_eq!(program.instructions[3], RegExpInstruction::split(4, 6));
        assert_eq!(program.instructions[5], RegExpInstruction::jump(3));
        assert_eq!(
            program.instructions[6],
            RegExpInstruction::lookbehind_end(7, 8, false)
        );
        assert_eq!(
            program.instructions[7],
            RegExpInstruction::lookbehind_failure(8, false)
        );
        assert_eq!(
            program.instructions[8],
            RegExpInstruction::literal_ascii(b'f')
        );
    }

    #[test]
    fn rejects_invalid_named_groups_and_same_path_duplicates() {
        for pattern in ["(?<1>x)", "(?<x>a)(?<x>b)"] {
            let error = RegExpProgram::compile(pattern, "").expect_err(pattern);
            assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
        }
        for pattern in [r"\k<missing>", r"\k<x"] {
            let error = RegExpProgram::compile(pattern, "u").expect_err(pattern);
            assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
        }
        assert!(RegExpProgram::compile("(?:(?<x>a)|(?<x>b))", "").is_ok());
    }

    /// DEFECT 1: `IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/``.
    ///
    /// The SOLIDUS alternative was missing, so `/\//u` — the way a RegExp
    /// literal escapes its own delimiter, and half of every URL pattern ever
    /// written — was answered `InvalidSyntax`, i.e. a `SyntaxError` for a legal
    /// program. `test262/vendor/test262/test/built-ins/RegExp/unicode_identity_escape.js`
    /// line 35 is `assert(/\//u.test("/"), …)`.
    #[test]
    fn unicode_identity_escape_accepts_the_solidus() {
        for flags in ["u", "v"] {
            let program = RegExpProgram::compile(r"\/", flags)
                .unwrap_or_else(|error| panic!("`\\/` under `{flags}` must compile: {error}"));
            assert_eq!(
                program.instructions,
                vec![
                    RegExpInstruction::literal_ascii(b'/'),
                    RegExpInstruction::accept(),
                ],
                "{flags}"
            );
        }

        // The class path always had the rule right. Its continued agreement is
        // what identified the atom path as the defect, so it is pinned too.
        //
        // The `A` is load-bearing and not decoration.
        // `class_needs_code_point_ranges` sends a class down `parse_class` only
        // when it contains one of `p P u x w W D S` after a backslash, or a
        // non-ASCII byte. A bare `[\/]` therefore goes to `parse_ascii_class`,
        // whose `parse_ascii_class_atom` takes no `RegExpUnicodeMode` and
        // accepts every escaped byte unconditionally — green whatever
        // `is_class_identity_escape` does, i.e. pinning nothing. A plain `A` is
        // not a trigger either — the second assertion below says so, because
        // that is the mistake this comment exists to stop — so the class carries
        // a `\w` and the `parse_class` / `parse_class_atom(_, _,
        // RegExpUnicodeMode::Unicode)` path is actually consulted.
        assert!(!class_needs_code_point_ranges(br"[\/]", 0));
        assert!(!class_needs_code_point_ranges(br"[\/A]", 0));
        assert!(class_needs_code_point_ranges(br"[\/\w]", 0));
        assert!(RegExpProgram::compile(r"[\/\w]", "u").is_ok());

        // The guard against the WRONG fix. Adding `/` to `is_syntax_character`
        // would satisfy the assertions above and simultaneously turn the bare
        // `/` in `new RegExp("a/b")` into an `UnsupportedFeature` verdict via
        // the metacharacter fallthrough in `parse_instruction_atom`.
        let unescaped = RegExpProgram::compile("a/b", "").expect("`a/b` must stay legal");
        assert_eq!(
            unescaped.instructions,
            vec![
                RegExpInstruction::literal_ascii(b'a'),
                RegExpInstruction::literal_ascii(b'/'),
                RegExpInstruction::literal_ascii(b'b'),
                RegExpInstruction::accept(),
            ]
        );

        // And `\q` is still not an identity escape, so the fix widened the set
        // by exactly one character rather than deleting the rule.
        assert_eq!(
            RegExpProgram::compile(r"\q", "u").unwrap_err().rule,
            Some(SyntaxRule::IdentityEscape)
        );
    }

    /// DEFECT 2: `ClassSetCharacter :: `\` ClassSetReservedPunctuator` in
    /// `v` mode.
    ///
    /// `parse_class_set` passed a literal `true` for what was a `unicode: bool`
    /// parameter, so `v`-mode class atoms were checked against the `u`-mode
    /// `ClassEscape` rule. All thirteen punctuators that `u` mode does not
    /// already accept were rejected as `SyntaxError`s.
    #[test]
    fn unicode_sets_class_accepts_reserved_punctuator_escapes() {
        // The full production is `& - ! # % , : ; < = > @ ` ~`; `-` is already a
        // `u`-mode `ClassEscape` alternative, so these are the thirteen that
        // were new in `v` mode and the thirteen HEAD refused.
        let punctuators = [
            b'&', b'!', b'#', b'%', b',', b':', b';', b'<', b'=', b'>', b'@', b'`', b'~',
        ];
        assert_eq!(punctuators.len(), 13);
        for punctuator in punctuators {
            let pattern = format!("[\\{}]", punctuator as char);
            let program = RegExpProgram::compile(&pattern, "v")
                .unwrap_or_else(|error| panic!("`{pattern}` under `v` must compile: {error}"));
            assert!(
                ranges_contain(&program.ranges, u32::from(punctuator)),
                "`{pattern}` must match `{}`",
                punctuator as char
            );
        }
        // `-` is the fourteenth alternative and is legal in `u` mode too, which
        // is why it was the one punctuator HEAD already accepted.
        //
        // Under `v` a bare `[\-]` reaches `parse_unicode_sets_class`
        // unconditionally, so it exercises the rule. Under `u` it does NOT: a
        // class with no `p P u x w W D S` escape and no non-ASCII byte is
        // handled by `parse_ascii_class`, which has no `RegExpUnicodeMode` and
        // accepts every escaped byte, so the `u` leg needs the trailing `\w` to force
        // the code-point path and reach `is_class_identity_escape` at all. The
        // same trap as the `[\/]` assertion in
        // `unicode_mode_accepts_solidus_identity_escape`.
        assert!(!class_needs_code_point_ranges(br"[\-]", 0));
        assert!(class_needs_code_point_ranges(br"[\-\w]", 0));
        for flags in ["u", "v"] {
            for pattern in [r"[\-]", r"[\-\w]"] {
                assert!(
                    RegExpProgram::compile(pattern, flags).is_ok(),
                    "the `-` alternative must not regress for `{pattern}` under `{flags}`"
                );
            }
        }

        // `u` mode is unchanged: these are NOT `u`-mode class escapes, and
        // widening both modes together would have been the wrong fix.
        for punctuator in punctuators {
            let pattern = format!("[\\{}\\u0041]", punctuator as char);
            let error = RegExpProgram::compile(&pattern, "u")
                .expect_err("`u` mode must keep rejecting reserved punctuator escapes");
            assert_eq!(error.rule, Some(SyntaxRule::ClassEscape), "{pattern}");
        }
    }

    /// DEFECT 3: `RegExpUnicodeEscapeSequence[+UnicodeMode] :: `u{` CodePoint `}``.
    ///
    /// Found while picking a witness for [`SyntaxRule::CodePointEscape`], and
    /// the same shape as DEFECT 1: only the class parser implemented the braced
    /// alternative, so `/[\u{41}]/u` compiled while `/\u{41}/u` — and therefore
    /// every astral pattern written the ordinary way — was a `SyntaxError`.
    #[test]
    fn unicode_mode_accepts_braced_code_point_escapes() {
        assert_eq!(
            RegExpProgram::compile(r"\u{41}", "u")
                .expect("`\\u{41}` must compile")
                .instructions,
            vec![
                RegExpInstruction::literal_ascii(b'A'),
                RegExpInstruction::accept(),
            ]
        );
        assert_eq!(
            RegExpProgram::compile(r"\u{1F600}", "v")
                .expect("an astral braced escape must compile")
                .instructions,
            vec![
                RegExpInstruction::literal_code_point(0x1_f600),
                RegExpInstruction::accept(),
            ]
        );
        // The class path, which always had this alternative, must still agree.
        assert!(RegExpProgram::compile(r"[\u{41}]", "u").is_ok());

        // Out of range stays a Syntax Error: the fix added the production, it
        // did not delete its early error.
        assert_eq!(
            RegExpProgram::compile(r"\u{110000}", "u").unwrap_err().rule,
            Some(SyntaxRule::CodePointEscapeRange)
        );
        // And in legacy mode `\u{41}` is still the Annex B identity escape `u`
        // followed by a literal brace group, not a code point.
        assert!(RegExpProgram::compile(r"\u{41}", "").is_ok());
    }

    /// One pinned `(pattern, flags)` witness per [`SyntaxRule`], asserted to
    /// produce that rule.
    ///
    /// This is the audit's standing form. A rejection site is a claim that a
    /// conforming engine refuses the pattern, and the cheapest way to keep that
    /// claim honest is to require a pattern that demonstrates it. A new
    /// `SyntaxRule` variant fails the exhaustive `match` in
    /// [`SyntaxRule::citation`]; once added there it fails HERE until it has a
    /// witness. A site that cannot produce one is not an `InvalidSyntax` site.
    ///
    /// Each witness is asserted to produce its OWN rule, not merely some
    /// `InvalidSyntax`, so a witness pattern that starts being answered by a
    /// different rule fails here rather than being silently absorbed.
    ///
    /// **This is per-RULE, not per-SITE, and it is not a site map.** 66
    /// `invalid_syntax` call sites map onto 24 rules and 24 witnesses:
    /// `RegExpIdentifierName` alone has 14 sites sharing one witness,
    /// `UnclosedCharacterClass` has 9, `CharacterEscape` has 3. A site given the
    /// WRONG variant is invisible to this test whenever some other site already
    /// witnesses both rules — which is exactly how `parse_modifier_group_prefix`
    /// shipped an `UnclosedGroup` citation on a `ModifierFlags` violation. Since
    /// `Display` puts `citation()` on the product path, that class of error is
    /// user-visible; a message-versus-citation read of each site is the only
    /// thing that catches it.
    #[test]
    fn every_syntax_rule_has_a_pinned_witness() {
        // (rule, pattern, flags). Written as one table rather than one test per
        // rule so that the coverage assertion below can be total.
        let witnesses: &[(SyntaxRule, &str, &str)] = &[
            (SyntaxRule::UnclosedGroup, "(a", ""),
            (SyntaxRule::StrayClosingParenthesis, "a)", ""),
            (SyntaxRule::ModifierFlags, "(?ii:a)", ""),
            (SyntaxRule::QuantifierWithoutAtom, "*", ""),
            (SyntaxRule::QuantifierAfterQuantifier, "a**", ""),
            (SyntaxRule::QuantifierBounds, "a{2,1}", ""),
            (SyntaxRule::UnescapedSyntaxCharacter, "{", "u"),
            (SyntaxRule::NamedBackreferenceSyntax, r"\ka", "u"),
            (SyntaxRule::UnknownGroupName, r"\k<missing>", "u"),
            (SyntaxRule::DuplicateGroupName, "(?<x>a)(?<x>b)", ""),
            (SyntaxRule::RegExpIdentifierName, "(?<1>a)", ""),
            (SyntaxRule::Flags, "a", "gg"),
            (SyntaxRule::CharacterEscape, "\\", ""),
            (SyntaxRule::IdentityEscape, r"\q", "u"),
            // The `\u0041` is load-bearing, not decoration. Written `[\q]`,
            // `class_needs_code_point_ranges` sees no `\p \P \u \x \w \W \D \S`
            // and no non-ASCII byte, picks `parse_ascii_class`, and that parser
            // does not apply the Unicode-mode `ClassEscape` rule at all -- so
            // `[\q]` under `u` compiles today. That divergence is the
            // under-rejection recorded in
            // `class_verdicts_do_not_depend_on_the_class_representation`.
            (SyntaxRule::ClassEscape, r"[\q\u0041]", "u"),
            (SyntaxRule::HexEscapeSequence, r"\xZZ", "u"),
            (SyntaxRule::UnicodeEscapeSequence, r"\uZZZZ", "u"),
            (SyntaxRule::CodePointEscape, r"\u{}", "u"),
            (SyntaxRule::CodePointEscapeRange, r"\u{110000}", "u"),
            (SyntaxRule::UnicodePropertyEscape, r"\p{ASCII", "u"),
            (SyntaxRule::UnicodePropertyName, r"\p{NotAProperty}", "u"),
            (SyntaxRule::UnclosedCharacterClass, "[a", ""),
            (SyntaxRule::ClassRangeOrder, "[z-a]", ""),
            (SyntaxRule::ClassRangeBound, r"[\D-a]", "u"),
        ];

        for (rule, pattern, flags) in witnesses {
            let Err(error) = RegExpProgram::compile(pattern, flags) else {
                panic!("`{pattern}` under `{flags}` must be rejected by {rule:?}");
            };
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::InvalidSyntax,
                "`{pattern}` under `{flags}`: {error}"
            );
            assert_eq!(
                error.rule,
                Some(*rule),
                "`{pattern}` under `{flags}` reached the wrong site: {error}"
            );
            assert!(
                !rule.citation().is_empty(),
                "{rule:?} must cite a production"
            );
        }

        for rule in SyntaxRule::ALL {
            assert!(
                witnesses.iter().any(|(witness, ..)| *witness == rule),
                "{rule:?} has no pinned witness. A rejection site whose rule \
                 cannot be demonstrated by a pattern is an UnsupportedFeature, \
                 not an InvalidSyntax -- see the SyntaxRule doc comment."
            );
        }
    }

    /// [`SyntaxRule::ALL`] is hand-maintained, so it is checked rather than
    /// trusted: sorted, duplicate-free, and the length the constant declares.
    ///
    /// It cannot be derived, and a copy-paste omission there would silently
    /// exempt a rule from the witness table above. Sorted-and-unique is the
    /// strongest property available without a derive.
    #[test]
    fn all_syntax_rules_are_listed_once() {
        let mut deduplicated = SyntaxRule::ALL.to_vec();
        deduplicated.sort_unstable();
        deduplicated.dedup();
        assert_eq!(
            deduplicated.len(),
            SyntaxRule::ALL.len(),
            "SyntaxRule::ALL contains a duplicate, so some rule is exempt from \
             the witness table"
        );
        // Declaration order is `Ord` here, so "sorted" means "listed in the
        // order the enum declares". A variant appended to the enum but inserted
        // in the middle of `ALL` -- or forgotten and then noticed later -- shows
        // up as an ordering failure rather than as a silent gap.
        assert!(
            SyntaxRule::ALL.windows(2).all(|pair| pair[0] < pair[1]),
            "SyntaxRule::ALL must list every variant once, in declaration order"
        );
        // Every rule's citation must be non-empty and must name a clause. This
        // is the cheap half of the audit: the expensive half is the witness.
        for rule in SyntaxRule::ALL {
            let citation = rule.citation();
            assert!(
                citation.starts_with("22.2."),
                "{rule:?} cites `{citation}`, which is not an ECMA-262 clause \
                 number. A rejection that cannot name one is an \
                 UnsupportedFeature."
            );
        }
    }

    /// The parser picks between an ASCII bitmap class and a code-point range
    /// class by inspecting the class body ([`class_needs_code_point_ranges`]),
    /// and the two are separate parsers with separate escape handling. A
    /// pattern's VERDICT must not depend on which one the compiler chose.
    ///
    /// Appending `A` adds `A` to the set and nothing else, and the `\u`
    /// is what forces the code-point path, so each witness is compiled through
    /// both representations and the two verdicts are compared.
    ///
    /// Positive witnesses only, deliberately. The negative direction diverges
    /// at this head — `[\q]` under `u` is accepted by the ASCII path and
    /// rejected by the code-point path — and that is an UNDER-rejection (a
    /// missing SyntaxError), the opposite of the over-rejection this lane owns.
    /// It is recorded in `target/lane-notes/re-verdict-b8-integration.md` rather
    /// than fixed blind here, because closing it means gating `\c<digit>`,
    /// legacy octal and the identity-escape fallthrough in
    /// `parse_ascii_class_atom`, which is a matcher-path change with its own
    /// blast radius.
    #[test]
    fn class_verdicts_do_not_depend_on_the_class_representation() {
        let bodies = [
            r"\/", r"\^", r"\$", r"\\", r"\.", r"\*", r"\+", r"\?", r"\(", r"\)", r"\[", r"\]",
            r"\{", r"\}", r"\|", r"\-", r"\b", r"\d", r"a-f", "abc",
        ];
        for body in bodies {
            // The premise the test's NAME rests on, asserted rather than
            // assumed: the two spellings must reach two different parsers.
            // `class_needs_code_point_ranges` is what routes them, and adding
            // any of `d`, `b`, `/` or `-` to its trigger set — a plausible edit,
            // since `\d` is already modelled differently by the two parsers —
            // would send BOTH forms to `parse_class` and leave every assertion
            // below green while compiling one parser twice.
            let bitmap_source = format!("[{body}]");
            let ranges_source = format!("[{body}\\u0041]");
            assert!(
                !class_needs_code_point_ranges(bitmap_source.as_bytes(), 0),
                "`{bitmap_source}` must take the ASCII bitmap parser"
            );
            assert!(
                class_needs_code_point_ranges(ranges_source.as_bytes(), 0),
                "`{ranges_source}` must take the code-point range parser"
            );
            for flags in ["", "u"] {
                let bitmap = RegExpProgram::compile(&bitmap_source, flags);
                let ranges = RegExpProgram::compile(&ranges_source, flags);
                assert!(
                    bitmap.is_ok(),
                    "`[{body}]` under `{flags}` must compile: {:?}",
                    bitmap.unwrap_err()
                );
                assert!(
                    ranges.is_ok(),
                    "`[{body}\\u0041]` under `{flags}` must compile through the \
                     code-point path too: {:?}",
                    ranges.unwrap_err()
                );
            }
            // `v` has one parser and no representation choice, but it must not
            // disagree with the other two about a legal class either.
            assert!(
                RegExpProgram::compile(&format!("[{body}]"), "v").is_ok(),
                "`[{body}]` under `v` must compile"
            );
        }
    }
}
