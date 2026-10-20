use super::linked_bodies::module_bodies;
use scriptw_front::{parse, ParseOptions};
use wasmparser::{Validator, WasmFeatures};
fn encoded(pairs: usize) -> (usize, usize, usize) {
    let elements = (0..pairs)
        .map(|index| format!("['paired',{index}]"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("function factory(){{return [{elements}];}} factory();");
    let parsed = parse(source, ParseOptions::script()).unwrap();
    let program = scriptw_ir::lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let artifact = scriptw_aot_wasm::emit(&program).unwrap();
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&artifact.bytes)
        .expect("actual compact Array program validates");
    let bodies = module_bodies(&artifact.bytes);
    let helper = bodies
        .iter()
        .filter(|body| body.name == "helper::compact_array_literal")
        .collect::<Vec<_>>();
    assert_eq!(helper.len(), 1, "one defined program recipe constructor");
    (
        helper[0].bytes(),
        bodies.iter().map(|body| body.bytes()).max().unwrap(),
        artifact.bytes.len(),
    )
}
#[test]
fn pure_nested_array_count_grows_passive_data_without_expanding_encoded_program_bodies() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let small = encoded(256);
            let large = encoded(4096);
            eprintln!(
                "compact Array helper/main maximum: {} / {} -> {} / {} bytes",
                small.0, small.1, large.0, large.1
            );
            assert!(
                large.2 > small.2 + 128 * 1024,
                "complete recipes remain in data"
            );
            assert!(large.0 <= 32 * 1024, "actual helper is {} bytes", large.0);
            assert_eq!(small.0, large.0, "one identical helper algorithm");
            assert!(
                large.1 <= 64 * 1024 && large.1 <= small.1 + 128,
                "actual source/helper maximum grew {} -> {}",
                small.1,
                large.1
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
