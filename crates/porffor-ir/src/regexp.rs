use std::error::Error;
use std::fmt;

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

    pub const fn positive_ascii_class(bitmap_low: u64, bitmap_high: u64) -> Self {
        Self {
            opcode: REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
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
    pub global: bool,
    pub sticky: bool,
}

/// A compiled, backend-neutral regular-expression matcher program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExpProgram {
    pub flags: RegExpFlags,
    pub instructions: Vec<RegExpInstruction>,
}

impl RegExpProgram {
    pub fn compile(pattern: &str, flags: &str) -> Result<Self, RegExpCompileError> {
        let flags = parse_flags(flags)?;
        let bytes = pattern.as_bytes();
        if bytes.is_empty() {
            return Err(RegExpCompileError::unsupported_feature(
                0,
                "empty regular-expression patterns are unsupported by this matcher-program grammar",
            ));
        }

        let mut instructions = Vec::with_capacity(bytes.len() + 1);
        let mut offset = 0;
        while offset < bytes.len() {
            let byte = bytes[offset];
            if !byte.is_ascii() {
                return Err(RegExpCompileError::unsupported_feature(
                    offset,
                    "non-ASCII regular-expression source is unsupported by this matcher-program grammar",
                ));
            }

            let atom = match byte {
                b'{' => {
                    // A complete braced quantifier cannot appear without a
                    // preceding atom. Incomplete/non-decimal forms are legacy
                    // literal braces in non-Unicode mode.
                    let mut probe = offset;
                    if parse_braced_quantifier(bytes, &mut probe)?.is_some() {
                        return Err(RegExpCompileError::invalid_syntax(
                            offset,
                            "regular-expression quantifier has no preceding atom",
                        ));
                    }
                    offset += 1;
                    RegExpInstruction::literal_ascii(byte)
                }
                b'}' => {
                    offset += 1;
                    RegExpInstruction::literal_ascii(byte)
                }
                b'[' => parse_positive_ascii_class(bytes, &mut offset)?,
                b'\\' => parse_escaped_atom(bytes, &mut offset)?,
                byte if is_regex_metacharacter(byte) => {
                    return Err(RegExpCompileError::unsupported_feature(
                        offset,
                        format!(
                            "unsupported regular-expression metacharacter `{}`",
                            byte as char
                        ),
                    ));
                }
                byte => {
                    offset += 1;
                    RegExpInstruction::literal_ascii(byte)
                }
            };
            let quantifier_offset = offset;
            let quantifier = parse_postfix_quantifier(bytes, &mut offset)?;
            emit_quantified_atom(&mut instructions, atom, quantifier, quantifier_offset)?;
        }

        instructions.push(RegExpInstruction::accept());
        Ok(Self {
            flags,
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
            b'g' => parsed.global = true,
            b'y' => parsed.sticky = true,
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

fn parse_escaped_atom(
    bytes: &[u8],
    offset: &mut usize,
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
    if escaped == b'd' {
        let mut bitmap_low = 0;
        let mut bitmap_high = 0;
        add_ascii_range(&mut bitmap_low, &mut bitmap_high, b'0', b'9');
        *offset += 2;
        return Ok(RegExpInstruction::positive_ascii_class(
            bitmap_low,
            bitmap_high,
        ));
    }
    if !is_regex_metacharacter(escaped) {
        return Err(RegExpCompileError::unsupported_feature(
            escape_offset,
            format!(
                "unsupported regular-expression escape `\\{}`",
                escaped as char
            ),
        ));
    }

    *offset += 2;
    Ok(RegExpInstruction::literal_ascii(escaped))
}

fn parse_positive_ascii_class(
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
    if first == b'^' {
        return Err(RegExpCompileError::unsupported_feature(
            cursor,
            "negated regular-expression character classes are unsupported",
        ));
    }
    if first == b']' {
        return Err(RegExpCompileError::unsupported_feature(
            class_offset,
            "empty regular-expression character classes are unsupported by this matcher-program grammar",
        ));
    }

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
        validate_class_member(member, cursor)?;
        cursor += 1;

        if bytes.get(cursor) == Some(&b'-') && bytes.get(cursor + 1) != Some(&b']') {
            let range_offset = cursor;
            cursor += 1;
            let Some(&range_end) = bytes.get(cursor) else {
                return Err(RegExpCompileError::invalid_syntax(
                    class_offset,
                    "regular-expression character class is unclosed",
                ));
            };
            validate_class_member(range_end, cursor)?;
            if range_end < member {
                return Err(RegExpCompileError::invalid_syntax(
                    range_offset,
                    "regular-expression character class range is reversed",
                ));
            }
            add_ascii_range(&mut bitmap_low, &mut bitmap_high, member, range_end);
            cursor += 1;
        } else {
            add_ascii_member(&mut bitmap_low, &mut bitmap_high, member);
        }
    }

    *offset = cursor + 1;
    Ok(RegExpInstruction::positive_ascii_class(
        bitmap_low,
        bitmap_high,
    ))
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
                })
            }
        },
        _ => {
            return Ok(Quantifier {
                min: 1,
                max: Some(1),
                lazy: false,
            })
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

fn emit_quantified_atom(
    instructions: &mut Vec<RegExpInstruction>,
    atom: RegExpInstruction,
    quantifier: Quantifier,
    offset: usize,
) -> Result<(), RegExpCompileError> {
    let optional = quantifier
        .max
        .map_or(0, |max| max.saturating_sub(quantifier.min));
    let additional = quantifier
        .min
        .checked_add(optional.checked_mul(2).unwrap_or(usize::MAX))
        .and_then(|n| n.checked_add(quantifier.max.is_none() as usize * 3))
        .unwrap_or(usize::MAX);
    if instructions
        .len()
        .checked_add(additional)
        .and_then(|n| n.checked_add(1))
        .filter(|&n| n <= REGEXP_MAX_INSTRUCTIONS)
        .is_none()
    {
        return Err(RegExpCompileError::unsupported_feature(offset, format!("regular-expression quantifier expands beyond the {REGEXP_MAX_INSTRUCTIONS}-instruction matcher-program limit")));
    }
    for _ in 0..quantifier.min {
        instructions.push(atom);
    }
    match quantifier.max {
        Some(max) => {
            for _ in quantifier.min..max {
                emit_optional(instructions, atom, quantifier.lazy);
            }
        }
        None => emit_star(instructions, atom, quantifier.lazy),
    }
    Ok(())
}

fn emit_optional(instructions: &mut Vec<RegExpInstruction>, atom: RegExpInstruction, lazy: bool) {
    let split = instructions.len();
    instructions.push(RegExpInstruction::split(0, 0));
    let atom_pc = instructions.len();
    instructions.push(atom);
    let after = instructions.len();
    instructions[split] = if lazy {
        RegExpInstruction::split(after, atom_pc)
    } else {
        RegExpInstruction::split(atom_pc, after)
    };
}
fn emit_star(instructions: &mut Vec<RegExpInstruction>, atom: RegExpInstruction, lazy: bool) {
    let split = instructions.len();
    instructions.push(RegExpInstruction::split(0, 0));
    let atom_pc = instructions.len();
    instructions.push(atom);
    instructions.push(RegExpInstruction::jump(split));
    let after = instructions.len();
    instructions[split] = if lazy {
        RegExpInstruction::split(after, atom_pc)
    } else {
        RegExpInstruction::split(atom_pc, after)
    };
}

fn validate_class_member(member: u8, offset: usize) -> Result<(), RegExpCompileError> {
    if !member.is_ascii() {
        return Err(RegExpCompileError::unsupported_feature(
            offset,
            "non-ASCII regular-expression source is unsupported by this matcher-program grammar",
        ));
    }
    if member == b'\\' {
        return Err(RegExpCompileError::unsupported_feature(
            offset,
            "regular-expression character class escapes are unsupported",
        ));
    }
    Ok(())
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
    fn neighboring_escape_forms_remain_unsupported() {
        for pattern in [r"\D", r"\s", r"\w", r"\p{Decimal_Number}"] {
            let error = RegExpProgram::compile(pattern, "").expect_err(pattern);
            assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
            assert_eq!(error.offset, 0, "{pattern}");
        }
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
        for pattern in ["a{4,2}", "a**", "a+*"] {
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
                    global: true,
                    sticky: true
                }
            );
            assert_eq!(
                program.instructions,
                vec![
                    RegExpInstruction::literal_ascii(b'a'),
                    RegExpInstruction::accept()
                ]
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

        let unsupported_cases = [
            ("", 0),
            ("[]", 0),
            ("é", 0),
            (".", 0),
            ("[^a]", 1),
            (r"[\d]", 1),
        ];
        for (pattern, offset) in unsupported_cases {
            let error =
                RegExpProgram::compile(pattern, "").expect_err("pattern should be unsupported");
            assert_eq!(
                error.kind,
                RegExpCompileErrorKind::UnsupportedFeature,
                "{pattern}"
            );
            assert_eq!(error.offset, offset, "{pattern}");
        }
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

        let error = RegExpProgram::compile("a", "dimsv")
            .expect_err("recognized flags should be unsupported");
        assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
        assert_eq!(error.offset, 0);
    }
}
