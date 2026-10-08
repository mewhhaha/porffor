//! The old raw publication mirrors retired with the shared GC bootstrap.
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_created_realm_builtin_function_prototypes.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/callable-function-prototype.md");

#[test]
fn consumer_oracle_pins_entry_and_created_realm_observables() {
    for witness in [
        "entry Function.prototype",
        "first Function.prototype",
        "second Function.prototype",
        "label + \" typeof\"",
        "label + \" tag\"",
        "function () { [native code] }",
        "label + \" empty call\"",
        "label + \" argument call\"",
        "label + \" length\"",
        "label + \" name\"",
        "label + \" publication\"",
        "label + \" own prototype\"",
        "label + \" construct error\"",
        "Function.prototype realm identity",
        "first realm evalScript",
        "second realm evalScript",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing fixture witness {witness}"
        );
    }

    for exact in [
        "S15.3.3.1_A1.js",
        "S15.3.4_A1.js",
        "S15.3.4_A2_T1.js",
        "S15.3.4_A2_T2.js",
        "S15.3.4_A2_T3.js",
    ] {
        assert!(CONTRACT.contains(exact), "missing exact witness {exact}");
    }
    assert!(CONTRACT.contains("not a current-HEAD execution result"));
}
