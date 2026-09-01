use std::fs;
use std::path::{Path, PathBuf};

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/conversion-abrupt-route-capabilities.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

fn rust_sources(path: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    entries.sort();

    for entry in entries {
        if entry.is_dir() {
            rust_sources(&entry, sources);
        } else if entry.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&entry)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", entry.display()));
            sources.push((entry, source));
        }
    }
}

#[test]
fn conversion_abrupt_routes_are_exact_capability_free_domains() {
    for (route, end, expected) in [
        (
            "ToPrimitiveAbruptRoute",
            "/// Where the Symbol throw admitted by primitive `ToString` must go.",
            "ActiveHandler,ReturnCurrentFunction,IteratorCloseAndReturn(IteratorCloseOnThrowLocals),}",
        ),
        (
            "PrimitiveToStringAbruptRoute",
            "/// Where a throw from one of the exceptional `ToLength` consumers must go.",
            "ActiveHandler,ReturnCurrentFunction,IteratorCloseAndReturn(IteratorCloseOnThrowLocals),}",
        ),
        (
            "ToLengthAbruptRoute",
            "/// Where the throw created by primitive `ToNumber` is owned.",
            "ActiveHandler,RejectArrayFromAsyncAndReturnPromise{capability_record_local:u32,promise_payload_local:u32,promise_tag_local:u32,},}",
        ),
    ] {
        let declaration = bounded(
            OPERATIONS_SOURCE,
            &format!("pub(crate) enum {route} {{"),
            end,
        );
        assert_eq!(normalized(declaration), expected);

        let prefix = OPERATIONS_SOURCE
            .split_once(&format!("pub(crate) enum {route} {{"))
            .expect("route declaration should exist")
            .0
            .rsplit_once('\n')
            .map_or("", |(_, line)| line);
        assert!(!prefix.trim_start().starts_with("#[derive("));
    }

    let mut sources = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    for route in [
        "ToPrimitiveAbruptRoute",
        "PrimitiveToStringAbruptRoute",
        "ToLengthAbruptRoute",
    ] {
        for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
            let implementation = format!("impl {capability} for {route}");
            assert!(
                sources
                    .iter()
                    .all(|(_, source)| !source.contains(&implementation)),
                "{route} must not implement {capability}",
            );
        }
    }
}

#[test]
fn conversion_abrupt_routes_move_into_one_exhaustive_finisher_each() {
    for (route, finisher, end, arms) in [
        (
            "ToPrimitiveAbruptRoute",
            "    fn finish_to_primitive_operation(",
            "    fn finish_primitive_to_string_throw(",
            3,
        ),
        (
            "PrimitiveToStringAbruptRoute",
            "    fn finish_primitive_to_string_throw(",
            "    fn finish_to_length_operation(",
            3,
        ),
        (
            "ToLengthAbruptRoute",
            "    fn finish_to_length_operation(",
            "    /// The first property-operation migration",
            2,
        ),
    ] {
        let consumer = normalized(bounded(OPERATIONS_SOURCE, finisher, end));
        assert!(consumer.contains(&format!("route:{route}")));
        assert!(!consumer.contains(&format!("route:&{route}")));
        assert_eq!(consumer.matches(&format!("{route}::")).count(), arms);
        assert_eq!(consumer.matches("matchroute{").count(), 1);
        assert!(!consumer.contains("_=>"));
        assert!(!consumer.contains("unreachable!"));
        assert!(!consumer.contains("todo!"));
    }

    for evidence in [CONTRACT, TASK] {
        for route in [
            "ToPrimitiveAbruptRoute",
            "PrimitiveToStringAbruptRoute",
            "ToLengthAbruptRoute",
        ] {
            assert!(evidence.contains(route));
        }
        assert!(evidence.contains("capability"));
        assert!(evidence.contains("exhaustive"));
    }
}
