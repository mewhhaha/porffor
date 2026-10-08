use super::*;

pub(super) struct Writer(Vec<u8>);
impl Writer {
    pub fn new(operation: RelativeHostOp, response: bool) -> Result<Self, RelativeWireError> {
        let mut writer = Self(Vec::new());
        writer.word(RELATIVE_WIRE_VERSION)?;
        writer.word(operation.code() * 2 + u64::from(response))?;
        Ok(writer)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), RelativeWireError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(RelativeWireError::Resource("message extent"))?;
        u32::try_from(length).map_err(|_| RelativeWireError::Resource("Wasm32 message extent"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| RelativeWireError::Resource("message allocation"))?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    pub fn word(&mut self, value: u64) -> Result<(), RelativeWireError> {
        self.append(&value.to_le_bytes())
    }
    pub fn text(&mut self, value: &str) -> Result<(), RelativeWireError> {
        self.word(value.len() as u64)?;
        self.append(value.as_bytes())
    }
    pub fn finish(self) -> Vec<u8> {
        self.0
    }
}
pub(super) struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    pub fn new(
        bytes: &'a [u8],
        operation: RelativeHostOp,
        response: bool,
    ) -> Result<Self, RelativeWireError> {
        u32::try_from(bytes.len())
            .map_err(|_| RelativeWireError::Resource("Wasm32 message extent"))?;
        let mut reader = Self(bytes);
        if reader.word()? != RELATIVE_WIRE_VERSION
            || reader.word()? != operation.code() * 2 + u64::from(response)
        {
            return Err(RelativeWireError::Malformed("version/operation/direction"));
        }
        Ok(reader)
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], RelativeWireError> {
        let (prefix, remaining) = self
            .0
            .split_at_checked(count)
            .ok_or(RelativeWireError::Malformed("truncated field"))?;
        self.0 = remaining;
        Ok(prefix)
    }
    pub fn word(&mut self) -> Result<u64, RelativeWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte word"),
        ))
    }
    pub fn domain<T>(&mut self, parse: fn(u64) -> Option<T>) -> Result<T, RelativeWireError> {
        parse(self.word()?).ok_or(RelativeWireError::Malformed("closed domain"))
    }
    pub fn count(&mut self, minimum_bytes: usize) -> Result<usize, RelativeWireError> {
        let count = u32::try_from(self.word()?)
            .map_err(|_| RelativeWireError::Malformed("Wasm32 count"))?
            as usize;
        if count > self.0.len() / minimum_bytes {
            return Err(RelativeWireError::Malformed("count exceeds owned message"));
        }
        Ok(count)
    }
    pub fn text(&mut self) -> Result<&'a str, RelativeWireError> {
        let length = u32::try_from(self.word()?)
            .map_err(|_| RelativeWireError::Malformed("Wasm32 text extent"))?
            as usize;
        std::str::from_utf8(self.take(length)?).map_err(|_| RelativeWireError::Malformed("UTF8"))
    }
    pub fn locale(
        &mut self,
        limits: &PartitionLimits,
    ) -> Result<CanonicalLocaleId, RelativeWireError> {
        let text = self.text()?;
        let owned =
            crate::number_format::owned_text(text, limits).map_err(RelativeTimeError::from)?;
        CanonicalLocaleId::from_data(owned)
            .map_err(|_| RelativeWireError::Malformed("canonical locale"))
    }
    pub fn finish(self) -> Result<(), RelativeWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(RelativeWireError::Malformed("trailing bytes"))
        }
    }
}
