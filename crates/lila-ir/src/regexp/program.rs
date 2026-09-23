use super::*;

/// A word in the versioned, allocation-relative immutable program descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum RegExpProgramWord {
    MagicVersion,
    ByteLength,
    InstructionCount,
    CaptureCount,
    RangeCount,
    SplitCount,
    RepeatableSplitCount,
    NamedGroupTableOffset,
}

impl RegExpProgramWord {
    pub const ALL: [Self; 8] = [
        Self::MagicVersion,
        Self::ByteLength,
        Self::InstructionCount,
        Self::CaptureCount,
        Self::RangeCount,
        Self::SplitCount,
        Self::RepeatableSplitCount,
        Self::NamedGroupTableOffset,
    ];

    pub const fn offset(self) -> u64 {
        (self as u64) * 8
    }
}

pub const REGEXP_PROGRAM_HEADER_SIZE: usize =
    (RegExpProgramWord::NamedGroupTableOffset as usize + 1) * 8;
pub const REGEXP_PROGRAM_MAGIC_VERSION: u64 = (1_u64 << 32) | u32::from_le_bytes(*b"RGPB") as u64;
pub const REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION: u64 =
    (2_u64 << 32) | u32::from_le_bytes(*b"NRGT") as u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegExpProgramValidationError {
    EmptyProgram,
    Capacity,
    InvalidInstruction { pc: usize },
    InvalidRanges,
    InvalidNamedGroups,
    NonConsumingCycle,
    InvalidDescriptor,
}

impl fmt::Display for RegExpProgramValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid RegExp program: {self:?}")
    }
}
impl Error for RegExpProgramValidationError {}

/// The sole serialization accepted by the backend. Its owned bytes cannot be
/// mutated after validation. All references inside the blob are relative, so
/// relocating the allocation does not change its semantic identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatedRegExpProgram {
    bytes: Vec<u8>,
}

