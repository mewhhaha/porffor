use std::ops::Deref;

use super::{raw, validation, DateTimeFormatError};

/// Prefixes captured from either of the two genuine consumed hour patterns.
/// The fields cannot be constructed without validating the complete branches.
pub(super) struct OffsetPattern {
    positive: Box<str>,
    negative: Box<str>,
}

impl OffsetPattern {
    pub(super) fn from_pattern(pattern: &str) -> Result<Self, DateTimeFormatError> {
        if !matches!(pattern, "+HH:mm;-HH:mm" | "+HH:mm;−HH:mm") {
            return Err(super::super::profile::invalid(
                "unreviewed localized offset pattern",
            ));
        }
        let (positive, negative) = pattern
            .split_once(';')
            .ok_or_else(|| super::super::profile::invalid("missing offset pattern branch"))?;
        let prefix = |branch: &str| {
            branch
                .strip_suffix("HH:mm")
                .map(Box::<str>::from)
                .ok_or_else(|| super::super::profile::invalid("invalid offset pattern fields"))
        };
        Ok(Self {
            positive: prefix(positive)?,
            negative: prefix(negative)?,
        })
    }

    fn prefix(&self, negative: bool) -> &str {
        if negative {
            &self.negative
        } else {
            &self.positive
        }
    }
}

/// Complete immutable geography, checked before a localized pool can use it.
pub(in crate::provider::datetime) struct Geography(raw::Geography);

impl Geography {
    pub(in crate::provider::datetime) fn from_raw(
        raw: raw::Geography,
    ) -> Result<Self, DateTimeFormatError> {
        validation::geography(&raw)?;
        Ok(Self(raw))
    }
}

impl Deref for Geography {
    type Target = raw::Geography;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The private field prevents an unchecked raw name block entering the pool.
pub(in crate::provider::datetime) struct ZoneNames {
    raw: raw::ZoneNames,
    offset: OffsetPattern,
}

impl ZoneNames {
    #[cfg(test)]
    pub(in crate::provider::datetime) fn from_raw(
        raw: raw::ZoneNames,
        geography: &Geography,
    ) -> Result<Self, DateTimeFormatError> {
        Self::from_selected_raw(raw, geography, None)
    }
    pub(in crate::provider::datetime) fn from_selected_raw(
        raw: raw::ZoneNames,
        geography: &Geography,
        selected: Option<&[Box<str>]>,
    ) -> Result<Self, DateTimeFormatError> {
        let offset = validation::names(&raw, geography, selected)?;
        Ok(Self { raw, offset })
    }

    pub(in crate::provider::datetime) fn offset_sign(&self, negative: bool) -> &str {
        self.offset.prefix(negative)
    }
}

impl Deref for ZoneNames {
    type Target = raw::ZoneNames;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
