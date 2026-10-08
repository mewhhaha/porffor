#![cfg(test)]

use super::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, ConstExpr, CustomSection, DataSection, ExportKind, ExportSection, Function,
    FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemorySection,
    MemoryType, Module, TypeSection, ValType,
};

struct MemoryCache {
    compiler: [u8; 32],
    entries: RefCell<HashMap<RuntimeArtifactCacheKey, Vec<u8>>>,
    removals: Cell<u32>,
}

impl MemoryCache {
    fn new() -> Self {
        Self {
            compiler: [17; 32],
            entries: RefCell::new(HashMap::new()),
            removals: Cell::new(0),
        }
    }
}

impl RuntimeArtifactCache for MemoryCache {
    fn compiler_identity(&self) -> &[u8; 32] {
        &self.compiler
    }

    fn load(&self, key: RuntimeArtifactCacheKey) -> Option<Vec<u8>> {
        self.entries.borrow().get(&key).cloned()
    }

    fn store(&self, key: RuntimeArtifactCacheKey, entry: Vec<u8>) -> bool {
        self.entries.borrow_mut().insert(key, entry);
        true
    }

    fn remove(&self, key: RuntimeArtifactCacheKey) {
        self.removals.set(self.removals.get() + 1);
        self.entries.borrow_mut().remove(&key);
    }
}

fn identity() -> RuntimeIdentity {
    RuntimeIdentity::from_sections(BTreeMap::from([(
        INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION,
        Arc::<[u8]>::from(&b"checked fixture"[..]),
    )]))
}

#[derive(Clone, Copy)]
enum Defect {
    None,
    FunctionResult,
    ExportKind,
    ForeignImport,
    DataOffset,
    IntlIdentity,
}

fn fixture(defect: Defect) -> Arc<RuntimeArtifact> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import(
        if matches!(defect, Defect::ForeignImport) {
            "lila_runtime"
        } else {
            HOST_IMPORT_MODULE
        },
        "fixture",
        wasm_encoder::EntityType::Function(0),
    );
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);
    let mut memories = MemorySection::new();
    memories.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memories);
    let global_type = GlobalType {
        val_type: ValType::I32,
        mutable: true,
        shared: false,
    };
    let mut globals = GlobalSection::new();
    globals.global(global_type, &ConstExpr::i32_const(0));
    module.section(&globals);
    let mut exports = ExportSection::new();
    if matches!(defect, Defect::ExportKind) {
        exports.export("f1", ExportKind::Global, 0);
    } else {
        exports.export("f1", ExportKind::Func, 1);
    }
    exports.export("g0", ExportKind::Global, 0);
    module.section(&exports);
    let mut body = Function::new([]);
    if matches!(defect, Defect::FunctionResult) {
        body.instruction(&Instruction::I64Const(7));
    } else {
        body.instruction(&Instruction::I32Const(7));
    }
    body.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&body);
    module.section(&code);
    let mut data = DataSection::new();
    data.passive([0, 0, 65, 0]);
    data.active(
        0,
        &ConstExpr::i32_const(
            STATIC_DATA_OFFSET as i32 + i32::from(matches!(defect, Defect::DataOffset)),
        ),
        [0, 65],
    );
    module.section(&data);
    module.section(&CustomSection {
        name: INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION.into(),
        data: if matches!(defect, Defect::IntlIdentity) {
            &b"wrong fixture"[..]
        } else {
            &b"checked fixture"[..]
        }
        .into(),
    });
    let bytes: Arc<[u8]> = Arc::from(module.finish());
    Arc::new(RuntimeArtifact {
        key: runtime_key(&bytes),
        bytes,
        layout: RuntimeLayout::new(
            1,
            vec![0],
            vec![global_type],
            PoolBoundary::from_runtime_cache(2, 4, 2).unwrap(),
        ),
    })
}

#[test]
fn runtime_cache_round_trip_reconstructs_the_encoded_types_and_pool() {
    let identity = identity();
    let cache = MemoryCache::new();
    let key = identity.cache_key(cache.compiler_identity());
    let runtime = fixture(Defect::None);
    let restored = decode(&encode(&runtime, key), key, &identity).expect("valid entry");
    assert_eq!(restored.bytes, runtime.bytes);
    assert_eq!(restored.key, runtime.key);
    assert_eq!(restored.layout, runtime.layout);
}

