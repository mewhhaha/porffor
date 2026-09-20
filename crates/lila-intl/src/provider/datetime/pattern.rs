use crate::datetime::{
    DateTimeFormatError, DateTimeFractionalDigits, DateTimeHourCycle, DateTimePartKind,
};
use crate::TimeZoneNameStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NameWidth {
    Abbreviated,
    Wide,
    Narrow,
    Short,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NameContext {
    Format,
    Standalone,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DayPeriod {
    Am,
    Pm,
    Midnight,
    Noon,
    Morning1,
    Morning2,
    Afternoon1,
    Afternoon2,
    Evening1,
    Evening2,
    Night1,
    Night2,
}

impl DayPeriod {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value {
            "am" => Some(Self::Am),
            "pm" => Some(Self::Pm),
            "midnight" => Some(Self::Midnight),
            "noon" => Some(Self::Noon),
            "morning1" => Some(Self::Morning1),
            "morning2" => Some(Self::Morning2),
            "afternoon1" => Some(Self::Afternoon1),
            "afternoon2" => Some(Self::Afternoon2),
            "evening1" => Some(Self::Evening1),
            "evening2" => Some(Self::Evening2),
            "night1" => Some(Self::Night1),
            "night2" => Some(Self::Night2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PeriodKind {
    AmPm,
    NoonMidnight,
    Flexible,
}

/// Width and symbol combinations exist only after the generated boundary checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Field {
    Era(NameWidth),
    Year(u8),
    RelatedYear(u8),
    CyclicYear(NameWidth),
    Month {
        context: NameContext,
        width: u8,
    },
    Day(u8),
    Weekday {
        context: NameContext,
        width: NameWidth,
    },
    DayPeriod {
        kind: PeriodKind,
        width: NameWidth,
    },
    Hour {
        cycle: DateTimeHourCycle,
        width: u8,
    },
    Minute(u8),
    Second(u8),
    Fraction(DateTimeFractionalDigits),
    ZoneName(TimeZoneNameStyle),
}

impl Field {
    pub(super) fn from_ldml(symbol: char, width: u8) -> Result<Self, DateTimeFormatError> {
        let text = || match width {
            1..=3 => Ok(NameWidth::Abbreviated),
            4 => Ok(NameWidth::Wide),
            5 => Ok(NameWidth::Narrow),
            _ => Err(invalid()),
        };
        let numeric = || {
            if (1..=2).contains(&width) {
                Ok(width)
            } else {
                Err(invalid())
            }
        };
        match symbol {
            'G' => Ok(Self::Era(text()?)),
            'y' | 'r' if (1..=6).contains(&width) => Ok(if symbol == 'y' {
                Self::Year(width)
            } else {
                Self::RelatedYear(width)
            }),
            'U' => Ok(Self::CyclicYear(text()?)),
            'M' | 'L' if (1..=5).contains(&width) => Ok(Self::Month {
                context: if symbol == 'M' {
                    NameContext::Format
                } else {
                    NameContext::Standalone
                },
                width,
            }),
            'd' => Ok(Self::Day(numeric()?)),
            'E' | 'e' | 'c' if (symbol == 'E' || width >= 3) && (1..=6).contains(&width) => {
                Ok(Self::Weekday {
                    context: if symbol == 'c' {
                        NameContext::Standalone
                    } else {
                        NameContext::Format
                    },
                    width: if width == 6 {
                        NameWidth::Short
                    } else {
                        text()?
                    },
                })
            }
            'a' | 'b' | 'B' => Ok(Self::DayPeriod {
                kind: match symbol {
                    'a' => PeriodKind::AmPm,
                    'b' => PeriodKind::NoonMidnight,
                    _ => PeriodKind::Flexible,
                },
                width: text()?,
            }),
            'h' | 'H' | 'K' | 'k' => Ok(Self::Hour {
                cycle: match symbol {
                    'h' => DateTimeHourCycle::H12,
                    'H' => DateTimeHourCycle::H23,
                    'K' => DateTimeHourCycle::H11,
                    _ => DateTimeHourCycle::H24,
                },
                width: numeric()?,
            }),
            'm' => Ok(Self::Minute(numeric()?)),
            's' => Ok(Self::Second(numeric()?)),
            'S' => Ok(Self::Fraction(
                DateTimeFractionalDigits::new(width).map_err(|_| invalid())?,
            )),
            'z' if (1..=4).contains(&width) => Ok(Self::ZoneName(if width == 4 {
                TimeZoneNameStyle::Long
            } else {
                TimeZoneNameStyle::Short
            })),
            'v' if matches!(width, 1 | 4) => Ok(Self::ZoneName(if width == 4 {
                TimeZoneNameStyle::LongGeneric
            } else {
                TimeZoneNameStyle::ShortGeneric
            })),
            'O' if matches!(width, 1 | 4) => Ok(Self::ZoneName(if width == 4 {
                TimeZoneNameStyle::LongOffset
            } else {
                TimeZoneNameStyle::ShortOffset
            })),
            _ => Err(invalid()),
        }
    }

    pub(super) const fn part(self) -> DateTimePartKind {
        match self {
            Self::Era(_) => DateTimePartKind::Era,
            Self::Year(_) => DateTimePartKind::Year,
            Self::RelatedYear(_) => DateTimePartKind::RelatedYear,
            Self::CyclicYear(_) => DateTimePartKind::YearName,
            Self::Month { .. } => DateTimePartKind::Month,
            Self::Day(_) => DateTimePartKind::Day,
            Self::Weekday { .. } => DateTimePartKind::Weekday,
            Self::DayPeriod { .. } => DateTimePartKind::DayPeriod,
            Self::Hour { .. } => DateTimePartKind::Hour,
            Self::Minute(_) => DateTimePartKind::Minute,
            Self::Second(_) => DateTimePartKind::Second,
            Self::Fraction(_) => DateTimePartKind::FractionalSecond,
            Self::ZoneName(_) => DateTimePartKind::TimeZoneName,
        }
    }

    pub(super) const fn numbering_symbol(self) -> Option<char> {
        match self {
            Self::Year(_) => Some('y'),
            Self::RelatedYear(_) => Some('r'),
            Self::Month {
                context: NameContext::Format,
                ..
            } => Some('M'),
            Self::Month {
                context: NameContext::Standalone,
                ..
            } => Some('L'),
            Self::Day(_) => Some('d'),
            Self::Hour { cycle, .. } => Some(match cycle {
                DateTimeHourCycle::H11 => 'K',
                DateTimeHourCycle::H12 => 'h',
                DateTimeHourCycle::H23 => 'H',
                DateTimeHourCycle::H24 => 'k',
            }),
            Self::Minute(_) => Some('m'),
            Self::Second(_) => Some('s'),
            Self::Fraction(_) => Some('S'),
            Self::Era(_)
            | Self::CyclicYear(_)
            | Self::Weekday { .. }
            | Self::DayPeriod { .. }
            | Self::ZoneName(_) => None,
        }
    }
}

fn invalid() -> DateTimeFormatError {
    DateTimeFormatError::InvalidProfile("unknown LDML field or width".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    Literal(String),
    Field(Field),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Pattern {
    pub(super) tokens: Vec<Token>,
    pub(super) numbering: Vec<(Option<char>, String)>,
    pub(super) skeleton: Vec<Field>,
}
impl Pattern {
    pub(super) fn new(tokens: Vec<Token>, numbering: Vec<(Option<char>, String)>) -> Self {
        let skeleton = tokens
            .iter()
            .filter_map(|token| match token {
                Token::Field(field) => Some(*field),
                Token::Literal(_) => None,
            })
            .collect();
        Self {
            tokens,
            numbering,
            skeleton,
        }
    }
    pub(super) fn single(field: Field) -> Self {
        Self::new(vec![Token::Field(field)], Vec::new())
    }
    pub(super) fn with_skeleton(mut self, source: &str) -> Result<Self, DateTimeFormatError> {
        let mut fields = Vec::new();
        let mut characters = source.chars().peekable();
        while let Some(symbol) = characters.next() {
            let mut width = 1_u8;
            while characters.peek() == Some(&symbol) {
                characters.next();
                width = width.checked_add(1).ok_or_else(invalid)?;
            }
            let field = Field::from_ldml(symbol, width)?;
            if fields
                .iter()
                .any(|other: &Field| other.part() == field.part())
            {
                return Err(invalid());
            }
            fields.push(field);
        }
        if fields.is_empty() {
            return Err(invalid());
        }
        self.skeleton = fields;
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GlueToken {
    Literal(String),
    First,
    Second,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Glue {
    pub(super) tokens: Vec<GlueToken>,
}

impl Glue {
    pub(super) fn combine(
        &self,
        first: &Pattern,
        second: &Pattern,
    ) -> Result<Pattern, DateTimeFormatError> {
        let mut tokens = Vec::new();
        for token in &self.tokens {
            match token {
                GlueToken::Literal(value) => tokens.push(Token::Literal(value.clone())),
                GlueToken::First => tokens.extend(first.tokens.iter().cloned()),
                GlueToken::Second => tokens.extend(second.tokens.iter().cloned()),
            }
        }
        let mut numbering: Vec<(Option<char>, String)> = Vec::new();
        // A bare override belongs to its source pattern. Expand it to that
        // pattern's numeric fields before composing independently styled parts.
        for pattern in [first, second] {
            for symbol in pattern.tokens.iter().filter_map(|token| match token {
                Token::Field(field) => field.numbering_symbol(),
                Token::Literal(_) => None,
            }) {
                let selected = pattern
                    .numbering
                    .iter()
                    .find(|(field, _)| *field == Some(symbol))
                    .or_else(|| pattern.numbering.iter().find(|(field, _)| field.is_none()));
                let Some((_, system)) = selected else {
                    continue;
                };
                if let Some((_, existing)) =
                    numbering.iter().find(|(field, _)| *field == Some(symbol))
                {
                    if existing != system {
                        return Err(DateTimeFormatError::InvalidProfile(
                            "incompatible numbering in composed date/time pattern".into(),
                        ));
                    }
                } else {
                    numbering.push((Some(symbol), system.clone()));
                }
            }
        }
        Ok(Pattern {
            tokens,
            numbering,
            skeleton: first
                .skeleton
                .iter()
                .chain(&second.skeleton)
                .copied()
                .collect(),
        })
    }
}
