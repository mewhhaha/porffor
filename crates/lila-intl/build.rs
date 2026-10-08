//! Export exact locked ICU data ahead of execution, never from host locale.

#[path = "src/image_build/calendar.rs"]
mod calendar;
#[path = "src/image_build/collator.rs"]
mod collator;
#[path = "src/image_build/keyword.rs"]
mod keyword;
#[path = "src/image_build/list.rs"]
mod list;
#[path = "src/image_build/locale.rs"]
mod locale;
#[path = "src/image_build/named_time_zones.rs"]
mod named_time_zones;
#[path = "src/image_build/segmenter.rs"]
mod segmenter;

fn main() {
    for source in [
        "build.rs",
        "src/image_build/locale.rs",
        "src/image_build/list.rs",
        "src/image_build/collator.rs",
        "src/image_build/segmenter.rs",
        "src/image_build/calendar.rs",
        "src/image_build/keyword.rs",
        "src/image_build/named_time_zones.rs",
        "src/provider/keyword_aliases/generated.rs",
        "data/iana-tzdb-2026a/catalogue.tsv",
        "data/locale-time-zones-iana2026a/zone.tab",
        "data/locale-time-zones-iana2026a/regions.tsv",
        "../../vendor/icu_calendar-2.0.6/src/provider.rs",
        "../../vendor/icu_calendar-2.0.6/src/provider/hijri.rs",
        "../../vendor/icu_calendar-2.0.6/src/cal/hijri/ummalqura_data.rs",
        "src/segmenter/profile.json",
        "Cargo.toml",
        "../../Cargo.lock",
        "../lila-intl-collator-data/src/baked.rs",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    let output =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    for (name, bytes) in [
        (
            "locale-data.blob",
            locale::export().expect("complete pinned ICU Locale data"),
        ),
        (
            "list-data.blob",
            list::export().expect("complete pinned ICU List data"),
        ),
        (
            "collator-data.blob",
            collator::export().expect("complete pinned ICU Collator data"),
        ),
        (
            "segmenter-data.blob",
            segmenter::export().expect("complete pinned ICU Segmenter data"),
        ),
        (
            "calendar-data.blob",
            calendar::export().expect("complete pinned selected calendar data"),
        ),
        (
            "keyword-data.json",
            keyword::export().expect("complete pinned Locale keyword data"),
        ),
        (
            "named-time-zone-data.blob",
            named_time_zones::export().expect("complete pinned IANA transition and country data"),
        ),
    ] {
        std::fs::write(output.join(name), bytes).expect("write immutable ICU image");
    }
}
