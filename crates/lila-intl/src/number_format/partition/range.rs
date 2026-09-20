use super::*;

struct RangeBuilder<'a> {
    parts: Vec<NumberRangePart>,
    bytes: usize,
    limits: &'a PartitionLimits,
}

impl<'a> RangeBuilder<'a> {
    fn new(limits: &'a PartitionLimits) -> Self {
        Self {
            parts: Vec::new(),
            bytes: 0,
            limits,
        }
    }

    fn reserve(&mut self, text: &str) -> Result<(), NumberFormatKernelError> {
        self.bytes = self
            .limits
            .check_bytes(self.bytes as u128 + text.len() as u128)?;
        self.limits.check_parts(self.parts.len() as u128 + 1)?;
        self.parts
            .try_reserve(1)
            .map_err(|_| NumberPartitionResourceError::Allocation)?;
        Ok(())
    }

    fn number(
        &mut self,
        source: RangePartSource,
        kind: NumberPartKind,
        text: &str,
    ) -> Result<(), NumberFormatKernelError> {
        if text.is_empty() {
            return Ok(());
        }
        if kind == NumberPartKind::Literal {
            if let Some(NumberRangePart::Number {
                source: previous_source,
                part,
            }) = self.parts.last_mut()
            {
                if *previous_source == source && part.kind() == NumberPartKind::Literal {
                    let bytes = self
                        .limits
                        .check_bytes(self.bytes as u128 + text.len() as u128)?;
                    let mut joined = String::new();
                    joined
                        .try_reserve_exact(part.text().len() + text.len())
                        .map_err(|_| NumberPartitionResourceError::Allocation)?;
                    joined.push_str(part.text());
                    joined.push_str(text);
                    *part = NumberPart::new(kind, joined.into_boxed_str());
                    self.bytes = bytes;
                    return Ok(());
                }
            }
        }
        self.reserve(text)?;
        self.parts.push(NumberRangePart::Number {
            source,
            part: NumberPart::new(kind, owned_text(text, self.limits)?),
        });
        Ok(())
    }

    fn pieces(
        &mut self,
        source: RangePartSource,
        pieces: &[Piece],
    ) -> Result<(), NumberFormatKernelError> {
        for piece in pieces {
            self.number(source, piece.part.kind(), piece.part.text())?;
        }
        Ok(())
    }

    fn approximately(&mut self, text: &str) -> Result<(), NumberFormatKernelError> {
        let mut start = 0;
        let mut current = None;
        for (position, character) in text.char_indices() {
            let literal = bidi(character);
            if current.is_some_and(|prior| prior != literal) {
                self.approximate_run(&text[start..position], current == Some(true))?;
                start = position;
            }
            current = Some(literal);
        }
        self.approximate_run(&text[start..], current == Some(true))
    }

    fn approximate_run(
        &mut self,
        text: &str,
        literal: bool,
    ) -> Result<(), NumberFormatKernelError> {
        if text.is_empty() {
            return Ok(());
        }
        if literal {
            return self.number(RangePartSource::Shared, NumberPartKind::Literal, text);
        }
        self.reserve(text)?;
        self.parts
            .push(NumberRangePart::ApproximatelySign(owned_text(
                text,
                self.limits,
            )?));
        Ok(())
    }

    fn last_character(&self) -> Option<char> {
        self.parts
            .last()
            .and_then(|part| part.text().chars().next_back())
    }

    fn finish(self) -> RangeNumberPartition {
        RangeNumberPartition::from_parts(self.parts.into_boxed_slice())
    }
}

#[derive(Clone, Copy)]
struct AffixBounds {
    prefix: usize,
    suffix: usize,
}

#[derive(Clone, Copy)]
enum AffixPolicy {
    Measurement,
    SignedPattern,
}

impl AffixPolicy {
    fn for_style(style: &NumberStyle) -> Self {
        match style {
            NumberStyle::Percent
            | NumberStyle::Currency {
                display:
                    CurrencyDisplay::Code | CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol,
                ..
            } => Self::SignedPattern,
            NumberStyle::Decimal
            | NumberStyle::Unit { .. }
            | NumberStyle::Currency {
                display: CurrencyDisplay::Name,
                ..
            } => Self::Measurement,
        }
    }

    fn contains(self, owner: Owner) -> bool {
        match (self, owner) {
            (_, Owner::Measurement) => true,
            (Self::SignedPattern, Owner::Sign | Owner::Structure) => true,
            (Self::Measurement, Owner::Sign | Owner::Structure)
            | (_, Owner::Number | Owner::Notation) => false,
        }
    }
}

