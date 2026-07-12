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

    pub const fn positive_ascii_class_contains(self, code_unit: u8) -> bool {
        if self.opcode != REGEXP_OPCODE_POSITIVE_ASCII_CLASS {
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

            match byte {
                b'[' => parse_positive_ascii_class(bytes, &mut offset, &mut instructions)?,
                b'\\' => parse_escaped_literal(bytes, &mut offset, &mut instructions)?,
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
                    instructions.push(RegExpInstruction::literal_ascii(byte));
                    offset += 1;
                }
            }
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
    for (offset, byte) in flags.bytes().enumerate() {
        if !byte.is_ascii() {
            return Err(RegExpCompileError::invalid_syntax(
                offset,
                "regular-expression flags must be ASCII",
            ));
        }

        match byte {
            b'g' if !parsed.global && !parsed.sticky => parsed.global = true,
            b'y' if !parsed.sticky => parsed.sticky = true,
            b'g' | b'y' => {
                return Err(RegExpCompileError::invalid_syntax(
                    offset,
                    "regular-expression flags must be in canonical, non-duplicate `gy` order",
                ));
            }
            byte => {
                return Err(RegExpCompileError::unsupported_feature(
                    offset,
                    format!("unsupported regular-expression flag `{}`", byte as char),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_escaped_literal(
    bytes: &[u8],
    offset: &mut usize,
    instructions: &mut Vec<RegExpInstruction>,
) -> Result<(), RegExpCompileError> {
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
    if !is_regex_metacharacter(escaped) {
        return Err(RegExpCompileError::unsupported_feature(
            escape_offset,
            format!(
                "unsupported regular-expression escape `\\{}`",
                escaped as char
            ),
        ));
    }

    instructions.push(RegExpInstruction::literal_ascii(escaped));
    *offset += 2;
    Ok(())
}

fn parse_positive_ascii_class(
    bytes: &[u8],
    offset: &mut usize,
    instructions: &mut Vec<RegExpInstruction>,
) -> Result<(), RegExpCompileError> {
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

    instructions.push(RegExpInstruction::positive_ascii_class(
        bitmap_low,
        bitmap_high,
    ));
    *offset = cursor + 1;
    Ok(())
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
    fn preserves_global_and_sticky_flags_outside_match_atoms() {
        let program = RegExpProgram::compile("a", "gy").expect("flags should compile");
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
            ("a+", 1),
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
    fn rejects_duplicate_noncanonical_and_unsupported_flags() {
        for flags in ["gg", "yg"] {
            let error = RegExpProgram::compile("a", flags).expect_err("flags should be invalid");
            assert_eq!(error.kind, RegExpCompileErrorKind::InvalidSyntax);
        }
        let error = RegExpProgram::compile("a", "i").expect_err("flag should be unsupported");
        assert_eq!(error.kind, RegExpCompileErrorKind::UnsupportedFeature);
        assert_eq!(error.offset, 0);
    }
}
