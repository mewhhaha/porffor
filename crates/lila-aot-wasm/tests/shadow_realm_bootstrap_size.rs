//! The native optional-alias cohort must share Realm bootstrap without moving
//! any emitted body across the Engine's whole-module size-optimization switch.

use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use std::collections::{BTreeMap, BTreeSet};
use wasmparser::{Operator, Parser, Payload, TypeRef, Validator, WasmFeatures};

// This is the exact sloppy-mode source used by the Engine's strict/sloppy
// semantic control, with the same empty preamble and default host policy.
const SOURCE: &str = include_str!(
    "../../lila-engine/tests/fixtures/shadow_realm/finite_candidates_receiver_aliases.js"
);

// Engine switches to SizeOptimized at >= 1 MiB for even one code body. This
// cohort previously emitted a 1,196,893-byte main body despite passing natively.
const SIZE_OPTIMIZED_MIN_BODY_BYTES: usize = 1024 * 1024;
const INITIALIZER: &str = "helper::realm_initialize_intrinsics";

struct EncodedBody {
    bytes: usize,
    calls: BTreeSet<u32>,
}

#[test]
fn shadow_realm_bootstrap_is_shared_and_keeps_every_body_below_native_size_switch() {
    let artifact = std::thread::Builder::new()
        .name("shadow-realm-bootstrap-size".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let parsed = parse(SOURCE, ParseOptions::script())
                .expect("native optional-alias fixture parses");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let artifact =
                lila_aot_wasm::emit(&program).expect("native optional-alias fixture emits");
            let mut features = WasmFeatures::default();
            for feature in [
                WasmFeatures::THREADS,
                WasmFeatures::MULTI_MEMORY,
                WasmFeatures::REFERENCE_TYPES,
                WasmFeatures::FUNCTION_REFERENCES,
                WasmFeatures::GC,
                WasmFeatures::EXCEPTIONS,
                WasmFeatures::TAIL_CALL,
            ] {
                features.set(feature, true);
            }
            Validator::new_with_features(features)
                .validate_all(&artifact.bytes)
                .expect("shared Realm initialization and its callers validate");
            artifact
        })
        .expect("compiler worker starts")
        .join()
        .expect("compiler worker completes");

    let unique_index = |selected: &str| {
        let mut matches = artifact
            .function_sizes
            .iter()
            .filter(|body| body.name == selected);
        let index = matches
            .next()
            .unwrap_or_else(|| panic!("missing emitted body: {selected}"))
            .wasm_index;
        assert!(
            matches.next().is_none(),
            "{selected} must have exactly one emitted body"
        );
        index
    };
    let initializer = unique_index(INITIALIZER);
    let callers = ["lila::main", "builtin::ShadowRealm"].map(|name| (name, unique_index(name)));
    let names = artifact
        .function_sizes
        .iter()
        .map(|body| (body.wasm_index, body.name.as_str()))
        .collect::<BTreeMap<_, _>>();

    // Measure the encoded bodies used by the Engine's policy. The emission
    // inventory only supplies names/indices; actual Wasm instructions must
    // establish that both bootstrap callers consume the same helper.
    let mut bodies = BTreeMap::<u32, EncodedBody>::new();
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
                let mut calls = BTreeSet::new();
                if callers.iter().any(|(_, index)| *index == next_function) {
                    for operator in body.get_operators_reader().expect("body opens") {
                        match operator.expect("operator decodes") {
                            Operator::Call { function_index }
                            | Operator::ReturnCall { function_index } => {
                                calls.insert(function_index);
                            }
                            _ => {}
                        }
                    }
                }
                bodies.insert(
                    next_function,
                    EncodedBody {
                        bytes: body.range().len(),
                        calls,
                    },
                );
                next_function += 1;
            }
            _ => {}
        }
    }

    assert_eq!(bodies.len(), artifact.function_sizes.len());
    let initializer_body = bodies
        .get(&initializer)
        .expect("shared initializer has an actual code body");
    eprintln!("{INITIALIZER}: {} bytes", initializer_body.bytes);
    for (name, index) in callers {
        let body = bodies
            .get(&index)
            .expect("bootstrap caller has a code body");
        eprintln!("{name}: {} bytes", body.bytes);
        assert!(
            body.calls.contains(&initializer),
            "{name} must directly call the shared {INITIALIZER}"
        );
    }

    let oversized = bodies
        .iter()
        .filter(|(_, body)| body.bytes >= SIZE_OPTIMIZED_MIN_BODY_BYTES)
        .map(|(&index, body)| (index, names.get(&index), body.bytes))
        .collect::<Vec<_>>();
    assert!(
        oversized.is_empty(),
        "every body, including main, must stay below the {SIZE_OPTIMIZED_MIN_BODY_BYTES}-byte \
         whole-module SizeOptimized switch; oversized bodies: {oversized:?}"
    );
}
