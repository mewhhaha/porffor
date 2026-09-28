//! Integration witnesses for the pinned runtime's native stack metadata.

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, TypeSection, ValType,
};
use wasmtime::{Caller, DefinedFuncIndex, Engine, Func, Linker, Module, Store};

fn probe_module() -> Vec<u8> {
    let mut module = wasm_encoder::Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], [ValType::I64]);
    types.ty().function([], [ValType::I64]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("host", "probe", EntityType::Function(0));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(0);
    functions.function(1);
    module.section(&functions);
    let mut exports = ExportSection::new();
    exports.export("recurse", ExportKind::Func, 1);
    exports.export("spills", ExportKind::Func, 2);
    module.section(&exports);
    let mut code = CodeSection::new();
    let mut recurse = Function::new([]);
    recurse.instruction(&Instruction::LocalGet(0));
    recurse.instruction(&Instruction::Call(0));
    recurse.instruction(&Instruction::Drop);
    recurse.instruction(&Instruction::LocalGet(0));
    recurse.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
    recurse.instruction(&Instruction::LocalGet(0));
    recurse.instruction(&Instruction::I32Const(1));
    recurse.instruction(&Instruction::I32Sub);
    recurse.instruction(&Instruction::Call(1));
    recurse.instruction(&Instruction::Else);
    recurse.instruction(&Instruction::I64Const(0));
    recurse.instruction(&Instruction::End);
    recurse.instruction(&Instruction::End);
    code.function(&recurse);
    // Distinct host results stay live across later calls, forcing native
    // spills. A count of Wasm locals alone would not prove native frame size.
    let mut spills = Function::new([(80, ValType::I64)]);
    for local in 0..80 {
        spills.instruction(&Instruction::I32Const(-1));
        spills.instruction(&Instruction::Call(0));
        spills.instruction(&Instruction::LocalSet(local));
    }
    spills.instruction(&Instruction::I64Const(0));
    for local in 0..80 {
        spills.instruction(&Instruction::LocalGet(local));
        spills.instruction(&Instruction::I64Add);
    }
    spills.instruction(&Instruction::End);
    code.function(&spills);
    module.section(&code);
    module.finish()
}

#[derive(Default)]
struct Observations {
    remaining: Vec<usize>,
    calls: i64,
}

#[test]
fn native_stack_metadata_tracks_spills_and_actual_recursion() {
    let config = crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, probe_module()).unwrap();
    let recursive_index = DefinedFuncIndex::from_u32(0);
    let spill_index = DefinedFuncIndex::from_u32(1);
    let recursive = module.wasm_function_stack_layout(recursive_index).unwrap();
    let spills = module.wasm_function_stack_layout(spill_index).unwrap();
    assert!(recursive.stack_size_bound_bytes() > 0);
    assert!(spills.stack_size_bound_bytes() > recursive.stack_size_bound_bytes());
    assert!(
        module
            .wasm_function_stack_layout(DefinedFuncIndex::from_u32(2))
            .is_none()
    );
    let expected_module = module.clone();
    let different_module = Module::new(&engine, probe_module()).unwrap();
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "host",
            "probe",
            move |mut caller: Caller<'_, Observations>, depth: i32| {
                let active = if depth < 0 {
                    spill_index
                } else {
                    recursive_index
                };
                let other = if depth < 0 {
                    recursive_index
                } else {
                    spill_index
                };
                assert!(
                    caller.wasm_stack_budget(active, other).is_none(),
                    "mismatched caller must be rejected"
                );
                assert!(
                    caller
                        .wasm_stack_budget_for_module(&different_module, active, active)
                        .is_none(),
                    "matching function indices in another module must not authenticate the caller"
                );
                let budget = caller
                    .wasm_stack_budget_for_module(&expected_module, active, active)
                    .unwrap();
                assert_eq!(
                    budget.remaining_bytes(),
                    caller
                        .wasm_stack_budget(active, active)
                        .unwrap()
                        .remaining_bytes()
                );
                let expected = if depth < 0 { spills } else { recursive };
                assert_eq!(budget.target_frame(), expected);
                assert_eq!(budget.guard_wrapper_frame(), expected);
                assert!(budget.remaining_bytes() > 0);
                if depth >= 0 {
                    caller.data_mut().remaining.push(budget.remaining_bytes());
                }
                caller.data_mut().calls += 1;
                caller.data().calls
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, Observations::default());
    store.set_epoch_deadline(1);
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let recurse = instance
        .get_typed_func::<i32, i64>(&mut store, "recurse")
        .unwrap();
    recurse.call(&mut store, 16).unwrap();
    assert_eq!(store.data().remaining.len(), 17);
    assert!(
        store
            .data()
            .remaining
            .windows(2)
            .all(|pair| pair[1] < pair[0])
    );
    instance
        .get_typed_func::<(), i64>(&mut store, "spills")
        .unwrap()
        .call(&mut store, ())
        .unwrap();
}

#[test]
fn native_stack_budget_is_absent_for_host_only_calls() {
    let config = crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
    let engine = Engine::new(&config).unwrap();
    let mut store = Store::new(&engine, ());
    store.set_epoch_deadline(1);
    let function = Func::wrap(&mut store, |caller: Caller<'_, ()>| {
        assert!(
            caller
                .wasm_stack_budget(DefinedFuncIndex::from_u32(0), DefinedFuncIndex::from_u32(0))
                .is_none()
        );
    });
    function
        .typed::<(), ()>(&store)
        .unwrap()
        .call(&mut store, ())
        .unwrap();
}
