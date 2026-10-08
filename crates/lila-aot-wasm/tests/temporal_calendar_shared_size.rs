//! Calendar consumers call the same bounded conversion and year algorithms.

use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use std::collections::{BTreeMap, BTreeSet};
use wasmparser::{Operator, Parser, Payload, TypeRef, Validator, WasmFeatures};

const HELPERS: [&str; 9] = [
    "helper::temporal_chinese_year",
    "helper::temporal_dangi_year",
    "helper::temporal_umalqura_year",
    "helper::temporal_umalqura_epoch",
    "helper::temporal_calendar_project_date",
    "helper::temporal_calendar_fields_to_iso",
    "helper::temporal_calendar_days_in_month",
    "helper::temporal_calendar_balance_year_month",
    "helper::temporal_calendar_difference_date",
];
const COMPOSITE_HELPERS: [&str; 2] = [
    "helper::temporal_calendar_project_date",
    "helper::temporal_calendar_fields_to_iso",
];
const YEAR_HELPERS: [&str; 4] = [
    "helper::temporal_chinese_year",
    "helper::temporal_dangi_year",
    "helper::temporal_umalqura_year",
    "helper::temporal_umalqura_epoch",
];
const ADDITION_HELPERS: [&str; 2] = [
    "helper::temporal_calendar_days_in_month",
    "helper::temporal_calendar_balance_year_month",
];
const DIFFERENCE_HELPER: &str = "helper::temporal_calendar_difference_date";
const CONVERSION_HELPERS: [&str; 2] = [
    "helper::temporal_zoned_date_time_convert",
    "helper::temporal_plain_date_convert",
];

// The original repeated year expansions produced 6.96 MB / 5.56 MB /
// 3.11 MB Duration bodies and 2.85 MB ZonedDateTime difference bodies.
// These limits bound each native compilation unit as well as requiring calls
// to the shared algorithms below; an unused helper cannot satisfy the control.
const CONSUMERS: [(&str, u32); 5] = [
    ("builtin::Temporal.Duration.prototype.round", 1024 * 1024),
    ("builtin::Temporal.Duration.prototype.total", 1024 * 1024),
    ("builtin::Temporal.Duration.compare", 1024 * 1024),
    (
        "builtin::Temporal.ZonedDateTime.prototype.until",
        1024 * 1024,
    ),
    (
        "builtin::Temporal.ZonedDateTime.prototype.since",
        1024 * 1024,
    ),
];

const SOURCE: &str = r#"
const duration = new Temporal.Duration(1, 2, 0, 3);
const relativeTo = new Temporal.PlainDate(2024, 2, 10, 'chinese');
duration.round({largestUnit: 'year', smallestUnit: 'month', relativeTo});
duration.total({unit: 'month', relativeTo});
Temporal.Duration.compare(duration, {months: 14}, {relativeTo});
const start = new Temporal.ZonedDateTime(0n, 'UTC', 'dangi');
const end = new Temporal.ZonedDateTime(3456000000000000n, 'UTC', 'dangi');
start.until(end, {largestUnit: 'year'});
end.since(start, {largestUnit: 'year'});
"#;

