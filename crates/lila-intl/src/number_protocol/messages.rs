use super::*;
use NumberWireDirection::{Request, Response};

impl NumberLocaleRequest {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::ResolveNumberLocale, Request)?;
        writer.word(self.matcher.wire_code())?;
        writer.text(
            self.numbering_system
                .as_ref()
                .map_or("", NumberingSystemOption::name),
        )?;
        writer.locales(&self.requested)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::ResolveNumberLocale, Request)?;
        let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
        let spelling = reader.text()?;
        let numbering_system = if spelling.is_empty() {
            None
        } else {
            Some(
                NumberingSystemOption::parse(spelling).map_err(|error| match error {
                    InvalidNumberingSystemOption::Syntax => {
                        NumberWireError::Malformed("invalid numbering-system option")
                    }
                    InvalidNumberingSystemOption::Allocation => {
                        NumberWireError::Resource("numbering-system option allocation")
                    }
                })?,
            )
        };
        let requested = reader.locales()?;
        reader.finish()?;
        Ok(Self {
            requested,
            matcher,
            numbering_system,
        })
    }
}
impl ResolvedNumberLocale {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::ResolveNumberLocale, Response)?;
        writer.resolved_locale(self)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &NumberProfiles) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::ResolveNumberLocale, Response)?;
        let result = reader.resolved_locale(profiles)?;
        reader.finish()?;
        Ok(result)
    }
}
impl NumberSupportedLocalesRequest {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SupportedNumberLocales, Request)?;
        writer.word(self.matcher.wire_code())?;
        writer.locales(&self.requested)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::SupportedNumberLocales, Request)?;
        let matcher = reader.domain(LocaleMatcher::from_wire_code)?;
        let requested = reader.locales()?;
        reader.finish()?;
        Ok(Self { requested, matcher })
    }
}
impl NumberSupportedLocalesResult {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::SupportedNumberLocales, Response)?;
        writer.locales(&self.locales)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, NumberWireError> {
        let mut reader =
            NumberWireReader::new(bytes, IntlHostOp::SupportedNumberLocales, Response)?;
        let locales = reader.locales()?;
        reader.finish()?;
        Ok(Self { locales })
    }
}
impl NumberFormatRequest {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::FormatNumberParts, Request)?;
        writer.configuration(&self.configuration)?;
        writer.input(&self.input)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &NumberProfiles) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::FormatNumberParts, Request)?;
        let configuration = reader.configuration(profiles)?;
        let input = reader.input()?;
        reader.finish()?;
        Ok(Self {
            configuration,
            input,
        })
    }
}
impl NumberRangeFormatRequest {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::FormatNumberRangeParts, Request)?;
        writer.configuration(&self.configuration)?;
        writer.input(&self.start)?;
        writer.input(&self.end)?;
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8], profiles: &NumberProfiles) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::FormatNumberRangeParts, Request)?;
        let configuration = reader.configuration(profiles)?;
        let start = reader.input()?;
        let end = reader.input()?;
        reader.finish()?;
        Ok(Self {
            configuration,
            start,
            end,
        })
    }
}
impl ScalarNumberPartition {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::FormatNumberParts, Response)?;
        writer.word(self.parts().len() as u64)?;
        for part in self.parts() {
            writer.word(part.kind().wire_code())?;
            writer.text(part.text())?;
        }
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, NumberWireError> {
        let mut reader = NumberWireReader::new(bytes, IntlHostOp::FormatNumberParts, Response)?;
        let count = reader.count(16)?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(count)
            .map_err(|_| NumberWireError::Resource("part list allocation"))?;
        for _ in 0..count {
            let kind = reader.domain(NumberPartKind::from_wire_code)?;
            parts.push(NumberPart::new(kind, reader.owned_text()?));
        }
        reader.finish()?;
        Ok(Self::from_parts(parts.into_boxed_slice()))
    }
}
impl RangeNumberPartition {
    pub fn encode(&self) -> Result<Vec<u8>, NumberWireError> {
        let mut writer = NumberWireWriter::new(IntlHostOp::FormatNumberRangeParts, Response)?;
        writer.word(self.parts().len() as u64)?;
        for part in self.parts() {
            let code = match part {
                NumberRangePart::Number { part, .. } => part.kind().wire_code(),
                NumberRangePart::ApproximatelySign(_) => NUMBER_APPROXIMATELY_SIGN_CODE,
            };
            writer.word(code)?;
            writer.word(part.source().wire_code())?;
            writer.text(part.text())?;
        }
        Ok(writer.finish())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, NumberWireError> {
        let mut reader =
            NumberWireReader::new(bytes, IntlHostOp::FormatNumberRangeParts, Response)?;
        let count = reader.count(24)?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(count)
            .map_err(|_| NumberWireError::Resource("range part list allocation"))?;
        for _ in 0..count {
            let code = reader.word()?;
            let source = reader.domain(RangePartSource::from_wire_code)?;
            let text = reader.owned_text()?;
            let part = if code == NUMBER_APPROXIMATELY_SIGN_CODE {
                if source != RangePartSource::Shared {
                    return Err(NumberWireError::Malformed("approximation is not shared"));
                }
                NumberRangePart::ApproximatelySign(text)
            } else {
                let kind = NumberPartKind::from_wire_code(code)
                    .ok_or(NumberWireError::Malformed("unknown range part kind"))?;
                NumberRangePart::Number {
                    source,
                    part: NumberPart::new(kind, text),
                }
            };
            parts.push(part);
        }
        reader.finish()?;
        Ok(Self::from_parts(parts.into_boxed_slice()))
    }
}
