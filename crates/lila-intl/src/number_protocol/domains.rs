use super::*;

// Exhaustive encoders couple accepted JavaScript spellings and wire values.
macro_rules! wire_domain {
    ($domain:ident { $($variant:ident = $code:literal),+ $(,)? }) => {
        impl $domain {
            pub const fn wire_code(self) -> u64 {
                match self { $(Self::$variant => $code),+ }
            }
            pub const fn from_wire_code(code: u64) -> Option<Self> {
                match code { $($code => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}
macro_rules! option_wire_domain {
    ($domain:ident { $($variant:ident = $code:literal),+ $(,)? }) => {
        wire_domain!($domain { $($variant = $code),+ });
        impl $domain {
            pub const OPTIONS: &'static [(&'static str, i64)] = &[
                $((Self::$variant.name(), Self::$variant.wire_code() as i64)),+
            ];
        }
    };
}
option_wire_domain!(LocaleMatcher { Lookup = 1, BestFit = 2 });
option_wire_domain!(StyleOption { Decimal = 1, Percent = 2, Currency = 3, Unit = 4 });
option_wire_domain!(CurrencyDisplay { Code = 1, Symbol = 2, NarrowSymbol = 3, Name = 4 });
option_wire_domain!(CurrencySign { Standard = 1, Accounting = 2 });
option_wire_domain!(UnitDisplay { Short = 1, Narrow = 2, Long = 3 });
option_wire_domain!(NotationOption { Standard = 1, Scientific = 2, Engineering = 3, Compact = 4 });
option_wire_domain!(CompactDisplay { Short = 1, Long = 2 });
option_wire_domain!(RoundingMode {
    Ceil = 1, Floor = 2, Expand = 3, Trunc = 4, HalfCeil = 5,
    HalfFloor = 6, HalfExpand = 7, HalfTrunc = 8, HalfEven = 9,
});
option_wire_domain!(RoundingPriority { Auto = 1, MorePrecision = 2, LessPrecision = 3 });
option_wire_domain!(TrailingZeroDisplay { Auto = 1, StripIfInteger = 2 });
option_wire_domain!(SignDisplay { Auto = 1, Never = 2, Always = 3, ExceptZero = 4, Negative = 5 });
wire_domain!(Grouping { Never = 0, Auto = 1, Always = 2, MinTwo = 3 });
wire_domain!(NumberPartKind {
    Literal = 0, Integer = 1, Group = 2, Decimal = 3, Fraction = 4,
    PlusSign = 5, MinusSign = 6, PercentSign = 7, Currency = 8, Unit = 9,
    Compact = 10, ExponentSeparator = 11, ExponentMinusSign = 12,
    ExponentInteger = 13, Infinity = 14, NaN = 15,
});
impl NumberPartKind {
    pub const ALL: &'static [Self] = &[
        Self::Literal,
        Self::Integer,
        Self::Group,
        Self::Decimal,
        Self::Fraction,
        Self::PlusSign,
        Self::MinusSign,
        Self::PercentSign,
        Self::Currency,
        Self::Unit,
        Self::Compact,
        Self::ExponentSeparator,
        Self::ExponentMinusSign,
        Self::ExponentInteger,
        Self::Infinity,
        Self::NaN,
    ];
}
wire_domain!(RangePartSource { Shared = 0, Start = 1, End = 2 });
impl RangePartSource {
    pub const ALL: &'static [Self] = &[Self::Shared, Self::Start, Self::End];
    pub const fn name(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Start => "startRange",
            Self::End => "endRange",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberNumericKind {
    String,
    Number,
    NegativeZero,
    BigInt,
}
wire_domain!(NumberNumericKind { String = 1, Number = 2, NegativeZero = 3, BigInt = 4 });
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPrecisionKind {
    Fraction,
    Significant,
    More,
    Less,
}
wire_domain!(NumberPrecisionKind { Fraction = 1, Significant = 2, More = 3, Less = 4 });

macro_rules! configuration_words {
    ($($variant:ident = $index:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum NumberConfigurationWord { $($variant),+ }
        impl NumberConfigurationWord {
            pub const ALL: [Self; NUMBER_CONFIGURATION_WORDS] = [$(Self::$variant),+];
            pub const fn index(self) -> usize { match self { $(Self::$variant => $index),+ } }
            pub const fn offset(self) -> u64 { self.index() as u64 * 8 }
        }
    };
}
configuration_words! {
    Style = 0, CurrencyDisplay = 1, CurrencySign = 2, UnitDisplay = 3,
    Notation = 4, CompactDisplay = 5, MinimumInteger = 6, Precision = 7,
    MinimumFraction = 8, MaximumFraction = 9, MinimumSignificant = 10,
    MaximumSignificant = 11, RoundingIncrement = 12, RoundingMode = 13,
    TrailingZero = 14, Grouping = 15, SignDisplay = 16,
}

impl NonUnitRoundingIncrement {
    pub const ALL: &'static [Self] = &[
        Self::Two,
        Self::Five,
        Self::Ten,
        Self::Twenty,
        Self::TwentyFive,
        Self::Fifty,
        Self::Hundred,
        Self::TwoHundred,
        Self::TwoHundredFifty,
        Self::FiveHundred,
        Self::Thousand,
        Self::TwoThousand,
        Self::TwoThousandFiveHundred,
        Self::FiveThousand,
    ];
    pub const fn from_wire_value(value: u64) -> Option<Self> {
        match value {
            2 => Some(Self::Two),
            5 => Some(Self::Five),
            10 => Some(Self::Ten),
            20 => Some(Self::Twenty),
            25 => Some(Self::TwentyFive),
            50 => Some(Self::Fifty),
            100 => Some(Self::Hundred),
            200 => Some(Self::TwoHundred),
            250 => Some(Self::TwoHundredFifty),
            500 => Some(Self::FiveHundred),
            1000 => Some(Self::Thousand),
            2000 => Some(Self::TwoThousand),
            2500 => Some(Self::TwoThousandFiveHundred),
            5000 => Some(Self::FiveThousand),
            _ => None,
        }
    }
}
