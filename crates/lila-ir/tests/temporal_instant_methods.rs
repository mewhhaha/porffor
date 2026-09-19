use lila_front::{parse, ParseOptions};
use lila_ir::{lower, StandardBuiltinId, ValueKind};

#[test]
fn instant_methods_preserve_result_shapes_and_observable_coercions() {
    for (name, builtin, arguments, property, kind) in [
        (
            "add",
            StandardBuiltinId::TemporalInstantPrototypeAdd,
            "{nanoseconds:1}",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "subtract",
            StandardBuiltinId::TemporalInstantPrototypeSubtract,
            "{nanoseconds:1}",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "round",
            StandardBuiltinId::TemporalInstantPrototypeRound,
            "'nanosecond'",
            "epochNanoseconds",
            ValueKind::BigInt,
        ),
        (
            "until",
            StandardBuiltinId::TemporalInstantPrototypeUntil,
            "new Temporal.Instant(1n)",
            "nanoseconds",
            ValueKind::Number,
        ),
        (
            "since",
            StandardBuiltinId::TemporalInstantPrototypeSince,
            "new Temporal.Instant(1n)",
            "nanoseconds",
            ValueKind::Number,
        ),
    ] {
        assert!(builtin.may_run_user_code_synchronously());
        assert_eq!(builtin.native_function_name(), Some(name));
        let source = format!("new Temporal.Instant(0n).{name}({arguments}).{property};");
        let unit = parse(&source, ParseOptions::script()).expect("Instant fixture parses");
        let program = lower(&unit);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert_eq!(
            program.script.expect("script IR").result_kind(),
            kind,
            "{source}"
        );
    }
}
