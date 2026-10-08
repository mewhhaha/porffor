//! ICU chooses templates; the collector never renders an input element.
use super::*;
use icu_list::parts;
use writeable::{Part, PartsWrite, Writeable};

struct ConditionInput<'a>(&'a [u16]);
impl Writeable for ConditionInput<'_> {
    fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result {
        // Selected ILD template policy only. These bytes never enter output.
        // Real pairs remain scalars; isolated surrogates become U+FFFD.
        for scalar in char::decode_utf16(self.0.iter().copied()) {
            sink.write_char(scalar.unwrap_or(char::REPLACEMENT_CHARACTER))?;
        }
        Ok(())
    }
}
struct Collector {
    parts: Vec<ListPart>,
    next: u32,
    input_count: u32,
    in_literal: bool,
    literal: Vec<u16>,
    failure: Option<ListFormatOperationError>,
    encoded_bytes: usize,
}
impl Collector {
    fn fail(&mut self, error: ListFormatOperationError) -> fmt::Result {
        self.failure = Some(error);
        Err(fmt::Error)
    }
    fn account(&mut self, additional: usize) -> fmt::Result {
        let Some(total) = self
            .encoded_bytes
            .checked_add(additional)
            .filter(|n| u32::try_from(*n).is_ok())
        else {
            return self.fail(ListFormatOperationError::Resource(
                "partition exceeds Wasm32 span",
            ));
        };
        self.encoded_bytes = total;
        Ok(())
    }
    fn push(&mut self, part: ListPart) -> fmt::Result {
        if self.parts.try_reserve(1).is_err() {
            return self.fail(ListFormatOperationError::Resource("part allocation"));
        }
        self.parts.push(part);
        Ok(())
    }
}
impl fmt::Write for Collector {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if !self.in_literal {
            return self.fail(ListFormatOperationError::InvalidPartition);
        }
        let count = text.encode_utf16().count();
        let Some(_) = self
            .literal
            .len()
            .checked_add(count)
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(16))
            .and_then(|n| n.checked_add(self.encoded_bytes))
            .filter(|n| u32::try_from(*n).is_ok())
        else {
            return self.fail(ListFormatOperationError::Resource(
                "literal exceeds Wasm32 span",
            ));
        };
        if self.literal.try_reserve(count).is_err() {
            return self.fail(ListFormatOperationError::Resource("literal allocation"));
        }
        self.literal.extend(text.encode_utf16());
        Ok(())
    }
}
impl PartsWrite for Collector {
    type SubPartsWrite = Self;
    fn with_part(
        &mut self,
        part: Part,
        mut closure: impl FnMut(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        if self.in_literal {
            return self.fail(ListFormatOperationError::InvalidPartition);
        }
        if part == parts::ELEMENT {
            if self.next >= self.input_count {
                return self.fail(ListFormatOperationError::InvalidPartition);
            }
            // The private input closure is pure. Skip rendering it and retain
            // its original index even when its JS string has zero code units.
            self.account(16)?;
            self.push(ListPart::Element(self.next))?;
            self.next += 1;
            Ok(())
        } else if part == parts::LITERAL {
            self.in_literal = true;
            let result = closure(self);
            self.in_literal = false;
            result?;
            let literal = core::mem::take(&mut self.literal);
            if !literal.is_empty() {
                let Some(extent) = literal.len().checked_mul(2).and_then(|n| n.checked_add(16))
                else {
                    return self.fail(ListFormatOperationError::Resource("literal extent"));
                };
                self.account(extent)?;
                self.push(ListPart::Literal(literal.into_boxed_slice()))?;
            }
            Ok(())
        } else {
            self.fail(ListFormatOperationError::InvalidPartition)
        }
    }
}
pub(super) fn format(
    request: &FormatListPartsRequest,
) -> Result<ListParts, ListFormatOperationError> {
    let count = u32::try_from(request.elements.len())
        .map_err(|_| ListFormatOperationError::Resource("element count"))?;
    let mut collector = Collector {
        parts: Vec::new(),
        next: 0,
        input_count: count,
        in_literal: false,
        literal: Vec::new(),
        failure: None,
        encoded_bytes: 24,
    };
    let config = &request.configuration;
    let formatted = config
        .profile()
        .formatter(config.kind, config.style)
        .format(request.elements.iter().map(|e| ConditionInput(e)));
    if formatted.write_to_parts(&mut collector).is_err() {
        return Err(collector
            .failure
            .unwrap_or(ListFormatOperationError::InvalidPartition));
    }
    ListParts::checked(collector.parts, count)
}
