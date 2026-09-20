use super::*;

pub(super) struct IntlHostProbe {
    pub(super) store: WasmtimeStore<WasmHostState>,
    pub(super) memory: WasmtimeMemory,
    call: wasmtime::TypedFunc<(i64, i64, i64), i64>,
}

impl IntlHostProbe {
    pub(super) fn new() -> Self {
        // A tiny Wasm caller exercises Caller::get_export and overlapping
        // spans through the same import callback as a compiled JS artifact.
        fn section(module: &mut Vec<u8>, id: u8, bytes: &[u8]) {
            assert!(bytes.len() < 128);
            module.extend_from_slice(&[id, bytes.len() as u8]);
            module.extend_from_slice(bytes);
        }
        let mut bytes = b"\0asm\x01\0\0\0".to_vec();
        section(&mut bytes, 1, &[1, 0x60, 3, 0x7e, 0x7e, 0x7e, 1, 0x7e]);
        let mut import = vec![1, WASM_HOST_IMPORT_NAMESPACE.len() as u8];
        import.extend_from_slice(WASM_HOST_IMPORT_NAMESPACE.as_bytes());
        import.push(WASM_HOST_IMPORT_INTL_CALL.len() as u8);
        import.extend_from_slice(WASM_HOST_IMPORT_INTL_CALL.as_bytes());
        import.extend_from_slice(&[0, 0]);
        section(&mut bytes, 2, &import);
        section(&mut bytes, 3, &[1, 0]);
        section(&mut bytes, 5, &[1, 0, 1]);
        section(&mut bytes, 7, b"\x02\x06memory\x02\x00\x04call\x00\x01");
        section(
            &mut bytes,
            10,
            &[1, 10, 0, 0x20, 0, 0x20, 1, 0x20, 2, 0x10, 0, 0x0b],
        );
        let wasm_engine = shared_wasm_engine().unwrap();
        let module = WasmtimeModule::new(&wasm_engine, bytes).unwrap();
        let realm = RealmBuilder::new().build();
        let mut store = WasmtimeStore::new(
            &wasm_engine,
            WasmHostState {
                monotonic_clock_origin: realm.host_clock().monotonic_instant(),
                realm,
                output_events: WasmOutputEvents::DelegateOnly,
                intl_kernel: shared_embedded_intl_kernel().unwrap(),
                can_block: true,
                shared_memory_backing: None,
                agent_group: None,
                agent_commands: None,
                agent_leaving: None,
                limits: WasmtimeStoreLimitsBuilder::new()
                    .memory_size(WASM_STORE_MEMORY_CAP_BYTES)
                    .build(),
            },
        );
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = WasmtimeLinker::new(&wasm_engine);
        linker
            .func_wrap(
                WASM_HOST_IMPORT_NAMESPACE,
                WASM_HOST_IMPORT_INTL_CALL,
                wasm_intl_call,
            )
            .unwrap();
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let call = instance
            .get_typed_func::<(i64, i64, i64), i64>(&mut store, "call")
            .unwrap();
        Self {
            store,
            memory,
            call,
        }
    }

    pub(super) fn invoke(
        &mut self,
        operation: IntlHostOp,
        request: IntlHostReadSpan,
        result: IntlHostWriteSpan,
    ) -> wasmtime::Result<i64> {
        self.call.call(
            &mut self.store,
            (operation.wire(), request.wire(), result.wire()),
        )
    }
}
