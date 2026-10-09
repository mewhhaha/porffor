use super::*;
use lila_ir::{ExprIr, StatementIr};
use std::collections::{BTreeMap, BTreeSet};

use wasmtime::{AsContextMut, OwnedRooted, Rooted, StructRef};

fn append_uleb(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        bytes.push(low | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            break;
        }
    }
}
fn read_uleb(bytes: &[u8], cursor: &mut usize) -> u32 {
    let mut value = 0;
    for shift in (0..35).step_by(7) {
        let byte = bytes[*cursor];
        *cursor += 1;
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
    }
    panic!("invalid test artifact length")
}

fn function_names(bytes: &[u8]) -> BTreeMap<u32, String> {
    let mut names = BTreeMap::new();
    for payload in WasmParser::new(0).parse_all(bytes) {
        let WasmPayload::CustomSection(section) = payload.unwrap() else {
            continue;
        };
        let wasmparser::KnownCustom::Name(subsections) = section.as_known() else {
            continue;
        };
        for subsection in subsections {
            if let wasmparser::Name::Function(functions) = subsection.unwrap() {
                for function in functions {
                    let function = function.unwrap();
                    names.insert(function.index, function.name.to_owned());
                }
            }
        }
    }
    names
}

/// Stop a copied main immediately before its one entry Evaluate call. The
/// original trusted IR and emitted artifact retain the mandatory entry owner;
/// private tests can inspect allocation and instantiation without executing source.
fn paused_before_entry_evaluation(bytes: &[u8]) -> (Vec<u8>, u32) {
    let evaluate = *function_names(bytes)
        .iter()
        .find(|(_, name)| name.as_str() == "helper::module_evaluate")
        .unwrap()
        .0;
    let mut types = Vec::new();
    let mut function_types = Vec::new();
    let mut imported = 0;
    let mut global_count = 0;
    let mut main = None;
    for payload in WasmParser::new(0).parse_all(bytes) {
        match payload.unwrap() {
            WasmPayload::TypeSection(groups) => {
                for group in groups {
                    types.extend(group.unwrap().into_types());
                }
            }
            WasmPayload::ImportSection(imports) => {
                for import in imports.into_imports() {
                    match import.unwrap().ty {
                        wasmparser::TypeRef::Func(index)
                        | wasmparser::TypeRef::FuncExact(index) => {
                            function_types.push(index);
                            imported += 1;
                        }
                        wasmparser::TypeRef::Global(_) => global_count += 1,
                        _ => {}
                    }
                }
            }
            WasmPayload::GlobalSection(globals) => global_count += globals.count(),
            WasmPayload::FunctionSection(functions) => {
                function_types.extend(functions.into_iter().map(Result::unwrap));
            }
            WasmPayload::ExportSection(exports) => {
                for export in exports {
                    let export = export.unwrap();
                    if export.name == "main" {
                        assert_eq!(export.kind, wasmparser::ExternalKind::Func);
                        assert!(main.replace(export.index).is_none());
                    }
                }
            }
            _ => {}
        }
    }
    let main = main.unwrap();
    let main_type = types[function_types[main as usize] as usize].unwrap_func();
    assert!(main_type.params().is_empty());
    assert_eq!(
        main_type.results(),
        &[
            wasmparser::ValType::I32,
            wasmparser::ValType::I64,
            wasmparser::ValType::Ref(wasmparser::RefType::EQREF),
            wasmparser::ValType::I32,
            wasmparser::ValType::I32,
        ]
    );
    let evaluate_type = types[function_types[evaluate as usize] as usize].unwrap_func();
    assert!(
        matches!(evaluate_type.params(), [wasmparser::ValType::Ref(reference)]
        if !reference.is_nullable()),
        "one typed ModuleRecord input"
    );
    assert_eq!(evaluate_type.results(), main_type.results());

    let mut defined = 0;
    let mut main_body = None;
    let mut calls = Vec::new();
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let WasmPayload::CodeSectionEntry(body) = payload.unwrap() {
            let index = imported + defined;
            defined += 1;
            if index != main {
                continue;
            }
            main_body = Some(body.range());
            let mut operators = body.get_operators_reader().unwrap();
            while !operators.eof() {
                let start = operators.original_position();
                let operator = operators.read().unwrap();
                if matches!(operator, wasmparser::Operator::Call { function_index } if function_index == evaluate)
                {
                    calls.push(start..operators.original_position());
                }
            }
        }
    }
    assert_eq!(calls.len(), 1, "replace exactly the entry Evaluate call");
    let call = calls.pop().unwrap();
    let main_body = main_body.unwrap();
    assert!(main_body.start <= call.start && call.end <= main_body.end);

    let mut rewritten = bytes[..8].to_vec();
    let mut cursor = 8;
    let mut replacements = 0;
    let mut added_global = false;
    // Test-only (mut eqref), initialized null. The intercepted typed input is
    // its only writer; the product publishes no private ModuleRecord export.
    let root_global = [0x6d, 0x01, 0xd0, 0x6d, 0x0b];
    while cursor < bytes.len() {
        let section_start = cursor;
        let id = bytes[cursor];
        cursor += 1;
        let length = read_uleb(bytes, &mut cursor) as usize;
        let end = cursor + length;
        if id == 6 {
            let count = read_uleb(bytes, &mut cursor);
            let mut globals = Vec::new();
            append_uleb(&mut globals, count + 1);
            globals.extend_from_slice(&bytes[cursor..end]);
            globals.extend_from_slice(&root_global);
            rewritten.push(6);
            append_uleb(&mut rewritten, globals.len().try_into().unwrap());
            rewritten.extend_from_slice(&globals);
            added_global = true;
            cursor = end;
            continue;
        }
        if id == 7 && !added_global {
            rewritten.extend_from_slice(&[6, 6, 1]);
            rewritten.extend_from_slice(&root_global);
            added_global = true;
        }
        if id != 10 {
            rewritten.extend_from_slice(&bytes[section_start..end]);
            cursor = end;
            continue;
        }
        let count = read_uleb(bytes, &mut cursor);
        let mut code = Vec::new();
        append_uleb(&mut code, count);
        for ordinal in 0..count {
            let body_length_start = cursor;
            let body_length = read_uleb(bytes, &mut cursor) as usize;
            let body_end = cursor + body_length;
            if imported + ordinal == main {
                assert_eq!(cursor..body_end, main_body);
                let mut body = bytes[cursor..call.start].to_vec();
                // Retain the actual typed record, then return a normal
                // Undefined Completion without entering source or draining jobs.
                body.push(0x24); // global.set
                append_uleb(&mut body, global_count);
                body.extend_from_slice(&[0x41, 0, 0x42, 0, 0xd0, 0x6d, 0x41, 0, 0x41, 0, 0x0f]);
                body.extend_from_slice(&bytes[call.end..body_end]);
                append_uleb(&mut code, body.len().try_into().unwrap());
                code.extend_from_slice(&body);
                replacements += 1;
            } else {
                code.extend_from_slice(&bytes[body_length_start..body_end]);
            }
            cursor = body_end;
        }
        assert_eq!(cursor, end);
        rewritten.push(id);
        append_uleb(&mut rewritten, code.len().try_into().unwrap());
        rewritten.extend_from_slice(&code);
    }
    assert_eq!(replacements, 1);
    assert!(added_global);
    (rewritten, global_count)
}

