use super::*;
use crate::wasmtime_config::base_config;
use lila_intl::{IntlCompilationProfile, IntlDataSelection};
use wasmtime::WasmBacktraceDetails;

#[test]
fn embedded_manifest_has_one_complete_framing_and_rejects_build_mismatches() {
    if MANIFEST.is_empty() {
        assert!(PACKAGE.is_empty() && NATIVE.is_empty());
        assert!(!cfg!(all(
            target_endian = "little",
            any(
                target_arch = "x86_64",
                target_arch = "aarch64",
                target_arch = "riscv64"
            )
        )));
        return;
    }
    for end in 0..MANIFEST.len() {
        assert!(manifest::Manifest::decode(&MANIFEST[..end]).is_none());
    }
    let mut extended = MANIFEST.to_vec();
    extended.push(0);
    assert!(manifest::Manifest::decode(&extended).is_none());
    assert!(EmbeddedNativeRuntime::admit().is_some());
    let mut previous_version = MANIFEST.to_vec();
    previous_version[8..12].copy_from_slice(&1_u32.to_le_bytes());
    assert!(manifest::Manifest::decode(&previous_version).is_none());
    // Both mode bytes have closed domains, separately from product admission.
    for offset in [16, 17] {
        let mut unknown_mode = MANIFEST.to_vec();
        unknown_mode[offset] = 2;
        assert!(manifest::Manifest::decode(&unknown_mode).is_none());
    }
    for field in [
        "source",
        "target",
        "configuration",
        "runtime mode",
        "execution mode",
        "package digest",
        "native digest",
    ] {
        let mut manifest = manifest::Manifest::decode(MANIFEST).unwrap();
        match field {
            "source" => manifest.source[0] ^= 1,
            "target" => manifest.target.push_str("-different"),
            "configuration" => manifest.configuration += 1,
            "runtime mode" => manifest.modes.runtime = WasmNativeCompilationMode::Fast,
            "execution mode" => manifest.modes.execution = WasmNativeCompilationMode::SizeOptimized,
            "package digest" => manifest.package_digest[0] ^= 1,
            "native digest" => manifest.native_digest[0] ^= 1,
            _ => unreachable!(),
        }
        // Only metadata is mutated. No corrupted native code reaches deserialize.
        assert!(
            EmbeddedNativeRuntime::admit_manifest(manifest).is_none(),
            "{field}"
        );
    }
}

#[cfg(all(
    target_endian = "little",
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )
))]
#[test]
fn embedded_runtime_matches_target_emission_and_preserves_native_compatibility_checks() {
    std::thread::Builder::new()
        .name("embedded-runtime-target-parity".to_owned())
        .stack_size(crate::ENGINE_WORKER_STACK_SIZE)
        .spawn(|| {
            let selection = IntlDataSelection::new(IntlCompilationProfile::Minimal);
            let runtime = lila_aot_wasm::runtime_artifact_with_inputs(&selection, inputs(None))
                .expect("embedded package passes full target Intl/Wasm/layout admission");
            let target = lila_aot_wasm::build_runtime_artifact(&selection, expected_source())
                .expect("target compiler independently emits R");
            assert_eq!(runtime.key(), target.key());
            assert_eq!(
                target.package_bytes(),
                PACKAGE,
                "the entire host/target raw package is byte-identical"
            );
            assert_eq!(
                runtime.bytes().as_ref(),
                target.wasm(),
                "host-produced R equals ordinary target R exactly"
            );
            let bundle = EmbeddedNativeRuntime::admit().unwrap();
            assert_eq!(
                bundle.manifest.modes.runtime,
                WasmNativeCompilationMode::SizeOptimized
            );
            assert_eq!(
                bundle.manifest.modes.execution,
                WasmNativeCompilationMode::Fast
            );
            let runtime_compiler = Engine::new(&base_config(
                bundle.manifest.modes.runtime,
                WasmBacktraceDetails::Disable,
            ))
            .unwrap();
            assert_eq!(
                runtime_compiler.get_cranelift_opt_level(),
                Some(wasmtime::OptLevel::SpeedAndSize),
                "the Cargo producer's selected R mode is optimized"
            );
            let config = base_config(
                WasmNativeCompilationMode::Fast,
                WasmBacktraceDetails::Disable,
            );
            let engine = Engine::new(&config).unwrap();
            assert_eq!(
                engine.get_cranelift_opt_level(),
                Some(wasmtime::OptLevel::None)
            );
            crate::wasmtime_policy::PRODUCT_WASMTIME_POLICY
                .verify_engine(&engine)
                .unwrap();
            let first = bundle
                .load(&engine, &runtime, WasmNativeCompilationMode::Fast)
                .expect("baseline ISA bundle loads on the actual target");
            assert!(
                Engine::same(first.engine(), &engine),
                "optimized R is owned by the actual Fast execution Engine"
            );
            assert!(
                !Engine::same(first.engine(), &runtime_compiler),
                "the build compiler is not an execution or retention owner"
            );
            let second = bundle
                .load(&engine, &runtime, WasmNativeCompilationMode::Fast)
                .expect("a second factory call owns a new native mapping");
            assert_ne!(
                first.image_range().start,
                second.image_range().start,
                "the embedded authority retains no native Module"
            );
            assert!(
                bundle
                    .load(
                        &runtime_compiler,
                        &runtime,
                        WasmNativeCompilationMode::SizeOptimized
                    )
                    .is_none(),
                "a matching R image mode cannot authorize a different execution mode"
            );
            assert!(
                load_native(&runtime_compiler, &runtime, WasmNativeCompilationMode::SizeOptimized)
                    .is_none(),
                "the product entry also preserves the ordinary path for unsupported execution modes"
            );
            let mut foreign_key = EmbeddedNativeRuntime::admit().unwrap();
            foreign_key.manifest.runtime_key[0] ^= 1;
            assert!(
                foreign_key
                    .load(&engine, &runtime, WasmNativeCompilationMode::Fast)
                    .is_none(),
                "native R cannot attach to another raw runtime key"
            );
            let mut mismatched = base_config(
                WasmNativeCompilationMode::Fast,
                WasmBacktraceDetails::Disable,
            );
            mismatched.epoch_interruption(false);
            let incompatible = Engine::new(&mismatched).unwrap();
            assert!(
                bundle
                    .load(&incompatible, &runtime, WasmNativeCompilationMode::Fast)
                    .is_none(),
                "known-valid native bytes with incompatible tunables are an optimization miss"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
