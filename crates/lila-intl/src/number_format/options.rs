//! Exact NumberFormat configuration and mathematical value domains.

macro_rules! closed_option {
    ($name:ident { $($variant:ident => $spelling:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub const fn name(self) -> &'static str {
                match self { $(Self::$variant => $spelling),+ }
            }
            pub fn parse(spelling: &str) -> Option<Self> {
                match spelling { $($spelling => Some(Self::$variant)),+, _ => None }
            }
        }
    };
}

closed_option!(LocaleMatcher { Lookup => "lookup", BestFit => "best fit" });
closed_option!(StyleOption { Decimal => "decimal", Percent => "percent", Currency => "currency", Unit => "unit" });
closed_option!(CurrencyDisplay { Code => "code", Symbol => "symbol", NarrowSymbol => "narrowSymbol", Name => "name" });
closed_option!(CurrencySign { Standard => "standard", Accounting => "accounting" });
closed_option!(UnitDisplay { Short => "short", Narrow => "narrow", Long => "long" });
closed_option!(NotationOption { Standard => "standard", Scientific => "scientific", Engineering => "engineering", Compact => "compact" });
closed_option!(CompactDisplay { Short => "short", Long => "long" });
closed_option!(RoundingMode {
    Ceil => "ceil", Floor => "floor", Expand => "expand", Trunc => "trunc",
    HalfCeil => "halfCeil", HalfFloor => "halfFloor", HalfExpand => "halfExpand",
    HalfTrunc => "halfTrunc", HalfEven => "halfEven"
});
closed_option!(RoundingPriority { Auto => "auto", MorePrecision => "morePrecision", LessPrecision => "lessPrecision" });
closed_option!(TrailingZeroDisplay { Auto => "auto", StripIfInteger => "stripIfInteger" });
closed_option!(SignDisplay { Auto => "auto", Never => "never", Always => "always", ExceptZero => "exceptZero", Negative => "negative" });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Grouping {
    Never,
    Auto,
    Always,
    MinTwo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Notation {
    Standard,
    Scientific,
    Engineering,
    Compact(CompactDisplay),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidNumberConfiguration {
    IntegerDigits,
    FractionDigits,
    SignificantDigits,
    ReversedDigitRange,
    CurrencyCode,
    UnsupportedUnit,
    NonUnitIncrementRequiresFixedFraction,
}

macro_rules! digit_count {
    ($name:ident, $min:literal, $max:literal, $error:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(u8);
        impl $name {
            pub fn new(value: u8) -> Result<Self, InvalidNumberConfiguration> {
                if ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidNumberConfiguration::$error)
                }
            }
            pub const fn get(self) -> u8 {
                self.0
            }
        }
    };
}
digit_count!(IntegerDigitCount, 1, 21, IntegerDigits);
digit_count!(FractionDigitCount, 0, 100, FractionDigits);
digit_count!(SignificantDigitCount, 1, 21, SignificantDigits);

