use std::sync::Arc;

use super::{format, raw, records::OffsetPattern, Profile, Snapshot, ZoneNames};
use crate::TimeZoneNameStyle;

#[test]
fn genuine_english_and_french_offset_patterns_keep_captured_signs_and_seconds() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("offset_patterns_cldr47.json")).unwrap();
    for (language, cases) in [
        (
            "en",
            [
                (0, "GMT", "GMT"),
                (-1800, "GMT-0:30", "GMT-00:30"),
                (-3600, "GMT-1", "GMT-01:00"),
                (-3601, "GMT-1:00:01", "GMT-01:00:01"),
                (-1, "GMT-0:00:01", "GMT-00:00:01"),
                (3601, "GMT+1:00:01", "GMT+01:00:01"),
            ],
        ),
        (
            "fr",
            [
                (0, "UTC", "UTC"),
                (-1800, "UTC−0:30", "UTC−00:30"),
                (-3600, "UTC−1", "UTC−01:00"),
                (-3601, "UTC−1:00:01", "UTC−01:00:01"),
                (-1, "UTC−0:00:01", "UTC−00:00:01"),
                (3601, "UTC+1:00:01", "UTC+01:00:01"),
            ],
        ),
    ] {
        // This private carrier checks the actual formatter with genuine en/fr
        // pattern scalars. Complete French locale admission belongs to208 data.
        let source = include_str!("../generated/profile.json");
        let mut profile = Profile::from_json(source).unwrap();
        let raw_profile: serde_json::Value = serde_json::from_str(source).unwrap();
        let index = raw_profile["locales"][0]["zone_name_ref"].as_u64().unwrap() as usize;
        let mut names = raw_profile["zone_name_pool"][index].clone();
        names["patterns"] = fixtures[language]["patterns"].clone();
        let names: raw::ZoneNames = serde_json::from_value(names).unwrap();
        let checked = ZoneNames::from_raw(names, &profile.geography).unwrap();
        profile.locales[0].zones = Arc::new(checked);
        let locale = &profile.locales[0];
        for (offset, short, long) in cases {
            for (style, expected) in [
                (TimeZoneNameStyle::ShortOffset, short),
                (TimeZoneNameStyle::LongOffset, long),
            ] {
                assert_eq!(
                    format(&profile, locale, "latn", &Snapshot::Fixed(offset), style).unwrap(),
                    expected,
                    "{language} {offset} {style:?}"
                );
            }
        }
    }
}

#[test]
fn an_unreviewed_offset_pattern_cannot_construct_a_checked_owner() {
    for pattern in [
        "+HH:mm;+HH:mm",
        "+HH:mm;−HH:mm;−HH:mm",
        "+H:mm;-H:mm",
        "+HH.mm;-HH.mm",
        "−HH:mm;−HH:mm",
        "+HH:mm;﹣HH:mm",
    ] {
        assert!(OffsetPattern::from_pattern(pattern).is_err(), "{pattern}");
    }
}
