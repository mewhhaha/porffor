use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{Parser, Payload, Validator, WasmFeatures};

fn artifact(source: &'static str) -> (Vec<u8>, String) {
    std::thread::Builder::new()
        .name("runtime-import-reachability".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let parsed = parse(source, ParseOptions::script()).expect("fixture parses");
            let program = lower(&parsed);
            let artifact = emit(&program).expect("fixture emits through ordinary AOT codegen");
            Validator::new_with_features(WasmFeatures::all())
                .validate_all(&artifact.bytes)
                .expect("runtime planning preserves valid Wasm indices");
            (artifact.bytes, artifact.debug_dump)
        })
        .expect("compiler worker spawns")
        .join()
        .expect("compiler worker does not panic")
}

fn imported_functions(bytes: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        if let Payload::ImportSection(section) = payload.expect("section decodes") {
            for import in section.into_imports() {
                let import = import.expect("import decodes");
                names.push(format!("{}.{}", import.module, import.name));
            }
        }
    }
    names
}

#[test]
fn runtime_free_ir_does_not_introduce_intl_or_clock_imports() {
    for source in [
        "",
        ";;",
        "262;",
        "true; null; 'text';",
        "-0;",
        "+7;",
        "~7;",
        "delete 1;",
        "!0;",
        "false || 'text';",
        "null ?? 7;",
        "!(null ?? 7);",
        "+(0 ?? 7);",
        "+(3 ?? 7);",
        "true ? 'yes' : 3;",
        "(1, 2, 3);",
        "{ 7; { 8; } }",
    ] {
        let (bytes, debug_dump) = artifact(source);
        let imports = imported_functions(&bytes);
        for forbidden in ["lila_host.intl_call", "lila_host.wall_clock_millis"] {
            assert!(
                !imports.iter().any(|name| name == forbidden),
                "{source}: {imports:?}"
            );
        }
        assert!(
            debug_dump.contains("runtime helper functions: 0"),
            "{source}\n{}",
            debug_dump
        );
        assert!(Parser::new(0).parse_all(&bytes).all(|payload| {
            !matches!(payload.expect("section decodes"), Payload::CustomSection(section)
                if section.name() == lila_intl::INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION)
        }));
    }
}

#[test]
fn observable_intrinsics_keep_callable_bodies_and_provider_imports() {
    for source in [
        "(1)['toLocaleString']('en-US');",
        "(1n)['toLocaleString']('en-US');",
        "Object.getPrototypeOf(1).toLocaleString.call(123, 'en-US');",
        "new Intl.NumberFormat('en-US').format(123);",
    ] {
        let (bytes, debug_dump) = artifact(source);
        assert!(
            imported_functions(&bytes)
                .iter()
                .any(|name| name == "lila_host.intl_call"),
            "{source}"
        );
        assert!(
            debug_dump.contains("Intl.NumberFormat Format Function"),
            "{source}"
        );
    }
}

#[test]
fn unproven_ir_retains_ordinary_realm_bootstrap() {
    for source in [
        // These expressions lower through coercive IR. Its generic emission
        // still requires runtime bodies even when the source operands are literals.
        "1 + 1;",
        "(7 - 2) * 3 / 2;",
        "5 % 2;",
        "void (1 + 2);",
        "1 < 2;",
        "+(null ?? 7);",
        "var answer = 262; answer;",
        "let answer = 262; answer;",
        "function answer() { return 262; } answer();",
        "({valueOf() { return 262; }}) + 1;",
        "this;",
        "try { throw 1; } catch (caught) { caught; }",
    ] {
        let (_, debug_dump) = artifact(source);
        assert!(debug_dump.contains("heap: enabled"), "{source}");
        assert!(
            !debug_dump.contains("runtime helper functions: 0"),
            "{source}"
        );
    }
}

#[test]
fn runtime_elision_keeps_the_experimental_wasm_gc_contract() {
    let (bytes, _) = artifact("262;");
    let mut features = WasmFeatures::all();
    features.set(WasmFeatures::GC, false);
    assert!(Validator::new_with_features(features)
        .validate_all(&bytes)
        .is_err());
}
