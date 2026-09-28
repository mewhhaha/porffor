//! Validate the emitted guard graph against the compiled module, then expose a
//! leaf stack-budget import. No JavaScript executes on the native callback stack.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use wasmparser::{FunctionBody, Operator, Parser, Payload, TypeRef};
use wasmtime::{Caller, DefinedFuncIndex, Linker, Module};

use super::{EngineError, WASM_HOST_IMPORT_NAMESPACE};

pub(super) const IMPORT_NAME: &str = "stack_guard_can_enter";
const MAGIC: &[u8; 8] = b"LILASTK\0";
const VERSION: u32 = 2;

fn invalid(reason: impl std::fmt::Display) -> EngineError {
    EngineError::new(format!("invalid Wasm stack-guard contract: {reason}"))
}

struct Metadata {
    original_count: u32,
    startup_body: u32,
    // Original wrapper -> relocated body, both in defined-function space.
    targets: BTreeMap<u32, u32>,
    helpers: BTreeSet<u32>,
}

impl Metadata {
    fn decode(mut bytes: &[u8]) -> Result<Self, EngineError> {
        if !bytes.starts_with(MAGIC) {
            return Err(invalid("missing metadata signature"));
        }
        bytes = &bytes[MAGIC.len()..];
        fn word(bytes: &mut &[u8]) -> Result<u32, EngineError> {
            let (value, rest) = bytes
                .split_at_checked(4)
                .ok_or_else(|| invalid("truncated metadata"))?;
            *bytes = rest;
            Ok(u32::from_le_bytes(
                value.try_into().expect("four-byte word"),
            ))
        }
        if word(&mut bytes)? != VERSION {
            return Err(invalid("unsupported metadata version"));
        }
        let original_count = word(&mut bytes)?;
        let startup_body = word(&mut bytes)?;
        let count = word(&mut bytes)?;
        if original_count.checked_add(count) != Some(startup_body) {
            return Err(invalid("inconsistent startup relocation"));
        }
        if count as usize > bytes.len() / 8 {
            return Err(invalid("truncated wrapper table"));
        }
        let mut targets = BTreeMap::new();
        for ordinal in 0..count {
            let wrapper = word(&mut bytes)?;
            let body = word(&mut bytes)?;
            if wrapper == 0
                || wrapper >= original_count
                || original_count.checked_add(ordinal) != Some(body)
                || targets.insert(wrapper, body).is_some()
            {
                return Err(invalid("inconsistent wrapper relocation"));
            }
        }
        let helper_count = word(&mut bytes)?;
        if helper_count as usize > bytes.len() / 4 {
            return Err(invalid("truncated helper table"));
        }
        let mut helpers = BTreeSet::new();
        for _ in 0..helper_count {
            let helper = word(&mut bytes)?;
            if helper == 0
                || helper >= original_count
                || targets.contains_key(&helper)
                || !helpers.insert(helper)
            {
                return Err(invalid("inconsistent unguarded helper table"));
            }
        }
        if !bytes.is_empty() || targets.is_empty() || helpers.is_empty() {
            return Err(invalid("empty guard plan or trailing metadata"));
        }
        if targets.len() + helpers.len() + 1 != original_count as usize {
            return Err(invalid(
                "original functions must partition into main, wrappers, and helpers",
            ));
        }
        Ok(Self {
            original_count,
            startup_body,
            targets,
            helpers,
        })
    }
}

fn frame_bound(module: &Module, index: u32) -> Result<u64, EngineError> {
    module
        .wasm_function_stack_layout(DefinedFuncIndex::from_u32(index))
        .map(|layout| layout.stack_size_bound_bytes())
        .ok_or_else(|| {
            invalid(format!(
                "native frame metadata missing for function {index}"
            ))
        })
}

