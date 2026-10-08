use super::*;

// These owners exist only after record construction and checked canonical lookup.
// Public gregory/iso8601 tags stay in locale resolution; only their data is shared.
pub(super) struct LocaleRecords {
    pub(super) calendars: CalendarRecords,
    pub(super) zones: Arc<super::super::zones::ZoneNames>,
}

pub(super) struct CalendarRecords {
    records: [Option<Arc<Calendar>>; 16],
}

impl CalendarRecords {
    pub(super) fn get(&self, calendar: DateTimeCalendar) -> Option<&Calendar> {
        self.records[CalendarId::from_admitted(calendar).index()].as_deref()
    }
    #[cfg(test)]
    pub(super) fn for_kind(&self, calendar: CalendarId) -> Option<&Calendar> {
        self.iter()
            .find_map(|(kind, record)| (kind == calendar).then_some(record))
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = (CalendarId, &Calendar)> {
        CalendarId::ALL
            .iter()
            .copied()
            .zip(self.records.iter())
            .filter_map(|(kind, record)| record.as_deref().map(|record| (kind, record)))
    }
}

pub(super) struct Pools {
    recipe: ProfileRecipe,
    calendars: Vec<Arc<Calendar>>,
    zones: Vec<Arc<super::super::zones::ZoneNames>>,
    used_calendars: Vec<bool>,
    used_zones: Vec<bool>,
}

impl Pools {
    pub(super) fn numbering_for_locale(
        &self,
        default: &str,
        digits: &BTreeMap<String, [char; 10]>,
    ) -> BTreeSet<String> {
        self.recipe.numbering_for_locale(default, digits)
    }
    pub(super) fn from_raw(
        calendars: Vec<raw::Calendar>,
        zones: Vec<raw::ZoneNames>,
        digits: &BTreeMap<String, [char; 10]>,
        algorithmic: &BTreeMap<String, AlgorithmicField>,
        geography: &super::super::zones::Geography,
        recipe: ProfileRecipe,
    ) -> Result<Self, DateTimeFormatError> {
        if calendars.is_empty() || zones.is_empty() {
            return Err(invalid("empty date/time pool"));
        }
        let used_calendars = vec![false; calendars.len()];
        let used_zones = vec![false; zones.len()];
        let calendars = calendars
            .into_iter()
            .map(|row| Calendar::from_raw(row, digits, algorithmic).map(Arc::new))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            calendars,
            zones: zones
                .into_iter()
                .map(|row| {
                    super::super::zones::ZoneNames::from_selected_raw(
                        row,
                        geography,
                        recipe.localized_zones(),
                    )
                    .map(Arc::new)
                })
                .collect::<Result<Vec<_>, _>>()?,
            used_calendars,
            used_zones,
            recipe,
        })
    }

    pub(super) fn for_locale(
        &mut self,
        rows: Vec<(String, u32)>,
        zone: u32,
        default: DateTimeCalendar,
    ) -> Result<LocaleRecords, DateTimeFormatError> {
        let expected = self.recipe.calendars_for_locale(default);
        if rows.len() != expected.len()
            || rows
                .iter()
                .map(|row| row.0.as_str())
                .ne(expected.iter().map(|kind| kind.as_str()))
        {
            return Err(invalid(
                "noncanonical, duplicate, incomplete or unordered calendar references",
            ));
        }
        let gregory = rows.iter().find(|row| row.0 == "gregory").map(|row| row.1);
        let iso = rows.iter().find(|row| row.0 == "iso8601").map(|row| row.1);
        if gregory
            .zip(iso)
            .is_some_and(|(gregory, iso)| gregory != iso)
        {
            return Err(invalid("Gregorian/ISO references differ"));
        }
        let mut records = std::array::from_fn(|_| None);
        for (kind, (_, index)) in expected.iter().zip(rows) {
            records[kind.index()] = Some(self.calendar(*kind, index)?);
        }
        let calendars = CalendarRecords { records };
        if calendars.get(default).is_none() {
            return Err(invalid("localized default calendar absent"));
        }
        let zone = usize::try_from(zone).map_err(|_| invalid("zone pool reference overflow"))?;
        let zones = Arc::clone(
            self.zones
                .get(zone)
                .ok_or_else(|| invalid("zone pool reference out of bounds"))?,
        );
        self.used_zones[zone] = true;
        Ok(LocaleRecords { calendars, zones })
    }

    fn calendar(
        &mut self,
        calendar: CalendarId,
        index: u32,
    ) -> Result<Arc<Calendar>, DateTimeFormatError> {
        let index =
            usize::try_from(index).map_err(|_| invalid("calendar pool reference overflow"))?;
        let record = self
            .calendars
            .get(index)
            .ok_or_else(|| invalid("calendar pool reference out of bounds"))?;
        if record.kind != CalendarDataKind::for_calendar(calendar) {
            return Err(invalid("calendar reference has the wrong physical domain"));
        }
        self.used_calendars[index] = true;
        Ok(Arc::clone(record))
    }

    pub(super) fn validate_coverage(&self) -> Result<(), DateTimeFormatError> {
        if self.used_calendars.contains(&false) || self.used_zones.contains(&false) {
            return Err(invalid("unused date/time pool records"));
        }
        Ok(())
    }
}
