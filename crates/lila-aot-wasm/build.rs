//! Validate immutable compiler data once, before any worker can use it.

#[path = "src/data/temporal_east_asian_years/construction.rs"]
mod construction;
// The format also defines the emitted calendar model's constants and readers.
#[allow(dead_code)]
#[path = "src/data/temporal_east_asian_years/format.rs"]
mod format;
#[path = "src/data/unicode_case_tables/construction.rs"]
mod unicode_case_construction;

fn main() {
    for input in [
        "build.rs",
        "Cargo.toml",
        "../../Cargo.lock",
        "src/data/temporal_east_asian_years/construction.rs",
        "src/data/temporal_east_asian_years/format.rs",
        "src/data/unicode_case_tables/construction.rs",
        "../../vendor/icu_calendar-2.0.6",
    ] {
        println!("cargo:rerun-if-changed={input}");
    }
    let bytes = construction::build_image()
        .expect("the pinned Chinese/Dangi provider must produce the complete checked catalog");
    let output =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    std::fs::write(output.join("temporal-east-asian-years.bin"), bytes)
        .expect("write the immutable Chinese/Dangi catalog");
    for (name, bytes) in [
        (
            "unicode-lowercase.bin",
            unicode_case_construction::lowercase(),
        ),
        (
            "unicode-uppercase.bin",
            unicode_case_construction::uppercase(),
        ),
    ] {
        std::fs::write(output.join(name), bytes).expect("write exact Unicode case mappings");
    }
    std::fs::write(
        output.join("unicode-case-version.bin"),
        [
            char::UNICODE_VERSION.0,
            char::UNICODE_VERSION.1,
            char::UNICODE_VERSION.2,
        ],
    )
    .expect("write the build's Unicode authority");
}
