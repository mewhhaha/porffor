use super::*;

/// Certifies the structured lifecycle before the graph proof can summarize a
/// RepeatEnd. Ordinary control flow cannot enter an active region, skip its
/// Guard, leave its body, or reset a parent counter from inside that region.
pub(super) fn validate_counted_regions(
    program: &RegExpProgram,
) -> Result<(), RegExpProgramValidationError> {
    let instructions = &program.instructions;
    let count = instructions.len();
    let repeat_count = instructions
        .iter()
        .filter(|instruction| instruction.opcode == REGEXP_OPCODE_REPEAT_BEGIN)
        .count();
    if repeat_count != program.repeat_bounds.len() {
        return Err(RegExpProgramValidationError::InvalidDescriptor);
    }
    let mut slots = vec![false; repeat_count];
    let mut owners = vec![None; count];
    let mut active: Vec<(usize, usize)> = Vec::new();
    let invalid = |pc| RegExpProgramValidationError::InvalidInstruction { pc };
    for (pc, instruction) in instructions.iter().enumerate() {
        owners[pc] = active.last().map(|&(begin, _)| begin);
        match RegExpOpcode::from_word(instruction.opcode).ok_or_else(|| invalid(pc))? {
            RegExpOpcode::RepeatBegin => {
                let guard = instructions.get(pc + 1).ok_or_else(|| invalid(pc))?;
                let end = usize::try_from(guard.operand0).map_err(|_| invalid(pc))?;
                let slot = usize::try_from(guard.operand1 >> 1).map_err(|_| invalid(pc))?;
                if guard.opcode != REGEXP_OPCODE_REPEAT_GUARD
                    || end < pc + 2
                    || end.checked_add(1).is_none_or(|exit| exit >= count)
                    || active
                        .last()
                        .is_some_and(|&(_, outer_end)| end >= outer_end)
                    || slot >= slots.len()
                    || slots[slot]
                    || instruction.operand0 != slot as u64
                    || instruction.operand1 != 0
                    || instructions[end] != RegExpInstruction::repeat_end(pc)
                    || instructions[end + 1] != RegExpInstruction::repeat_exit(pc)
                {
                    return Err(invalid(pc));
                }
                slots[slot] = true;
                active.push((pc, end));
            }
            RegExpOpcode::RepeatGuard => {
                if active.last().is_none_or(|&(begin, _)| pc != begin + 1) {
                    return Err(invalid(pc));
                }
            }
            RegExpOpcode::RepeatEnd => {
                if active
                    .last()
                    .is_none_or(|&(begin, end)| pc != end || instruction.operand0 != begin as u64)
                {
                    return Err(invalid(pc));
                }
            }
            RegExpOpcode::RepeatExit => {
                if active.last().is_none_or(|&(begin, end)| {
                    pc != end + 1 || instruction.operand0 != begin as u64
                }) {
                    return Err(invalid(pc));
                }
                active.pop();
            }
            RegExpOpcode::Accept if !active.is_empty() => return Err(invalid(pc)),
            _ => {}
        }
    }
    if !active.is_empty() || slots.iter().any(|present| !present) {
        return Err(invalid(count - 1));
    }
    for (pc, instruction) in instructions.iter().copied().enumerate() {
        let opcode = RegExpOpcode::from_word(instruction.opcode).ok_or_else(|| invalid(pc))?;
        for target in opcode
            .successors(instruction, pc, count)
            .into_iter()
            .flatten()
        {
            let target_opcode = RegExpOpcode::from_word(instructions[target].opcode)
                .ok_or_else(|| invalid(target))?;
            if target_opcode == RegExpOpcode::RepeatGuard
                && !matches!(opcode, RegExpOpcode::RepeatBegin | RegExpOpcode::RepeatEnd)
                || target_opcode == RegExpOpcode::RepeatExit && opcode != RegExpOpcode::RepeatGuard
            {
                return Err(invalid(pc));
            }
            let expected_owner = match opcode {
                RegExpOpcode::RepeatBegin => Some(pc),
                RegExpOpcode::RepeatExit => owners[instruction.operand0 as usize],
                _ => owners[pc],
            };
            if owners[target] != expected_owner {
                return Err(invalid(pc));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiled(source: &str) -> RegExpProgram {
        RegExpProgram::compile(source, "").expect("small counted source")
    }

    #[test]
    fn counted_descriptor_retains_one_body_and_exact_bounds() {
        for (source, minimum, maximum) in [
            ("a{32768}", 32768, 32768),
            ("a{32768,32770}?", 32768, 32770),
            ("(?:a|){3,5}", 3, 5),
            ("(?:(a?){2,3}){4,6}?", 4, 6),
        ] {
            let program = compiled(source);
            let descriptor = ValidatedRegExpProgram::from_program(&program).expect(source);
            let begin = program
                .instructions
                .iter()
                .find(|i| i.opcode == REGEXP_OPCODE_REPEAT_BEGIN)
                .unwrap();
            assert_eq!((begin.operand0, begin.operand1), (0, 0), "{source}");
            assert_eq!(
                program.repeat_bounds[0].minimum(),
                &RegExpNatural::from_u64(minimum)
            );
            assert_eq!(
                program.repeat_bounds[0].maximum(),
                &RegExpRepeatMaximum::Finite(RegExpNatural::from_u64(maximum))
            );
            assert!(program.instructions.len() < 64, "{source}");
            assert!(descriptor.word(RegExpProgramWord::RepeatSlotCount) > 0);
            assert_eq!(
                ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
                descriptor
            );
        }
    }

    #[test]
    fn rejects_repeat_pair_slot_nesting_and_region_damage() {
        let original = compiled("(?:(a?){2,3}){4,6}");
        let begins: Vec<_> = original
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(pc, i)| (i.opcode == REGEXP_OPCODE_REPEAT_BEGIN).then_some(pc))
            .collect();
        assert_eq!(begins.len(), 2);
        for damage in 0..5 {
            let mut program = original.clone();
            let outer = begins[0];
            let inner = begins[1];
            let end = program.instructions[outer + 1].operand0 as usize;
            match damage {
                0 => program.instructions[outer + 1].operand0 = inner as u64,
                1 => {
                    program.instructions[inner + 1].operand1 =
                        program.instructions[outer + 1].operand1
                }
                2 => program.instructions[end].operand0 = inner as u64,
                3 => program.instructions[inner + 2] = RegExpInstruction::jump(outer + 1),
                4 => program.instructions[inner + 2] = RegExpInstruction::jump(end + 2),
                _ => unreachable!(),
            }
            assert!(
                ValidatedRegExpProgram::from_program(&program).is_err(),
                "damage {damage}"
            );
        }
    }

    #[test]
    fn counted_progress_does_not_hide_counter_reset_cycles() {
        let mut program = compiled("(?:){2}");
        let last = program.instructions.len() - 1;
        program.instructions[last] = RegExpInstruction::jump(0);
        assert_eq!(
            ValidatedRegExpProgram::from_program(&program),
            Err(RegExpProgramValidationError::NonConsumingCycle)
        );
    }

    #[test]
    fn bounds_admission_never_accepts_saturated_nullable_counts() {
        let natural = |digits: &str| RegExpNatural::from_decimal_digits(digits.as_bytes()).unwrap();
        assert!(
            RegExpRepeatBounds::new(natural("3"), RegExpRepeatMaximum::Finite(natural("2")))
                .is_none()
        );
        let impossible = compiled("a{4294967296}");
        for (bounds, minimum, maximum) in [
            (
                "18446744073709551615,18446744073709551616",
                "18446744073709551615",
                "18446744073709551616",
            ),
            (
                "18446744073709551616,18446744073709551616",
                "18446744073709551616",
                "18446744073709551616",
            ),
            (
                "18446744073709551616,18446744073709551617",
                "18446744073709551616",
                "18446744073709551617",
            ),
            (
                "00018446744073709551615,00018446744073709551616",
                "18446744073709551615",
                "18446744073709551616",
            ),
            (
                "18446744073709551615",
                "18446744073709551615",
                "18446744073709551615",
            ),
            (
                "18446744073709551616",
                "18446744073709551616",
                "18446744073709551616",
            ),
            (
                "1000000000000000000000000000000",
                "1000000000000000000000000000000",
                "1000000000000000000000000000000",
            ),
        ] {
            let consuming = compiled(&format!("a{{{bounds}}}"));
            ValidatedRegExpProgram::from_program(&consuming).expect(bounds);
            assert_eq!(consuming.instructions, impossible.instructions, "{bounds}");
            let nullable = compiled(&format!("(){{{bounds}}}"));
            assert_eq!(
                nullable.repeat_bounds[0].minimum().digits(),
                minimum.as_bytes()
            );
            assert_eq!(
                nullable.repeat_bounds[0].maximum(),
                &RegExpRepeatMaximum::Finite(natural(maximum))
            );
            assert!(
                nullable.instructions.len() < 16,
                "one source-sized body: {bounds}"
            );
            let descriptor = ValidatedRegExpProgram::from_program(&nullable).unwrap();
            assert!(
                descriptor.bytes().len() < 512,
                "source-sized bound storage: {bounds}"
            );
            assert!(descriptor.word(RegExpProgramWord::RepeatStateByteLength) < 128);
            assert_eq!(
                ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
                descriptor
            );
        }
        let unbounded = compiled("(){00018446744073709551616,}");
        assert_eq!(
            unbounded.repeat_bounds[0].minimum().digits(),
            b"18446744073709551616"
        );
        assert_eq!(
            unbounded.repeat_bounds[0].maximum(),
            &RegExpRepeatMaximum::Unbounded
        );
        for source in [
            "(?:(){18446744073709551616}){2,3}",
            "(?<=(){18446744073709551616})a",
            "(?=(){18446744073709551616})a",
            "(?!(?:){18446744073709551616})a",
        ] {
            let program = compiled(source);
            let descriptor = ValidatedRegExpProgram::from_program(&program).expect(source);
            assert_eq!(
                ValidatedRegExpProgram::from_bytes(descriptor.bytes().to_vec()).unwrap(),
                descriptor
            );
        }
        for bounds in [
            "18446744073709551616,18446744073709551615",
            "18446744073709551617,18446744073709551616",
            "00018446744073709551617,00018446744073709551616",
            "4,2",
        ] {
            for atom in ["a", "()"] {
                assert_eq!(
                    RegExpProgram::compile(&format!("{atom}{{{bounds}}}"), "")
                        .unwrap_err()
                        .kind,
                    RegExpCompileErrorKind::InvalidSyntax,
                    "original decimal order takes precedence: {bounds}"
                );
            }
        }
    }

    #[test]
    fn exact_decimal_archive_rejects_alias_order_padding_and_state_damage() {
        let program = compiled("(?<named>){100000000000000000001,100000000000000000002}");
        let descriptor = ValidatedRegExpProgram::from_program(&program).unwrap();
        let record = REGEXP_PROGRAM_HEADER_SIZE
            + program.instructions.len() * REGEXP_INSTRUCTION_WIDTH
            + program.ranges.len() * REGEXP_RANGE_ENTRY_WIDTH;
        let read = |word: RegExpRepeatBoundWord| {
            let offset = record + word.offset() as usize;
            u64::from_le_bytes(descriptor.bytes()[offset..offset + 8].try_into().unwrap()) as usize
        };
        let minimum = read(RegExpRepeatBoundWord::MinimumDigitsOffset);
        let maximum = read(RegExpRepeatBoundWord::MaximumDigitsOffset);
        let maximum_length = read(RegExpRepeatBoundWord::MaximumDigitsLength);
        let put = |bytes: &mut Vec<u8>, offset: usize, value: u64| {
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        };
        for damage in 0..12 {
            let mut bytes = descriptor.bytes().to_vec();
            match damage {
                0 => put(&mut bytes, record, record as u64),
                1 => put(
                    &mut bytes,
                    record + RegExpRepeatBoundWord::MinimumDigitsLength.offset() as usize,
                    0,
                ),
                2 => bytes[minimum] = b'x',
                3 => bytes[minimum] = b'0',
                4 => put(
                    &mut bytes,
                    record + RegExpRepeatBoundWord::MaximumKind.offset() as usize,
                    2,
                ),
                5 => put(
                    &mut bytes,
                    record + RegExpRepeatBoundWord::MaximumKind.offset() as usize,
                    RegExpRepeatMaximumKind::Unbounded.word(),
                ),
                6 => put(
                    &mut bytes,
                    record + RegExpRepeatBoundWord::MaximumDigitsOffset.offset() as usize,
                    minimum as u64,
                ),
                7 => bytes[maximum + maximum_length - 1] = b'0',
                8 => put(
                    &mut bytes,
                    record + RegExpRepeatBoundWord::StateOffset.offset() as usize,
                    8,
                ),
                9 => put(
                    &mut bytes,
                    RegExpProgramWord::RepeatStateByteLength.offset() as usize,
                    descriptor.word(RegExpProgramWord::RepeatStateByteLength) + 8,
                ),
                10 => bytes[maximum + maximum_length] = 1,
                11 => put(&mut bytes, REGEXP_PROGRAM_HEADER_SIZE + 16, 1),
                _ => unreachable!(),
            }
            assert!(
                ValidatedRegExpProgram::from_bytes(bytes).is_err(),
                "damage {damage}"
            );
        }
        let mut unused = program.clone();
        unused.repeat_bounds.push(unused.repeat_bounds[0].clone());
        assert!(ValidatedRegExpProgram::from_program(&unused).is_err());
        let mut mismatch = program;
        mismatch.instructions[0].operand0 = 1;
        assert!(ValidatedRegExpProgram::from_program(&mismatch).is_err());
    }

    #[test]
    fn reverse_counted_body_retains_capture_and_sequence_direction() {
        for (source, lazy) in [("(?<=^(ab){2,3})c", 0), ("(?<=^(ab){2,3}?)c", 1)] {
            let program = compiled(source);
            ValidatedRegExpProgram::from_program(&program).expect("reverse counted region");
            let begin = program
                .instructions
                .iter()
                .position(|i| i.opcode == REGEXP_OPCODE_REPEAT_BEGIN)
                .unwrap();
            let guard = program.instructions[begin + 1];
            let body = &program.instructions[begin + 2..guard.operand0 as usize];
            assert_eq!(guard.operand1 & 1, lazy);
            assert_eq!(
                body,
                &[
                    RegExpInstruction::clear_capture_range(1, 2),
                    RegExpInstruction::capture_end(1),
                    RegExpInstruction::literal_ascii(b'b'),
                    RegExpInstruction::literal_ascii(b'a'),
                    RegExpInstruction::capture_start(1),
                ]
            );
        }
    }
}
