use super::linked_bodies::module_bodies;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{Validator, WasmFeatures};

fn candidate_body(literals: usize) -> (usize, usize) {
    let values = (0..literals)
        .map(|index| format!("'candidate_{index:04}'"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "var values = [{values}]; new RegExp(values[values.length - 1]).test('candidate');"
    );
    let parsed = parse(&source, ParseOptions::script()).expect("candidate source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let artifact = lila_aot_wasm::emit(&program).expect("finite candidate program emits");
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&artifact.bytes)
        .expect("the actual compact candidate module validates");
    let bodies = module_bodies(&artifact.bytes);
    let matches = bodies
        .iter()
        .filter(|body| body.name == "helper::regexp_program_candidate")
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "one defined program-owned candidate hook");
    (matches[0].bytes(), artifact.bytes.len())
}

#[test]
fn finite_candidate_count_grows_data_without_expanding_the_emitted_lookup() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let (small_body, small_module) = candidate_body(16);
            let (large_body, large_module) = candidate_body(512);
            eprintln!("RegExp candidate body: {small_body} -> {large_body} bytes");
            assert!(large_module > small_module + 16 * 1024);
            assert!(
                large_body <= 4096,
                "actual helper body is {large_body} bytes"
            );
            assert!(
                large_body <= small_body + 32,
                "finite candidates must grow data: {small_body} -> {large_body} code bytes"
            );
        })
        .expect("bounded compiler worker starts")
        .join()
        .expect("finite candidate emission completes");
}
