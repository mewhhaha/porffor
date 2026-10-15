use super::*;
use lila_front::{parse, ParseOptions};

fn with_lowerer(source: &str, check: impl FnOnce(&ScriptLowerer<'_>)) {
    let mut interner = Interner::default();
    let scope = Scope::new_global();
    let script = Parser::new(Source::from_bytes(source.as_bytes()))
        .parse_script(&scope, &mut interner)
        .expect("capture fixture should parse");
    let analysis = AnalysisBuilder::default().finish(&script, &interner, source);
    let lowerer = ScriptLowerer::new(
        &interner,
        &analysis,
        source.into(),
        RootThisBinding::GlobalObject,
        SCRIPT_OWNER_ID.to_string(),
        HostSurfacePolicy::default(),
    );
    check(&lowerer);
}

#[test]
fn capture_membership_keeps_physical_shadow_aliases_and_source_names() {
    with_lowerer(
        r#"
'use strict';
function owner() {
    let value = 1;
    let uncaptured = 2;
    function outerRead() { return value; }
    {
        let value = 3;
        function innerRead() { return value; }
        return [outerRead, innerRead];
    }
}
"#,
        |lowerer| {
            let storage = |name| {
                lowerer
                    .analysis
                    .function_plans
                    .values()
                    .find(|plan| plan.name == name)
                    .expect("named reader is analyzed")
                    .captures
                    .iter()
                    .find_map(|(storage, capture)| {
                        (capture.source_name == "value").then_some(storage.as_str())
                    })
                    .expect("reader captures its resolved value binding")
            };
            let outer = storage("outerRead");
            let inner = storage("innerRead");
            assert_ne!(outer, inner, "shadowed cells retain distinct storage");
            assert_ne!(inner, "value", "block capture uses its physical alias");
            assert!(lowerer.captured_binding_names.contains(outer));
            assert!(lowerer.captured_binding_names.contains(inner));
            assert!(lowerer.captured_binding_names.contains("value"));
            assert!(!lowerer.captured_binding_names.contains("uncaptured"));
        },
    );
}

#[test]
fn capture_membership_belongs_to_each_analyzed_source_owner() {
    for (present, absent) in [("first", "second"), ("second", "first")] {
        let source = format!(
            "function owner() {{ let {present} = 1; return function read() {{ return {present}; }}; }}"
        );
        with_lowerer(&source, |lowerer| {
            assert!(lowerer.captured_binding_names.contains(present));
            assert!(!lowerer.captured_binding_names.contains(absent));
        });
    }
}

#[test]
fn each_unknown_effect_still_invalidates_mutable_captures_and_preserves_local_constants() {
    for declaration in ["var", "let", "const"] {
        for captured in [false, true] {
            if declaration == "const" && !captured {
                continue;
            }
            let hook = if captured && declaration != "const" {
                // A fresh known assignment between the two effects must not
                // make a repeated invalidation look redundant.
                "function write() { value = 'changed'; } unknownHook(write); \
                 value = 3; unknownHook(write);"
            } else if captured {
                "function read() { return value; } unknownHook(read);"
            } else {
                "unknownHook(); unknownHook();"
            };
            let source = format!(
                "function owner() {{ {declaration} value = 1; {hook} return value + 1; }} owner();"
            );
            let parsed = parse(&source, ParseOptions::script()).expect("flow fixture should parse");
            let program = crate::lower(&parsed);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let owner = program
                .script
                .as_ref()
                .expect("script")
                .functions
                .iter()
                .find(|function| function.name == "owner")
                .expect("owner function");
            let result = owner
                .body
                .statements
                .iter()
                .find_map(|statement| match statement {
                    StatementIr::Return(value) => Some(value),
                    _ => None,
                })
                .expect("owner returns its original addition");
            if captured && declaration != "const" {
                assert!(
                    matches!(result.expr, ExprIr::CoerciveAdd { .. }),
                    "{source}: {result:?}"
                );
            } else {
                assert_eq!(
                    result.possible_kinds,
                    KindSet::from_kind(ValueKind::Number),
                    "uncaptured/immutable primitive remains Number: {source}: {result:?}"
                );
            }
        }
    }
}