/// Compute the maximum weighted path without recursive Rust traversal. A
/// cycle, an indirect call, or an edge outside the declared unguarded helper
/// set prevents a finite reserve from being established and rejects the module.
fn helper_path_bound(
    metadata: &Metadata,
    bodies: &[FunctionBody<'_>],
    imported_count: u32,
    module: &Module,
) -> Result<u64, EngineError> {
    let indices: BTreeMap<_, _> = metadata
        .helpers
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, index)| (index, ordinal))
        .collect();
    let mut bounds = Vec::with_capacity(indices.len());
    let mut pending = vec![0_usize; indices.len()];
    let mut parents = vec![Vec::new(); indices.len()];
    for (&index, &ordinal) in &indices {
        bounds.push(frame_bound(module, index)?);
        let body = bodies
            .get(index as usize)
            .ok_or_else(|| invalid("missing helper body"))?;
        let mut operators = body.get_operators_reader().map_err(invalid)?;
        let mut children = BTreeSet::new();
        while !operators.eof() {
            match operators.read().map_err(invalid)? {
                Operator::Call { function_index } | Operator::ReturnCall { function_index } => {
                    let child = function_index
                        .checked_sub(imported_count)
                        .and_then(|index| indices.get(&index).copied())
                        .ok_or_else(|| {
                            invalid(format!("unguarded helper {index} calls function {function_index} outside its bounded graph"))
                        })?;
                    children.insert(child);
                }
                Operator::CallIndirect { .. }
                | Operator::ReturnCallIndirect { .. }
                | Operator::CallRef { .. }
                | Operator::ReturnCallRef { .. } => {
                    return Err(invalid("unguarded helper makes an indirect call"));
                }
                _ => {}
            }
        }
        pending[ordinal] = children.len();
        for child in children {
            parents[child].push(ordinal);
        }
    }
    maximum_acyclic_path(&bounds, pending, &parents)
}

fn maximum_acyclic_path(
    own_bounds: &[u64],
    mut pending_children: Vec<usize>,
    parents: &[Vec<usize>],
) -> Result<u64, EngineError> {
    let mut paths = own_bounds.to_vec();
    let mut ready: VecDeque<_> = pending_children
        .iter()
        .enumerate()
        .filter_map(|(index, &count)| (count == 0).then_some(index))
        .collect();
    let mut visited = 0;
    while let Some(child) = ready.pop_front() {
        visited += 1;
        for &parent in &parents[child] {
            let path = own_bounds[parent]
                .checked_add(paths[child])
                .ok_or_else(|| invalid("helper frame reserve overflows u64"))?;
            paths[parent] = paths[parent].max(path);
            pending_children[parent] -= 1;
            if pending_children[parent] == 0 {
                ready.push_back(parent);
            }
        }
    }
    if visited != own_bounds.len() {
        return Err(invalid("unguarded helper graph is recursive"));
    }
    Ok(paths.into_iter().max().unwrap_or(0))
}

