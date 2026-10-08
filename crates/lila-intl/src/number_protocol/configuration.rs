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
        let rounding = rounding_wire_words(&RoundingSettings::from(self));
        words[W::MinimumInteger.index()] = rounding[0];
        words[W::Precision.index()] = rounding[1];
        words[W::MinimumFraction.index()] = rounding[2];
        words[W::MaximumFraction.index()] = rounding[3];
        words[W::MinimumSignificant.index()] = rounding[4];
        words[W::MaximumSignificant.index()] = rounding[5];
        words[W::RoundingIncrement.index()] = rounding[6];
        words[W::RoundingMode.index()] = rounding[7];
        words[W::TrailingZero.index()] = rounding[8];
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
    fn zeros(&self, fields: &[W]) -> Result<(), NumberWireError> {
        if fields.iter().any(|field| self.word(*field) != 0) {
            Err(NumberWireError::Malformed("nonzero inactive option"))
        } else {
            Ok(())
        }
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
        let rounding = decode_rounding_words([
            self.word(W::MinimumInteger),
            self.word(W::Precision),
            self.word(W::MinimumFraction),
            self.word(W::MaximumFraction),
            self.word(W::MinimumSignificant),
            self.word(W::MaximumSignificant),
            self.word(W::RoundingIncrement),
            self.word(W::RoundingMode),
            self.word(W::TrailingZero),
        ])?;
        Ok(NumberFormatOptions {
            style,
            notation,
            precision: rounding.precision,
            minimum_integer_digits: rounding.minimum_integer_digits,
            rounding_mode: rounding.mode,
            trailing_zero_display: rounding.trailing_zero_display,
            grouping: self.domain(W::Grouping, Grouping::from_wire_code)?,
            sign_display: self.domain(W::SignDisplay, SignDisplay::from_wire_code)?,
        })
    }
}
impl NumberWireReader<'_> {
    pub(super) fn configuration(
        &mut self,
        profiles: &Arc<NumberProfiles>,
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