/// Export private operations and the single module root from a copied artifact.
/// Product artifacts never publish these names or private memory mutation APIs.
fn with_private_exports(bytes: &[u8], module_root: u32) -> Vec<u8> {
    let names = function_names(bytes);
    let mut exports = vec![("test_module_record", 3, module_root)];
    for (export, helper) in [
        ("test_module_evaluate", "helper::module_evaluate"),
        ("test_module_execute", "helper::module_execute"),
        ("test_module_ready", "helper::module_ready"),
    ] {
        let index = *names
            .iter()
            .find(|(_, name)| name.as_str() == helper)
            .unwrap()
            .0;
        exports.push((export, 0, index));
    }
    let mut rewritten = bytes[..8].to_vec();
    let mut cursor = 8;
    let mut found = false;
    while cursor < bytes.len() {
        let id = bytes[cursor];
        cursor += 1;
        let length = read_uleb(bytes, &mut cursor) as usize;
        let end = cursor + length;
        if id == 7 {
            found = true;
            let count = read_uleb(bytes, &mut cursor);
            let mut body = Vec::new();
            append_uleb(&mut body, count + exports.len() as u32);
            body.extend_from_slice(&bytes[cursor..end]);
            for &(name, kind, index) in &exports {
                append_uleb(&mut body, name.len() as u32);
                body.extend_from_slice(name.as_bytes());
                body.push(kind);
                append_uleb(&mut body, index);
            }
            rewritten.push(id);
            append_uleb(&mut rewritten, body.len() as u32);
            rewritten.extend_from_slice(&body);
        } else {
            rewritten.push(id);
            append_uleb(&mut rewritten, length as u32);
            rewritten.extend_from_slice(&bytes[cursor..end]);
        }
        cursor = end;
    }
    assert!(found);
    rewritten
}