pub(super) fn register<T: 'static>(
    linker: &mut Linker<T>,
    module: &Module,
    bytes: &[u8],
) -> Result<(), EngineError> {
    let mut metadata_bytes = None;
    let mut imported_count = 0_u32;
    let mut guard_imports = 0;
    let mut bodies = Vec::new();
    let mut main_index = None;
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(invalid)? {
            Payload::CustomSection(section)
                if section.name() == lila_aot_wasm::STACK_GUARD_CUSTOM_SECTION =>
            {
                if metadata_bytes.replace(section.data()).is_some() {
                    return Err(invalid("duplicate metadata section"));
                }
            }
            Payload::ImportSection(section) => {
                for import in section.into_imports() {
                    let import = import.map_err(invalid)?;
                    let is_function = matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_));
                    if is_function {
                        imported_count = imported_count
                            .checked_add(1)
                            .ok_or_else(|| invalid("function import count overflows"))?;
                    }
                    if import.module == WASM_HOST_IMPORT_NAMESPACE && import.name == IMPORT_NAME {
                        if !is_function {
                            return Err(invalid("guard import is not a function"));
                        }
                        guard_imports += 1;
                    }
                }
            }
            Payload::ExportSection(section) => {
                for export in section {
                    let export = export.map_err(invalid)?;
                    if export.name == "main" {
                        if export.kind != wasmparser::ExternalKind::Func
                            || main_index.replace(export.index).is_some()
                        {
                            return Err(invalid("invalid main export"));
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => bodies.push(body),
            _ => {}
        }
    }
    let Some(metadata_bytes) = metadata_bytes else {
        return if guard_imports == 0 {
            Ok(())
        } else {
            Err(invalid("guard import lacks metadata"))
        };
    };
    if guard_imports != 1 {
        return Err(invalid("metadata requires exactly one guard import"));
    }
    let metadata = Metadata::decode(metadata_bytes)?;
    if main_index != Some(imported_count) {
        return Err(invalid("main must be defined function zero"));
    }
    if metadata.original_count as usize + metadata.targets.len() + 1 != bodies.len() {
        return Err(invalid("relocation does not match module function count"));
    }
    let helper_reserve = helper_path_bound(&metadata, &bodies, imported_count, module)?;
    frame_bound(module, metadata.startup_body)?;
    let mut wrapper_reserve = frame_bound(module, 0)?;
    for (&wrapper, &body) in &metadata.targets {
        wrapper_reserve = wrapper_reserve.max(frame_bound(module, wrapper)?);
        frame_bound(module, body)?;
    }
    let reserve = wrapper_reserve
        .checked_add(helper_reserve)
        .ok_or_else(|| invalid("stack guard reserve overflows u64"))?;
    let expected_module = module.clone();
    linker
        .func_wrap(
            WASM_HOST_IMPORT_NAMESPACE,
            IMPORT_NAME,
            move |caller: Caller<'_, T>, target: i32, wrapper: i32| -> wasmtime::Result<i32> {
                let (target, wrapper) = (target as u32, wrapper as u32);
                let startup = wrapper == 0 && target == metadata.startup_body;
                if !startup && metadata.targets.get(&wrapper) != Some(&target) {
                    return Err(wasmtime::Error::msg(
                        "invalid stack-guard wrapper/body pair",
                    ));
                }
                let budget = caller
                    .wasm_stack_budget_for_module(
                        &expected_module,
                        DefinedFuncIndex::from_u32(target),
                        DefinedFuncIndex::from_u32(wrapper),
                    )
                    .ok_or_else(|| {
                        wasmtime::Error::msg("stack guard called outside its compiled module")
                    })?;
                let required = budget
                    .target_frame()
                    .stack_size_bound_bytes()
                    .checked_add(reserve);
                let remaining = u64::try_from(budget.remaining_bytes()).unwrap_or(u64::MAX);
                let allowed = required.is_some_and(|required| remaining >= required);
                if startup && !allowed {
                    return Err(wasmtime::Error::msg("Wasm entry stack budget cannot accommodate the compiled main function and runtime reserve"));
                }
                Ok(i32::from(allowed))
            },
        )
        .map_err(invalid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_reserve_uses_longest_live_chain_and_rejects_cycles() {
        // A calls B and C sequentially; both call D. The reserve is A+C+D,
        // with no double counting of either sequential branch or shared D.
        assert_eq!(
            maximum_acyclic_path(
                &[10, 20, 30, 40],
                vec![2, 1, 1, 0],
                &[vec![], vec![0], vec![0], vec![1, 2]]
            )
            .unwrap(),
            80
        );
        assert!(maximum_acyclic_path(&[10, 20], vec![1, 1], &[vec![1], vec![0]]).is_err());
        assert!(maximum_acyclic_path(&[u64::MAX, 1], vec![1, 0], &[vec![], vec![0]]).is_err());
    }

    #[test]
    fn metadata_rejects_truncated_and_conflicting_relocations() {
        let mut bytes = MAGIC.to_vec();
        for word in [VERSION, 3, 4, 1, 1, 3, 1, 2] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let parsed = Metadata::decode(&bytes).unwrap();
        assert_eq!(parsed.targets.get(&1), Some(&3));
        assert!(parsed.helpers.contains(&2));
        for len in 0..bytes.len() {
            assert!(Metadata::decode(&bytes[..len]).is_err());
        }
        let mut omitted = bytes.clone();
        omitted[12..16].copy_from_slice(&4_u32.to_le_bytes());
        omitted[16..20].copy_from_slice(&5_u32.to_le_bytes());
        omitted[28..32].copy_from_slice(&4_u32.to_le_bytes());
        assert!(Metadata::decode(&omitted).is_err());
        let mut main_as_wrapper = bytes.clone();
        main_as_wrapper[24..28].copy_from_slice(&0_u32.to_le_bytes());
        assert!(Metadata::decode(&main_as_wrapper).is_err());
        let mut conflicting = bytes;
        let end = conflicting.len();
        conflicting[end - 4..].copy_from_slice(&1_u32.to_le_bytes());
        assert!(Metadata::decode(&conflicting).is_err());
    }
    #[test]
    fn startup_guard_rejects_a_main_frame_larger_than_the_available_stack() {
        use wasm_encoder::{
            CodeSection, CustomSection, EntityType, ExportKind, ExportSection, Function,
            FunctionSection, ImportSection, Instruction, TypeSection, ValType,
        };
        let mut bytes = wasm_encoder::Module::new();
        let mut types = TypeSection::new();
        types
            .ty()
            .function([ValType::I32, ValType::I32], [ValType::I32]);
        types.ty().function([], [ValType::I64]);
        bytes.section(&types);
        let mut imports = ImportSection::new();
        imports.import(
            WASM_HOST_IMPORT_NAMESPACE,
            IMPORT_NAME,
            EntityType::Function(0),
        );
        imports.import("probe", "value", EntityType::Function(1));
        bytes.section(&imports);
        let mut functions = FunctionSection::new();
        for _ in 0..5 {
            functions.function(1);
        }
        bytes.section(&functions);
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 2);
        bytes.section(&exports);
        let mut code = CodeSection::new();
        // Defined functions: startup wrapper, ordinary wrapper, helper,
        // ordinary body, startup body. Two imports precede these indices.
        for (wrapper, target) in [(0, 4), (1, 3)] {
            let mut function = Function::new([]);
            function.instruction(&Instruction::I32Const(target));
            function.instruction(&Instruction::I32Const(wrapper));
            function.instruction(&Instruction::Call(0));
            function.instruction(&Instruction::Drop);
            function.instruction(&Instruction::Call(target as u32 + 2));
            function.instruction(&Instruction::End);
            code.function(&function);
        }
        for _ in 0..2 {
            let mut function = Function::new([]);
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::End);
            code.function(&function);
        }
        // Each opaque host result remains live across later calls. This
        // forces a compiled spill frame larger than the 64 KiB Wasm budget.
        const VALUES: u32 = 16_384;
        let mut main = Function::new([(VALUES, ValType::I64)]);
        for local in 0..VALUES {
            main.instruction(&Instruction::Call(1));
            main.instruction(&Instruction::LocalSet(local));
        }
        main.instruction(&Instruction::I64Const(0));
        for local in 0..VALUES {
            main.instruction(&Instruction::LocalGet(local));
            main.instruction(&Instruction::I64Add);
        }
        main.instruction(&Instruction::End);
        code.function(&main);
        bytes.section(&code);
        let mut metadata = MAGIC.to_vec();
        for word in [VERSION, 3, 4, 1, 1, 3, 1, 2] {
            metadata.extend_from_slice(&word.to_le_bytes());
        }
        bytes.section(&CustomSection {
            name: lila_aot_wasm::STACK_GUARD_CUSTOM_SECTION.into(),
            data: metadata.into(),
        });
        let bytes = bytes.finish();
        let mut config =
            crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
        config.max_wasm_stack(64 * 1024);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let module = Module::new(&engine, &bytes).unwrap();
        assert!(frame_bound(&module, 4).unwrap() > 64 * 1024);
        let mut linker = Linker::new(&engine);
        register(&mut linker, &module, &bytes).unwrap();
        linker
            .func_wrap("probe", "value", || -> i64 {
                panic!("the oversized main body must never begin execution")
            })
            .unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        store.set_epoch_deadline(1);
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let error = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .unwrap()
            .call(&mut store, ())
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("Wasm entry stack budget cannot accommodate"),
            "{error:#}"
        );
    }

    #[test]
    fn refused_proxy_call_in_foreign_promise_job_uses_job_realm() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        };
        use wasmtime::{Engine as WasmtimeEngine, ExternType, Store};

        let source = r#"
            const other = __lilaCreateRealm();
            const foreign = other.evalScript('(function handler(value) { return value; })');
            const handler = new Proxy(foreign, {});
            Promise.resolve(1).then(handler).then(
                () => print('unexpected fulfillment'),
                error => print(error instanceof other.global.RangeError &&
                               !(error instanceof RangeError))
            );
            print('arm');
        "#;
        let lila = crate::Engine::new(crate::RealmBuilder::new().build());
        let unit = lila
            .compile_script(
                source,
                crate::CompileOptions {
                    host_surface_policy: crate::HostSurfacePolicy::Test262,
                    ..crate::CompileOptions::default()
                },
            )
            .unwrap();
        let artifact = lila.emit_wasm(&unit).unwrap();
        let config =
            crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
        let engine = WasmtimeEngine::new(&config).unwrap();
        let module = Module::new(&engine, &artifact.bytes).unwrap();

        let mut metadata = None;
        let mut proxy_wrapper_absolute = None;
        let mut bodies = Vec::new();
        for payload in Parser::new(0).parse_all(&artifact.bytes) {
            match payload.unwrap() {
                Payload::CustomSection(section)
                    if section.name() == lila_aot_wasm::STACK_GUARD_CUSTOM_SECTION =>
                {
                    metadata = Some(Metadata::decode(section.data()).unwrap());
                }
                Payload::CustomSection(section) => {
                    if let wasmparser::KnownCustom::Name(subsections) = section.as_known() {
                        for subsection in subsections {
                            if let wasmparser::Name::Function(names) = subsection.unwrap() {
                                for name in names {
                                    let name = name.unwrap();
                                    if name.name == "stack_guard::wrapper::helper::proxy_call" {
                                        proxy_wrapper_absolute = Some(name.index);
                                    }
                                }
                            }
                        }
                    }
                }
                Payload::CodeSectionEntry(body) => bodies.push(body),
                _ => {}
            }
        }
        let metadata = metadata.unwrap();
        let imported_count = u32::try_from(
            module
                .imports()
                .filter(|import| matches!(import.ty(), ExternType::Func(_)))
                .count(),
        )
        .unwrap();
        let proxy_wrapper = proxy_wrapper_absolute.unwrap() - imported_count;
        let proxy_body = metadata.targets[&proxy_wrapper];
        let helper_reserve =
            helper_path_bound(&metadata, &bodies, imported_count, &module).unwrap();
        let wrapper_reserve = std::iter::once(0)
            .chain(metadata.targets.keys().copied())
            .map(|index| frame_bound(&module, index).unwrap())
            .max()
            .unwrap();
        let reserve = wrapper_reserve.checked_add(helper_reserve).unwrap();

        let armed = Arc::new(AtomicBool::new(false));
        let denied = Arc::new(AtomicBool::new(false));
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut store = Store::new(&engine, ());
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = Linker::new(&engine);
        let expected_module = module.clone();
        let guard_armed = Arc::clone(&armed);
        let guard_denied = Arc::clone(&denied);
        linker
            .func_wrap(
                WASM_HOST_IMPORT_NAMESPACE,
                IMPORT_NAME,
                move |caller: Caller<'_, ()>, target: i32, wrapper: i32| -> wasmtime::Result<i32> {
                    let (target, wrapper) = (target as u32, wrapper as u32);
                    let startup = wrapper == 0 && target == metadata.startup_body;
                    if !startup && metadata.targets.get(&wrapper) != Some(&target) {
                        return Err(wasmtime::Error::msg("invalid guard pair"));
                    }
                    let budget = caller
                        .wasm_stack_budget_for_module(
                            &expected_module,
                            DefinedFuncIndex::from_u32(target),
                            DefinedFuncIndex::from_u32(wrapper),
                        )
                        .ok_or_else(|| wasmtime::Error::msg("invalid guard frame"))?;
                    if target == proxy_body && guard_armed.swap(false, Ordering::SeqCst) {
                        guard_denied.store(true, Ordering::SeqCst);
                        return Ok(0);
                    }
                    let allowed = budget
                        .target_frame()
                        .stack_size_bound_bytes()
                        .checked_add(reserve)
                        .is_some_and(|required| budget.remaining_bytes() as u64 >= required);
                    if startup && !allowed {
                        return Err(wasmtime::Error::msg("startup budget exhausted"));
                    }
                    Ok(i32::from(allowed))
                },
            )
            .unwrap();
        let print_armed = Arc::clone(&armed);
        let print_output = Arc::clone(&output);
        linker
            .func_wrap(
                WASM_HOST_IMPORT_NAMESPACE,
                crate::WASM_HOST_IMPORT_PRINT_LINE_UTF8,
                move |mut caller: Caller<'_, ()>, ptr: i32, len: i32| -> wasmtime::Result<()> {
                    let memory = caller
                        .get_export("memory")
                        .and_then(wasmtime::Extern::into_memory)
                        .ok_or_else(|| wasmtime::Error::msg("missing print memory"))?;
                    let mut bytes = vec![0; usize::try_from(len)?];
                    memory.read(&caller, usize::try_from(ptr)?, &mut bytes)?;
                    let text = String::from_utf8(bytes)?;
                    if text == "arm" {
                        print_armed.store(true, Ordering::SeqCst);
                    }
                    print_output.lock().unwrap().push(text);
                    Ok(())
                },
            )
            .unwrap();
        for import in module.imports() {
            if import.module() == WASM_HOST_IMPORT_NAMESPACE
                && (import.name() == IMPORT_NAME
                    || import.name() == crate::WASM_HOST_IMPORT_PRINT_LINE_UTF8)
            {
                continue;
            }
            match import.ty() {
                ExternType::Memory(memory_type)
                    if import.module() == WASM_HOST_IMPORT_NAMESPACE
                        && import.name() == crate::WASM_HOST_IMPORT_PRIVATE_MEMORY =>
                {
                    assert!(!memory_type.is_shared());
                    let memory = wasmtime::Memory::new(&mut store, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                ExternType::Memory(memory_type)
                    if import.module() == WASM_HOST_IMPORT_NAMESPACE
                        && import.name() == crate::WASM_HOST_IMPORT_SHARED_MEMORY =>
                {
                    assert!(memory_type.is_shared());
                    let memory = wasmtime::SharedMemory::new(&engine, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                ExternType::Func(_)
                    if import.module() == WASM_HOST_IMPORT_NAMESPACE
                        && import.name() == crate::WASM_HOST_IMPORT_AGENT_CAN_SUSPEND =>
                {
                    linker
                        .func_wrap(import.module(), import.name(), || -> i32 { 1 })
                        .unwrap();
                }
                ExternType::Func(_)
                    if import.module() == WASM_HOST_IMPORT_NAMESPACE
                        && import.name() == crate::WASM_HOST_IMPORT_MONOTONIC_CLOCK_NANOS =>
                {
                    // The main Promise checkpoint polls async-wait deadlines even
                    // though this fixture never starts an Atomics.waitAsync waiter.
                    linker
                        .func_wrap(import.module(), import.name(), || -> i64 { 0 })
                        .unwrap();
                }
                ExternType::Func(signature) => {
                    let name = format!("{}.{}", import.module(), import.name());
                    linker
                        .func_new(import.module(), import.name(), signature, move |_, _, _| {
                            Err(wasmtime::Error::msg(format!(
                                "unexpected host import {name}"
                            )))
                        })
                        .unwrap();
                }
                other => panic!("unexpected import in Promise fixture: {other:?}"),
            }
        }
        let instance = linker.instantiate(&mut store, &module).unwrap();
        instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .unwrap()
            .call(&mut store, ())
            .unwrap();
        assert!(
            denied.load(Ordering::SeqCst),
            "ProxyCall guard was not refused"
        );
        assert_eq!(
            output.lock().unwrap().as_slice(),
            &["arm".to_string(), "true".to_string()]
        );
    }
}
