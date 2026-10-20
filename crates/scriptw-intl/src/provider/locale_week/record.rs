use crate::CanonicalLocaleId;

/// ISO 8601's closed Monday=1 through Sunday=7 domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum IsoWeekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl IsoWeekday {
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    pub const fn iso_number(self) -> u8 {
        self as u8
    }

    pub const fn from_iso_number(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Monday),
            2 => Some(Self::Tuesday),
            3 => Some(Self::Wednesday),
            4 => Some(Self::Thursday),
            5 => Some(Self::Friday),
            6 => Some(Self::Saturday),
            7 => Some(Self::Sunday),
            _ => None,
        }
    }

    pub(super) fn from_unicode_value(value: &str) -> Option<Self> {
        match value {
            "mon" => Some(Self::Monday),
            "tue" => Some(Self::Tuesday),
            "wed" => Some(Self::Wednesday),
            "thu" => Some(Self::Thursday),
            "fri" => Some(Self::Friday),
            "sat" => Some(Self::Saturday),
            "sun" => Some(Self::Sunday),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidLocaleWeekInfo {
    FirstDay,
    EmptyWeekend,
    WeekendOrder,
    WeekendMask,
}

/// Only valid, nonempty, ascending and unique weekends may leave the kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleWeekInfo {
    first_day: IsoWeekday,
    weekend: Box<[IsoWeekday]>,
}

impl LocaleWeekInfo {
    pub fn new(
        first_day: IsoWeekday,
        weekend: impl Into<Box<[IsoWeekday]>>,
    ) -> Result<Self, InvalidLocaleWeekInfo> {
        let weekend = weekend.into();
        if weekend.is_empty() {
            return Err(InvalidLocaleWeekInfo::EmptyWeekend);
        }
        if weekend.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(InvalidLocaleWeekInfo::WeekendOrder);
        }
        Ok(Self { first_day, weekend })
    }

    /// Mask bit zero is Monday; bit six is Sunday. Higher bits are forbidden.
    pub fn from_iso_mask(first_day: u32, weekend_mask: u32) -> Result<Self, InvalidLocaleWeekInfo> {
        let first_day =
            IsoWeekday::from_iso_number(first_day).ok_or(InvalidLocaleWeekInfo::FirstDay)?;
        if weekend_mask & !0x7f != 0 {
            return Err(InvalidLocaleWeekInfo::WeekendMask);
        }
        let weekend: Vec<_> = IsoWeekday::ALL
            .into_iter()
            .filter(|day| weekend_mask & (1 << (day.iso_number() - 1)) != 0)
            .collect();
        Self::new(first_day, weekend.into_boxed_slice())
    }

    pub const fn first_day(&self) -> IsoWeekday {
        self.first_day
    }

    pub fn weekend(&self) -> &[IsoWeekday] {
        &self.weekend
    }

    pub fn weekend_mask(&self) -> u8 {
        self.weekend
            .iter()
            .fold(0, |mask, day| mask | (1 << (day.iso_number() - 1)))
    }

    pub(super) fn with_first_day(&self, first_day: IsoWeekday) -> Self {
        Self {
            first_day,
            weekend: self.weekend.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleWeekRequest {
    locale: CanonicalLocaleId,
}

impl LocaleWeekRequest {
    pub fn new(locale: CanonicalLocaleId) -> Self {
        Self { locale }
    }

    pub fn into_locale(self) -> CanonicalLocaleId {
        self.locale
    }
}