fn bounds(pieces: &Pieces, policy: AffixPolicy) -> AffixBounds {
    if !pieces.rows.iter().any(|piece| piece.owner == Owner::Number) {
        return AffixBounds {
            prefix: 0,
            suffix: pieces.rows.len(),
        };
    }
    let prefix = pieces
        .rows
        .iter()
        .take_while(|piece| policy.contains(piece.owner))
        .count();
    let suffix_count = pieces
        .rows
        .iter()
        .rev()
        .take_while(|piece| policy.contains(piece.owner))
        .count();
    AffixBounds {
        prefix,
        suffix: pieces.rows.len() - suffix_count,
    }
}

fn has_measurement(pieces: &[Piece]) -> bool {
    pieces.iter().any(|piece| {
        matches!(
            piece.part.kind(),
            NumberPartKind::Currency | NumberPartKind::Unit | NumberPartKind::PercentSign
        )
    })
}

fn pattern_extent(prefix: &[Piece], suffix: &[Piece]) -> usize {
    prefix
        .iter()
        .chain(suffix)
        .flat_map(|piece| piece.part.text().chars())
        .filter(|character| !bidi(*character))
        .count()
}

fn equivalent(pieces: &[Piece], other: &[Piece]) -> bool {
    pieces.len() == other.len()
        && pieces
            .iter()
            .zip(other)
            .all(|(left, right)| left.part == right.part && left.owner == right.owner)
}

struct Endpoint<'a> {
    shared_prefix: &'a [Piece],
    body: &'a [Piece],
    shared_suffix: &'a [Piece],
    source: RangePartSource,
}

impl Endpoint<'_> {
    fn first(&self) -> Option<char> {
        self.shared_prefix
            .iter()
            .chain(self.body)
            .chain(self.shared_suffix)
            .find_map(|piece| piece.part.text().chars().next())
    }

    fn last(&self) -> Option<char> {
        self.shared_prefix
            .iter()
            .chain(self.body)
            .chain(self.shared_suffix)
            .rev()
            .find_map(|piece| piece.part.text().chars().next_back())
    }

    fn append(&self, output: &mut RangeBuilder<'_>) -> Result<(), NumberFormatKernelError> {
        output.pieces(RangePartSource::Shared, self.shared_prefix)?;
        output.pieces(self.source, self.body)?;
        output.pieces(RangePartSource::Shared, self.shared_suffix)
    }
}