macro_rules! digit_range {
    ($name:ident, $count:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name {
            minimum: $count,
            maximum: $count,
        }
        impl $name {
            pub fn new(
                minimum: $count,
                maximum: $count,
            ) -> Result<Self, InvalidNumberConfiguration> {
                if minimum.get() <= maximum.get() {
                    Ok(Self { minimum, maximum })
                } else {
                    Err(InvalidNumberConfiguration::ReversedDigitRange)
                }
            }
            pub const fn minimum(self) -> $count {
                self.minimum
            }
            pub const fn maximum(self) -> $count {
                self.maximum
            }
        }
    };
}
digit_range!(FractionDigitRange, FractionDigitCount);
digit_range!(SignificantDigitRange, SignificantDigitCount);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NonUnitRoundingIncrement {
    Two,
    Five,
    Ten,
    Twenty,
    TwentyFive,
    Fifty,
    Hundred,
    TwoHundred,
    TwoHundredFifty,
    FiveHundred,
    Thousand,
    TwoThousand,
    TwoThousandFiveHundred,
    FiveThousand,
}
impl NonUnitRoundingIncrement {
    pub const fn value(self) -> u16 {
        match self {
            Self::Two => 2,
            Self::Five => 5,
            Self::Ten => 10,
            Self::Twenty => 20,
            Self::TwentyFive => 25,
            Self::Fifty => 50,
            Self::Hundred => 100,
            Self::TwoHundred => 200,
            Self::TwoHundredFifty => 250,
            Self::FiveHundred => 500,
            Self::Thousand => 1000,
            Self::TwoThousand => 2000,
            Self::TwoThousandFiveHundred => 2500,
            Self::FiveThousand => 5000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingIncrement {
    One,
    Multiple(NonUnitRoundingIncrement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FractionPrecision {
    Range(FractionDigitRange),
    Increment {
        digits: FractionDigitCount,
        increment: NonUnitRoundingIncrement,
    },
}

/// Non-unit increments cannot coexist with significant/combined precision or
/// different minimum and maximum fractional digit counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Precision {
    Fraction(FractionPrecision),
    Significant(SignificantDigitRange),
    More {
        fraction: FractionDigitRange,
        significant: SignificantDigitRange,
    },
    Less {
        fraction: FractionDigitRange,
        significant: SignificantDigitRange,
    },
}
impl Precision {
    pub const fn computed_rounding_priority(self) -> RoundingPriority {
        match self {
            Self::Fraction(_) | Self::Significant(_) => RoundingPriority::Auto,
            Self::More { .. } => RoundingPriority::MorePrecision,
            Self::Less { .. } => RoundingPriority::LessPrecision,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyCode([u8; 3]);
impl CurrencyCode {
    pub fn parse(value: &str) -> Result<Self, InvalidNumberConfiguration> {
        let bytes = value.as_bytes();
        if bytes.len() != 3 || !bytes.iter().all(u8::is_ascii_alphabetic) {
            return Err(InvalidNumberConfiguration::CurrencyCode);
        }
        Ok(Self([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
            bytes[2].to_ascii_uppercase(),
        ]))
    }
    pub const fn ascii(self) -> [u8; 3] {
        self.0
    }
}

closed_option!(SingleUnit {
    Acre => "acre",
    Bit => "bit",
    Byte => "byte",
    Celsius => "celsius",
    Centimeter => "centimeter",
    Day => "day",
    Degree => "degree",
    Fahrenheit => "fahrenheit",
    FluidOunce => "fluid-ounce",
    Foot => "foot",
    Gallon => "gallon",
    Gigabit => "gigabit",
    Gigabyte => "gigabyte",
    Gram => "gram",
    Hectare => "hectare",
    Hour => "hour",
    Inch => "inch",
    Kilobit => "kilobit",
    Kilobyte => "kilobyte",
    Kilogram => "kilogram",
    Kilometer => "kilometer",
    Liter => "liter",
    Megabit => "megabit",
    Megabyte => "megabyte",
    Meter => "meter",
    Microsecond => "microsecond",
    Mile => "mile",
    MileScandinavian => "mile-scandinavian",
    Milliliter => "milliliter",
    Millimeter => "millimeter",
    Millisecond => "millisecond",
    Minute => "minute",
    Month => "month",
    Nanosecond => "nanosecond",
    Ounce => "ounce",
    Percent => "percent",
    Petabyte => "petabyte",
    Pound => "pound",
    Second => "second",
    Stone => "stone",
    Terabit => "terabit",
    Terabyte => "terabyte",
    Week => "week",
    Yard => "yard",
    Year => "year",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitIdentifier {
    Single(SingleUnit),
    Per {
        numerator: SingleUnit,
        denominator: SingleUnit,
    },
}
impl UnitIdentifier {
    pub fn parse(value: &str) -> Result<Self, InvalidNumberConfiguration> {
        if let Some(unit) = SingleUnit::parse(value) {
            return Ok(Self::Single(unit));
        }
        if let Some((numerator, denominator)) = value.split_once("-per-") {
            if let (Some(numerator), Some(denominator)) =
                (SingleUnit::parse(numerator), SingleUnit::parse(denominator))
            {
                return Ok(Self::Per {
                    numerator,
                    denominator,
                });
            }
        }
        Err(InvalidNumberConfiguration::UnsupportedUnit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumberStyle {
    Decimal,
    Percent,
    Currency {
        code: CurrencyCode,
        display: CurrencyDisplay,
        sign: CurrencySign,
    },
    Unit {
        identifier: UnitIdentifier,
        display: UnitDisplay,
    },
}

/// Complete after JS option observations and cross-option validation. Inactive
/// style fields are absent; compact display only exists in compact notation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumberFormatOptions {
    pub style: NumberStyle,
    pub notation: Notation,
    pub minimum_integer_digits: IntegerDigitCount,
    pub precision: Precision,
    pub rounding_mode: RoundingMode,
    pub trailing_zero_display: TrailingZeroDisplay,
    pub grouping: Grouping,
    pub sign_display: SignDisplay,
}
