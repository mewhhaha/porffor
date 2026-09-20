use super::*;
use lila_ir::{ExprIr, StatementIr};
use std::collections::{BTreeMap, BTreeSet};

type ModuleArguments = (i64, i64, i64, i64, i64, i64, i64);
type ModuleResult = (i64, i64, i64, i64);

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
fn paused_before_entry_evaluation(bytes: &[u8]) -> Vec<u8> {
    let evaluate = *function_names(bytes)
        .iter()
        .find(|(_, name)| name.as_str() == "helper::module_evaluate")
        .unwrap()
        .0;
    let mut types = Vec::new();
    let mut function_types = Vec::new();
    let mut imported = 0;
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
                    if let wasmparser::TypeRef::Func(index)
                    | wasmparser::TypeRef::FuncExact(index) = import.unwrap().ty
                    {
                        function_types.push(index);
                        imported += 1;
                    }
                }
            }
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
    assert_eq!(main_type.results(), &[wasmparser::ValType::I64]);
    let evaluate_type = types[function_types[evaluate as usize] as usize].unwrap_func();
    assert_eq!(evaluate_type.params(), &[wasmparser::ValType::I64; 7]);
    assert_eq!(evaluate_type.results(), &[wasmparser::ValType::I64; 4]);

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
    while cursor < bytes.len() {
        let section_start = cursor;
        let id = bytes[cursor];
        cursor += 1;
        let length = read_uleb(bytes, &mut cursor) as usize;
        let end = cursor + length;
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
                // Discard the complete private-call ABI, return main's i64
                // result, and retain its remaining code as unreachable bytes.
                body.extend_from_slice(&[0x1a; 7]); // drop
                body.extend_from_slice(&[0x42, 0x00, 0x0f]); // i64.const 0; return
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
    rewritten
}

/// Export private operations and the single module root from a copied artifact.
/// Product artifacts never publish these names or private memory mutation APIs.
fn with_private_exports(bytes: &[u8]) -> Vec<u8> {
    let names = function_names(bytes);
    let mut module_root = None;
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let WasmPayload::GlobalSection(globals) = payload.unwrap() {
            for (index, global) in globals.into_iter().enumerate() {
                if global.unwrap().ty.content_type == wasmparser::ValType::I64 {
                    module_root = Some(index as u32);
                }
            }
        }
    }
    let mut exports = vec![("test_module_record", 3, module_root.unwrap())];
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

fn word(memory: &[u8], record: i64, offset: usize) -> i64 {
    let start = usize::try_from(record).unwrap() + offset;
    i64::from_le_bytes(memory[start..start + 8].try_into().unwrap())
}

