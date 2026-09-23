use std::collections::BTreeMap;

use crate::datetime::DateTimeFormatError;

use super::pattern::{DayPeriod, NameContext, NameWidth};
use super::raw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum NameKey {
    Era(NameWidth, u8),
    Month(NameContext, NameWidth, u8),
    Weekday(NameContext, NameWidth, u8),
    Period(NameContext, NameWidth, DayPeriod),
    CyclicYear(NameWidth, u8),
    LeapMonth(NameContext, NameWidth),
    NumericLeapMonth,
}

pub(super) struct FieldNames(BTreeMap<NameKey, String>);
impl FieldNames {
    pub(super) fn from_raw(records: Vec<raw::Name>) -> Result<Self, DateTimeFormatError> {
        let mut names = BTreeMap::new();
        for record in records {
            let width = record
                .width
                .as_deref()
                .map(|width| match width {
                    "abbreviated" => Ok(NameWidth::Abbreviated),
                    "wide" => Ok(NameWidth::Wide),
                    "narrow" => Ok(NameWidth::Narrow),
                    "short" => Ok(NameWidth::Short),
                    _ => Err(invalid()),
                })
                .transpose()?;
            let context = record
                .context
                .as_deref()
                .map(|context| match context {
                    "format" => Ok(NameContext::Format),
                    "stand-alone" => Ok(NameContext::Standalone),
                    _ => Err(invalid()),
                })
                .transpose()?;
            let key = match (
                record.kind.as_str(),
                context,
                width,
                record.index,
                record.period.as_deref(),
            ) {
                ("era", None, Some(width), Some(index @ 0..=1), None) => NameKey::Era(width, index),
                ("month", Some(context), Some(width), Some(index @ 1..=12), None) => {
                    NameKey::Month(context, width, index)
                }
                ("weekday", Some(context), Some(width), Some(index @ 0..=6), None) => {
                    NameKey::Weekday(context, width, index)
                }
                ("day_period", Some(context), Some(width), None, Some(period)) => NameKey::Period(
                    context,
                    width,
                    DayPeriod::parse(period).ok_or_else(invalid)?,
                ),
                (
                    "cyclic_year",
                    Some(NameContext::Format),
                    Some(width),
                    Some(index @ 1..=60),
                    None,
                ) => NameKey::CyclicYear(width, index),
                ("leap_month", None, None, None, None) => NameKey::NumericLeapMonth,
                ("leap_month", Some(context), Some(width), None, None) => {
                    NameKey::LeapMonth(context, width)
                }
                _ => return Err(invalid()),
            };
            if record.value.is_empty() || names.insert(key, record.value).is_some() {
                return Err(invalid());
            }
        }
        Ok(Self(names))
    }
    pub(super) fn get(&self, key: NameKey) -> Result<&str, DateTimeFormatError> {
        self.optional(key).ok_or_else(|| {
            DateTimeFormatError::InvalidProfile(format!("missing required calendar name: {key:?}"))
        })
    }
    pub(super) fn optional(&self, key: NameKey) -> Option<&str> {
        self.0.get(&key).map(String::as_str)
    }
}
fn invalid() -> DateTimeFormatError {
    DateTimeFormatError::InvalidProfile("invalid or duplicate calendar name key".into())
}

pub(super) enum PeriodRule {
    At(u32, DayPeriod),
    Range {
        start: u32,
        end: u32,
        period: DayPeriod,
    },
}
pub(super) struct PeriodRules(Vec<PeriodRule>);
impl PeriodRules {
    pub(super) fn from_raw(records: Vec<raw::PeriodRule>) -> Result<Self, DateTimeFormatError> {
        let mut rules = Vec::new();
        for record in records {
            let period = DayPeriod::parse(&record.period).ok_or_else(invalid)?;
            let rule = match (record.at, record.from, record.before) {
                (Some(at), None, None) => PeriodRule::At(parse_minute(&at)?, period),
                (None, Some(from), Some(before)) => PeriodRule::Range {
                    start: parse_minute(&from)?,
                    end: parse_minute(&before)?,
                    period,
                },
                _ => return Err(invalid()),
            };
            rules.push(rule);
        }
        for minute in 0..1440 {
            if rules.iter().filter(|rule| matches!(rule, PeriodRule::Range { start, end, .. } if contains(*start, *end, minute))).count() != 1
                || rules.iter().filter(|rule| matches!(rule, PeriodRule::At(at, _) if *at == minute)).count() > 1 { return Err(invalid()); }
        }
        Ok(Self(rules))
    }
    pub(super) fn periods(&self) -> impl Iterator<Item = DayPeriod> + '_ {
        self.0.iter().map(|rule| match rule {
            PeriodRule::At(_, period) | PeriodRule::Range { period, .. } => *period,
        })
    }
    pub(super) fn select(
        &self,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<DayPeriod, DateTimeFormatError> {
        let minute = u32::from(hour) * 60 + u32::from(minute);
        if second == 0 && nanosecond == 0 {
            if let Some(period) = self.0.iter().find_map(|rule| match rule {
                // Intl has no start/end-of-day context to disambiguate midnight.
                // LDML Day Period Rules allows omitting that exact label.
                PeriodRule::At(at, period) if *at == minute && *period != DayPeriod::Midnight => {
                    Some(*period)
                }
                _ => None,
            }) {
                return Ok(period);
            }
        }
        self.0
            .iter()
            .find_map(|rule| match rule {
                PeriodRule::Range { start, end, period } if contains(*start, *end, minute) => {
                    Some(*period)
                }
                _ => None,
            })
            .ok_or_else(invalid)
    }
}
fn contains(start: u32, end: u32, minute: u32) -> bool {
    if start < end {
        (start..end).contains(&minute)
    } else {
        minute >= start || minute < end
    }
}
fn parse_minute(value: &str) -> Result<u32, DateTimeFormatError> {
    let (hour, minute) = value.split_once(':').ok_or_else(invalid)?;
    let hour = hour.parse::<u32>().map_err(|_| invalid())?;
    let minute = minute.parse::<u32>().map_err(|_| invalid())?;
    if hour > 24 || minute >= 60 || (hour == 24 && minute != 0) {
        return Err(invalid());
    }
    Ok(hour * 60 + minute)
}
