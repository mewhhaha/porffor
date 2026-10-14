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
                let mut descriptor_initializations = 0;
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
                                    match array_data_index {
                                        0 => initializations += 1,
                                        2 => descriptor_initializations += 1,
                                        _ => panic!("unexpected pooled literal data segment {array_data_index}"),
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                assert!(initializations > 0, "the literal image must initialize real GC arrays");
                assert_eq!(descriptor_initializations, 1, "one private program descriptor image");
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

#[test]
fn literal_count_grows_descriptor_data_without_expanding_main_initialization() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let emit = |count: usize| {
                // The function holds many source literals but is never invoked.
                // Main's own operations are fixed, isolating literal-root code
                // from the independently emitted body and array-literal code.
                let literals = (0..count)
                    .map(|index| format!("value = 'program_pool_{index:05}';"))
                    .collect::<String>();
                let source = format!("function strings(value) {{{literals}return value;}} strings;");
                let parsed = parse(&source, ParseOptions::script()).expect("pool source parses");
                let program = lower(&parsed);
                assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
                let artifact = lila_aot_wasm::emit(&program).expect("pooled string program emits");
                Validator::new_with_features(WasmFeatures::all())
                    .validate_all(&artifact.bytes).expect("program descriptor module validates");
                let body = named_function_body(&artifact.bytes, "lila::main");
                let parsed_body = wasmparser::FunctionBody::new(wasmparser::BinaryReader::new(&body, 0));
                let mut data_initializers = [0; 3];
                for operator in parsed_body.get_operators_reader().expect("main opens") {
                    if let Operator::ArrayNewData { array_data_index, .. } = operator.expect("main decodes") {
                        data_initializers[array_data_index as usize] += 1;
                    }
                }
                assert_eq!(data_initializers, [1, 0, 1],
                    "one dynamic string creation and one private descriptor table, independent of count");
                let segments = Parser::new(0).parse_all(&artifact.bytes)
                    .find_map(|payload| match payload.expect("program decodes") {
                        Payload::DataSection(reader) => Some(reader.into_iter()
                            .map(|row| row.expect("passive segment decodes").data.to_vec())
                            .collect::<Vec<_>>()),
                        _ => None,
                    }).expect("program passive images exist");
                assert!(segments.len() >= 3);
                assert_eq!(segments[2].len() % 12, 0);
                assert!(segments[2].len() >= count * 12,
                    "every unique source literal needs its descriptor row");
                (artifact, body.len(), segments[2].len())
            };
            let (small, small_main, small_descriptors) = emit(16);
            let (large, large_main, large_descriptors) = emit(2048);
            assert_eq!(small.runtime().unwrap().bytes(), large.runtime().unwrap().bytes(),
                "program string count must preserve the complete shared runtime");
            assert!(large_descriptors >= small_descriptors + (2048 - 16) * 12);
            assert!(large.bytes.len() > small.bytes.len() + 32 * 1024);
            eprintln!("pooled string count main body: {small_main} -> {large_main} bytes");
            assert!(large_main <= 4096, "actual main body is {large_main} bytes");
            assert!(large_main <= small_main + 64,
                "descriptor count grows data, not unrolled main code: {small_main} -> {large_main}");
        })
        .expect("bounded compiler worker starts")
        .join()
        .expect("pooled string descriptor emission completes");
}