pub fn partition_number_range(
    configuration: &NumberFormatConfiguration,
    range: &NumberRange,
    profiles: &NumberProfiles,
    limits: &PartitionLimits,
) -> Result<RangeNumberPartition, NumberFormatKernelError> {
    let context = FormatContext::new(configuration, profiles, limits)?;
    let start = context.format(range.start().into(), None)?;
    let end = context.format(range.end().into(), None)?;
    let mut result = RangeBuilder::new(limits);
    if start.pieces.same_text(&end.pieces) {
        result.pieces(
            RangePartSource::Shared,
            &start.pieces.rows[..start.approximate_at],
        )?;
        result.approximately(profiles.text(context.symbols.approximately))?;
        result.pieces(
            RangePartSource::Shared,
            &start.pieces.rows[start.approximate_at..],
        )?;
        return Ok(result.finish());
    }
    let policy = AffixPolicy::for_style(&context.options.style);
    let start_bounds = bounds(&start.pieces, policy);
    let end_bounds = bounds(&end.pieces, policy);
    let can_prefix = has_measurement(&start.pieces.rows[..start_bounds.prefix])
        && has_measurement(&end.pieces.rows[..end_bounds.prefix]);
    let can_suffix = has_measurement(&start.pieces.rows[start_bounds.suffix..])
        && has_measurement(&end.pieces.rows[end_bounds.suffix..]);
    let category =
        profiles.range_category(context.locale.plural_ranges, start.category, end.category);
    let candidates = if can_prefix || can_suffix {
        Some((
            context.format(range.start().into(), Some(category))?,
            context.format(range.end().into(), Some(category))?,
        ))
    } else {
        None
    };
    let mut shared_prefix: &[Piece] = &[];
    let mut shared_suffix: &[Piece] = &[];
    if let Some((left, right)) = &candidates {
        let left_bounds = bounds(&left.pieces, policy);
        let right_bounds = bounds(&right.pieces, policy);
        let left_prefix = &left.pieces.rows[..left_bounds.prefix];
        let right_prefix = &right.pieces.rows[..right_bounds.prefix];
        let left_suffix = &left.pieces.rows[left_bounds.suffix..];
        let right_suffix = &right.pieces.rows[right_bounds.suffix..];
        match policy {
            AffixPolicy::Measurement => {
                if can_prefix
                    && has_measurement(left_prefix)
                    && equivalent(left_prefix, right_prefix)
                {
                    shared_prefix = left_prefix;
                }
                if can_suffix
                    && has_measurement(right_suffix)
                    && equivalent(left_suffix, right_suffix)
                {
                    shared_suffix = right_suffix;
                }
            }
            AffixPolicy::SignedPattern => {
                // A currency/percent pattern owns its signs and spacing on both
                // sides. Collapsing just one side would detach that ownership.
                if equivalent(left_prefix, right_prefix)
                    && equivalent(left_suffix, right_suffix)
                    && pattern_extent(left_prefix, left_suffix) > 1
                {
                    shared_prefix = left_prefix;
                    shared_suffix = right_suffix;
                }
            }
        }
    }
    let collapse_prefix = !shared_prefix.is_empty();
    let collapse_suffix = !shared_suffix.is_empty();
    let start_endpoint = Endpoint {
        shared_prefix,
        body: &start.pieces.rows[if collapse_prefix {
            start_bounds.prefix
        } else {
            0
        }..if collapse_suffix {
            start_bounds.suffix
        } else {
            start.pieces.rows.len()
        }],
        shared_suffix: &[],
        source: RangePartSource::Start,
    };
    let end_endpoint = Endpoint {
        shared_prefix: &[],
        body: &end.pieces.rows[if collapse_prefix {
            end_bounds.prefix
        } else {
            0
        }..if collapse_suffix {
            end_bounds.suffix
        } else {
            end.pieces.rows.len()
        }],
        shared_suffix,
        source: RangePartSource::End,
    };
    let pattern = profiles.pattern(context.numbering.range_pattern);
    let first_argument = pattern
        .0
        .iter()
        .position(|token| matches!(token, Token::Number | Token::Argument1))
        .expect("validated range pattern");
    let last_argument = pattern
        .0
        .iter()
        .rposition(|token| matches!(token, Token::Number | Token::Argument1))
        .expect("validated range pattern");
    let (first_endpoint, last_endpoint) = if matches!(pattern.0[first_argument], Token::Number) {
        (&start_endpoint, &end_endpoint)
    } else {
        (&end_endpoint, &start_endpoint)
    };
    let add_spacing = first_endpoint
        .last()
        .is_some_and(|character| !profiles.is_digit(character))
        || last_endpoint
            .first()
            .is_some_and(|character| !profiles.is_digit(character));
    for (position, token) in pattern.0.iter().enumerate() {
        if add_spacing
            && position == last_argument
            && result
                .last_character()
                .is_some_and(|character| !profiles.is_whitespace(character))
        {
            result.number(RangePartSource::Shared, NumberPartKind::Literal, " ")?;
        }
        match token {
            Token::Number => start_endpoint.append(&mut result)?,
            Token::Argument1 => end_endpoint.append(&mut result)?,
            Token::Literal(text) => result.number(
                RangePartSource::Shared,
                NumberPartKind::Literal,
                profiles.text(*text),
            )?,
            Token::MinusSign
            | Token::PlusSign
            | Token::PercentSign
            | Token::Currency
            | Token::Compact(_)
            | Token::Unit(_) => unreachable!("validated range message role"),
        }
        if add_spacing && position == first_argument {
            let next_is_space = match pattern.0.get(position + 1) {
                Some(Token::Literal(text)) => profiles
                    .text(*text)
                    .chars()
                    .next()
                    .is_some_and(|character| profiles.is_whitespace(character)),
                Some(Token::Number | Token::Argument1) => false,
                None
                | Some(
                    Token::MinusSign
                    | Token::PlusSign
                    | Token::PercentSign
                    | Token::Currency
                    | Token::Compact(_)
                    | Token::Unit(_),
                ) => unreachable!("validated range message role"),
            };
            if !next_is_space {
                result.number(RangePartSource::Shared, NumberPartKind::Literal, " ")?;
            }
        }
    }
    Ok(result.finish())
}
