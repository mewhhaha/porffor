//! Test262's collection hook invokes the selected Wasm runtime's collector.
//!
//! This collects native Wasm GC references, including argument vectors and
//! deferred invocation frames. It does not trace integer addresses in Lila's
//! still-unmigrated JavaScript heap. Registered native weak edges participate in
//! the same collection; JavaScript weak builtins still need the reference ABI.

use wasmtime::{AsContextMut, Caller, Linker};

use super::{EngineError, WASM_HOST_IMPORT_NAMESPACE};

pub(super) fn register<T: 'static>(linker: &mut Linker<T>) -> Result<(), EngineError> {
    linker
        .func_wrap(
            WASM_HOST_IMPORT_NAMESPACE,
            "gc",
            |mut caller: Caller<'_, T>| -> wasmtime::Result<()> {
                // Collect while Wasm frames are suspended at the host call.
                // Wasmtime owns their stack maps and all native reference roots.
                // Propagate collection errors instead of reporting success.
                caller.as_context_mut().gc(None)
            },
        )
        .map_err(|error| EngineError::new(format!("wasmtime GC linker setup failed: {error}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use wasm_encoder::{
        CodeSection, EntityType, ExportKind, ExportSection, FieldType, Function, FunctionSection,
        HeapType, ImportSection, Instruction, RefType, StorageType, TypeSection, ValType,
    };
    use wasmtime::{Engine, ExternRef, Module, RootScope, Store};

    struct DropWitness(Arc<AtomicUsize>);

    impl Drop for DropWitness {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn host_collection_reclaims_cycles_and_preserves_a_live_wasm_local() {
        let config = crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast)
            .expect("product runtime configuration");
        let engine = Engine::new(&config).unwrap();
        let mut types = TypeSection::new();
        types.ty().function([], []);
        types.ty().struct_([
            FieldType {
                element_type: StorageType::Val(ValType::Ref(RefType::EQREF)),
                mutable: true,
            },
            FieldType {
                element_type: StorageType::Val(ValType::Ref(RefType::EXTERNREF)),
                mutable: false,
            },
        ]);
        types
            .ty()
            .function([ValType::Ref(RefType::EXTERNREF)], [ValType::I32]);
        let mut imports = ImportSection::new();
        imports.import(WASM_HOST_IMPORT_NAMESPACE, "gc", EntityType::Function(0));
        let mut functions = FunctionSection::new();
        functions.function(2);
        let mut exports = ExportSection::new();
        exports.export("collect", ExportKind::Func, 0);
        exports.export("keep_alive", ExportKind::Func, 1);
        let mut body = Function::new_with_locals_types([ValType::Ref(RefType {
            nullable: true,
            heap_type: HeapType::Concrete(1),
        })]);
        // Make a self-cycle whose only Wasm root is local 1. Read it again
        // after the host callback has collected with this frame suspended.
        body.instruction(&Instruction::RefNull(RefType::EQREF.heap_type));
        body.instruction(&Instruction::LocalGet(0));
        body.instruction(&Instruction::StructNew(1));
        body.instruction(&Instruction::LocalTee(1));
        body.instruction(&Instruction::LocalGet(1));
        body.instruction(&Instruction::StructSet {
            struct_type_index: 1,
            field_index: 0,
        });
        body.instruction(&Instruction::Call(0));
        body.instruction(&Instruction::LocalGet(1));
        body.instruction(&Instruction::StructGet {
            struct_type_index: 1,
            field_index: 0,
        });
        body.instruction(&Instruction::LocalGet(1));
        body.instruction(&Instruction::RefEq);
        body.instruction(&Instruction::End);
        let mut code = CodeSection::new();
        code.function(&body);
        let mut wasm = wasm_encoder::Module::new();
        wasm.section(&types)
            .section(&imports)
            .section(&functions)
            .section(&exports)
            .section(&code);
        let module = Module::new(&engine, wasm.finish()).unwrap();
        let mut store = Store::new(&engine, ());
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = Linker::new(&engine);
        register(&mut linker).unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let keep_alive = instance
            .get_typed_func::<Option<wasmtime::Rooted<ExternRef>>, i32>(&mut store, "keep_alive")
            .unwrap();
        let collect = instance
            .get_typed_func::<(), ()>(&mut store, "collect")
            .unwrap();
        let dropped = Arc::new(AtomicUsize::new(0));
        {
            let mut scope = RootScope::new(&mut store);
            let witness = ExternRef::new(&mut scope, DropWitness(dropped.clone())).unwrap();
            assert_eq!(keep_alive.call(&mut scope, Some(witness)).unwrap(), 1);
            assert_eq!(dropped.load(Ordering::SeqCst), 0);
        }
        collect.call(&mut store, ()).unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        collect.call(&mut store, ()).unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        // The witness must be dropped by collection, before Store teardown.
    }
}
