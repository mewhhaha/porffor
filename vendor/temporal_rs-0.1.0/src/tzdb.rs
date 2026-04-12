// We can clean these imports up eventually

#[cfg(feature = "compiled_data")]
#[cfg(test)]
pub use timezone_provider::tzif::CompiledTzdbProvider;
#[cfg(test)]
pub use timezone_provider::tzif::FsTzdbProvider;

#[cfg(test)]
mod tests {
    use crate::{
        builtins::calendar::CalendarFields, partial::PartialZonedDateTime, TimeZone, ZonedDateTime,
    };

    use super::FsTzdbProvider;

    #[test]
    fn canonical_time_zone() {
        let provider = FsTzdbProvider::default();
        let valid_iana_identifiers = [
            ("AFRICA/Bissau", "Africa/Bissau", "-01:00"),
            ("America/Belem", "America/Belem", "-03:00"),
            ("Europe/Vienna", "Europe/Vienna", "+01:00"),
            ("America/New_York", "America/New_York", "-05:00"),
            ("Africa/CAIRO", "Africa/Cairo", "+02:00"),
            ("Asia/Ulan_Bator", "Asia/Ulan_Bator", "+07:00"),
            ("GMT", "GMT", "+00:00"),
            ("etc/gmt", "Etc/GMT", "+00:00"),
            (
                "1994-11-05T08:15:30-05:00[America/New_York]",
                "America/New_York",
                "-05:00",
            ),
            (
                "1994-11-05T08:15:30-05[America/Chicago]",
                "America/Chicago",
                "-06:00",
            ),
            ("EuROpe/DUBLIn", "Europe/Dublin", "+01:00"),
        ];

        for (valid_iana_identifier, canonical, offset) in valid_iana_identifiers {
            let time_zone =
                TimeZone::try_from_str_with_provider(valid_iana_identifier, &provider).unwrap();

            assert_eq!(
                time_zone.identifier_with_provider(&provider).unwrap(),
                canonical
            );
            let result = ZonedDateTime::from_partial_with_provider(
                PartialZonedDateTime::default()
                    .with_calendar_fields(
                        CalendarFields::new()
                            .with_year(1970)
                            .with_month(1)
                            .with_day(1),
                    )
                    .with_timezone(Some(time_zone)),
                None,
                None,
                None,
                &provider,
            )
            .unwrap();
            assert_eq!(result.offset(), offset);
        }
    }

    #[test]
    fn temporal_primary_identifier_uses_ecma402_link_mapping() {
        let provider = FsTzdbProvider::default();

        let jan_mayen =
            TimeZone::try_from_identifier_str_with_provider("Atlantic/Jan_Mayen", &provider)
                .unwrap();
        let longyearbyen =
            TimeZone::try_from_identifier_str_with_provider("Arctic/Longyearbyen", &provider)
                .unwrap();
        assert!(jan_mayen
            .time_zone_equals_with_provider(&longyearbyen, &provider)
            .unwrap());

        let asmera = TimeZone::try_from_identifier_str_with_provider("Africa/Asmera", &provider)
            .unwrap();
        let asmara = TimeZone::try_from_identifier_str_with_provider("Africa/Asmara", &provider)
            .unwrap();
        assert!(asmera
            .time_zone_equals_with_provider(&asmara, &provider)
            .unwrap());
    }
}
