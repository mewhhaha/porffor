//! Same checked primitive framing as the admitted relative-time wire boundary.
use super::*;

pub(super) struct Writer(Vec<u8>);
impl Writer {
    pub fn new(op: DurationHostOp, response: bool) -> Result<Self, DurationWireError> {
        let mut writer = Self(Vec::new());
        writer.word(DURATION_WIRE_VERSION)?;
        writer.word(u64::from(op.proposed_global_tag()) * 2 + u64::from(response))?;
        Ok(writer)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), DurationWireError> {
        let length = self
            .0
            .len()
            .checked_add(bytes.len())
            .ok_or(DurationWireError::Resource("message extent"))?;
        u32::try_from(length).map_err(|_| DurationWireError::Resource("Wasm32 message extent"))?;
        self.0
            .try_reserve(bytes.len())
            .map_err(|_| DurationWireError::Resource("message allocation"))?;
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    pub fn word(&mut self, value: u64) -> Result<(), DurationWireError> {
        self.append(&value.to_le_bytes())
    }
    pub fn text(&mut self, text: &str) -> Result<(), DurationWireError> {
        self.word(text.len() as u64)?;
        self.append(text.as_bytes())
    }
    pub fn finish(self) -> Vec<u8> {
        self.0
    }
}
pub(super) struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    pub fn new(
        bytes: &'a [u8],
        op: DurationHostOp,
        response: bool,
    ) -> Result<Self, DurationWireError> {
        u32::try_from(bytes.len())
            .map_err(|_| DurationWireError::Resource("Wasm32 message extent"))?;
        let mut reader = Self(bytes);
        if reader.word()? != DURATION_WIRE_VERSION
            || reader.word()? != u64::from(op.proposed_global_tag()) * 2 + u64::from(response)
        {
            return Err(DurationWireError::Malformed("version/operation/direction"));
        }
        Ok(reader)
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], DurationWireError> {
        let (prefix, remaining) = self
            .0
            .split_at_checked(count)
            .ok_or(DurationWireError::Malformed("truncated field"))?;
        self.0 = remaining;
        Ok(prefix)
    }
    pub fn word(&mut self) -> Result<u64, DurationWireError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte word"),
        ))
    }
    pub fn count(&mut self, minimum_bytes: usize) -> Result<usize, DurationWireError> {
        let count = u32::try_from(self.word()?)
            .map_err(|_| DurationWireError::Malformed("Wasm32 count"))?
            as usize;
        if count > self.0.len() / minimum_bytes {
            return Err(DurationWireError::Malformed("count exceeds owned message"));
        }
        Ok(count)
    }
    pub fn text(&mut self) -> Result<&'a str, DurationWireError> {
        let length = u32::try_from(self.word()?)
            .map_err(|_| DurationWireError::Malformed("Wasm32 text extent"))?
            as usize;
        std::str::from_utf8(self.take(length)?).map_err(|_| DurationWireError::Malformed("UTF8"))
    }
    pub fn locale(
        &mut self,
        limits: &PartitionLimits,
    ) -> Result<CanonicalLocaleId, DurationWireError> {
        let owned =
            crate::number_format::owned_text(self.text()?, limits).map_err(DurationError::from)?;
        CanonicalLocaleId::from_data(owned)
            .map_err(|_| DurationWireError::Malformed("canonical locale"))
    }
    pub fn finish(self) -> Result<(), DurationWireError> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(DurationWireError::Malformed("trailing bytes"))
        }
    }
}
