use super::*;

pub(super) fn write_locales(
    writer: &mut Writer,
    locales: &[CanonicalLocaleId],
) -> Result<(), DurationWireError> {
    writer.word(locales.len() as u64)?;
    for locale in locales {
        writer.text(locale.as_str())?;
    }
    Ok(())
}
pub(super) fn read_locales(
    reader: &mut Reader<'_>,
    limits: &PartitionLimits,
) -> Result<Box<[CanonicalLocaleId]>, DurationWireError> {
    let count = reader.count(8)?;
    if count as u128 > u128::from(limits.part_count()) {
        return Err(DurationWireError::Resource("locale count"));
    }
    let mut locales = Vec::new();
    locales
        .try_reserve_exact(count)
        .map_err(|_| DurationWireError::Resource("locale allocation"))?;
    let mut bytes = 0_u128;
    for _ in 0..count {
        let locale = reader.locale(limits)?;
        bytes += locale.as_str().len() as u128;
        if bytes > u128::from(limits.output_bytes()) {
            return Err(DurationWireError::Resource("locale text extent"));
        }
        locales.push(locale);
    }
    Ok(locales.into_boxed_slice())
}
pub(super) fn write_resolved(
    writer: &mut Writer,
    locale: &ResolvedDurationLocale,
) -> Result<(), DurationWireError> {
    writer.text(locale.resolved().as_str())?;
    writer.text(locale.numbering_system())
}
pub(super) fn read_resolved_fields(
    reader: &mut Reader<'_>,
    limits: &PartitionLimits,
) -> Result<(CanonicalLocaleId, Box<str>), DurationWireError> {
    let locale = reader.locale(limits)?;
    let numbering =
        crate::number_format::owned_text(reader.text()?, limits).map_err(DurationError::from)?;
    Ok((locale, numbering))
}
pub(super) fn admit_resolved(
    locale: CanonicalLocaleId,
    numbering: &str,
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
) -> Result<ResolvedDurationLocale, DurationWireError> {
    let option = NumberingSystemOption::parse(numbering)
        .map_err(|_| DurationWireError::Malformed("numbering system syntax"))?;
    let request = NumberLocaleRequest {
        requested: vec![locale.clone()].into_boxed_slice(),
        matcher: LocaleMatcher::Lookup,
        numbering_system: Some(option),
    };
    let resolved = profiles.resolve_locale(&request, numbers)?;
    if resolved.resolved() != &locale || resolved.numbering_system() != numbering {
        return Err(DurationWireError::Malformed(
            "resolved locale/numbering association",
        ));
    }
    Ok(resolved)
}

pub fn encode_duration_request(
    request: &DurationWireRequest,
) -> Result<Vec<u8>, DurationWireError> {
    let mut writer = Writer::new(request.operation(), false)?;
    match request {
        DurationWireRequest::Resolve(request) => {
            write_locales(&mut writer, &request.requested)?;
            writer.word(request.matcher.wire_code())?;
            writer.word(u64::from(request.numbering_system.is_some()))?;
            if let Some(numbering) = &request.numbering_system {
                writer.text(numbering.name())?;
            }
        }
        DurationWireRequest::SupportedLocales(request) => {
            write_locales(&mut writer, &request.requested)?;
            writer.word(request.matcher.wire_code())?;
        }
        DurationWireRequest::Parts(request) => {
            write_resolved(&mut writer, request.configuration().locale())?;
            for word in request.configuration().wire_words() {
                writer.word(word)?;
            }
            for &bits in request.number_bits() {
                writer.word(bits)?;
            }
        }
    }
    Ok(writer.finish())
}
pub fn decode_duration_request(
    operation: DurationHostOp,
    bytes: &[u8],
    profiles: &DurationProfiles,
    numbers: &Arc<NumberProfiles>,
    limits: &PartitionLimits,
) -> Result<DurationWireRequest, DurationWireError> {
    let mut reader = Reader::new(bytes, operation, false)?;
    match operation {
        DurationHostOp::Resolve => {
            let requested = read_locales(&mut reader, limits)?;
            let matcher = LocaleMatcher::from_wire_code(reader.word()?)
                .ok_or(DurationWireError::Malformed("locale matcher"))?;
            let numbering_system = match reader.word()? {
                0 => None,
                1 => {
                    let text = crate::number_format::owned_text(reader.text()?, limits)
                        .map_err(DurationError::from)?;
                    Some(
                        NumberingSystemOption::parse(&text)
                            .map_err(|_| DurationWireError::Malformed("numbering system syntax"))?,
                    )
                }
                _ => return Err(DurationWireError::Malformed("numbering presence")),
            };
            reader.finish()?;
            Ok(DurationWireRequest::Resolve(NumberLocaleRequest {
                requested,
                matcher,
                numbering_system,
            }))
        }
        DurationHostOp::SupportedLocales => {
            let requested = read_locales(&mut reader, limits)?;
            let matcher = LocaleMatcher::from_wire_code(reader.word()?)
                .ok_or(DurationWireError::Malformed("locale matcher"))?;
            reader.finish()?;
            Ok(DurationWireRequest::SupportedLocales(
                DurationSupportedLocalesRequest { requested, matcher },
            ))
        }
        DurationHostOp::Parts => {
            let (locale, numbering) = read_resolved_fields(&mut reader, limits)?;
            let mut words = [0_u64; DURATION_CONFIGURATION_WORDS];
            for word in &mut words {
                *word = reader.word()?;
            }
            let mut bits = [0_u64; DURATION_RECORD_WORDS];
            for field in &mut bits {
                *field = reader.word()?;
            }
            reader.finish()?;
            // Structural completion precedes semantic admission. Neither a raw
            // configuration nor a partial record escapes this boundary.
            let options = configuration::options(words)?;
            let locale = admit_resolved(locale, &numbering, profiles, numbers)?;
            let checked = CheckedDurationConfiguration::new(locale, options)?;
            if checked.wire_words() != words {
                return Err(DurationWireError::Malformed(
                    "noncanonical effective configuration",
                ));
            }
            let request = DurationPartitionRequest::from_completed_number_fields(
                checked,
                bits.map(f64::from_bits),
            )?;
            Ok(DurationWireRequest::Parts(request))
        }
    }
}
