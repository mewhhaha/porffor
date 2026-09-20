use super::*;
use NumberConfigurationWord as W;

impl NumberFormatOptions {
    pub fn wire_words(&self) -> [u64; NUMBER_CONFIGURATION_WORDS] {
        let mut words = [0; NUMBER_CONFIGURATION_WORDS];
        match &self.style {
            NumberStyle::Decimal => words[W::Style.index()] = StyleOption::Decimal.wire_code(),
            NumberStyle::Percent => words[W::Style.index()] = StyleOption::Percent.wire_code(),
            NumberStyle::Currency { display, sign, .. } => {
                words[W::Style.index()] = StyleOption::Currency.wire_code();
                words[W::CurrencyDisplay.index()] = display.wire_code();
                words[W::CurrencySign.index()] = sign.wire_code();
            }
            NumberStyle::Unit { display, .. } => {
                words[W::Style.index()] = StyleOption::Unit.wire_code();
                words[W::UnitDisplay.index()] = display.wire_code();
            }
        }
        words[W::Notation.index()] = match self.notation {
            Notation::Standard => NotationOption::Standard.wire_code(),
            Notation::Scientific => NotationOption::Scientific.wire_code(),
            Notation::Engineering => NotationOption::Engineering.wire_code(),
            Notation::Compact(display) => {
                words[W::CompactDisplay.index()] = display.wire_code();
                NotationOption::Compact.wire_code()
            }
        };
        words[W::MinimumInteger.index()] = u64::from(self.minimum_integer_digits.get());
        let (kind, fraction, significant, increment) = match self.precision {
            Precision::Fraction(FractionPrecision::Range(range)) => {
                (NumberPrecisionKind::Fraction, Some(range), None, 1)
            }
            Precision::Fraction(FractionPrecision::Increment { digits, increment }) => (
                NumberPrecisionKind::Fraction,
                Some(FractionDigitRange::new(digits, digits).expect("equal checked digit counts")),
                None,
                u64::from(increment.value()),
            ),
            Precision::Significant(range) => {
                (NumberPrecisionKind::Significant, None, Some(range), 1)
            }
            Precision::More {
                fraction,
                significant,
            } => (
                NumberPrecisionKind::More,
                Some(fraction),
                Some(significant),
                1,
            ),
            Precision::Less {
                fraction,
                significant,
            } => (
                NumberPrecisionKind::Less,
                Some(fraction),
                Some(significant),
                1,
            ),
        };
        words[W::Precision.index()] = kind.wire_code();
        words[W::RoundingIncrement.index()] = increment;
        if let Some(range) = fraction {
            words[W::MinimumFraction.index()] = u64::from(range.minimum().get());
            words[W::MaximumFraction.index()] = u64::from(range.maximum().get());
        }
        if let Some(range) = significant {
            words[W::MinimumSignificant.index()] = u64::from(range.minimum().get());
            words[W::MaximumSignificant.index()] = u64::from(range.maximum().get());
        }
        words[W::RoundingMode.index()] = self.rounding_mode.wire_code();
        words[W::TrailingZero.index()] = self.trailing_zero_display.wire_code();
        words[W::Grouping.index()] = self.grouping.wire_code();
        words[W::SignDisplay.index()] = self.sign_display.wire_code();
        words
    }
}

impl NumberWireWriter {
    pub(super) fn configuration(
        &mut self,
        configuration: &NumberFormatConfiguration,
    ) -> Result<(), NumberWireError> {
        self.resolved_locale(&configuration.locale)?;
        for word in configuration.options.wire_words() {
            self.word(word)?;
        }
        match &configuration.options.style {
            NumberStyle::Decimal | NumberStyle::Percent => self.text(""),
            NumberStyle::Currency { code, .. } => self.bytes(&code.clone().ascii()),
            NumberStyle::Unit { identifier, .. } => match identifier {
                UnitIdentifier::Single(unit) => self.text(unit.name()),
                UnitIdentifier::Per {
                    numerator,
                    denominator,
                } => self.text_fragments(&[numerator.name(), "-per-", denominator.name()]),
            },
        }
    }
}