#[test]
fn duration_and_zoned_difference_share_bounded_calendar_year_helpers() {
    let artifact = std::thread::Builder::new()
        .name("temporal-calendar-shared-size".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let parsed = parse(SOURCE, ParseOptions::script()).expect("Temporal source parses");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let artifact = lila_aot_wasm::emit(&program).expect("Temporal consumers emit");
            eprintln!(
                "calendar artifact: {} bytes, {} code-body bytes",
                artifact.bytes.len(),
                artifact
                    .function_sizes
                    .iter()
                    .map(|body| u64::from(body.body_bytes.bytes()))
                    .sum::<u64>()
            );
            for body in &artifact.function_sizes {
                if body.name == "lila::main"
                    || HELPERS.contains(&body.name.as_str())
                    || CONVERSION_HELPERS.contains(&body.name.as_str())
                    || CONSUMERS.iter().any(|(name, _)| *name == body.name)
                {
                    eprintln!("{}: {} bytes", body.name, body.body_bytes.bytes());
                }
            }
            Validator::new_with_features(WasmFeatures::all())
                .validate_all(&artifact.bytes)
                .expect("shared scalar and whole-Completion results and callers validate");
            artifact
        })
        .expect("compiler worker starts")
        .join()
        .expect("compiler worker completes");

    let summary = |name: &str| {
        let mut matches = artifact
            .function_sizes
            .iter()
            .filter(|body| body.name == name);
        let body = matches
            .next()
            .unwrap_or_else(|| panic!("missing emitted body: {name}"));
        assert!(
            matches.next().is_none(),
            "{name} must have one emitted body"
        );
        body
    };
    let mut selected = BTreeSet::new();
    let helpers = HELPERS.map(|name| {
        let body = summary(name);
        assert!(
            (128..=64 * 1024).contains(&body.body_bytes.bytes()),
            "{name} must have a real bounded body, got {} bytes",
            body.body_bytes.bytes()
        );
        selected.insert(body.wasm_index);
        (name, body.wasm_index)
    });
    for name in CONVERSION_HELPERS {
        let body = summary(name);
        assert!(
            (128..=1024 * 1024).contains(&body.body_bytes.bytes()),
            "{name} must have one real bounded conversion body, got {} bytes",
            body.body_bytes.bytes()
        );
        selected.insert(body.wasm_index);
    }
    for (name, limit) in CONSUMERS {
        let body = summary(name);
        assert!(
            body.body_bytes.bytes() <= limit,
            "{name} is {} bytes; per-body limit is {limit}",
            body.body_bytes.bytes()
        );
        selected.insert(body.wasm_index);
    }

    // Decode only the selected bodies. The typed emission inventory supplies
    // their actual indices, so function names and hard-coded offsets cannot
    // manufacture the required calls.
    let mut calls = BTreeMap::<u32, BTreeSet<u32>>::new();
    let mut next_function = 0u32;
    for payload in Parser::new(0).parse_all(&artifact.bytes) {
        match payload.expect("module decodes") {
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    if matches!(
                        import.expect("import decodes").ty,
                        TypeRef::Func(_) | TypeRef::FuncExact(_)
                    ) {
                        next_function += 1;
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                if selected.contains(&next_function) {
                    let mut body_calls = BTreeSet::new();
                    for operator in body.get_operators_reader().expect("body opens") {
                        match operator.expect("operator decodes") {
                            Operator::Call { function_index }
                            | Operator::ReturnCall { function_index } => {
                                body_calls.insert(function_index);
                            }
                            _ => {}
                        }
                    }
                    calls.insert(next_function, body_calls);
                }
                next_function += 1;
            }
            _ => {}
        }
    }
    for (name, _) in CONSUMERS {
        let body_calls = calls
            .get(&summary(name).wasm_index)
            .expect("consumer body exists");
        assert!(
            body_calls.contains(&summary(CONVERSION_HELPERS[0]).wasm_index),
            "{name} must call the shared complete ZonedDateTime conversion"
        );
        if name.starts_with("builtin::Temporal.Duration") {
            assert!(
                body_calls.contains(&summary(CONVERSION_HELPERS[1]).wasm_index),
                "{name} must call the shared complete PlainDate conversion"
            );
        }
        for helper in COMPOSITE_HELPERS.into_iter().chain(ADDITION_HELPERS) {
            assert!(
                body_calls.contains(&summary(helper).wasm_index),
                "{name} must call {helper}"
            );
        }
        if name != "builtin::Temporal.Duration.compare" {
            assert!(
                body_calls.contains(&summary(DIFFERENCE_HELPER).wasm_index),
                "{name} must call the shared complete calendar difference"
            );
        }
    }
    for (helper, index) in helpers {
        if YEAR_HELPERS.contains(&helper) {
            assert!(
                COMPOSITE_HELPERS.iter().any(|name| {
                    calls
                        .get(&summary(name).wasm_index)
                        .expect("calendar composite body exists")
                        .contains(&index)
                }),
                "the shared calendar conversions must call {helper}"
            );
        }
    }
    let difference_calls = calls
        .get(&summary(DIFFERENCE_HELPER).wasm_index)
        .expect("complete calendar difference body exists");
    for helper in COMPOSITE_HELPERS
        .into_iter()
        .chain(["helper::temporal_calendar_days_in_month"])
    {
        assert!(
            difference_calls.contains(&summary(helper).wasm_index),
            "the shared calendar difference must call {helper}"
        );
    }
}

#[test]
fn plain_date_only_source_collects_shared_converter_dependencies() {
    std::thread::Builder::new()
        .name("temporal-shared-converter-dependencies".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // No ZonedDateTime builtin or option spelling can supply the shared
            // converter's literals before the dependency fixpoint is reached.
            let parsed = parse(
                "Temporal.PlainDate.from('2024-02-29').year;",
                ParseOptions::script(),
            )
            .expect("PlainDate source parses");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let artifact = lila_aot_wasm::emit(&program)
                .expect("shared converters collect their dependencies in the initial pass");
            Validator::new_with_features(WasmFeatures::all())
                .validate_all(&artifact.bytes)
                .expect("all emitted shared conversion bodies validate");
            for helper in CONVERSION_HELPERS {
                assert!(
                    artifact
                        .function_sizes
                        .iter()
                        .any(|body| { body.name == helper && body.body_bytes.bytes() > 20 }),
                    "shared converter must have a real body: {helper}"
                );
            }
        })
        .expect("compiler worker starts")
        .join()
        .expect("compiler worker completes");
}

#[test]
fn partial_date_constructors_collect_shared_converter_host_dependencies() {
    std::thread::Builder::new()
        .name("temporal-shared-converter-imports".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // Neither constructor declares a direct named-zone provider call.
            // The emitted shared converter owns that import in the first pass.
            let parsed = parse(
                "new Temporal.PlainMonthDay(2, 29); new Temporal.PlainYearMonth(2024, 2);",
                ParseOptions::script(),
            )
            .expect("partial-date source parses");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let artifact = lila_aot_wasm::emit(&program)
                .expect("emitted converter owns its checked host import");
            Validator::new_with_features(WasmFeatures::all())
                .validate_all(&artifact.bytes)
                .expect("shared converter and imported host signatures validate");
        })
        .expect("compiler worker starts")
        .join()
        .expect("compiler worker completes");
}