struct ModuleStore {
    limits: WasmtimeStoreLimits,
    source_allowed: bool,
    reentrant_calls: usize,
    reentrant_promise: i64,
}
struct ModuleFixture {
    engine: WasmtimeEngine,
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
        let paused = paused_before_entry_evaluation(&original.bytes);
        let bytes = with_private_exports(&paused);
        let engine = shared_wasm_engine().unwrap();
        let module = WasmtimeModule::new(&engine, bytes).unwrap();
        Self { engine, module }
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
                reentrant_promise: 0,
            },
        );
        store.limiter(|state| &mut state.limits);
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = WasmtimeLinker::<ModuleStore>::new(&self.engine);
        for import in self.module.imports() {
            match import.ty() {
                WasmtimeExternType::Func(signature) => {
                    let is_print = import.name() == WASM_HOST_IMPORT_PRINT_LINE_UTF8;
                    linker.func_new(import.module(), import.name(), signature, move |mut caller, _, _| {
                        if !is_print || !caller.data().source_allowed {
                            return Err(wasmtime::Error::msg("module allocation/instantiation called source or host code"));
                        }
                        let root = caller.get_export("test_module_record").unwrap().into_global().unwrap();
                        let module = root.get(&mut caller).i64().unwrap();
                        let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                        assert_eq!(word(memory.data(&caller), module, 184), 1, "body is Executing before a reentrant call");
                        let promise = word(memory.data(&caller), module, 96);
                        assert_ne!(promise, 0, "Evaluate owns its capability before source entry");
                        let evaluate = caller.get_export("test_module_evaluate").unwrap().into_func().unwrap()
                            .typed::<ModuleArguments, ModuleResult>(&caller).unwrap();
                        let result = evaluate.call(&mut caller, (module, 0, 0, 0, 0, 0, 0))?;
                        assert_eq!(result.0, promise, "reentrant Evaluate returns the owned capability");
                        assert_eq!(result.2, 0);
                        caller.data_mut().reentrant_calls += 1;
                        caller.data_mut().reentrant_promise = promise;
                        Ok(())
                    }).unwrap();
                }
                WasmtimeExternType::Memory(memory_type) => {
                    assert!(!memory_type.is_shared());
                    let memory = WasmtimeMemory::new(&mut store, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                other => panic!("unexpected fixture import: {other:?}"),
            }
        }
        let instance = linker.instantiate(&mut store, &self.module).unwrap();
        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .unwrap();
        main.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let record = instance
            .get_global(&mut store, "test_module_record")
            .unwrap()
            .get(&mut store)
            .i64()
            .unwrap();
        let evaluate = instance
            .get_typed_func(&mut store, "test_module_evaluate")
            .unwrap();
        let execute = instance
            .get_typed_func(&mut store, "test_module_execute")
            .unwrap();
        let ready = instance
            .get_typed_func(&mut store, "test_module_ready")
            .unwrap();
        assert_ne!(record, 0);
        assert_eq!(
            word(
                memory.data(&store),
                word(memory.data(&store), record, 168),
                0
            ),
            1,
            "single-record fixture root"
        );
        PrivateModule {
            store,
            memory,
            record,
            evaluate,
            execute,
            ready,
        }
    }
}
struct PrivateModule {
    store: WasmtimeStore<ModuleStore>,
    memory: WasmtimeMemory,
    record: i64,
    evaluate: wasmtime::TypedFunc<ModuleArguments, ModuleResult>,
    execute: wasmtime::TypedFunc<ModuleArguments, ModuleResult>,
    ready: wasmtime::TypedFunc<ModuleArguments, ModuleResult>,
}
impl PrivateModule {
    fn read(&self, record: i64, offset: usize) -> i64 {
        word(self.memory.data(&self.store), record, offset)
    }
    fn write(&mut self, record: i64, offset: usize, value: i64) {
        self.memory
            .write(
                &mut self.store,
                record as usize + offset,
                &value.to_le_bytes(),
            )
            .unwrap();
    }
}

#[test]
fn private_module_instantiation_has_no_source_effect_or_await_job_and_keeps_body_promise_pending() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        let runtime = fixture.instantiate();
        let record = runtime.record;
        let activation = runtime.read(record, 0);
        assert_eq!(runtime.read(record, 80), 1); // Async owner.
        assert_eq!(runtime.read(record, 16), 0); // Linked, never Evaluating.
        assert_eq!(runtime.read(record, 176), 0); // No completion.
        assert_eq!(runtime.read(record, 184), 0); // Body not started.
        assert_eq!(runtime.read(record, 96), 0); // Evaluate not requested.
        assert_eq!(runtime.read(activation, 152), 2); // Execute mode after instantiation.
        assert_eq!(runtime.read(activation, 48), 1); // Private boundary only.
        assert_eq!(runtime.read(activation, 112), 0); // Body not completed.
        assert_eq!(runtime.read(activation, 88), 1); // One initialized invocation.
        assert_ne!(runtime.read(record, 72), 0);
        assert_eq!(runtime.read(record, 72), runtime.read(activation, 144));
        assert_ne!(runtime.read(activation, 80), 0);
        assert_eq!(runtime.read(runtime.read(activation, 104), 0), 0); // Intrinsic body Promise Pending.
        assert_eq!(runtime.store.data().reentrant_calls, 0);
    });
}