impl ValidatedRegExpProgram {
    pub fn from_program(program: &RegExpProgram) -> Result<Self, RegExpProgramValidationError> {
        validate_program(program)?;
        let instruction_count = program.instructions.len();
        let split_count = program
            .instructions
            .iter()
            .filter(|instruction| {
                RegExpOpcode::from_word(instruction.opcode).is_some_and(RegExpOpcode::is_choice)
            })
            .count();
        let repeatable_split_count = repeatable_split_count(program);
        let candidate_count: usize = program
            .named_groups
            .iter()
            .map(|group| group.capture_ids.len())
            .sum();
        let instructions_and_ranges = instruction_count * REGEXP_INSTRUCTION_WIDTH
            + program.ranges.len() * REGEXP_RANGE_ENTRY_WIDTH;
        let candidates_offset = program
            .named_groups
            .len()
            .checked_mul(24)
            .and_then(|length| length.checked_add(32))
            .ok_or(RegExpProgramValidationError::Capacity)?;
        let names_offset = candidate_count
            .checked_mul(8)
            .and_then(|length| length.checked_add(candidates_offset))
            .ok_or(RegExpProgramValidationError::Capacity)?;
        let named_byte_length = if program.named_groups.is_empty() {
            0
        } else {
            program
                .named_groups
                .iter()
                .try_fold(names_offset, |length, group| {
                    length.checked_add(group.name.len())
                })
                .ok_or(RegExpProgramValidationError::Capacity)?
        };
        let byte_length = REGEXP_PROGRAM_HEADER_SIZE
            .checked_add(instructions_and_ranges)
            .and_then(|length| length.checked_add(named_byte_length))
            .and_then(|length| u32::try_from(length).ok())
            .ok_or(RegExpProgramValidationError::Capacity)?;
        let mut bytes = Vec::with_capacity(byte_length as usize);
        bytes.resize(REGEXP_PROGRAM_HEADER_SIZE, 0);
        bytes.extend_from_slice(&program.encode());
        let named_offset = if program.named_groups.is_empty() {
            0
        } else {
            bytes.len()
        };
        if !program.named_groups.is_empty() {
            let mut candidate_offset = candidates_offset;
            let mut name_offset = names_offset;
            for word in [
                REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION,
                program.named_groups.len() as u64,
                candidate_count as u64,
                32,
            ] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            for group in &program.named_groups {
                let name_payload = ((name_offset as u64) << 32) | group.name.len() as u64;
                for word in [
                    name_payload,
                    candidate_offset as u64,
                    group.capture_ids.len() as u64,
                ] {
                    bytes.extend_from_slice(&word.to_le_bytes());
                }
                candidate_offset += group.capture_ids.len() * 8;
                name_offset += group.name.len();
            }
            for group in &program.named_groups {
                for &capture_id in &group.capture_ids {
                    bytes.extend_from_slice(&(capture_id as u64).to_le_bytes());
                }
            }
            for group in &program.named_groups {
                bytes.extend_from_slice(group.name.as_bytes());
            }
        }
        for word in RegExpProgramWord::ALL {
            let value = match word {
                RegExpProgramWord::MagicVersion => REGEXP_PROGRAM_MAGIC_VERSION,
                RegExpProgramWord::ByteLength => byte_length as u64,
                RegExpProgramWord::InstructionCount => instruction_count as u64,
                RegExpProgramWord::CaptureCount => program.capture_count as u64,
                RegExpProgramWord::RangeCount => program.ranges.len() as u64,
                RegExpProgramWord::SplitCount => split_count as u64,
                RegExpProgramWord::RepeatableSplitCount => repeatable_split_count as u64,
                RegExpProgramWord::NamedGroupTableOffset => named_offset as u64,
            };
            let offset = word.offset() as usize;
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        Ok(Self { bytes })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn word(&self, word: RegExpProgramWord) -> u64 {
        let offset = word.offset() as usize;
        u64::from_le_bytes(self.bytes[offset..offset + 8].try_into().unwrap())
    }

    /// Validates an encoded allocation, including canonical section ownership.
    /// The static serializer uses this boundary before publishing bytes too.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RegExpProgramValidationError> {
        let invalid = RegExpProgramValidationError::InvalidDescriptor;
        if u32::try_from(bytes.len()).is_err() {
            return Err(RegExpProgramValidationError::Capacity);
        }
        let read = |offset: usize| -> Result<u64, RegExpProgramValidationError> {
            let slice = bytes
                .get(offset..offset.checked_add(8).ok_or_else(|| invalid.clone())?)
                .ok_or_else(|| invalid.clone())?;
            Ok(u64::from_le_bytes(slice.try_into().unwrap()))
        };
        if read(0)? != REGEXP_PROGRAM_MAGIC_VERSION || read(8)? != bytes.len() as u64 {
            return Err(invalid);
        }
        let count = usize::try_from(read(RegExpProgramWord::InstructionCount.offset() as usize)?)
            .map_err(|_| invalid.clone())?;
        let capture_count = u32::try_from(read(RegExpProgramWord::CaptureCount.offset() as usize)?)
            .map_err(|_| invalid.clone())?;
        let range_count = usize::try_from(read(RegExpProgramWord::RangeCount.offset() as usize)?)
            .map_err(|_| invalid.clone())?;
        if count == 0 || count > REGEXP_MAX_INSTRUCTIONS || range_count > REGEXP_MAX_RANGE_ENTRIES {
            return Err(invalid);
        }
        let ranges_offset = REGEXP_PROGRAM_HEADER_SIZE + count * REGEXP_INSTRUCTION_WIDTH;
        let names_offset = ranges_offset + range_count * REGEXP_RANGE_ENTRY_WIDTH;
        if names_offset > bytes.len() {
            return Err(invalid);
        }
        let mut instructions = Vec::with_capacity(count);
        for pc in 0..count {
            let offset = REGEXP_PROGRAM_HEADER_SIZE + pc * REGEXP_INSTRUCTION_WIDTH;
            instructions.push(RegExpInstruction {
                opcode: read(offset)?,
                operand0: read(offset + 8)?,
                operand1: read(offset + 16)?,
            });
        }
        let mut ranges = Vec::with_capacity(range_count);
        for index in 0..range_count {
            let word = read(ranges_offset + index * REGEXP_RANGE_ENTRY_WIDTH)?;
            ranges.push((word as u32, (word >> 32) as u32));
        }
        let named_offset = read(RegExpProgramWord::NamedGroupTableOffset.offset() as usize)?;
        let mut named_groups = Vec::new();
        if named_offset == 0 {
            if names_offset != bytes.len() {
                return Err(invalid);
            }
        } else {
            if named_offset != names_offset as u64
                || read(names_offset)? != REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION
            {
                return Err(invalid);
            }
            let named_count =
                usize::try_from(read(names_offset + 8)?).map_err(|_| invalid.clone())?;
            let candidate_count =
                usize::try_from(read(names_offset + 16)?).map_err(|_| invalid.clone())?;
            if named_count == 0
                || named_count > capture_count as usize
                || candidate_count > capture_count as usize
                || read(names_offset + 24)? != 32
            {
                return Err(invalid);
            }
            let candidates = named_count
                .checked_mul(24)
                .and_then(|length| length.checked_add(32))
                .ok_or_else(|| invalid.clone())?;
            let candidates_end = candidate_count
                .checked_mul(8)
                .and_then(|length| length.checked_add(candidates))
                .ok_or_else(|| invalid.clone())?;
            let mut next_candidate = candidates;
            let mut next_name = candidates_end;
            if next_name > bytes.len() - names_offset {
                return Err(invalid);
            }
            for index in 0..named_count {
                let record = names_offset + 32 + index * 24;
                let name_payload = read(record)?;
                let name_length = name_payload as u32 as usize;
                let name_offset = (name_payload >> 32) as usize;
                let candidate_offset =
                    usize::try_from(read(record + 8)?).map_err(|_| invalid.clone())?;
                let count = usize::try_from(read(record + 16)?).map_err(|_| invalid.clone())?;
                if name_offset != next_name
                    || candidate_offset != next_candidate
                    || count == 0
                    || count > candidate_count
                    || count > (candidates_end - next_candidate) / 8
                {
                    return Err(invalid);
                }
                let name_end = name_offset
                    .checked_add(name_length)
                    .ok_or_else(|| invalid.clone())?;
                let name = bytes
                    .get(
                        names_offset + name_offset
                            ..names_offset
                                .checked_add(name_end)
                                .ok_or_else(|| invalid.clone())?,
                    )
                    .ok_or_else(|| invalid.clone())?;
                let name = std::str::from_utf8(name)
                    .map_err(|_| invalid.clone())?
                    .to_owned();
                let mut capture_ids = Vec::with_capacity(count);
                for candidate in 0..count {
                    capture_ids.push(
                        u32::try_from(read(names_offset + candidate_offset + candidate * 8)?)
                            .map_err(|_| invalid.clone())?,
                    );
                }
                named_groups.push(RegExpNamedGroup { name, capture_ids });
                next_candidate += count * 8;
                next_name = name_end;
            }
            if next_candidate != candidates_end || names_offset + next_name != bytes.len() {
                return Err(invalid);
            }
        }
        let program = RegExpProgram {
            flags: RegExpFlags::default(),
            capture_count,
            named_groups,
            instructions,
            ranges,
        };
        let validated = Self::from_program(&program)?;
        if validated.bytes != bytes {
            return Err(invalid);
        }
        Ok(validated)
    }
}

fn validate_program(program: &RegExpProgram) -> Result<(), RegExpProgramValidationError> {
    use RegExpProgramValidationError as Failure;
    let count = program.instructions.len();
    if count == 0 {
        return Err(Failure::EmptyProgram);
    }
    if count > REGEXP_MAX_INSTRUCTIONS || program.ranges.len() > REGEXP_MAX_RANGE_ENTRIES {
        return Err(Failure::Capacity);
    }
    if program
        .ranges
        .iter()
        .any(|&(start, end)| start > end || end > 0x10ffff)
    {
        return Err(Failure::InvalidRanges);
    }
    let mut names = BTreeSet::new();
    let mut named_captures = BTreeSet::new();
    for group in &program.named_groups {
        if group.name.is_empty() || !names.insert(&group.name) || group.capture_ids.is_empty() {
            return Err(Failure::InvalidNamedGroups);
        }
        for &id in &group.capture_ids {
            if id == 0 || id > program.capture_count || !named_captures.insert(id) {
                return Err(Failure::InvalidNamedGroups);
            }
        }
    }
    for (pc, instruction) in program.instructions.iter().enumerate() {
        let RegExpInstruction {
            opcode,
            operand0: a,
            operand1: b,
        } = *instruction;
        let target = |value: u64| value < count as u64;
        let capture = |value: u64| value != 0 && value <= program.capture_count as u64;
        let range = || {
            let length = b >> 1;
            if a > program.ranges.len() as u64 || length > program.ranges.len() as u64 - a {
                return false;
            }
            program.ranges[a as usize..(a + length) as usize]
                .windows(2)
                .all(|pair| pair[0].1 < pair[1].0)
        };
        let Some(opcode) = RegExpOpcode::from_word(opcode) else {
            return Err(Failure::InvalidInstruction { pc });
        };
        let valid = match opcode.operand_rule() {
            RegExpOperandRule::Zero => a == 0 && b == 0,
            RegExpOperandRule::Ascii => a <= 0x7f && b == 0,
            RegExpOperandRule::CodePoint => a <= 0x10ffff && b == 0,
            RegExpOperandRule::Bitmap => true,
            RegExpOperandRule::TargetPair => target(a) && target(b),
            RegExpOperandRule::Target => target(a) && b == 0,
            RegExpOperandRule::Capture => capture(a) && b == 0,
            RegExpOperandRule::CaptureRange => {
                a > 0 && a <= b && b <= program.capture_count as u64 + 1
            }
            RegExpOperandRule::Modifier => a <= 2 && b == 0,
            RegExpOperandRule::Range => range(),
            RegExpOperandRule::NonemptyRange => b >> 1 != 0 && range(),
            RegExpOperandRule::NamedReference => {
                a < program.named_groups.len() as u64 && b & !REGEXP_BACKREFERENCE_IGNORE_CASE == 0
            }
            RegExpOperandRule::NumberedReference => {
                capture(a)
                    && b & !(REGEXP_BACKREFERENCE_NONEMPTY | REGEXP_BACKREFERENCE_IGNORE_CASE) == 0
            }
            RegExpOperandRule::LookaroundStart => a <= 1 && b == 0,
            RegExpOperandRule::LookaroundEnd => {
                target(a)
                    && target(b & 0x3fff_ffff_ffff_ffff)
                    && program.instructions[a as usize].opcode == REGEXP_OPCODE_LOOKAROUND_FAILURE
            }
            RegExpOperandRule::LookaroundFailure => target(a) && b <= 3,
            RegExpOperandRule::ProgressSplit => target(a) && target(b >> 1),
            RegExpOperandRule::ProgressCheck => {
                target(a)
                    && target(b)
                    && program.instructions[a as usize].opcode == REGEXP_OPCODE_PROGRESS_SPLIT
            }
        };
        if !valid {
            return Err(Failure::InvalidInstruction { pc });
        }
        if pc + 1 == count && opcode.control_flow() == RegExpControlFlow::Next {
            return Err(Failure::InvalidInstruction { pc });
        }
    }
    if has_non_consuming_cycle(program) {
        return Err(Failure::NonConsumingCycle);
    }
    Ok(())
}

/// Counts every choice instruction with a path back to itself, including
/// progress-checked repetition and lookaround bodies.
fn repeatable_split_count(program: &RegExpProgram) -> u32 {
    let instructions = &program.instructions;
    let successors = |pc: usize| {
        RegExpOpcode::from_word(instructions[pc].opcode)
            .expect("validated opcode")
            .successors(instructions[pc], pc, instructions.len())
    };
    instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| {
            RegExpOpcode::from_word(instruction.opcode).is_some_and(RegExpOpcode::is_choice)
        })
        .filter(|(root, _)| {
            let mut visited = vec![false; instructions.len()];
            let mut stack = vec![*root];
            visited[*root] = true;
            while let Some(pc) = stack.pop() {
                for next in successors(pc).into_iter().flatten() {
                    if next == *root {
                        return true;
                    }
                    if !visited[next] {
                        visited[next] = true;
                        stack.push(next);
                    }
                }
            }
            false
        })
        .count() as u32
}

