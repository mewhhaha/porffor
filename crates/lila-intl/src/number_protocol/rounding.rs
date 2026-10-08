use super::*;
use crate::number_format::numeric::RoundingSettings;
#[derive(Clone, Copy)]
enum W {
    MinimumInteger,
    Precision,
    MinimumFraction,
    MaximumFraction,
    MinimumSignificant,
    MaximumSignificant,
    RoundingIncrement,
    RoundingMode,
    TrailingZero,
}
impl W {
    const fn index(self) -> usize {
        match self {
            Self::MinimumInteger => 0,
            Self::Precision => 1,
            Self::MinimumFraction => 2,
            Self::MaximumFraction => 3,
            Self::MinimumSignificant => 4,
            Self::MaximumSignificant => 5,
            Self::RoundingIncrement => 6,
            Self::RoundingMode => 7,
            Self::TrailingZero => 8,
        }
    }
}
const WORDS: usize = 9;
pub(crate) fn rounding_wire_words(settings: &RoundingSettings) -> [u64; WORDS] {
    let mut words = [0; WORDS];
    words[W::MinimumInteger.index()] = u64::from(settings.minimum_integer_digits.get());
    let (kind, fraction, significant, increment) = match settings.precision {
        Precision::Fraction(FractionPrecision::Range(range)) => {
            (NumberPrecisionKind::Fraction, Some(range), None, 1)
        }
        Precision::Fraction(FractionPrecision::Increment { digits, increment }) => (
            NumberPrecisionKind::Fraction,
            Some(FractionDigitRange::new(digits, digits).expect("equal checked digit counts")),
            None,
            u64::from(increment.value()),
        ),
        Precision::Significant(range) => (NumberPrecisionKind::Significant, None, Some(range), 1),
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
    words[W::RoundingMode.index()] = settings.mode.wire_code();
    words[W::TrailingZero.index()] = settings.trailing_zero_display.wire_code();
    words
}
struct EncodedRounding([u64; WORDS]);
impl EncodedRounding {
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
    fn decode(self) -> Result<RoundingSettings, NumberWireError> {
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
        Ok(RoundingSettings {
            precision,
            minimum_integer_digits: IntegerDigitCount::new(self.byte(W::MinimumInteger)?)?,
            mode: self.domain(W::RoundingMode, RoundingMode::from_wire_code)?,
            trailing_zero_display: self
                .domain(W::TrailingZero, TrailingZeroDisplay::from_wire_code)?,
        })
    }
}
pub(crate) fn decode_rounding_words(
    words: [u64; WORDS],
) -> Result<RoundingSettings, NumberWireError> {
    EncodedRounding(words).decode()
}
