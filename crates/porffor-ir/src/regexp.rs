use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

use regress::Regex;

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
/// Match membership in one of the supported Unicode properties.
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

/// The Unicode `ASCII` binary property.
pub const REGEXP_UNICODE_PROPERTY_ASCII: u64 = 0;
/// The complement of the Unicode `ASCII` binary property.
pub const REGEXP_UNICODE_PROPERTY_NOT_ASCII: u64 = 1;
/// The Unicode `Script=Han` property.
pub const REGEXP_UNICODE_PROPERTY_SCRIPT_HAN: u64 = 2;

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

    pub const fn unicode_property(property: u64) -> Self {
        assert!(property <= REGEXP_UNICODE_PROPERTY_SCRIPT_HAN);
        Self {
            opcode: REGEXP_OPCODE_UNICODE_PROPERTY,
            operand0: property,
            operand1: 0,
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

/// RegExp flags that affect matching wrappers rather than match instructions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegExpFlags {
    pub has_indices: bool,
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub dot_all: bool,
    pub sticky: bool,
    pub unicode: bool,
    pub unicode_sets: bool,
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
}

impl RegExpProgram {
    pub fn compile(pattern: &str, flags: &str) -> Result<Self, RegExpCompileError> {
        let flags = parse_flags(flags)?;
        let parsed = parse_pattern(
            pattern,
            flags.unicode || flags.unicode_sets,
            flags.unicode_sets,
        )?;
        let mut instructions = Vec::with_capacity(pattern.len() + 1);
        let mut lowerer =
            ProgramLowerer::new(&mut instructions, pattern.len(), &parsed.named_groups);
        lowerer.alternatives(&parsed.alternatives)?;
        lowerer.error_offset = pattern.len();
        lowerer.push(RegExpInstruction::accept())?;
        if flags.ignore_case {
            apply_ascii_ignore_case(&mut instructions);
        }
        Ok(Self {
            flags,
            capture_count: parsed.capture_count,
            named_groups: parsed.named_groups,
            instructions,
        })
    }

    /// Encodes only match instructions. Flags remain wrapper behavior.
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(self.instructions.len() * REGEXP_INSTRUCTION_WIDTH);
        for instruction in &self.instructions {
            encoded.extend_from_slice(&instruction.opcode.to_le_bytes());
            encoded.extend_from_slice(&instruction.operand0.to_le_bytes());
            encoded.extend_from_slice(&instruction.operand1.to_le_bytes());
        }
        encoded
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegExpCompileErrorKind {
    InvalidSyntax,
    UnsupportedFeature,
}

/// A compile failure with the byte offset in the pattern or flag string supplied
/// to [`RegExpProgram::compile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExpCompileError {
    pub kind: RegExpCompileErrorKind,
    pub offset: usize,
    pub message: String,
}

impl RegExpCompileError {
    fn invalid_syntax(offset: usize, message: impl Into<String>) -> Self {
        Self {
            kind: RegExpCompileErrorKind::InvalidSyntax,
            offset,
            message: message.into(),
        }
    }

    fn unsupported_feature(offset: usize, message: impl Into<String>) -> Self {
        Self {
            kind: RegExpCompileErrorKind::UnsupportedFeature,
            offset,
            message: message.into(),
        }
    }
}

impl fmt::Display for RegExpCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl Error for RegExpCompileError {}

struct ParsedPattern {
    alternatives: Vec<Vec<ParsedTerm>>,
    capture_count: u32,
    named_groups: Vec<RegExpNamedGroup>,
}

struct ParsedTerm {
    atom: ParsedAtom,
    quantifier: Quantifier,
    quantifier_offset: usize,
}

enum ParsedAtom {
    Instruction(RegExpInstruction),
    InstructionSequence(Vec<RegExpInstruction>),
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
    unicode: bool,
    unicode_sets: bool,
) -> Result<ParsedPattern, RegExpCompileError> {
    if pattern.is_empty() {
        return Ok(ParsedPattern {
            alternatives: vec![Vec::new()],
            capture_count: 0,
            named_groups: Vec::new(),
        });
    }

    let (total_capture_count, has_named_capture_syntax) = regexp_capture_syntax(pattern.as_bytes());
    let mut parser = PatternParser {
        bytes: pattern.as_bytes(),
        offset: 0,
        capture_count: 0,
        unicode,
        unicode_sets,
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
    })
}