fn has_non_consuming_cycle(program: &RegExpProgram) -> bool {
    let instructions = &program.instructions;
    let mut colors = vec![0_u8; instructions.len()];
    for root in 0..instructions.len() {
        let mut stack = vec![(root, 0)];
        while let Some((pc, edge)) = stack.last_mut() {
            let instruction = instructions[*pc];
            let opcode = RegExpOpcode::from_word(instruction.opcode).expect("validated opcode");
            if colors[*pc] == 2 || opcode.stops_non_consuming_walk(instruction.operand1) {
                colors[*pc] = 2;
                stack.pop();
                continue;
            }
            colors[*pc] = 1;
            if *edge == 2 {
                colors[*pc] = 2;
                stack.pop();
                continue;
            }
            let next = opcode.successors(instruction, *pc, instructions.len())[*edge];
            *edge += 1;
            if let Some(next) = next {
                match colors[next] {
                    1 => return true,
                    0 => stack.push((next, 0)),
                    2 => {}
                    _ => unreachable!(),
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn program(instructions: Vec<RegExpInstruction>) -> RegExpProgram {
        RegExpProgram {
            flags: RegExpFlags::default(),
            capture_count: 0,
            named_groups: Vec::new(),
            instructions,
            ranges: Vec::new(),
        }
    }

    #[test]
    fn iterative_graph_validation_handles_the_full_instruction_capacity() {
        let mut instructions = vec![RegExpInstruction::assert_start(); REGEXP_MAX_INSTRUCTIONS - 1];
        instructions.push(RegExpInstruction::accept());
        let mut full = program(instructions);
        let descriptor =
            ValidatedRegExpProgram::from_program(&full).expect("acyclic maximum-width program");
        assert_eq!(
            descriptor.word(RegExpProgramWord::InstructionCount),
            REGEXP_MAX_INSTRUCTIONS as u64
        );
        full.instructions[REGEXP_MAX_INSTRUCTIONS - 1] = RegExpInstruction::jump(0);
        assert_eq!(
            ValidatedRegExpProgram::from_program(&full),
            Err(RegExpProgramValidationError::NonConsumingCycle)
        );
    }

    #[test]
    fn choice_accounting_preserves_progress_and_lookaround_edges() {
        for (source, choices, repeatable) in [
            ("(?:a?)*", 2, 2),
            ("(?:a?){0,2}", 4, 0),
            ("(?=a*)b", 2, 1),
            (r"(?:\b)*", 1, 1),
        ] {
            let compiled = RegExpProgram::compile(source, "").expect("graph fixture compiles");
            let descriptor =
                ValidatedRegExpProgram::from_program(&compiled).expect("valid progress graph");
            assert_eq!(
                descriptor.word(RegExpProgramWord::SplitCount),
                choices,
                "{source}"
            );
            assert_eq!(
                descriptor.word(RegExpProgramWord::RepeatableSplitCount),
                repeatable,
                "{source}"
            );
        }
    }

    #[test]
    fn rejects_non_consuming_program_cycles_without_rejecting_valid_repetition() {
        let self_jump = program(vec![RegExpInstruction::jump(0)]);
        assert!(has_non_consuming_cycle(&self_jump));

        let failed_consuming_loop = program(vec![
            RegExpInstruction::split(1, 2),
            RegExpInstruction::literal_ascii(b'a'),
            RegExpInstruction::jump(0),
        ]);
        assert!(has_non_consuming_cycle(&failed_consuming_loop));

        let valid_star = RegExpProgram::compile("a*", "").expect("star should compile");
        assert!(!has_non_consuming_cycle(&valid_star));
        let valid_lazy_star = RegExpProgram::compile("a*?b", "").expect("lazy star should compile");
        assert!(!has_non_consuming_cycle(&valid_lazy_star));
    }

    #[test]
    fn word_boundaries_remain_zero_width_in_program_cycle_validation() {
        for pattern in [r"\b", r"\B"] {
            let mut boundary = RegExpProgram::compile(pattern, "").unwrap();
            boundary.instructions.pop();
            boundary.instructions.push(RegExpInstruction::jump(0));
            assert!(has_non_consuming_cycle(&boundary), "{pattern}");
        }
        for pattern in [r"(?:\b)*", r"(\B)+", r"(?:(\b)|x)+"] {
            let guarded = RegExpProgram::compile(pattern, "").unwrap();
            assert!(!has_non_consuming_cycle(&guarded), "{pattern}");
        }
    }

    #[test]
    fn repeatable_split_analysis_walks_through_word_boundaries() {
        let mut boundary = RegExpProgram::compile(r"\b", "").unwrap();
        let assertion = boundary.instructions[0];
        let accept = *boundary.instructions.last().unwrap();
        boundary.instructions = vec![
            RegExpInstruction::split(1, 3),
            assertion,
            RegExpInstruction::jump(0),
            accept,
        ];
        assert_eq!(repeatable_split_count(&boundary), 1);
        assert!(has_non_consuming_cycle(&boundary));
    }

    #[test]
    fn case_folding_bit_does_not_prove_that_a_numbered_reference_consumes() {
        let mut nullable = program(vec![
            RegExpInstruction::numbered_backreference(1, CaseFolding::Legacy),
            RegExpInstruction::jump(0),
        ]);
        nullable.capture_count = 1;
        assert!(has_non_consuming_cycle(&nullable));
        nullable.instructions[0] =
            RegExpInstruction::nonempty_numbered_backreference(1, CaseFolding::Legacy);
        assert!(!has_non_consuming_cycle(&nullable));
        for flags in ["i", "ui", "vi"] {
            let guarded = RegExpProgram::compile(r"(a*)\1*", flags).unwrap();
            assert!(!has_non_consuming_cycle(&guarded));
        }
    }

    #[test]
    fn counts_repeatable_splits_inside_lookbehind() {
        let program =
            RegExpProgram::compile(r"(?<=\w+)f", "").expect("lookbehind repetition should compile");
        assert_eq!(repeatable_split_count(&program), 1);
    }
    #[test]
    fn descriptors_round_trip_composed_programs_and_own_all_named_bytes() {
        for (pattern, flags) in [
            ("", ""),
            (r"(a|b)*?c", ""),
            (r"(?<left>Ā)|(?<left>Ă)", "d"),
            (r"(?<x>[Ā-Ă])\k<x>", "iu"),
            (r"(?<=\w+)f\b", ""),
            (r"(?:(a?)|b){2,4}", ""),
            (r"[\q{ab|c}&&\q{ab}]", "v"),
        ] {
            let program = RegExpProgram::compile(pattern, flags).unwrap();
            let encoded = ValidatedRegExpProgram::from_program(&program).unwrap();
            assert_eq!(
                encoded.word(RegExpProgramWord::ByteLength),
                encoded.bytes().len() as u64
            );
            assert_eq!(
                encoded,
                ValidatedRegExpProgram::from_bytes(encoded.bytes().to_vec()).unwrap(),
                "{pattern}"
            );
        }
    }

    #[test]
    fn descriptors_retain_captures_omitted_by_zero_repetition_lowering() {
        for pattern in [r"(a(b(c))){0}", r"(?<outer>a(?<inner>b)){0}"] {
            let program = RegExpProgram::compile(pattern, "d").unwrap();
            assert!(program.capture_count as usize > program.instructions.len());
            let encoded = ValidatedRegExpProgram::from_program(&program).unwrap();
            assert_eq!(
                encoded.word(RegExpProgramWord::CaptureCount),
                program.capture_count as u64
            );
            assert_eq!(
                encoded,
                ValidatedRegExpProgram::from_bytes(encoded.bytes().to_vec()).unwrap()
            );
        }
    }

    #[test]
    fn descriptor_round_trip_preserves_two_hundred_nested_captures() {
        let pattern = format!("{}a{}", "(".repeat(200), ")".repeat(200));
        let program = RegExpProgram::compile(&pattern, "").unwrap();
        let encoded = ValidatedRegExpProgram::from_program(&program).unwrap();
        assert_eq!(encoded.word(RegExpProgramWord::CaptureCount), 200);
        assert_eq!(
            encoded,
            ValidatedRegExpProgram::from_bytes(encoded.bytes().to_vec()).unwrap()
        );
    }

    #[test]
    fn corrupted_sections_cannot_borrow_neighboring_bytes() {
        let program = RegExpProgram::compile(r"(?<x>[Ā-Ă])", "").unwrap();
        let encoded = ValidatedRegExpProgram::from_program(&program).unwrap();
        let names = encoded.word(RegExpProgramWord::NamedGroupTableOffset) as usize;
        let corruptions = [
            (RegExpProgramWord::MagicVersion.offset() as usize, 0),
            (
                RegExpProgramWord::ByteLength.offset() as usize,
                encoded.bytes().len() as u64 + 8,
            ),
            (
                RegExpProgramWord::RangeCount.offset() as usize,
                encoded.word(RegExpProgramWord::RangeCount) + 1,
            ),
            (names + 24, 0),
            (names + 32, 1),
            (names + 40, encoded.bytes().len() as u64),
            (names + 48, u64::MAX),
        ];
        for (offset, word) in corruptions {
            let mut bytes = encoded.bytes().to_vec();
            bytes[offset..offset + 8].copy_from_slice(&word.to_le_bytes());
            assert!(
                ValidatedRegExpProgram::from_bytes(bytes).is_err(),
                "offset {offset}"
            );
        }
        let mut invalid_name = encoded.bytes().to_vec();
        *invalid_name.last_mut().unwrap() = 0xff;
        assert!(ValidatedRegExpProgram::from_bytes(invalid_name).is_err());
        let mut with_neighbor = encoded.bytes().to_vec();
        with_neighbor.extend_from_slice(encoded.bytes());
        assert!(ValidatedRegExpProgram::from_bytes(with_neighbor).is_err());
    }

    #[test]
    fn invalid_opcode_target_capture_and_range_never_become_descriptors() {
        let original = RegExpProgram::compile(r"(?<x>[Ā-Ă])", "").unwrap();
        for replacement in [
            RegExpInstruction {
                opcode: u64::MAX,
                operand0: 0,
                operand1: 0,
            },
            RegExpInstruction::jump(original.instructions.len()),
            RegExpInstruction::capture_start(2),
            RegExpInstruction::code_point_range_set(original.ranges.len() as u32, 1, false),
        ] {
            let mut program = original.clone();
            program.instructions[0] = replacement;
            assert!(ValidatedRegExpProgram::from_program(&program).is_err());
        }
    }
}