#[test]
fn invalid_private_module_modes_and_lifecycle_words_trap_before_source_entry() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        for (offset, value) in [(152, 99), (152, 1), (48, 0), (48, i64::MAX)] {
            let mut runtime = fixture.instantiate();
            let activation = runtime.read(runtime.record, 0);
            runtime.write(activation, offset, value);
            let result = runtime
                .execute
                .call(&mut runtime.store, (runtime.record, 0, 0, 0, 0, 0, 0));
            assert!(
                result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some(),
                "private mode/state corruption must be a Wasm trap"
            );
            assert_eq!(runtime.store.data().reentrant_calls, 0);
        }
        for (offset, execute) in [(16, false), (184, true), (80, true)] {
            let mut runtime = fixture.instantiate();
            runtime.write(runtime.record, offset, 99);
            let operation = if execute {
                &runtime.execute
            } else {
                &runtime.ready
            };
            let result = operation.call(&mut runtime.store, (runtime.record, 0, 0, 0, 0, 0, 0));
            assert!(result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some());
        }
        let mut runtime = fixture.instantiate();
        runtime.write(runtime.record, 16, 3);
        runtime.write(runtime.record, 176, 99);
        let result = runtime
            .evaluate
            .call(&mut runtime.store, (runtime.record, 0, 0, 0, 0, 0, 0));
        assert!(result.unwrap_err().downcast_ref::<WasmtimeTrap>().is_some());
    });
}

#[test]
fn reentrant_evaluate_while_an_async_body_is_executing_reuses_its_owned_capability() {
    run_on_sized_stack(|| {
        let fixture = ModuleFixture::new();
        let mut runtime = fixture.instantiate();
        runtime.store.data_mut().source_allowed = true;
        let result = runtime
            .evaluate
            .call(&mut runtime.store, (runtime.record, 0, 0, 0, 0, 0, 0))
            .unwrap();
        assert_eq!(result.2, 0);
        assert_eq!(runtime.store.data().reentrant_calls, 1);
        assert_eq!(runtime.store.data().reentrant_promise, result.0);
        assert_eq!(runtime.read(runtime.record, 184), 1); // Suspended body remains Executing.
        assert_eq!(runtime.read(runtime.record, 16), 2); // DFS closed as EvaluatingAsync.
        assert_eq!(runtime.read(runtime.record, 96), result.0);
        let duplicate = runtime
            .execute
            .call(&mut runtime.store, (runtime.record, 0, 0, 0, 0, 0, 0));
        assert!(duplicate
            .unwrap_err()
            .downcast_ref::<WasmtimeTrap>()
            .is_some());
        assert_eq!(runtime.store.data().reentrant_calls, 1);
    });
}

fn compiled(source: &str, module: bool) -> Vec<u8> {
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = if module {
        engine.compile_module(source, CompileOptions::default())
    } else {
        engine.compile_script(source, CompileOptions::default())
    }
    .unwrap();
    engine.emit_wasm(&unit).unwrap().bytes
}

#[test]
fn graphless_scripts_have_only_unreachable_module_slots_and_no_edges_to_them() {
    run_on_sized_stack(|| {
        let bytes = compiled("const values = [1]; values.length;", false);
        let names = function_names(&bytes);
        let helpers: BTreeSet<_> = names
            .iter()
            .filter_map(|(&index, name)| name.starts_with("helper::module_").then_some(index))
            .collect();
        assert_eq!(helpers.len(), 7);
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
            total_module_bytes, 21,
            "seven three-byte reserved bodies, including local declarations"
        );
        // Array intrinsic installation also roots Array.fromAsync and its
        // Promise dependencies. Graph isolation is the absence of module edges,
        // together with the seven minimal unreachable bodies asserted above.
    });
}

#[test]
fn synchronous_module_graph_roots_real_evaluation_bodies_and_private_generator_resume() {
    run_on_sized_stack(|| {
        let bytes = compiled("export const value = 1;", true);
        let names = function_names(&bytes);
        let helpers: BTreeSet<_> = names
            .iter()
            .filter_map(|(&index, name)| name.starts_with("helper::module_").then_some(index))
            .collect();
        assert_eq!(helpers.len(), 7);
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
        assert_eq!(sizes.len(), 7);
        assert!(
            sizes.iter().all(|&size| size > 3 && size < 1024 * 1024),
            "outlined module bodies have bounded per-function size: {sizes:?}"
        );
        assert!(
            names
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
