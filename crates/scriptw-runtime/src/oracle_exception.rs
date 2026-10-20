//! Closed observation vocabulary; no constructor can mint exception provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleExceptionPhase {
    Runtime,
    Resolution,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleExceptionType {
    Error,
    Eval,
    Range,
    Reference,
    Syntax,
    Type,
    Uri,
    Aggregate,
    Suppressed,
    Primitive,
    UnclassifiedObject,
}
impl OracleExceptionType {
    pub const fn constructor_name(self) -> Option<&'static str> {
        match self {
            Self::Error => Some("Error"),
            Self::Eval => Some("EvalError"),
            Self::Range => Some("RangeError"),
            Self::Reference => Some("ReferenceError"),
            Self::Syntax => Some("SyntaxError"),
            Self::Type => Some("TypeError"),
            Self::Uri => Some("URIError"),
            Self::Aggregate => Some("AggregateError"),
            Self::Suppressed => Some("SuppressedError"),
            Self::Primitive | Self::UnclassifiedObject => None,
        }
    }
}