// Ordinals of the actual typed GC schemas. Reads/writes use Wasmtime's field
// APIs, so a field-kind mismatch fails rather than indexing unrelated bytes.
mod record_field {
    pub const GRAPH: usize = 0;
    pub const ENVIRONMENT: usize = 3;
    pub const ACTIVATION_KIND: usize = 6;
    pub const ASYNC_ACTIVATION: usize = 8;
    pub const EVALUATION_PROMISE: usize = 11;
    pub const STATE: usize = 12;
    pub const COMPLETION: usize = 13;
    pub const BODY_STATE: usize = 14;
}
mod activation_field {
    pub const FRAME: usize = 0;
    pub const PROMISE: usize = 3;
    pub const RESUME_POINT: usize = 4;
    pub const COMPLETED: usize = 5;
    pub const MODULE_ENTRY_MODE: usize = 7;
}

fn structure(store: impl AsContext, value: WasmtimeVal) -> Rooted<StructRef> {
    let WasmtimeVal::AnyRef(Some(value)) = value else {
        panic!("non-null GC record")
    };
    value.as_struct(store).unwrap().expect("GC struct")
}
fn reference_field(
    store: &mut impl AsContextMut,
    record: Rooted<StructRef>,
    index: usize,
) -> Rooted<StructRef> {
    let value = record.field(&mut *store, index).unwrap();
    structure(store, value)
}
fn scalar_field(store: &mut impl AsContextMut, record: Rooted<StructRef>, index: usize) -> i32 {
    record
        .field(store, index)
        .unwrap()
        .i32()
        .expect("typed scalar field")
}
fn null_field(store: &mut impl AsContextMut, record: Rooted<StructRef>, index: usize) -> bool {
    matches!(
        record.field(store, index).unwrap(),
        WasmtimeVal::AnyRef(None)
    )
}
fn same_record(store: impl AsContext, left: Rooted<StructRef>, right: Rooted<StructRef>) {
    assert!(Rooted::ref_eq(store, &left, &right).unwrap());
}
fn call_completion(
    store: &mut impl AsContextMut,
    operation: wasmtime::Func,
    record: Rooted<StructRef>,
) -> wasmtime::Result<[WasmtimeVal; 5]> {
    let mut result = [
        WasmtimeVal::I32(0),
        WasmtimeVal::I64(0),
        WasmtimeVal::AnyRef(None),
        WasmtimeVal::I32(0),
        WasmtimeVal::I32(0),
    ];
    operation.call(store, &[WasmtimeVal::from(record)], &mut result)?;
    Ok(result)
}