struct PatternParser<'a> {
    bytes: &'a [u8],
    offset: usize,
    capture_count: u32,
    unicode: bool,
    unicode_sets: bool,
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
                            opening,
                            "regular-expression capturing group is unclosed",
                        ));
                    }
                    break;
                }
                Some(b')') => {
                    if opening.is_none() {
                        return Err(RegExpCompileError::invalid_syntax(
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
            ParsedAtom::Instruction(if negative {
                RegExpInstruction::negative_ascii_lookahead(code_unit)
            } else {
                RegExpInstruction::positive_ascii_lookahead(code_unit)
            })
        } else if self.bytes[self.offset] == b'(' {
            if self.bytes.get(self.offset + 1) == Some(&b'?') {
                match self.bytes.get(self.offset + 2).copied() {
                    Some(b':') => {
                        self.offset += 3;
                        let subtree_start = self.capture_count + 1;
                        let body = self.alternatives(Some(atom_offset))?;
                        ParsedAtom::NonCapture {
                            body,
                            subtree_start,
                            subtree_end: self.capture_count + 1,
                        }
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
                            ParsedAtom::Lookbehind { negative, body }
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
                            ParsedAtom::Capture {
                                id,
                                body,
                                subtree_end: self.capture_count + 1,
                            }
                        }
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
                ParsedAtom::Capture {
                    id,
                    body,
                    subtree_end: self.capture_count + 1,
                }
            }
        } else {
            let atom = parse_instruction_atom(
                self.bytes,
                &mut self.offset,
                self.unicode,
                self.unicode_sets,
                self.total_capture_count,
                self.has_named_capture_syntax,
            )?;
            match atom {
                ParsedAtom::NumberedBackreference {
                    capture_id,
                    nullable: _,
                } => ParsedAtom::NumberedBackreference {
                    capture_id,
                    nullable: self
                        .capture_nullability
                        .get(&capture_id)
                        .copied()
                        .unwrap_or(true),
                },
                atom => atom,
            }
        };
        let quantifier_offset = self.offset;
        let mut quantifier = parse_postfix_quantifier(self.bytes, &mut self.offset)?;
        if matches!(
            atom,
            ParsedAtom::Instruction(RegExpInstruction {
                opcode: REGEXP_OPCODE_POSITIVE_ASCII_LOOKAHEAD
                    | REGEXP_OPCODE_NEGATIVE_ASCII_LOOKAHEAD
                    | REGEXP_OPCODE_ASSERT_START
                    | REGEXP_OPCODE_ASSERT_END,
                ..
            })
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
        if matches!(atom, ParsedAtom::InstructionSequence(_)) && self.offset != quantifier_offset {
            return Err(RegExpCompileError::unsupported_feature(
                quantifier_offset,
                "postfix quantifiers on direct astral source are unsupported in non-Unicode mode",
            ));
        }
        if quantifier.max.is_none() && atom_nullable(&atom) {
            return Err(RegExpCompileError::unsupported_feature(
                quantifier_offset,
                "unbounded quantifier over a nullable atom is unsupported by this matcher-program grammar",
            ));
        }
        Ok(ParsedTerm {
            atom,
            quantifier,
            quantifier_offset,
        })
    }

    fn parse_group_name(&mut self) -> Result<String, RegExpCompileError> {
        let start = self.offset + 3;
        let (name, end) = parse_regexp_identifier_name(self.bytes, start, "named capture group")?;
        self.offset = end;
        Ok(name)
    }
}

fn parse_instruction_atom(
    bytes: &[u8],
    offset: &mut usize,
    unicode: bool,
    unicode_sets: bool,
    total_capture_count: u32,
    has_named_capture_syntax: bool,
) -> Result<ParsedAtom, RegExpCompileError> {
    let atom_offset = *offset;
    let byte = bytes[atom_offset];
    if bytes.get(atom_offset..atom_offset + 2) == Some(b"\\k") {
        if !unicode && !has_named_capture_syntax {
            *offset += 2;
            return Ok(ParsedAtom::Instruction(RegExpInstruction::literal_ascii(
                b'k',
            )));
        }
        if bytes.get(atom_offset + 2) != Some(&b'<') {
            return Err(RegExpCompileError::invalid_syntax(
                atom_offset,
                "malformed named backreference",
            ));
        }
        let (name, end) =
            parse_regexp_identifier_name(bytes, atom_offset + 3, "named backreference")?;
        *offset = end;
        return Ok(ParsedAtom::NamedBackreference {
            name,
            offset: atom_offset,
        });
    }
    if byte == b'\\' {
        if let Some(digit @ b'1'..=b'9') = bytes.get(atom_offset + 1).copied() {
            let capture_id = u32::from(digit - b'0');
            if capture_id <= total_capture_count {
                *offset += 2;
                return Ok(ParsedAtom::NumberedBackreference {
                    capture_id,
                    nullable: true,
                });
            }
        }
    }
    if !byte.is_ascii() {
        if unicode {
            let source = std::str::from_utf8(&bytes[atom_offset..]).map_err(|_| {
                RegExpCompileError::invalid_syntax(
                    atom_offset,
                    "regular-expression source is not valid UTF-8",
                )
            })?;
            let ch = source.chars().next().expect("non-empty source");
            *offset += ch.len_utf8();
            return Ok(ParsedAtom::Instruction(
                RegExpInstruction::literal_code_point(ch as u32),
            ));
        }
        let source = std::str::from_utf8(&bytes[atom_offset..]).map_err(|_| {
            RegExpCompileError::invalid_syntax(
                atom_offset,
                "regular-expression source is not valid UTF-8",
            )
        })?;
        let ch = source.chars().next().expect("non-empty source");
        *offset += ch.len_utf8();
        let code_point = ch as u32;
        if code_point <= 0xffff {
            return Ok(ParsedAtom::Instruction(
                RegExpInstruction::literal_code_point(code_point),
            ));
        }
        let supplementary = code_point - 0x1_0000;
        return Ok(ParsedAtom::InstructionSequence(vec![
            RegExpInstruction::literal_code_point(0xD800 + (supplementary >> 10)),
            RegExpInstruction::literal_code_point(0xDC00 + (supplementary & 0x3ff)),
        ]));
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
                    atom_offset,
                    "regular-expression quantifier has no preceding atom",
                ));
            }
            if unicode {
                return Err(RegExpCompileError::invalid_syntax(
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
        b'[' if unicode_sets => {
            return Err(RegExpCompileError::unsupported_feature(
                atom_offset,
                "character classes are unsupported in Unicode-sets mode",
            ));
        }
        b'[' if unicode => {
            if let Some(instruction) = parse_single_unicode_class(bytes, offset)? {
                instruction
            } else {
                parse_ascii_class(bytes, offset)?
            }
        }
        b'[' => parse_ascii_class(bytes, offset)?,
        b'\\' => parse_escaped_atom(bytes, offset, unicode)?,
        b'.' => {
            *offset += 1;
            RegExpInstruction::dot()
        }
        b'*' | b'+' | b'?' => {
            return Err(RegExpCompileError::invalid_syntax(
                atom_offset,
                "regular-expression quantifier has no preceding atom",
            ));
        }
        byte if is_regex_metacharacter(byte) => {
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
    Ok(ParsedAtom::Instruction(instruction))
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
        ParsedAtom::InstructionSequence(_) => false,
        ParsedAtom::Capture { body, .. } | ParsedAtom::NonCapture { body, .. } => body
            .iter()
            .any(|sequence| sequence.iter().all(|term| term_nullable(term))),
        ParsedAtom::NamedBackreference { .. } => true,
        ParsedAtom::NumberedBackreference { nullable, .. } => *nullable,
        ParsedAtom::Lookbehind { .. } => true,
    }
}

fn lookbehind_body_supported(alternatives: &[Vec<ParsedTerm>]) -> bool {
    alternatives.iter().flatten().all(|term| match &term.atom {
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
        ParsedAtom::InstructionSequence(_)
        | ParsedAtom::NamedBackreference { .. }
        | ParsedAtom::NumberedBackreference { .. }
        | ParsedAtom::Lookbehind { .. } => false,
    })
}
fn term_nullable(term: &ParsedTerm) -> bool {
    term.quantifier.min == 0 || atom_nullable(&term.atom)
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

static REGEXP_ID_START_CLASSIFIER: OnceLock<Regex> = OnceLock::new();
static REGEXP_ID_CONTINUE_CLASSIFIER: OnceLock<Regex> = OnceLock::new();

fn ascii_hex_value(byte: u8) -> Option<u32> {
    let byte = byte.to_ascii_lowercase();
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a' + 10)),
        _ => None,
    }
}

fn unicode_property_contains(
    classifier: &'static OnceLock<Regex>,
    pattern: &'static str,
    code_point: char,
) -> bool {
    let classifier = classifier.get_or_init(|| {
        Regex::with_flags(pattern, "u")
            .unwrap_or_else(|error| panic!("Unicode classifier `{pattern}` must compile: {error}"))
    });
    let mut utf8 = [0_u8; 4];
    classifier.find(code_point.encode_utf8(&mut utf8)).is_some()
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
                cursor,
                format!("{description} identifier is unclosed"),
            ));
        };
        if byte == b'>' {
            if name.is_empty() {
                return Err(RegExpCompileError::invalid_syntax(
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
                        cursor,
                        format!("{description} contains an empty Unicode identifier escape"),
                    ));
                }
                loop {
                    let Some(&digit) = bytes.get(cursor) else {
                        return Err(RegExpCompileError::invalid_syntax(
                            cursor,
                            format!("{description} contains an unclosed Unicode identifier escape"),
                        ));
                    };
                    if digit == b'}' {
                        break;
                    }
                    let Some(digit) = ascii_hex_value(digit) else {
                        return Err(RegExpCompileError::invalid_syntax(
                            cursor,
                            format!("{description} contains a malformed Unicode identifier escape"),
                        ));
                    };
                    value = value
                        .checked_mul(16)
                        .and_then(|value| value.checked_add(digit))
                        .ok_or_else(|| {
                            RegExpCompileError::invalid_syntax(
                                code_point_offset,
                                format!("{description} Unicode identifier escape is out of range"),
                            )
                        })?;
                    cursor += 1;
                }
                cursor += 1;
                char::from_u32(value).ok_or_else(|| {
                    RegExpCompileError::invalid_syntax(
                        code_point_offset,
                        format!("{description} Unicode identifier escape is not a scalar value"),
                    )
                })?
            } else {
                let digits = bytes.get(cursor + 2..cursor + 6).ok_or_else(|| {
                    RegExpCompileError::invalid_syntax(
                        bytes.len(),
                        format!("{description} contains an incomplete Unicode identifier escape"),
                    )
                })?;
                let invalid_digit = digits.iter().position(|digit| !digit.is_ascii_hexdigit());
                if let Some(invalid_digit) = invalid_digit {
                    return Err(RegExpCompileError::invalid_syntax(
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
                                code_point_offset,
                                format!("{description} contains an unpaired lead surrogate escape"),
                            ));
                        }
                    } else {
                        return Err(RegExpCompileError::invalid_syntax(
                            code_point_offset,
                            format!("{description} contains an unpaired lead surrogate escape"),
                        ));
                    }
                } else if (0xDC00..=0xDFFF).contains(&high) {
                    return Err(RegExpCompileError::invalid_syntax(
                        code_point_offset,
                        format!("{description} contains an unpaired trail surrogate escape"),
                    ));
                } else {
                    char::from_u32(u32::from(high)).expect("non-surrogate u16 is a scalar")
                }
            }
        } else {
            let source = std::str::from_utf8(&bytes[cursor..]).map_err(|_| {
                RegExpCompileError::invalid_syntax(
                    cursor,
                    format!("{description} is not valid UTF-8"),
                )
            })?;
            let code_point = source.chars().next().expect("source is non-empty");
            cursor += code_point.len_utf8();
            code_point
        };

        let valid = if name.is_empty() {
            matches!(code_point, '$' | '_')
                || unicode_property_contains(
                    &REGEXP_ID_START_CLASSIFIER,
                    r"^\p{ID_Start}$",
                    code_point,
                )
        } else {
            matches!(code_point, '$' | '_' | '\u{200C}' | '\u{200D}')
                || unicode_property_contains(
                    &REGEXP_ID_CONTINUE_CLASSIFIER,
                    r"^\p{ID_Continue}$",
                    code_point,
                )
        };
        if !valid {
            let position = if name.is_empty() {
                "start"
            } else {
                "continuation"
            };
            return Err(RegExpCompileError::invalid_syntax(
                code_point_offset,
                format!(
                    "{description} code point U+{:04X} is not valid in identifier {position}",
                    u32::from(code_point)
                ),
            ));
        }
        name.push(code_point);
    }
}

