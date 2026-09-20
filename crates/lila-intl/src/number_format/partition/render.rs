use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplaySign {
    None,
    Minus,
    Plus,
}

fn display_sign(
    input: Input<'_>,
    rounded: Option<&RoundedDecimal>,
    display: SignDisplay,
) -> DisplaySign {
    let (negative, zero, nan) = match input {
        Input::Finite(value) => (
            value.sign() == NumberSign::Negative,
            rounded.is_some_and(RoundedDecimal::is_zero),
            false,
        ),
        Input::Infinity(sign) => (sign == NumberSign::Negative, false, false),
        Input::NaN => (false, false, true),
    };
    match display {
        SignDisplay::Never => DisplaySign::None,
        SignDisplay::Auto if negative => DisplaySign::Minus,
        SignDisplay::Auto => DisplaySign::None,
        SignDisplay::Always if negative => DisplaySign::Minus,
        SignDisplay::Always => DisplaySign::Plus,
        SignDisplay::ExceptZero if zero || nan => DisplaySign::None,
        SignDisplay::ExceptZero if negative => DisplaySign::Minus,
        SignDisplay::ExceptZero => DisplaySign::Plus,
        SignDisplay::Negative if negative && !zero => DisplaySign::Minus,
        SignDisplay::Negative => DisplaySign::None,
    }
}

