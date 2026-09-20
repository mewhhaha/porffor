use super::*;

pub(super) struct NumberWireWriter(Vec<u8>);
impl NumberWireWriter {
    pub(super) fn new(
        operation: IntlHostOp,
        direction: NumberWireDirection,
    ) -> Result<Self, NumberWireError> {
        let mut writer = Self(Vec::new());
        writer.word(NUMBER_WIRE_VERSION)?;
        writer.word(direction.message_code(operation))?;
        Ok(writer)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), NumberWireError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(NumberWireError::Resource("message length overflow"))?;
        u32::try_from(length)
            .map_err(|_| NumberWireError::Resource("message exceeds Wasm32 span"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| NumberWireError::Resource("message allocation"))?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    pub(super) fn word(&mut self, value: u64) -> Result<(), NumberWireError> {
        self.append(&value.to_le_bytes())
    }
    pub(super) fn bytes(&mut self, bytes: &[u8]) -> Result<(), NumberWireError> {
        self.word(bytes.len() as u64)?;
        self.append(bytes)
    }
    pub(super) fn text(&mut self, text: &str) -> Result<(), NumberWireError> {
        self.bytes(text.as_bytes())
    }
    pub(super) fn text_fragments(&mut self, fragments: &[&str]) -> Result<(), NumberWireError> {
        let length = fragments
            .iter()
            .try_fold(0usize, |sum, part| sum.checked_add(part.len()))
            .ok_or(NumberWireError::Resource("text length overflow"))?;
        self.word(length as u64)?;
        for part in fragments {
            self.append(part.as_bytes())?;
        }
        Ok(())
    }
    pub(super) fn locales(&mut self, locales: &[CanonicalLocaleId]) -> Result<(), NumberWireError> {
        self.word(locales.len() as u64)?;
        for locale in locales {
            self.text(locale.as_str())?;
        }
        Ok(())
    }
    pub(super) fn resolved_locale(
        &mut self,
        locale: &ResolvedNumberLocale,
    ) -> Result<(), NumberWireError> {
        self.text(locale.resolved().as_str())?;
        self.text(locale.formatting().as_str())?;
        self.text(locale.numbering_system().name())
    }
    pub(super) fn input(&mut self, input: &ObservedNumericInput) -> Result<(), NumberWireError> {
        match input {
            ObservedNumericInput::StringNumericLiteral(units) => {
                self.word(NumberNumericKind::String.wire_code())?;
                let bytes = units
                    .len()
                    .checked_mul(2)
                    .ok_or(NumberWireError::Resource("UTF16 input extent"))?;
                self.word(bytes as u64)?;
                for unit in units {
                    self.append(&unit.to_le_bytes())?;
                }
                Ok(())
            }
            ObservedNumericInput::NumberShortestDecimal(text) => {
                self.word(NumberNumericKind::Number.wire_code())?;
                self.text(text)
            }
            ObservedNumericInput::NegativeZero => {
                self.word(NumberNumericKind::NegativeZero.wire_code())?;
                self.bytes(&[])
            }
            ObservedNumericInput::BigIntDecimal(text) => {
                self.word(NumberNumericKind::BigInt.wire_code())?;
                self.text(text)
            }
        }
    }
    pub(super) fn finish(self) -> Vec<u8> {
        self.0
    }
}

pub(super) struct NumberWireReader<'a>(&'a [u8]);
impl<'a> NumberWireReader<'a> {
    pub(super) fn new(
        bytes: &'a [u8],
        operation: IntlHostOp,
        direction: NumberWireDirection,
    ) -> Result<Self, NumberWireError> {
        u32::try_from(bytes.len())
            .map_err(|_| NumberWireError::Resource("message exceeds Wasm32 span"))?;
        let mut reader = Self(bytes);
        if reader.word()? != NUMBER_WIRE_VERSION
            || reader.word()? != direction.message_code(operation)
        {
            return Err(NumberWireError::Malformed("version or operation"));
        }
        Ok(reader)
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], NumberWireError> {
        let (prefix, remaining) = self
            .0
            .split_at_checked(count)
            .ok_or(NumberWireError::Malformed("truncated field"))?;
        self.0 = remaining;
        Ok(prefix)
    }
    pub(super) fn word(&mut self) -> Result<u64, NumberWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte word"),
        ))
    }
    pub(super) fn domain<T>(&mut self, decode: fn(u64) -> Option<T>) -> Result<T, NumberWireError> {
        decode(self.word()?).ok_or(NumberWireError::Malformed("unknown closed-domain code"))
    }
    pub(super) fn count(&mut self, minimum_record_bytes: usize) -> Result<usize, NumberWireError> {
        let count = u32::try_from(self.word()?)
            .map_err(|_| NumberWireError::Malformed("count exceeds Wasm32"))?
            as usize;
        if count > self.0.len() / minimum_record_bytes {
            return Err(NumberWireError::Malformed("count exceeds owned message"));
        }
        Ok(count)
    }
    pub(super) fn bytes(&mut self) -> Result<&'a [u8], NumberWireError> {
        let length = u32::try_from(self.word()?)
            .map_err(|_| NumberWireError::Malformed("field exceeds Wasm32"))?
            as usize;
        self.take(length)
    }
    pub(super) fn text(&mut self) -> Result<&'a str, NumberWireError> {
        core::str::from_utf8(self.bytes()?)
            .map_err(|_| NumberWireError::Malformed("invalid UTF8 text"))
    }
    pub(super) fn owned_text(&mut self) -> Result<Box<str>, NumberWireError> {
        let text = self.text()?;
        Self::copy_text(text)
    }
    fn copy_text(text: &str) -> Result<Box<str>, NumberWireError> {
        let mut owned = String::new();
        owned
            .try_reserve_exact(text.len())
            .map_err(|_| NumberWireError::Resource("text allocation"))?;
        owned.push_str(text);
        Ok(owned.into_boxed_str())
    }
    pub(super) fn locale(&mut self) -> Result<CanonicalLocaleId, NumberWireError> {
        CanonicalLocaleId::from_data(self.owned_text()?)
            .map_err(|_| NumberWireError::Malformed("invalid canonical locale"))
    }
    pub(super) fn locales(&mut self) -> Result<Box<[CanonicalLocaleId]>, NumberWireError> {
        let count = self.count(8)?;
        let mut locales = Vec::new();
        locales
            .try_reserve_exact(count)
            .map_err(|_| NumberWireError::Resource("locale list allocation"))?;
        for _ in 0..count {
            locales.push(self.locale()?);
        }
        Ok(locales.into_boxed_slice())
    }
    pub(super) fn resolved_locale(
        &mut self,
        profiles: &NumberProfiles,
    ) -> Result<ResolvedNumberLocale, NumberWireError> {
        let resolved = self.locale()?;
        let formatting = self.locale()?;
        let numbering = self.text()?;
        Ok(ResolvedNumberLocale::from_resolved(
            resolved, formatting, numbering, profiles,
        )?)
    }
    pub(super) fn input(&mut self) -> Result<ObservedNumericInput, NumberWireError> {
        let kind = self.domain(NumberNumericKind::from_wire_code)?;
        let bytes = self.bytes()?;
        match kind {
            NumberNumericKind::String => {
                if bytes.len() % 2 != 0 {
                    return Err(NumberWireError::Malformed("odd UTF16 byte length"));
                }
                let mut units = Vec::new();
                units
                    .try_reserve_exact(bytes.len() / 2)
                    .map_err(|_| NumberWireError::Resource("UTF16 allocation"))?;
                for pair in bytes.chunks_exact(2) {
                    units.push(u16::from_le_bytes([pair[0], pair[1]]));
                }
                Ok(ObservedNumericInput::StringNumericLiteral(
                    units.into_boxed_slice(),
                ))
            }
            NumberNumericKind::Number | NumberNumericKind::BigInt => {
                if !bytes.is_ascii() {
                    return Err(NumberWireError::Malformed(
                        "non-ASCII intrinsic numeric text",
                    ));
                }
                let text = Self::copy_text(core::str::from_utf8(bytes).expect("ASCII is UTF8"))?;
                Ok(match kind {
                    NumberNumericKind::Number => ObservedNumericInput::NumberShortestDecimal(text),
                    NumberNumericKind::BigInt => ObservedNumericInput::BigIntDecimal(text),
                    NumberNumericKind::String | NumberNumericKind::NegativeZero => {
                        unreachable!("selected text kind")
                    }
                })
            }
            NumberNumericKind::NegativeZero => {
                if !bytes.is_empty() {
                    return Err(NumberWireError::Malformed("negative zero has a payload"));
                }
                Ok(ObservedNumericInput::NegativeZero)
            }
        }
    }
    pub(super) fn finish(self) -> Result<(), NumberWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(NumberWireError::Malformed("trailing bytes"))
        }
    }
}
