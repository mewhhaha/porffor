//! Scalar and range parts are the shared formatting and serialization authority.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberPartKind {
    Literal,
    Integer,
    Group,
    Decimal,
    Fraction,
    PlusSign,
    MinusSign,
    PercentSign,
    Currency,
    Unit,
    Compact,
    ExponentSeparator,
    ExponentMinusSign,
    ExponentInteger,
    Infinity,
    NaN,
}
impl NumberPartKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Literal => "literal",
            Self::Integer => "integer",
            Self::Group => "group",
            Self::Decimal => "decimal",
            Self::Fraction => "fraction",
            Self::PlusSign => "plusSign",
            Self::MinusSign => "minusSign",
            Self::PercentSign => "percentSign",
            Self::Currency => "currency",
            Self::Unit => "unit",
            Self::Compact => "compact",
            Self::ExponentSeparator => "exponentSeparator",
            Self::ExponentMinusSign => "exponentMinusSign",
            Self::ExponentInteger => "exponentInteger",
            Self::Infinity => "infinity",
            Self::NaN => "nan",
        }
    }
}

/// Text is already localized, including numbering-system digits and bidi
/// literals. AOT must copy it unchanged; it never substitutes digits again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberPart {
    kind: NumberPartKind,
    text: Box<str>,
}
impl NumberPart {
    pub fn new(kind: NumberPartKind, text: Box<str>) -> Self {
        Self { kind, text }
    }
    pub const fn kind(&self) -> NumberPartKind {
        self.kind
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangePartSource {
    Start,
    End,
    Shared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberRangePart {
    Number {
        source: RangePartSource,
        part: NumberPart,
    },
    /// This always has source "shared" and cannot appear in scalar parts.
    ApproximatelySign(Box<str>),
}
impl NumberRangePart {
    pub const fn source(&self) -> RangePartSource {
        match self {
            Self::Number { source, .. } => *source,
            Self::ApproximatelySign(_) => RangePartSource::Shared,
        }
    }
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Number { part, .. } => part.kind().name(),
            Self::ApproximatelySign(_) => "approximatelySign",
        }
    }
    pub fn text(&self) -> &str {
        match self {
            Self::Number { part, .. } => part.text(),
            Self::ApproximatelySign(text) => text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarNumberPartition(Box<[NumberPart]>);
impl ScalarNumberPartition {
    pub fn from_parts(parts: Box<[NumberPart]>) -> Self {
        Self(parts)
    }
    pub fn parts(&self) -> &[NumberPart] {
        &self.0
    }
    pub fn to_text(&self) -> String {
        self.0.iter().map(NumberPart::text).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeNumberPartition(Box<[NumberRangePart]>);
impl RangeNumberPartition {
    pub fn from_parts(parts: Box<[NumberRangePart]>) -> Self {
        Self(parts)
    }
    pub fn parts(&self) -> &[NumberRangePart] {
        &self.0
    }
    pub fn to_text(&self) -> String {
        self.0.iter().map(NumberRangePart::text).collect()
    }
}