struct ModuleStore {
    limits: WasmtimeStoreLimits,
    source_allowed: bool,
    reentrant_calls: usize,
    record: Option<Rooted<StructRef>>,
    evaluate: Option<wasmtime::Func>,
    reentrant_promise: Option<OwnedRooted<StructRef>>,
}
struct ModuleFixture {
    engine: WasmtimeEngine,
    runtime: WasmtimeModule,
    module: WasmtimeModule,
}
impl ModuleFixture {
    fn new() -> Self {
        configure_compilation_jobs(1).unwrap();
        let compiler = Engine::new(RealmBuilder::new().build());
        let unit = compiler
            .compile_module(
                "print('body'); await 0; print('resumed');",
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
            )
            .unwrap();
        assert!(unit
            .ir
            .script
            .as_ref()
            .unwrap()
            .module_entry_evaluation()
            .is_some());
        let original = compiler.emit_wasm(&unit).unwrap();
        let (paused, root) = paused_before_entry_evaluation(&original.bytes);
        let bytes = with_private_exports(&paused, root);
        let engine = shared_wasm_engine().unwrap();
        let runtime = WasmtimeModule::new(
            &engine,
            original.runtime.as_ref().expect("linked runtime").0.bytes(),
        )
        .unwrap();
        let module = WasmtimeModule::new(&engine, bytes).unwrap();
        Self {
            engine,
            runtime,
            module,
        }
    }
    fn instantiate(&self) -> PrivateModule {
        let mut store = WasmtimeStore::new(
            &self.engine,
            ModuleStore {
                limits: WasmtimeStoreLimitsBuilder::new()
                    .memory_size(64 * 1024 * 1024)
                    .build(),
                source_allowed: false,
                reentrant_calls: 0,
                record: None,
                evaluate: None,
                reentrant_promise: None,
            },
        );
        store.limiter(|state| &mut state.limits);
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = WasmtimeLinker::<ModuleStore>::new(&self.engine);
        for import in self.runtime.imports() {
            match import.ty() {
                WasmtimeExternType::Func(signature) => {
                    let is_print = import.name() == WASM_HOST_IMPORT_PRINT_LINE_UTF8;
                    linker.func_new(import.module(), import.name(), signature, move |mut caller, _, _| {
                        if !is_print || !caller.data().source_allowed {
                            return Err(wasmtime::Error::msg("module allocation/instantiation called source or host code"));
                        }
                        // This import belongs to R, so P's private handles are
                        // held in Store data instead of Caller::get_export.
                        let module = caller.data().record.expect("initialized program record");
                        assert_eq!(scalar_field(&mut caller, module, record_field::BODY_STATE), 1,
                            "body is Executing before a reentrant call");
                        let promise = reference_field(&mut caller, module, record_field::EVALUATION_PROMISE);
                        let evaluate = caller.data().evaluate.unwrap();
                        let result = call_completion(&mut caller, evaluate, module)?;
                        assert_eq!(result[3].i32(), Some(0));
                        let returned = structure(&caller, result[2]);
                        same_record(&caller, promise, returned);
                        caller.data_mut().reentrant_calls += 1;
                        let retained = promise.to_owned_rooted(&mut caller)?;
                        caller.data_mut().reentrant_promise = Some(retained);
                        Ok(())
                    }).unwrap();
                }
                WasmtimeExternType::Memory(memory_type) if memory_type.is_shared() => {
                    let memory = WasmtimeSharedMemory::new(&self.engine, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                WasmtimeExternType::Memory(memory_type) => {
                    let memory = WasmtimeMemory::new(&mut store, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                other => panic!("unexpected fixture import: {other:?}"),
            }
        }
        let runtime = linker.instantiate(&mut store, &self.runtime).unwrap();
        linker
            .instance(&mut store, lila_aot_wasm::RUNTIME_IMPORT_NAMESPACE, runtime)
            .unwrap();
        let instance = linker.instantiate(&mut store, &self.module).unwrap();
        let main = instance.get_func(&mut store, "main").unwrap();
        let mut completion = [
            WasmtimeVal::I32(0),
            WasmtimeVal::I64(0),
            WasmtimeVal::AnyRef(None),
            WasmtimeVal::I32(0),
            WasmtimeVal::I32(0),
        ];
        main.call(&mut store, &[], &mut completion).unwrap();
        assert_eq!(completion[3].i32(), Some(0), "paused main returns Normal");
        let root = instance
            .get_global(&mut store, "test_module_record")
            .unwrap()
            .get(&mut store);
        let record = structure(&store, root);
        assert_eq!(
            record.ty(&store).unwrap().fields().len(),
            22,
            "ModuleRecord schema"
        );
        let evaluate = instance
            .get_func(&mut store, "test_module_evaluate")
            .unwrap();
        let execute = instance
            .get_func(&mut store, "test_module_execute")
            .unwrap();
        let ready = instance.get_func(&mut store, "test_module_ready").unwrap();
        let graph = reference_field(&mut store, record, record_field::GRAPH);
        let WasmtimeVal::AnyRef(Some(modules)) = graph.field(&mut store, 0).unwrap() else {
            panic!("ModuleGraph owns its registry")
        };
        let modules = modules.as_array(&store).unwrap().unwrap();
        assert_eq!(
            modules.len(&store).unwrap(),
            1,
            "single-record fixture root"
        );
        let member = modules.get(&mut store, 0).unwrap();
        same_record(&store, record, structure(&store, member));
        store.data_mut().record = Some(record);
        store.data_mut().evaluate = Some(evaluate);
        PrivateModule {
            store,
            record,
            evaluate,
            execute,
            ready,
        }
    }
}
struct PrivateModule {
    store: WasmtimeStore<ModuleStore>,
    record: Rooted<StructRef>,
    evaluate: wasmtime::Func,
    execute: wasmtime::Func,
    ready: wasmtime::Func,
}
impl PrivateModule {
    fn write(&mut self, record: Rooted<StructRef>, index: usize, value: i32) {
        record
            .set_field(&mut self.store, index, WasmtimeVal::I32(value))
            .unwrap();
    }
}

#[test]
fn private_module_instantiation_has_no_source_effect_or_await_job_and_keeps_body_promise_pending() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        let mut runtime = fixture.instantiate();
        let record = runtime.record;
        let activation =
            reference_field(&mut runtime.store, record, record_field::ASYNC_ACTIVATION);
        assert_eq!(activation.ty(&runtime.store).unwrap().fields().len(), 8);
        for (field, expected) in [
            (record_field::ACTIVATION_KIND, 1),
            (record_field::STATE, 0),
            (record_field::COMPLETION, 0),
            (record_field::BODY_STATE, 0),
        ] {
            assert_eq!(scalar_field(&mut runtime.store, record, field), expected);
        }
        assert!(null_field(
            &mut runtime.store,
            record,
            record_field::EVALUATION_PROMISE
        ));
        for (field, expected) in [
            (activation_field::MODULE_ENTRY_MODE, 2),
            (activation_field::RESUME_POINT, 1),
            (activation_field::COMPLETED, 0),
        ] {
            assert_eq!(
                scalar_field(&mut runtime.store, activation, field),
                expected
            );
        }
        let frame = reference_field(&mut runtime.store, activation, activation_field::FRAME);
        assert_eq!(
            scalar_field(&mut runtime.store, frame, 13),
            1,
            "InvocationFrame initialized"
        );
        let environment = reference_field(&mut runtime.store, record, record_field::ENVIRONMENT);
        let invocation_environment = reference_field(&mut runtime.store, frame, 3);
        same_record(&runtime.store, environment, invocation_environment);
        reference_field(&mut runtime.store, frame, 1); // Live lexical execution environment.
        let promise = reference_field(&mut runtime.store, activation, activation_field::PROMISE);
        assert_eq!(
            scalar_field(&mut runtime.store, promise, 4),
            0,
            "body Promise Pending"
        );
        assert_eq!(runtime.store.data().reentrant_calls, 0);
    });
}

#[test]
fn invalid_private_module_modes_and_lifecycle_words_trap_before_source_entry() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        for (field, value) in [
            (activation_field::MODULE_ENTRY_MODE, 99),
            (activation_field::MODULE_ENTRY_MODE, 1),
            (activation_field::RESUME_POINT, 0),
            (activation_field::RESUME_POINT, i32::MAX),
        ] {
            let mut runtime = fixture.instantiate();
            let activation = reference_field(
                &mut runtime.store,
                runtime.record,
                record_field::ASYNC_ACTIVATION,
            );
            runtime.write(activation, field, value);
            let result = call_completion(&mut runtime.store, runtime.execute, runtime.record);
            assert!(
                result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some(),
                "private mode/state corruption must be a Wasm trap"
            );
            assert_eq!(runtime.store.data().reentrant_calls, 0);
        }
        for (field, execute) in [
            (record_field::STATE, false),
            (record_field::BODY_STATE, true),
            (record_field::ACTIVATION_KIND, true),
        ] {
            let mut runtime = fixture.instantiate();
            runtime.write(runtime.record, field, 99);
            let result = if execute {
                call_completion(&mut runtime.store, runtime.execute, runtime.record).map(|_| ())
            } else {
                runtime.ready.call(
                    &mut runtime.store,
                    &[WasmtimeVal::from(runtime.record)],
                    &mut [WasmtimeVal::I32(0)],
                )
            };
            assert!(result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some());
            assert_eq!(runtime.store.data().reentrant_calls, 0);
        }
        let mut runtime = fixture.instantiate();
        runtime.write(runtime.record, record_field::STATE, 3);
        runtime.write(runtime.record, record_field::COMPLETION, 99);
        let result = call_completion(&mut runtime.store, runtime.evaluate, runtime.record);
        assert!(result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some());
        assert_eq!(runtime.store.data().reentrant_calls, 0);
    });
}

