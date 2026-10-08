use lila_aot_wasm::WasmArtifact;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{
    DataKind, KnownCustom, Name, Operator, Parser, Payload, TypeRef, Validator, WasmFeatures,
};

fn named_function_body(bytes: &[u8], name: &str) -> Vec<u8> {
    let mut imported_functions = 0u32;
    let mut function_index = None;
    let mut bodies = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.expect("runtime module decodes") {
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    if matches!(
                        import.expect("runtime import decodes").ty,
                        TypeRef::Func(_) | TypeRef::FuncExact(_)
                    ) {
                        imported_functions += 1;
                    }
                }
            }
            Payload::CodeSectionEntry(body) => bodies.push(body.range()),
            Payload::CustomSection(section) => {
                if let KnownCustom::Name(subsections) = section.as_known() {
                    for subsection in subsections {
                        if let Name::Function(map) = subsection.expect("name subsection decodes") {
                            for naming in map {
                                let naming = naming.expect("function name decodes");
                                if naming.name == name {
                                    assert!(
                                        function_index.replace(naming.index).is_none(),
                                        "the shared helper must have exactly one named body"
                                    );
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let ordinal = function_index
        .expect("shared coercion helper is emitted in the runtime")
        .checked_sub(imported_functions)
        .expect("shared coercion helper is defined, not imported");
    let range = bodies
        .get(ordinal as usize)
        .expect("helper index identifies an actual code body");
    bytes[range.clone()].to_vec()
}

fn emit_literal(length: usize, marker: &str) -> (WasmArtifact, u32, Vec<u8>) {
    let source = format!(
        "const marker = '{marker}'; const text = '{}\\u0000\\uD800\\uDC00\\uD800'; text.length;",
        "x".repeat(length)
    );
    let parsed = parse(&source, ParseOptions::script()).expect("literal source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let artifact =
        lila_aot_wasm::emit(&program).expect("literal emits through the product compiler");
    let main_size = artifact
        .function_sizes
        .iter()
        .find(|body| body.name == "lila::main")
        .expect("actual main body")
        .body_bytes
        .bytes();
    let runtime = artifact.runtime().expect("literal program links a runtime");
    let helper = named_function_body(runtime.bytes(), "helper::value_to_primitive_number");
    (artifact, main_size, helper)
}

#[test]
fn literal_length_grows_passive_data_without_expanding_bootstrap_code() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // These source-only strings sort on opposite sides of every
            // ordinary builtin spelling. Neither may renumber builtin slots.
            let (short, short_main, short_helper) = emit_literal(1024, "\\u0000pool-before");
            let (long, long_main, long_helper) = emit_literal(16 * 1024, "\\u{10ffff}pool-after");
            assert_eq!(short_helper, long_helper,
                "unrelated source literals must preserve the shared coercion helper bytes");
            for artifact in [&short, &long] {
                let runtime = artifact.runtime().expect("literal program links a runtime");
                for bytes in [artifact.bytes.as_slice(), runtime.bytes().as_ref()] {
                    Validator::new_with_features(WasmFeatures::all())
                        .validate_all(bytes)
                        .expect("runtime and program GC literal data and their counts validate");
                }
                let mut passive_image = None;
                let mut initializations = 0;
                for payload in Parser::new(0).parse_all(&artifact.bytes) {
                    match payload.expect("module decodes") {
                        Payload::DataSection(reader) => {
                            let first = reader.into_iter().next().expect("literal data exists")
                                .expect("literal data decodes");
                            assert!(matches!(first.kind, DataKind::Passive));
                            passive_image = Some(first.data);
                        }
                        Payload::CodeSectionEntry(body) => {
                            for operator in body.get_operators_reader().expect("body opens") {
                                if let Operator::ArrayNewData { array_data_index, .. } =
                                    operator.expect("operator decodes")
                                {
                                    assert_eq!(array_data_index, 0);
                                    initializations += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                assert!(initializations > 0, "the literal image must initialize real GC arrays");
                let expected = [0u16, 0xd800, 0xdc00, 0xd800]
                    .into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>();
                assert!(passive_image.expect("passive literal image")
                    .windows(expected.len()).any(|units| units == expected),
                    "NUL, a surrogate pair and an isolated surrogate retain their exact UTF-16 units");
            }
            eprintln!("pooled literal bootstrap: {short_main} -> {long_main} bytes");
            assert!(long_main <= short_main + 128,
                "literal contents must grow data, not instructions: {short_main} -> {long_main}");
            assert!(long.bytes.len() > short.bytes.len() + 15 * 1024);
        })
        .expect("compiler worker starts")
        .join()
        .expect("compiler worker completes");
}