#[test]
fn runtime_cache_rejects_corruption_truncation_stale_identity_and_trailing_bytes() {
    let identity = identity();
    let key = identity.cache_key(&[17; 32]);
    let runtime = fixture(Defect::None);
    let entry = encode(&runtime, key);
    for offset in [0, 8, 12, 44, 52, 60, 64, 72, entry.len() - 1] {
        let mut corrupt = entry.clone();
        corrupt[offset] ^= 1;
        assert!(
            decode(&corrupt, key, &identity).is_none(),
            "offset {offset}"
        );
    }
    for length in 0..104 {
        assert!(decode(&entry[..length], key, &identity).is_none());
    }
    assert!(decode(&entry[..entry.len() - 1], key, &identity).is_none());
    let mut trailing = entry.clone();
    trailing.push(0);
    assert!(decode(&trailing, key, &identity).is_none());
    assert!(decode(&entry, identity.cache_key(&[18; 32]), &identity).is_none());

    // Repair the checksum after changing the declared static-data boundary:
    // integrity alone must not admit a layout inconsistent with actual Wasm.
    let mut wrong_boundary = entry;
    wrong_boundary[44..52].copy_from_slice(&3u64.to_le_bytes());
    let digest_offset = wrong_boundary.len() - DIGEST_BYTES;
    let digest = Sha256::digest(&wrong_boundary[..digest_offset]);
    wrong_boundary[digest_offset..].copy_from_slice(&digest);
    assert!(decode(&wrong_boundary, key, &identity).is_none());
}

#[test]
fn runtime_cache_rejects_validly_checksummed_wrong_abi_or_unvalidated_wasm() {
    let identity = identity();
    let key = identity.cache_key(&[17; 32]);
    for defect in [
        Defect::FunctionResult,
        Defect::ExportKind,
        Defect::ForeignImport,
        Defect::DataOffset,
        Defect::IntlIdentity,
    ] {
        assert!(decode(&encode(&fixture(defect), key), key, &identity).is_none());
    }
}

#[test]
fn runtime_cache_identity_binds_compiler_and_exact_admitted_sections() {
    let first = identity();
    let mut changed_sections = first.sections.clone();
    changed_sections.insert(
        INTL_SERVICE_SELECTION_CUSTOM_SECTION,
        Arc::from(&b"services"[..]),
    );
    let services = RuntimeIdentity::from_sections(changed_sections.clone());
    changed_sections.insert(
        INTL_ARTIFACT_IDENTITY_CUSTOM_SECTION,
        Arc::from(&b"new data"[..]),
    );
    let data = RuntimeIdentity::from_sections(changed_sections);
    assert_ne!(first.cache_key(&[17; 32]), first.cache_key(&[18; 32]));
    assert_ne!(first.cache_key(&[17; 32]), services.cache_key(&[17; 32]));
    assert_ne!(services.cache_key(&[17; 32]), data.cache_key(&[17; 32]));
}

#[test]
fn runtime_cache_rejects_and_replaces_a_corrupt_entry_once() {
    let identity = identity();
    let cache = MemoryCache::new();
    let key = identity.cache_key(cache.compiler_identity());
    let runtime = fixture(Defect::None);
    cache.store(key, vec![0, 1, 2, 3]);
    let replacements = Cell::new(0);
    let first = load_or_emit(&identity, Some(&cache), || {
        replacements.set(replacements.get() + 1);
        Ok(runtime.clone())
    })
    .unwrap();
    let second = load_or_emit(&identity, Some(&cache), || {
        panic!("a validated hit must not emit")
    })
    .unwrap();
    assert_eq!(replacements.get(), 1);
    assert_eq!(cache.removals.get(), 1);
    assert_eq!(first.bytes, second.bytes);
}

#[test]
fn runtime_cache_round_trips_real_minimal_and_selected_custom_runtimes_without_emission() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let custom = lila_intl::CustomIntlProfile::for_services(
                lila_intl::CustomProfileId::parse("runtime-cache-list-only").unwrap(),
                &["ListFormat"],
            )
            .unwrap();
            for profile in [
                lila_intl::IntlCompilationProfile::Minimal,
                lila_intl::IntlCompilationProfile::CustomProjection(custom),
            ] {
                let selection = lila_intl::IntlDataSelection::new(profile);
                let identity = RuntimeIdentity::admit(&selection).expect("Intl selection admitted");
                let emitted =
                    super::super::emit_runtime_artifact(&selection).expect("actual runtime emits");
                let cache = MemoryCache::new();
                let first = load_or_emit(&identity, Some(&cache), || Ok(emitted.clone())).unwrap();
                let second = load_or_emit(&identity, Some(&cache), || {
                    panic!("a real-runtime cache hit must not rebuild R")
                })
                .expect("actual runtime and typed layout reload");
                assert_eq!(first.bytes, second.bytes);
                assert_eq!(first.key, second.key);
                assert_eq!(first.layout, second.layout);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
