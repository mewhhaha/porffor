use super::*;

fn write_locale_request(
    op: SegmenterWireOperation,
    request: &SegmenterLocaleRequest,
) -> Result<Vec<u8>, SegmenterWireError> {
    let mut writer = Writer::new(op, false)?;
    writer.locales(&request.requested)?;
    writer.word(match request.matcher {
        LocaleMatcher::Lookup => 1,
        LocaleMatcher::BestFit => 2,
    })?;
    Ok(writer.0)
}
fn read_locale_request(
    op: SegmenterWireOperation,
    data: &[u8],
) -> Result<SegmenterLocaleRequest, SegmenterWireError> {
    let mut reader = Reader::new(data, op, false)?;
    let requested = reader.locales()?;
    let matcher = match reader.word()? {
        1 => LocaleMatcher::Lookup,
        2 => LocaleMatcher::BestFit,
        _ => return Err(SegmenterWireError::Malformed("locale matcher")),
    };
    reader.finish()?;
    Ok(SegmenterLocaleRequest { requested, matcher })
}
pub fn encode_resolve_segmenter_locale_request(
    request: &SegmenterLocaleRequest,
) -> Result<Vec<u8>, SegmenterWireError> {
    write_locale_request(SegmenterWireOperation::ResolveLocale, request)
}
pub fn decode_resolve_segmenter_locale_request(
    data: &[u8],
) -> Result<SegmenterLocaleRequest, SegmenterWireError> {
    read_locale_request(SegmenterWireOperation::ResolveLocale, data)
}
pub fn encode_resolve_segmenter_locale_response(
    locale: &ResolvedSegmenterLocale,
) -> Result<Vec<u8>, SegmenterWireError> {
    let mut writer = Writer::new(SegmenterWireOperation::ResolveLocale, true)?;
    writer.text(locale.resolved().as_str())?;
    Ok(writer.0)
}
pub fn decode_resolve_segmenter_locale_response(
    profiles: &SegmenterProfiles,
    data: &[u8],
) -> Result<ResolvedSegmenterLocale, SegmenterWireError> {
    let mut reader = Reader::new(data, SegmenterWireOperation::ResolveLocale, true)?;
    let locale = reader.locale()?;
    reader.finish()?;
    profiles.admit(locale).map_err(Into::into)
}
pub fn encode_supported_segmenter_locales_request(
    request: &SegmenterLocaleRequest,
) -> Result<Vec<u8>, SegmenterWireError> {
    write_locale_request(SegmenterWireOperation::SupportedLocales, request)
}
pub fn decode_supported_segmenter_locales_request(
    data: &[u8],
) -> Result<SegmenterLocaleRequest, SegmenterWireError> {
    read_locale_request(SegmenterWireOperation::SupportedLocales, data)
}
pub fn encode_supported_segmenter_locales_response(
    result: &SegmenterSupportedLocalesResult,
) -> Result<Vec<u8>, SegmenterWireError> {
    let mut writer = Writer::new(SegmenterWireOperation::SupportedLocales, true)?;
    writer.locales(&result.locales)?;
    Ok(writer.0)
}
pub fn decode_supported_segmenter_locales_response(
    profiles: &SegmenterProfiles,
    request: &SegmenterLocaleRequest,
    data: &[u8],
) -> Result<SegmenterSupportedLocalesResult, SegmenterWireError> {
    let mut reader = Reader::new(data, SegmenterWireOperation::SupportedLocales, true)?;
    let locales = reader.locales()?;
    reader.finish()?;
    // Returned tags preserve the supported original requests. Binding to the
    // checked catalogue and request excludes invented or reordered responses.
    let expected = profiles.supported_locales(request);
    if locales != expected.locales {
        return Err(SegmenterWireError::Malformed(
            "supported locale/request association",
        ));
    }
    Ok(SegmenterSupportedLocalesResult { locales })
}
pub fn encode_segment_utf16_request(
    request: &SegmentUtf16Request,
) -> Result<Vec<u8>, SegmenterWireError> {
    let mut writer = Writer::new(SegmenterWireOperation::SegmentUtf16, false)?;
    writer.text(request.configuration().locale().resolved().as_str())?;
    for word in request.configuration().wire_words() {
        writer.word(word)?;
    }
    let bytes = request
        .input()
        .len()
        .checked_mul(2)
        .ok_or(SegmenterWireError::Resource("UTF16 extent"))?;
    writer.word(bytes as u64)?;
    for unit in request.input() {
        writer.append(&unit.to_le_bytes())?;
    }
    Ok(writer.0)
}
pub fn decode_segment_utf16_request(
    profiles: &SegmenterProfiles,
    data: &[u8],
) -> Result<SegmentUtf16Request, SegmenterWireError> {
    let mut reader = Reader::new(data, SegmenterWireOperation::SegmentUtf16, false)?;
    let locale = profiles.admit(reader.locale()?)?;
    let granularity = SegmenterGranularity::from_wire_code(reader.word()?)
        .ok_or(SegmenterWireError::Malformed("granularity"))?;
    let bytes = reader.count(1)?;
    if bytes % 2 != 0 {
        return Err(SegmenterWireError::Malformed("odd UTF16 byte length"));
    }
    let mut input = Vec::new();
    input
        .try_reserve_exact(bytes / 2)
        .map_err(|_| SegmenterWireError::Resource("UTF16 allocation"))?;
    for _ in 0..bytes / 2 {
        input.push(u16::from_le_bytes(
            reader.take(2)?.try_into().expect("two bytes"),
        ));
    }
    reader.finish()?;
    SegmentUtf16Request::new(
        CheckedSegmenterConfiguration::new(locale, granularity),
        input.into_boxed_slice(),
    )
    .map_err(Into::into)
}
pub fn encode_segment_utf16_response(
    result: &SegmenterResult,
) -> Result<Vec<u8>, SegmenterWireError> {
    let mut writer = Writer::new(SegmenterWireOperation::SegmentUtf16, true)?;
    writer.word(result.granularity().wire_code())?;
    writer.word(result.input().len() as u64)?;
    writer.word(result.boundaries().len() as u64)?;
    for row in result.boundaries() {
        writer.word(u64::from(row.end()))?;
        writer.word(match row.is_word_like() {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        })?;
    }
    Ok(writer.0)
}
pub fn decode_segment_utf16_response(
    request: &SegmentUtf16Request,
    data: &[u8],
) -> Result<SegmenterResult, SegmenterWireError> {
    let mut reader = Reader::new(data, SegmenterWireOperation::SegmentUtf16, true)?;
    if reader.word()? != request.configuration().granularity().wire_code()
        || reader.word()? != request.input().len() as u64
    {
        return Err(SegmenterWireError::Malformed(
            "input or granularity association",
        ));
    }
    let count = reader.count(16)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| SegmenterWireError::Resource("boundary allocation"))?;
    for _ in 0..count {
        let end = u32::try_from(reader.word()?)
            .map_err(|_| SegmenterWireError::Malformed("boundary index"))?;
        let word = match reader.word()? {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return Err(SegmenterWireError::Malformed("word annotation")),
        };
        rows.push((end, word));
    }
    reader.finish()?;
    SegmenterResult::from_wire(request, rows).map_err(Into::into)
}