#[test]
fn reentrant_evaluate_while_an_async_body_is_executing_reuses_its_owned_capability() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        let mut runtime = fixture.instantiate();
        runtime.store.data_mut().source_allowed = true;
        let result = call_completion(&mut runtime.store, runtime.evaluate, runtime.record).unwrap();
        assert_eq!(result[3].i32(), Some(0));
        let promise = structure(&runtime.store, result[2]);
        assert_eq!(runtime.store.data().reentrant_calls, 1);
        let observed = runtime.store.data_mut().reentrant_promise.take().unwrap();
        let observed = observed.to_rooted(&mut runtime.store);
        same_record(&runtime.store, observed, promise);
        assert_eq!(
            scalar_field(&mut runtime.store, runtime.record, record_field::BODY_STATE),
            1,
            "suspended body remains Executing"
        );
        assert_eq!(
            scalar_field(&mut runtime.store, runtime.record, record_field::STATE),
            2,
            "DFS closed as EvaluatingAsync"
        );
        let retained = reference_field(
            &mut runtime.store,
            runtime.record,
            record_field::EVALUATION_PROMISE,
        );
        same_record(&runtime.store, retained, promise);
        let duplicate = call_completion(&mut runtime.store, runtime.execute, runtime.record);
        assert!(duplicate
            .unwrap_err()
            .downcast_ref::<WasmtimeTrap>()
            .is_some());
        assert_eq!(runtime.store.data().reentrant_calls, 1);
    });
}