impl FormatContext<'_> {
    fn ordinary_pattern(&self) -> SignedPatternId {
        match &self.options.style {
            NumberStyle::Decimal
            | NumberStyle::Unit { .. }
            | NumberStyle::Currency {
                display: CurrencyDisplay::Name,
                ..
            } => self.numbering.decimal,
            NumberStyle::Percent => self.numbering.percent,
            NumberStyle::Currency { sign, .. } => {
                if let Some(pattern) = self.currency_override().and_then(|row| row.pattern) {
                    return pattern;
                }
                match sign {
                    CurrencySign::Standard => self.numbering.currency,
                    CurrencySign::Accounting => self.numbering.accounting,
                }
            }
        }
    }

    fn currency_label(
        &self,
        rounded: Option<&RoundedDecimal>,
        category: CardinalCategory,
    ) -> Result<Box<str>, NumberFormatKernelError> {
        let NumberStyle::Currency { code, display, .. } = &self.options.style else {
            return owned_text("", self.limits);
        };
        let ascii = code.clone().ascii();
        let mut literal = [0u8; 3];
        literal.copy_from_slice(&ascii);
        let fallback = core::str::from_utf8(&literal)
            .map_err(|_| NumberFormatKernelError::InvalidResolvedLocale)?;
        let labels = self.profiles.currencies(self.locale.currencies).find(ascii);
        let text = match (display, labels) {
            (CurrencyDisplay::Code, _) | (_, None) => fallback,
            (CurrencyDisplay::Symbol, Some(row)) => self.profiles.text(row.symbol),
            (CurrencyDisplay::NarrowSymbol, Some(row)) => self.profiles.text(row.narrow),
            (CurrencyDisplay::Name, Some(row)) => {
                let operands =
                    rounded.map(|value| PluralSelectionPurpose::Measurement.operands(value));
                self.profiles.text(
                    self.profiles
                        .string_choices(row.names)
                        .select(category, operands),
                )
            }
        };
        owned_text(text, self.limits)
    }

    fn compact_pattern(&self, rounded: &RoundedDecimal) -> Option<CompactPattern> {
        let magnitude = rounded.notation().compact_pattern_magnitude()?;
        let row = self.compact_set().row(magnitude)?;
        let operands = PluralSelectionPurpose::CompactPattern.operands(rounded);
        let category = self
            .profiles
            .rules(self.locale.plural_rules)
            .select(operands);
        Some(
            self.profiles
                .compact_choices(row.choices)
                .select(category, Some(operands)),
        )
    }

    fn alpha_pattern(
        &self,
        selected: SignedPatternId,
        compact: Option<&RoundedDecimal>,
        label: &str,
    ) -> SignedPatternId {
        let positive = &self
            .profiles
            .pattern(self.profiles.signed(selected).positive)
            .0;
        let currency = positive
            .iter()
            .position(|token| matches!(token, Token::Currency));
        let number = positive
            .iter()
            .position(|token| matches!(token, Token::Number));
        let Some(currency) = currency else {
            return selected;
        };
        let edge = if number.is_some_and(|number| currency < number) {
            label.chars().rev().find(|character| !bidi(*character))
        } else {
            label.chars().find(|character| !bidi(*character))
        };
        if !edge.is_some_and(|character| self.profiles.is_letter(character)) {
            return selected;
        }
        if let Some(rounded) = compact {
            if let Some(row) =
                rounded
                    .notation()
                    .compact_pattern_magnitude()
                    .and_then(|magnitude| {
                        self.profiles
                            .compact(self.numbering.compact_currency_alpha)
                            .row(magnitude)
                    })
            {
                let operands = PluralSelectionPurpose::CompactPattern.operands(rounded);
                let category = self
                    .profiles
                    .rules(self.locale.plural_rules)
                    .select(operands);
                return match self
                    .profiles
                    .compact_choices(row.choices)
                    .select(category, Some(operands))
                {
                    CompactPattern::Pattern(pattern) => pattern,
                    CompactPattern::Normal => selected,
                };
            }
            return selected;
        }
        if self
            .currency_override()
            .and_then(|row| row.pattern)
            .is_some()
        {
            return selected;
        }
        if selected == self.numbering.accounting {
            self.numbering.accounting_alpha
        } else if selected == self.numbering.currency {
            self.numbering.currency_alpha
        } else {
            selected
        }
    }

    fn numeric_parts(
        &self,
        input: Input<'_>,
        rounded: Option<&RoundedDecimal>,
    ) -> Result<Pieces, NumberFormatKernelError> {
        let mut result = Pieces::new();
        let Some(rounded) = rounded else {
            let (kind, text) = match input {
                Input::Infinity(_) => (NumberPartKind::Infinity, self.symbols.infinity),
                Input::NaN => (NumberPartKind::NaN, self.symbols.nan),
                Input::Finite(_) => return Err(NumberFormatKernelError::InvalidResolvedLocale),
            };
            result.symbol(kind, self.profiles.text(text), Owner::Number, self.limits)?;
            return Ok(result);
        };
        let currency = matches!(self.options.style, NumberStyle::Currency { .. });
        let decimal = self
            .currency_override()
            .and_then(|row| row.decimal)
            .unwrap_or(if currency {
                self.symbols.currency_decimal
            } else {
                self.symbols.decimal
            });
        let group = self
            .currency_override()
            .and_then(|row| row.group)
            .unwrap_or(if currency {
                self.symbols.currency_group
            } else {
                self.symbols.group
            });
        let widths = self.profiles.signed(self.ordinary_pattern()).grouping;
        let grouping_minimum = match self.options.grouping {
            Grouping::Never => None,
            Grouping::Auto => Some(self.numbering.minimum_grouping),
            Grouping::Always => Some(1),
            Grouping::MinTwo => Some(2),
        };
        let digits = rounded.integer_digits();
        let grouped = !matches!(
            rounded.notation(),
            SelectedNotation::Scientific { .. } | SelectedNotation::Engineering { .. }
        ) && widths.primary != 0
            && grouping_minimum.is_some_and(|minimum| {
                digits.len() >= usize::from(widths.primary) + usize::from(minimum)
            });
        let mut position = 0;
        while position < digits.len() {
            let remaining = digits.len() - position;
            let length = if !grouped || remaining <= usize::from(widths.primary) {
                remaining
            } else {
                let left = remaining - usize::from(widths.primary);
                let width = usize::from(widths.secondary);
                let first = left % width;
                if first == 0 {
                    width
                } else {
                    first
                }
            };
            result.push(
                NumberPartKind::Integer,
                &digit_text(
                    &digits[position..position + length],
                    self.system,
                    self.limits,
                )?,
                Owner::Number,
                self.limits,
            )?;
            position += length;
            if position < digits.len() {
                result.symbol(
                    NumberPartKind::Group,
                    self.profiles.text(group),
                    Owner::Number,
                    self.limits,
                )?;
            }
        }
        if !rounded.fraction_digits().is_empty() {
            result.symbol(
                NumberPartKind::Decimal,
                self.profiles.text(decimal),
                Owner::Number,
                self.limits,
            )?;
            result.push(
                NumberPartKind::Fraction,
                &digit_text(rounded.fraction_digits(), self.system, self.limits)?,
                Owner::Number,
                self.limits,
            )?;
        }
        match rounded.notation() {
            SelectedNotation::Scientific { exponent }
            | SelectedNotation::Engineering { exponent } => {
                result.symbol(
                    NumberPartKind::ExponentSeparator,
                    self.profiles.text(self.symbols.exponent),
                    Owner::Notation,
                    self.limits,
                )?;
                if exponent < 0 {
                    result.symbol(
                        NumberPartKind::ExponentMinusSign,
                        self.profiles.text(self.symbols.minus),
                        Owner::Notation,
                        self.limits,
                    )?;
                }
                let mut magnitude = exponent.unsigned_abs();
                let mut digits = [0u8; 20];
                let mut start = digits.len();
                loop {
                    start -= 1;
                    digits[start] = (magnitude % 10) as u8;
                    magnitude /= 10;
                    if magnitude == 0 {
                        break;
                    }
                }
                result.push(
                    NumberPartKind::ExponentInteger,
                    &digit_text(&digits[start..], self.system, self.limits)?,
                    Owner::Notation,
                    self.limits,
                )?;
            }
            SelectedNotation::Standard | SelectedNotation::Compact { .. } => {}
        }
        Ok(result)
    }

    fn message(
        &self,
        pattern: PatternId,
        number: &Pieces,
        argument: &Pieces,
    ) -> Result<Pieces, NumberFormatKernelError> {
        let mut result = Pieces::new();
        for token in &self.profiles.pattern(pattern).0 {
            match token {
                Token::Number => result.append(number, self.limits)?,
                Token::Argument1 => result.append(argument, self.limits)?,
                Token::Literal(text) => result.push(
                    NumberPartKind::Literal,
                    self.profiles.text(*text),
                    Owner::Measurement,
                    self.limits,
                )?,
                Token::Unit(text) => result.symbol(
                    NumberPartKind::Unit,
                    self.profiles.text(*text),
                    Owner::Measurement,
                    self.limits,
                )?,
                Token::MinusSign
                | Token::PlusSign
                | Token::PercentSign
                | Token::Currency
                | Token::Compact(_) => unreachable!("validated message pattern role"),
            }
        }
        Ok(result)
    }

    fn unit_message(
        &self,
        identifier: UnitIdentifier,
        display: UnitDisplay,
        rounded: Option<&RoundedDecimal>,
        category: CardinalCategory,
        number: &Pieces,
    ) -> Result<Pieces, NumberFormatKernelError> {
        let width = match display {
            UnitDisplay::Short => 0,
            UnitDisplay::Narrow => 1,
            UnitDisplay::Long => 2,
        };
        let set = self.profiles.units(self.locale.units[width]);
        let index = |unit: SingleUnit| {
            SingleUnit::ALL
                .iter()
                .position(|candidate| *candidate == unit)
                .expect("closed sanctioned unit inventory")
        };
        let operands = rounded.map(|value| PluralSelectionPurpose::Measurement.operands(value));
        let select = |id| self.profiles.pattern_choices(id).select(category, operands);
        match identifier {
            UnitIdentifier::Single(unit) => self.message(
                select(set.simple[index(unit)].choices),
                number,
                &Pieces::new(),
            ),
            UnitIdentifier::Per {
                numerator,
                denominator,
            } => {
                let numerator = index(numerator);
                let denominator = index(denominator);
                if let Some(pair) = set.pair(numerator, denominator) {
                    return self.message(select(pair), number, &Pieces::new());
                }
                let numerator = self.message(
                    select(set.simple[numerator].choices),
                    number,
                    &Pieces::new(),
                )?;
                if let Some(per) = set.simple[denominator].per_unit {
                    return self.message(per, &numerator, &Pieces::new());
                }
                let denominator = self.message(
                    set.simple[denominator].denominator,
                    &Pieces::new(),
                    &Pieces::new(),
                )?;
                self.message(set.per_pattern, &numerator, &denominator)
            }
        }
    }

    fn number_pattern(
        &self,
        selected: SignedPatternId,
        sign: DisplaySign,
        number: &Pieces,
        currency: &str,
    ) -> Result<(Pieces, usize), NumberFormatKernelError> {
        let signed = self.profiles.signed(selected);
        let negative = self.profiles.pattern(signed.negative);
        let replace_minus = sign == DisplaySign::Plus
            && negative
                .0
                .iter()
                .any(|token| matches!(token, Token::MinusSign));
        let pattern = self
            .profiles
            .pattern(if sign == DisplaySign::Minus || replace_minus {
                signed.negative
            } else {
                signed.positive
            });
        let mut result = Pieces::new();
        let mut approximate_at = None;
        if sign == DisplaySign::Plus && !replace_minus {
            approximate_at = Some(0);
            result.symbol(
                NumberPartKind::PlusSign,
                self.profiles.text(self.symbols.plus),
                Owner::Sign,
                self.limits,
            )?;
        }
        let mut number_start = 0;
        let mut number_end = 0;
        let mut currency_start = 0;
        for (index, token) in pattern.0.iter().enumerate() {
            match token {
                Token::Number => {
                    number_start = result.rows.len();
                    result.append(number, self.limits)?;
                    number_end = result.rows.len();
                }
                Token::MinusSign | Token::PlusSign => {
                    approximate_at.get_or_insert(result.rows.len());
                    let plus = matches!(token, Token::PlusSign) || replace_minus;
                    result.symbol(
                        if plus {
                            NumberPartKind::PlusSign
                        } else {
                            NumberPartKind::MinusSign
                        },
                        self.profiles.text(if plus {
                            self.symbols.plus
                        } else {
                            self.symbols.minus
                        }),
                        Owner::Sign,
                        self.limits,
                    )?;
                }
                Token::Currency => {
                    currency_start = result.rows.len();
                    if index > 0 && matches!(pattern.0[index - 1], Token::Number) {
                        self.currency_spacing(&mut result, currency, number, true)?;
                    }
                    result.symbol(
                        NumberPartKind::Currency,
                        currency,
                        Owner::Measurement,
                        self.limits,
                    )?;
                    if matches!(pattern.0.get(index + 1), Some(Token::Number)) {
                        self.currency_spacing(&mut result, currency, number, false)?;
                    }
                }
                Token::PercentSign => result.symbol(
                    NumberPartKind::PercentSign,
                    self.profiles.text(self.symbols.percent),
                    Owner::Measurement,
                    self.limits,
                )?,
                Token::Compact(text) => result.symbol(
                    NumberPartKind::Compact,
                    self.profiles.text(*text),
                    Owner::Notation,
                    self.limits,
                )?,
                Token::Literal(text) => {
                    let text = self.profiles.text(*text);
                    let adjacent_currency = index
                        .checked_sub(1)
                        .and_then(|i| pattern.0.get(i))
                        .is_some_and(|token| matches!(token, Token::Currency | Token::PercentSign))
                        || pattern.0.get(index + 1).is_some_and(|token| {
                            matches!(token, Token::Currency | Token::PercentSign)
                        });
                    let adjacent_notation = index
                        .checked_sub(1)
                        .and_then(|i| pattern.0.get(i))
                        .is_some_and(|token| matches!(token, Token::Compact(_)))
                        || pattern
                            .0
                            .get(index + 1)
                            .is_some_and(|token| matches!(token, Token::Compact(_)));
                    let space = text
                        .chars()
                        .all(|character| self.profiles.is_whitespace(character) || bidi(character));
                    let owner = if adjacent_currency && space {
                        Owner::Measurement
                    } else if adjacent_notation && space {
                        Owner::Notation
                    } else if sign != DisplaySign::None {
                        Owner::Sign
                    } else {
                        Owner::Structure
                    };
                    if owner == Owner::Sign {
                        approximate_at.get_or_insert(result.rows.len());
                    }
                    result.push(NumberPartKind::Literal, text, owner, self.limits)?;
                }
                Token::Unit(_) | Token::Argument1 => unreachable!("validated number pattern role"),
            }
        }
        let approximate_at = approximate_at.unwrap_or_else(|| {
            let minus = negative
                .0
                .iter()
                .position(|token| matches!(token, Token::MinusSign));
            let negative_number = negative
                .0
                .iter()
                .position(|token| matches!(token, Token::Number));
            let negative_currency = negative
                .0
                .iter()
                .position(|token| matches!(token, Token::Currency));
            match minus {
                Some(minus) if negative_number.is_some_and(|number| minus > number) => number_end,
                Some(minus) if negative_currency.is_some_and(|currency| minus > currency) => {
                    number_start
                }
                Some(_) if negative_currency.is_some() => currency_start.min(number_start),
                Some(_) => number_start,
                None => 0,
            }
        });
        Ok((result, approximate_at))
    }

    fn currency_spacing(
        &self,
        result: &mut Pieces,
        currency: &str,
        number: &Pieces,
        before: bool,
    ) -> Result<(), NumberFormatKernelError> {
        let (rule, symbol_edge, number_edge) = if before {
            (
                &self.numbering.before_currency,
                currency.chars().next(),
                number
                    .rows
                    .last()
                    .and_then(|piece| piece.part.text().chars().next_back()),
            )
        } else {
            (
                &self.numbering.after_currency,
                currency.chars().next_back(),
                number
                    .rows
                    .first()
                    .and_then(|piece| piece.part.text().chars().next()),
            )
        };
        if let (Some(symbol), Some(number)) = (symbol_edge, number_edge) {
            if self.profiles.in_set(rule.currency, symbol)
                && self.profiles.in_set(rule.surrounding, number)
            {
                result.push(
                    NumberPartKind::Literal,
                    self.profiles.text(rule.insert),
                    Owner::Measurement,
                    self.limits,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn format(
        &self,
        input: Input<'_>,
        category_override: Option<CardinalCategory>,
    ) -> Result<Formatted, NumberFormatKernelError> {
        let mut settings =
            DecimalFormatSettings::from_options(self.options, &self.compact_set().exponents);
        let mut rounded = match input {
            Input::Finite(value) => Some(format_decimal(value, &settings, &self.limits.numeric())?),
            Input::Infinity(_) | Input::NaN => None,
        };
        let mut compact = rounded
            .as_ref()
            .and_then(|value| self.compact_pattern(value));
        if compact == Some(CompactPattern::Normal) {
            settings.notation = NotationScaling::Standard;
            if let Input::Finite(value) = input {
                rounded = Some(format_decimal(value, &settings, &self.limits.numeric())?);
            }
            compact = None;
        }
        let category = rounded.as_ref().map_or(CardinalCategory::Other, |value| {
            self.profiles
                .rules(self.locale.plural_rules)
                .select(PluralSelectionPurpose::Measurement.operands(value))
        });
        let selected_category = category_override.unwrap_or(category);
        let sign = display_sign(input, rounded.as_ref(), self.options.sign_display);
        let name_quantity = if category_override.is_some() {
            None
        } else {
            rounded.as_ref()
        };
        let currency = self.currency_label(name_quantity, selected_category)?;
        let mut number = self.numeric_parts(input, rounded.as_ref())?;
        let mut pattern = self.ordinary_pattern();
        let direct_compact = matches!(
            self.options.style,
            NumberStyle::Decimal
                | NumberStyle::Currency {
                    display: CurrencyDisplay::Code
                        | CurrencyDisplay::Symbol
                        | CurrencyDisplay::NarrowSymbol,
                    ..
                }
        );
        if let Some(CompactPattern::Pattern(selected)) = compact {
            if direct_compact {
                pattern = selected;
            } else {
                number = self
                    .number_pattern(selected, DisplaySign::None, &number, "")?
                    .0;
            }
        }
        match &self.options.style {
            NumberStyle::Unit {
                identifier,
                display,
            } => {
                number = self.unit_message(
                    *identifier,
                    *display,
                    name_quantity,
                    selected_category,
                    &number,
                )?
            }
            NumberStyle::Currency {
                display: CurrencyDisplay::Name,
                ..
            } => {
                let operands =
                    name_quantity.map(|value| PluralSelectionPurpose::Measurement.operands(value));
                let message = self
                    .profiles
                    .pattern_choices(self.numbering.currency_unit_pattern)
                    .select(selected_category, operands);
                let mut label = Pieces::new();
                label.symbol(
                    NumberPartKind::Currency,
                    &currency,
                    Owner::Measurement,
                    self.limits,
                )?;
                number = self.message(message, &number, &label)?;
            }
            NumberStyle::Decimal | NumberStyle::Percent | NumberStyle::Currency { .. } => {}
        }
        if matches!(
            self.options.style,
            NumberStyle::Currency {
                display: CurrencyDisplay::Code
                    | CurrencyDisplay::Symbol
                    | CurrencyDisplay::NarrowSymbol,
                ..
            }
        ) {
            pattern = self.alpha_pattern(
                pattern,
                if compact.is_some() {
                    rounded.as_ref()
                } else {
                    None
                },
                &currency,
            );
        }
        let (pieces, approximate_at) = self.number_pattern(pattern, sign, &number, &currency)?;
        Ok(Formatted {
            pieces,
            category,
            approximate_at,
        })
    }
}
