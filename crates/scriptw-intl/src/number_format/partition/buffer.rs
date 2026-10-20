use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Owner {
    Number,
    Sign,
    Notation,
    Measurement,
    Structure,
}

#[derive(Debug)]
pub(super) struct Piece {
    pub part: NumberPart,
    pub owner: Owner,
}

#[derive(Debug)]
pub(super) struct Pieces {
    pub rows: Vec<Piece>,
    bytes: usize,
}

pub(super) fn bidi(character: char) -> bool {
    matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

impl Pieces {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            bytes: 0,
        }
    }

    pub fn push(
        &mut self,
        kind: NumberPartKind,
        text: &str,
        owner: Owner,
        limits: &PartitionLimits,
    ) -> Result<(), NumberFormatKernelError> {
        if text.is_empty() {
            return Ok(());
        }
        let bytes = limits.check_bytes(self.bytes as u128 + text.len() as u128)?;
        limits.check_parts(self.rows.len() as u128 + 1)?;
        self.rows
            .try_reserve(1)
            .map_err(|_| NumberPartitionResourceError::Allocation)?;
        self.rows.push(Piece {
            part: NumberPart::new(kind, owned_text(text, limits)?),
            owner,
        });
        self.bytes = bytes;
        Ok(())
    }

    pub fn symbol(
        &mut self,
        kind: NumberPartKind,
        text: &str,
        owner: Owner,
        limits: &PartitionLimits,
    ) -> Result<(), NumberFormatKernelError> {
        let mut start = 0;
        let mut current = None;
        for (position, character) in text.char_indices() {
            let literal = bidi(character);
            if current.is_some_and(|prior| prior != literal) {
                self.push(
                    if current == Some(true) {
                        NumberPartKind::Literal
                    } else {
                        kind
                    },
                    &text[start..position],
                    owner,
                    limits,
                )?;
                start = position;
            }
            current = Some(literal);
        }
        self.push(
            if current == Some(true) {
                NumberPartKind::Literal
            } else {
                kind
            },
            &text[start..],
            owner,
            limits,
        )
    }

    pub fn append(
        &mut self,
        other: &Self,
        limits: &PartitionLimits,
    ) -> Result<(), NumberFormatKernelError> {
        for piece in &other.rows {
            self.push(piece.part.kind(), piece.part.text(), piece.owner, limits)?;
        }
        Ok(())
    }

    pub fn same_text(&self, other: &Self) -> bool {
        self.rows
            .iter()
            .flat_map(|piece| piece.part.text().bytes())
            .eq(other
                .rows
                .iter()
                .flat_map(|piece| piece.part.text().bytes()))
    }
}

pub(super) fn digit_text(
    digits: &[u8],
    system: &NumberingSystem,
    limits: &PartitionLimits,
) -> Result<Box<str>, NumberFormatKernelError> {
    let bytes: u128 = digits
        .iter()
        .map(|digit| system.digits[usize::from(*digit)].len_utf8() as u128)
        .sum();
    let bytes = limits.check_bytes(bytes)?;
    let mut result = String::new();
    result
        .try_reserve_exact(bytes)
        .map_err(|_| NumberPartitionResourceError::Allocation)?;
    result.extend(
        digits
            .iter()
            .map(|digit| system.digits[usize::from(*digit)]),
    );
    Ok(result.into_boxed_str())
}
