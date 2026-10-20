use super::*;
use requests::{read_locales, read_resolved_fields, write_locales, write_resolved};

fn write_partition(
    writer: &mut Writer,
    partition: &DurationPartition,
) -> Result<(), DurationWireError> {
    writer.word(partition.parts().len() as u64)?;
    for part in partition.parts() {
        writer.word(part.kind().wire_code())?;
        writer.text(part.text())?;
        writer.word(u64::from(part.unit().is_some()))?;
        if let Some(unit) = part.unit() {
            writer.word(unit.index() as u64)?;
        }
    }
    Ok(())
}
pub fn encode_duration_response(response: &DurationResponse) -> Result<Vec<u8>, DurationWireError> {
    let operation = match response {
        DurationResponse::Resolved(_) => DurationHostOp::Resolve,
        DurationResponse::SupportedLocales(_) => DurationHostOp::SupportedLocales,
        DurationResponse::Parts(_) => DurationHostOp::Parts,
    };
    let mut writer = Writer::new(operation, true)?;
    match response {
        DurationResponse::Resolved(locale) => {
            write_resolved(&mut writer, locale)?;
            writer.word(u64::from(locale.two_digit_hours()))?;
        }
        DurationResponse::SupportedLocales(locales) => write_locales(&mut writer, locales)?,
        DurationResponse::Parts(partition) => write_partition(&mut writer, partition)?,
    }
    Ok(writer.finish())
}
fn read_partition(
    reader: &mut Reader<'_>,
    limits: &PartitionLimits,
) -> Result<DurationWirePartition, DurationWireError> {
    let count = reader.count(24)?;
    if count as u128 > u128::from(limits.part_count()) {
        return Err(DurationWireError::Resource("part count"));
    }
    let mut parts = Vec::new();
    parts
        .try_reserve_exact(count)
        .map_err(|_| DurationWireError::Resource("part allocation"))?;
    let mut bytes = 0_usize;
    let mut previous = None;
    let mut minus_signs = 0;
    for _ in 0..count {
        let kind = NumberPartKind::from_wire_code(reader.word()?)
            .ok_or(DurationWireError::Malformed("number part kind"))?;
        let text = reader.text()?;
        let unit = match reader.word()? {
            0 => None,
            1 => {
                let code = reader.word()?;
                Some(match code {
                    0 => DurationUnit::Year,
                    1 => DurationUnit::Month,
                    2 => DurationUnit::Week,
                    3 => DurationUnit::Day,
                    4 => DurationUnit::Hour,
                    5 => DurationUnit::Minute,
                    6 => DurationUnit::Second,
                    7 => DurationUnit::Millisecond,
                    8 => DurationUnit::Microsecond,
                    9 => DurationUnit::Nanosecond,
                    _ => return Err(DurationWireError::Malformed("duration unit")),
                })
            }
            _ => return Err(DurationWireError::Malformed("unit presence")),
        };
        if text.is_empty() || text.contains('\0') {
            return Err(DurationWireError::Malformed("empty/NUL part"));
        }
        if !matches!(
            kind,
            NumberPartKind::Literal
                | NumberPartKind::Integer
                | NumberPartKind::Group
                | NumberPartKind::Decimal
                | NumberPartKind::Fraction
                | NumberPartKind::MinusSign
                | NumberPartKind::Unit
        ) {
            return Err(DurationWireError::Malformed("non-duration number part"));
        }
        if let Some(unit) = unit {
            if previous.is_some_and(|previous: usize| previous > unit.index()) {
                return Err(DurationWireError::Malformed("duration unit order"));
            }
            previous = Some(unit.index());
        } else if kind != NumberPartKind::Literal {
            return Err(DurationWireError::Malformed("unitless nonliteral"));
        }
        if kind == NumberPartKind::MinusSign {
            minus_signs += 1;
        }
        if minus_signs > 1 {
            return Err(DurationWireError::Malformed("duplicate duration sign"));
        }
        bytes = bytes
            .checked_add(text.len())
            .ok_or(DurationWireError::Resource("part text extent"))?;
        if bytes as u128 > u128::from(limits.output_bytes()) {
            return Err(DurationWireError::Resource("part text extent"));
        }
        let text = crate::number_format::owned_text(text, limits).map_err(DurationError::from)?;
        parts.push(DurationWirePart {
            part: NumberPart::new(kind, text),
            unit,
        });
    }
    Ok(DurationWirePartition {
        parts: parts.into_boxed_slice(),
        text_bytes: bytes,
    })
}
pub fn decode_duration_response(
    request: &DurationWireRequest,
    bytes: &[u8],
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<DurationWireResponse, DurationWireError> {
    let mut reader = Reader::new(bytes, request.operation(), true)?;
    match request {
        DurationWireRequest::Resolve(request) => {
            let (locale, numbering) = read_resolved_fields(&mut reader, limits)?;
            let two_digit_hours = match reader.word()? {
                0 => false,
                1 => true,
                _ => return Err(DurationWireError::Malformed("two-digit-hours Boolean")),
            };
            reader.finish()?;
            let expected = profiles.resolve_locale(request, numbers)?;
            if expected.resolved() != &locale
                || expected.numbering_system() != numbering.as_ref()
                || expected.two_digit_hours() != two_digit_hours
            {
                return Err(DurationWireError::Malformed(
                    "resolved response/request association",
                ));
            }
            Ok(DurationWireResponse::Resolved(expected))
        }
        DurationWireRequest::SupportedLocales(request) => {
            let locales = read_locales(&mut reader, limits)?;
            reader.finish()?;
            if locales != profiles.supported_locales(request.clone()) {
                return Err(DurationWireError::Malformed(
                    "supported response/request association",
                ));
            }
            Ok(DurationWireResponse::SupportedLocales(locales))
        }
        DurationWireRequest::Parts(request) => {
            let partition = read_partition(&mut reader, limits)?;
            reader.finish()?;
            profiles.ensure_numbers(numbers)?;
            profiles.ensure_configuration(request.configuration())?;
            Ok(DurationWireResponse::Parts(partition))
        }
    }
}