fn compiled(source: &str, module: bool) -> Artifact {
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = if module {
        engine.compile_module(source, CompileOptions::default())
    } else {
        engine.compile_script(source, CompileOptions::default())
    }
    .unwrap();
    engine.emit_wasm(&unit).unwrap()
}

#[test]
fn graphless_scripts_have_only_unreachable_module_slots_and_no_edges_to_them() {
    run_on_sized_stack(|| {
        for source in [
            "const values = [1]; values.length;",
            "const realm = new ShadowRealm(); realm.evaluate('1');",
        ] {
            let artifact = compiled(source, false);
            let bytes = &artifact.bytes;
            let names = function_names(bytes);
            let helpers: BTreeSet<_> = names
                .iter()
                .filter_map(|(&index, name)| name.starts_with("helper::module_").then_some(index))
                .collect();
            assert_eq!(
                helpers
                    .iter()
                    .map(|index| names[index].as_str())
                    .collect::<BTreeSet<_>>(),
                [
                    "helper::module_initialize",
                    "helper::module_evaluate",
                    "helper::module_ready",
                    "helper::module_gather",
                    "helper::module_execute",
                    "helper::module_fulfilled",
                    "helper::module_rejected",
                    "helper::module_deferred_import",
                    "helper::module_body_reaction"
                ]
                .into_iter()
                .collect()
            );
            let mut imported = 0;
            let mut defined = 0;
            let mut total_module_bytes = 0;
            for payload in WasmParser::new(0).parse_all(&bytes) {
                match payload.unwrap() {
                    WasmPayload::ImportSection(imports) => {
                        for import in imports.into_imports() {
                            if matches!(
                                import.unwrap().ty,
                                wasmparser::TypeRef::Func(_) | wasmparser::TypeRef::FuncExact(_)
                            ) {
                                imported += 1;
                            }
                        }
                    }
                    WasmPayload::CodeSectionEntry(body) => {
                        let index = imported + defined;
                        defined += 1;
                        let operators = body
                            .get_operators_reader()
                            .unwrap()
                            .into_iter()
                            .map(Result::unwrap)
                            .collect::<Vec<_>>();
                        if helpers.contains(&index) {
                            total_module_bytes += body.range().len();
                            assert!(matches!(
                                &operators[..],
                                [wasmparser::Operator::Unreachable, wasmparser::Operator::End]
                            ));
                        }
                        for operator in operators {
                            if let wasmparser::Operator::Call { function_index }
                            | wasmparser::Operator::RefFunc { function_index } = operator
                            {
                                assert!(
                                    !helpers.contains(&function_index),
                                    "graphless artifact has a caller edge to a private module slot"
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            assert_eq!(
                total_module_bytes,
                helpers.len() * 3,
                "one three-byte reserved body per exact program module role"
            );
            // Array intrinsic installation also roots Array.fromAsync and its
            // Promise dependencies. Graph isolation is the absence of module edges,
            // together with the nine minimal unreachable bodies asserted above.
        }
    });
}

#[test]
fn synchronous_module_graph_roots_real_evaluation_bodies_and_private_generator_resume() {
    run_on_sized_stack(|| {
        let artifact = compiled("export const value = 1;", true);
        let bytes = &artifact.bytes;
        let names = function_names(bytes);
        let helpers: BTreeSet<_> = names
            .iter()
            .filter_map(|(&index, name)| name.starts_with("helper::module_").then_some(index))
            .collect();
        assert_eq!(
            helpers
                .iter()
                .map(|index| names[index].as_str())
                .collect::<BTreeSet<_>>(),
            [
                "helper::module_initialize",
                "helper::module_evaluate",
                "helper::module_ready",
                "helper::module_gather",
                "helper::module_execute",
                "helper::module_fulfilled",
                "helper::module_rejected",
                "helper::module_deferred_import",
                "helper::module_body_reaction"
            ]
            .into_iter()
            .collect()
        );
        let mut imported = 0;
        let mut defined = 0;
        let mut sizes = Vec::new();
        for payload in WasmParser::new(0).parse_all(&bytes) {
            match payload.unwrap() {
                WasmPayload::ImportSection(imports) => {
                    for import in imports.into_imports() {
                        if matches!(
                            import.unwrap().ty,
                            wasmparser::TypeRef::Func(_) | wasmparser::TypeRef::FuncExact(_)
                        ) {
                            imported += 1;
                        }
                    }
                }
                WasmPayload::CodeSectionEntry(body) => {
                    let index = imported + defined;
                    defined += 1;
                    if helpers.contains(&index) {
                        sizes.push(body.range().len());
                    }
                }
                _ => {}
            }
        }
        assert_eq!(sizes.len(), helpers.len());
        assert!(
            sizes.iter().all(|&size| size > 3 && size < 1024 * 1024),
            "outlined module bodies have bounded per-function size: {sizes:?}"
        );
        assert!(
            function_names(artifact.runtime.as_ref().expect("linked runtime").0.bytes())
                .values()
                .any(|name| name == "builtin::Generator.prototype.next"),
            "private synchronous body resume must have an emitted intrinsic body: {names:?}"
        );
    });
}

#[test]
fn trusted_module_operations_without_their_graph_are_rejected_before_emission() {
    run_on_sized_stack(|| {
        let engine = Engine::new(RealmBuilder::new().build());
        let mut unit = engine
            .compile_module("await 0;", CompileOptions::default())
            .unwrap();
        unit.ir.script.as_mut().unwrap().body.statements.retain(|statement| !matches!(statement,
            StatementIr::Expression(expression) if matches!(&expression.expr, ExprIr::ModuleExecutionGraph(_))));
        let error = engine.emit_wasm(&unit).unwrap_err();
        assert!(
            error.to_string().contains("validated execution graph"),
            "{error}"
        );
    });
}