fn parse_flags(flags: &str) -> Result<RegExpFlags, RegExpCompileError> {
    let mut parsed = RegExpFlags::default();
    let mut seen_flags = 0_u8;
    let mut first_unsupported = None;
    for (offset, byte) in flags.bytes().enumerate() {
        if !byte.is_ascii() {
            return Err(RegExpCompileError::invalid_syntax(
                offset,
                "regular-expression flags must be ASCII",
            ));
        }

        let flag_bit = match byte {
            b'd' => 1 << 0,
            b'g' => 1 << 1,
            b'i' => 1 << 2,
            b'm' => 1 << 3,
            b's' => 1 << 4,
            b'u' => 1 << 5,
            b'v' => 1 << 6,
            b'y' => 1 << 7,
            byte => {
                return Err(RegExpCompileError::invalid_syntax(
                    offset,
                    format!("unknown regular-expression flag `{}`", byte as char),
                ));
            }
        };
        if seen_flags & flag_bit != 0 {
            return Err(RegExpCompileError::invalid_syntax(
                offset,
                format!("duplicate regular-expression flag `{}`", byte as char),
            ));
        }
        if (byte == b'u' && seen_flags & (1 << 6) != 0)
            || (byte == b'v' && seen_flags & (1 << 5) != 0)
        {
            return Err(RegExpCompileError::invalid_syntax(
                offset,
                "regular-expression flags `u` and `v` are mutually exclusive",
            ));
        }
        seen_flags |= flag_bit;

        match byte {
            b'd' => parsed.has_indices = true,
            b'g' => parsed.global = true,
            b'i' => parsed.ignore_case = true,
            b'm' => parsed.multiline = true,
            b's' => parsed.dot_all = true,
            b'y' => parsed.sticky = true,
            b'u' => parsed.unicode = true,
            b'v' => parsed.unicode_sets = true,
            byte if first_unsupported.is_none() => first_unsupported = Some((offset, byte)),
            _ => {}
        }
    }

    if let Some((offset, byte)) = first_unsupported {
        return Err(RegExpCompileError::unsupported_feature(
            offset,
            format!("unsupported regular-expression flag `{}`", byte as char),
        ));
    }
    Ok(parsed)
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
    unicode: bool,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let escape_offset = *offset;
    let Some(&escaped) = bytes.get(escape_offset + 1) else {
        return Err(RegExpCompileError::invalid_syntax(
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
        return parse_unicode_property_escape(bytes, offset);
    }
    if escaped == b'u' {
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
    if !is_regex_metacharacter(escaped) {
        if unicode {
            return Err(RegExpCompileError::invalid_syntax(
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

fn parse_unicode_property_escape(
    bytes: &[u8],
    offset: &mut usize,
) -> Result<RegExpInstruction, RegExpCompileError> {
    let escape_offset = *offset;
    let complement = bytes[escape_offset + 1] == b'P';
    if bytes.get(escape_offset + 2) != Some(&b'{') {
        return Err(RegExpCompileError::invalid_syntax(
            escape_offset,
            "malformed Unicode property escape",
        ));
    }
    let value_start = escape_offset + 3;
    let Some(relative_end) = bytes[value_start..].iter().position(|byte| *byte == b'}') else {
        return Err(RegExpCompileError::invalid_syntax(
            escape_offset,
            "malformed Unicode property escape",
        ));
    };
    let value_end = value_start + relative_end;
    let value = &bytes[value_start..value_end];
    if value.is_empty() {
        return Err(RegExpCompileError::invalid_syntax(
            escape_offset,
            "malformed Unicode property escape",
        ));
    }

    let property = match (complement, value) {
        (false, b"ASCII") => REGEXP_UNICODE_PROPERTY_ASCII,
        (true, b"ASCII") => REGEXP_UNICODE_PROPERTY_NOT_ASCII,
        (false, b"Script=Han") => REGEXP_UNICODE_PROPERTY_SCRIPT_HAN,
        _ => {
            return Err(RegExpCompileError::unsupported_feature(
                escape_offset,
                "unsupported Unicode property escape",
            ));
        }
    };
    *offset = value_end + 1;
    Ok(RegExpInstruction::unicode_property(property))
}

fn parse_unicode_escape(bytes: &[u8], start: usize) -> Result<(u16, usize), RegExpCompileError> {
    if bytes.get(start..start + 2) != Some(b"\\u") {
        return Err(RegExpCompileError::invalid_syntax(
            start,
            "malformed Unicode escape",
        ));
    }
    let digits = bytes
        .get(start + 2..start + 6)
        .ok_or_else(|| RegExpCompileError::invalid_syntax(start, "malformed Unicode escape"))?;
    if digits.len() != 4 || !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(RegExpCompileError::invalid_syntax(
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
                    class_offset,
                    "regular-expression character class is unclosed",
                ));
            }
            let range_end = parse_ascii_class_atom(bytes, &mut cursor)?;
            match (range_start.singleton, range_end.singleton) {
                (Some(start), Some(end)) if end < start => {
                    return Err(RegExpCompileError::invalid_syntax(
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
            class_offset,
            "regular-expression character class is unclosed",
        ));
    };
    let end = class_offset + 1 + relative_end;
    let source = std::str::from_utf8(&bytes[class_offset + 1..end]).map_err(|_| {
        RegExpCompileError::invalid_syntax(
            class_offset,
            "regular-expression class source is not valid UTF-8",
        )
    })?;
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
            self.quantified(&term.atom, term.quantifier, term.quantifier_offset)?;
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
            ParsedAtom::InstructionSequence(instructions) => {
                for instruction in instructions {
                    self.push(*instruction)?;
                }
                Ok(())
            }
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
            self.reverse_quantified(&term.atom, term.quantifier, term.quantifier_offset)?;
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

fn is_regex_metacharacter(byte: u8) -> bool {
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
                    unicode: false,
                    unicode_sets: false,
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
        assert!(unicode.flags.unicode);
        assert!(!unicode.flags.unicode_sets);
        let unicode_sets = RegExpProgram::compile("𠮷", "v").unwrap();
        assert!(!unicode_sets.flags.unicode);
        assert!(unicode_sets.flags.unicode_sets);
        assert_eq!(unicode.instructions, unicode_sets.instructions);
    }

    #[test]
    fn compiles_exact_supported_unicode_properties() {
        for flags in ["u", "v"] {
            let program =
                RegExpProgram::compile(r"\p{ASCII}\P{ASCII}\p{Script=Han}", flags).unwrap();
            assert_eq!(
                program.instructions,
                vec![
                    RegExpInstruction::unicode_property(REGEXP_UNICODE_PROPERTY_ASCII),
                    RegExpInstruction::unicode_property(REGEXP_UNICODE_PROPERTY_NOT_ASCII),
                    RegExpInstruction::unicode_property(REGEXP_UNICODE_PROPERTY_SCRIPT_HAN),
                    RegExpInstruction::accept(),
                ]
            );
            let encoded = program.encode();
            assert_eq!(&encoded[..8], &REGEXP_OPCODE_UNICODE_PROPERTY.to_le_bytes());
            assert_eq!(
                &encoded[8..16],
                &REGEXP_UNICODE_PROPERTY_ASCII.to_le_bytes()
            );
            assert_eq!(&encoded[16..24], &[0; 8]);
        }
    }

    #[test]
    fn rejects_unsupported_and_malformed_unicode_properties() {
        for pattern in [r"\p{Letter}", r"\P{Script=Han}", r"\p{script=Han}"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "u").unwrap_err().kind,
                RegExpCompileErrorKind::UnsupportedFeature,
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
    fn rejects_character_classes_in_unicode_sets_mode() {
        let error = RegExpProgram::compile("[a]", "v").unwrap_err();
        assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
        assert_eq!(error.offset, 0);
    }

    #[test]
    fn direct_non_unicode_source_uses_utf16_code_units() {
        assert_eq!(
            RegExpProgram::compile("é𠮷", "").unwrap().instructions,
            vec![
                RegExpInstruction::literal_code_point(0xE9),
                RegExpInstruction::literal_code_point(0xD842),
                RegExpInstruction::literal_code_point(0xDFB7),
                RegExpInstruction::accept(),
            ]
        );
        for pattern in ["𠮷?", "𠮷{1}", "𠮷+"] {
            assert_eq!(
                RegExpProgram::compile(pattern, "").unwrap_err().kind,
                RegExpCompileErrorKind::UnsupportedFeature,
                "{pattern}"
            );
        }
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

    #[test]
    fn distinguishes_duplicate_unknown_and_unsupported_flags() {
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
        assert!(flags.unicode_sets);
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
}