struct EncodedNumberOptions([u64; NUMBER_CONFIGURATION_WORDS]);
impl EncodedNumberOptions {
    fn word(&self, field: W) -> u64 {
        self.0[field.index()]
    }
    fn domain<T>(&self, field: W, decode: fn(u64) -> Option<T>) -> Result<T, NumberWireError> {
        decode(self.word(field)).ok_or(NumberWireError::Malformed("invalid option code"))
    }
    fn byte(&self, field: W) -> Result<u8, NumberWireError> {
        u8::try_from(self.word(field))
            .map_err(|_| NumberWireError::Malformed("digit count exceeds byte"))
    }
    fn zeros(&self, fields: &[W]) -> Result<(), NumberWireError> {
        if fields.iter().any(|field| self.word(*field) != 0) {
            Err(NumberWireError::Malformed("nonzero inactive option"))
        } else {
            Ok(())
        }
    }
    fn fraction(&self) -> Result<FractionDigitRange, NumberWireError> {
        Ok(FractionDigitRange::new(
            FractionDigitCount::new(self.byte(W::MinimumFraction)?)?,
            FractionDigitCount::new(self.byte(W::MaximumFraction)?)?,
        )?)
    }
    fn significant(&self) -> Result<SignificantDigitRange, NumberWireError> {
        Ok(SignificantDigitRange::new(
            SignificantDigitCount::new(self.byte(W::MinimumSignificant)?)?,
            SignificantDigitCount::new(self.byte(W::MaximumSignificant)?)?,
        )?)
    }
    fn decode(self, active_style: &str) -> Result<NumberFormatOptions, NumberWireError> {
        let style = match self.domain(W::Style, StyleOption::from_wire_code)? {
            StyleOption::Decimal => {
                self.zeros(&[W::CurrencyDisplay, W::CurrencySign, W::UnitDisplay])?;
                if !active_style.is_empty() {
                    return Err(NumberWireError::Malformed("inactive style text"));
                }
                NumberStyle::Decimal
            }
            StyleOption::Percent => {
                self.zeros(&[W::CurrencyDisplay, W::CurrencySign, W::UnitDisplay])?;
                if !active_style.is_empty() {
                    return Err(NumberWireError::Malformed("inactive style text"));
                }
                NumberStyle::Percent
            }
            StyleOption::Currency => {
                self.zeros(&[W::UnitDisplay])?;
                let code = CurrencyCode::parse(active_style)?;
                if code.clone().ascii().as_slice() != active_style.as_bytes() {
                    return Err(NumberWireError::Malformed("noncanonical currency code"));
                }
                NumberStyle::Currency {
                    code,
                    display: self.domain(W::CurrencyDisplay, CurrencyDisplay::from_wire_code)?,
                    sign: self.domain(W::CurrencySign, CurrencySign::from_wire_code)?,
                }
            }
            StyleOption::Unit => {
                self.zeros(&[W::CurrencyDisplay, W::CurrencySign])?;
                NumberStyle::Unit {
                    identifier: UnitIdentifier::parse(active_style)?,
                    display: self.domain(W::UnitDisplay, UnitDisplay::from_wire_code)?,
                }
            }
        };
        let notation = match self.domain(W::Notation, NotationOption::from_wire_code)? {
            NotationOption::Standard => {
                self.zeros(&[W::CompactDisplay])?;
                Notation::Standard
            }
            NotationOption::Scientific => {
                self.zeros(&[W::CompactDisplay])?;
                Notation::Scientific
            }
            NotationOption::Engineering => {
                self.zeros(&[W::CompactDisplay])?;
                Notation::Engineering
            }
            NotationOption::Compact => {
                Notation::Compact(self.domain(W::CompactDisplay, CompactDisplay::from_wire_code)?)
            }
        };
        let kind = self.domain(W::Precision, NumberPrecisionKind::from_wire_code)?;
        let increment = self.word(W::RoundingIncrement);
        if increment != 1 && kind != NumberPrecisionKind::Fraction {
            return Err(InvalidNumberConfiguration::NonUnitIncrementRequiresFixedFraction.into());
        }
        let precision = match kind {
            NumberPrecisionKind::Fraction => {
                self.zeros(&[W::MinimumSignificant, W::MaximumSignificant])?;
                let range = self.fraction()?;
                if increment == 1 {
                    Precision::Fraction(FractionPrecision::Range(range))
                } else {
                    let increment = NonUnitRoundingIncrement::from_wire_value(increment)
                        .ok_or(NumberWireError::Malformed("invalid rounding increment"))?;
                    if range.minimum() != range.maximum() {
                        return Err(
                            InvalidNumberConfiguration::NonUnitIncrementRequiresFixedFraction
                                .into(),
                        );
                    }
                    Precision::Fraction(FractionPrecision::Increment {
                        digits: range.minimum(),
                        increment,
                    })
                }
            }
            NumberPrecisionKind::Significant => {
                self.zeros(&[W::MinimumFraction, W::MaximumFraction])?;
                Precision::Significant(self.significant()?)
            }
            NumberPrecisionKind::More => Precision::More {
                fraction: self.fraction()?,
                significant: self.significant()?,
            },
            NumberPrecisionKind::Less => Precision::Less {
                fraction: self.fraction()?,
                significant: self.significant()?,
            },
        };
        Ok(NumberFormatOptions {
            style,
            notation,
            precision,
            minimum_integer_digits: IntegerDigitCount::new(self.byte(W::MinimumInteger)?)?,
            rounding_mode: self.domain(W::RoundingMode, RoundingMode::from_wire_code)?,
            trailing_zero_display: self
                .domain(W::TrailingZero, TrailingZeroDisplay::from_wire_code)?,
            grouping: self.domain(W::Grouping, Grouping::from_wire_code)?,
            sign_display: self.domain(W::SignDisplay, SignDisplay::from_wire_code)?,
        })
    }
}
impl NumberWireReader<'_> {
    pub(super) fn configuration(
        &mut self,
        profiles: &NumberProfiles,
    ) -> Result<NumberFormatConfiguration, NumberWireError> {
        let locale = self.resolved_locale(profiles)?;
        let mut words = [0; NUMBER_CONFIGURATION_WORDS];
        for field in W::ALL {
            words[field.index()] = self.word()?;
        }
        let options = EncodedNumberOptions(words).decode(self.text()?)?;
        Ok(NumberFormatConfiguration { locale, options })
    }
}
