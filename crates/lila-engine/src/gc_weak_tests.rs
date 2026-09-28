//! Collector acceptance tests, independent of Lila's still-scalar JS heap.
//! Drop witnesses prove reclamation before Store teardown. No observation of
//! an object uses an integer address, and every temporary host root is scoped.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use wasmtime::{
    AnyRef, AsContextMut, Engine, ExternRef, FieldType, GcEphemeronTable, GcFinalizationRegistry,
    GcWeakHeap, GcWeakRef, Mutability, OwnedRooted, RootScope, Rooted, StorageType, Store,
    StructRef, StructRefPre, StructType, Val, ValType, I31,
};

#[derive(Clone, Default)]
struct Deaths(Arc<AtomicUsize>);

impl Deaths {
    fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

struct Witness(Deaths);

impl Drop for Witness {
    fn drop(&mut self) {
        self.0 .0.fetch_add(1, Ordering::SeqCst);
    }
}

fn setup() -> (Store<()>, StructRefPre) {
    let config = crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
    let engine = Engine::new(&config).unwrap();
    let mut store = Store::new(&engine, ());
    store.set_epoch_deadline(u64::MAX / 2);
    let ty = StructType::new(
        &engine,
        [
            FieldType::new(Mutability::Var, StorageType::ValType(ValType::ANYREF)),
            FieldType::new(Mutability::Const, StorageType::ValType(ValType::EXTERNREF)),
        ],
    )
    .unwrap();
    let allocator = StructRefPre::new(&mut store, ty);
    (store, allocator)
}

fn object(
    mut store: impl AsContextMut,
    allocator: &StructRefPre,
    deaths: &Deaths,
) -> Rooted<AnyRef> {
    let witness = ExternRef::new(&mut store, Witness(deaths.clone())).unwrap();
    StructRef::new(
        &mut store,
        allocator,
        &[Val::AnyRef(None), Val::ExternRef(Some(witness))],
    )
    .unwrap()
    .into()
}

fn owned(store: &mut Store<()>, allocator: &StructRefPre, deaths: &Deaths) -> OwnedRooted<AnyRef> {
    let mut scope = RootScope::new(store);
    object(&mut scope, allocator, deaths)
        .to_owned_rooted(&mut scope)
        .unwrap()
}

fn link(mut store: impl AsContextMut, from: Rooted<AnyRef>, to: Rooted<AnyRef>) {
    let from = from.as_struct(&mut store).unwrap().unwrap();
    let to = to.as_struct(&mut store).unwrap().unwrap().to_anyref();
    from.set_field(&mut store, 0, Val::AnyRef(Some(to)))
        .unwrap();
}

#[test]
fn weak_target_is_kept_for_the_job_then_an_unreachable_cycle_is_reclaimed() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let deaths = Deaths::default();
    {
        let mut scope = RootScope::new(&mut store);
        let target = object(&mut scope, &allocator, &deaths);
        link(&mut scope, target, target);
        GcWeakRef::initialize(&mut scope, &owner, &target).unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(
        deaths.count(),
        0,
        "constructor keeps its target for this job"
    );
    {
        let mut scope = RootScope::new(&mut store);
        assert!(GcWeakRef::get(&mut scope, &owner).unwrap().is_some());
    }
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 0, "deref keeps its target for this job");
    GcWeakHeap::clear_kept_objects(&mut store).unwrap();
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 1);
    assert!(GcWeakRef::get(&mut store, &owner).unwrap().is_none());
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 1);
}

#[test]
fn weak_reference_tracks_a_strong_root_across_repeated_moves() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let deaths = Deaths::default();
    let target = owned(&mut store, &allocator, &deaths);
    GcWeakRef::initialize(&mut store, &owner, &target).unwrap();
    for _ in 0..8 {
        GcWeakHeap::clear_kept_objects(&mut store).unwrap();
        store.gc(None).unwrap();
        let mut scope = RootScope::new(&mut store);
        let observed = GcWeakRef::get(&mut scope, &owner).unwrap().unwrap();
        assert!(Rooted::ref_eq(&scope, &target, &observed).unwrap());
        assert_eq!(deaths.count(), 0);
    }
    drop(target);
    GcWeakHeap::clear_kept_objects(&mut store).unwrap();
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 1);
    assert!(GcWeakRef::get(&mut store, &owner).unwrap().is_none());
}

#[test]
fn weak_registry_does_not_keep_dead_owners_alive() {
    let (mut store, allocator) = setup();
    let owners = Deaths::default();
    let targets = Deaths::default();
    let target = owned(&mut store, &allocator, &targets);
    for _ in 0..32 {
        {
            let mut scope = RootScope::new(&mut store);
            let owner = object(&mut scope, &allocator, &owners);
            GcWeakRef::initialize(&mut scope, &owner, &target).unwrap();
        }
        GcWeakHeap::clear_kept_objects(&mut store).unwrap();
        store.gc(None).unwrap();
    }
    assert_eq!(owners.count(), 32);
    assert_eq!(targets.count(), 0);
    drop(target);
    store.gc(None).unwrap();
    assert_eq!(targets.count(), 1);
}

#[test]
fn ephemeron_retains_value_until_key_dies_and_reclaims_back_edge_cycle() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    GcEphemeronTable::initialize(&mut store, &owner).unwrap();
    let keys = Deaths::default();
    let values = Deaths::default();
    let key = owned(&mut store, &allocator, &keys);
    {
        let mut scope = RootScope::new(&mut store);
        let value = object(&mut scope, &allocator, &values);
        let key = key.to_rooted(&mut scope);
        link(&mut scope, value, key);
        GcEphemeronTable::set(&mut scope, &owner, &key, &value).unwrap();
    }
    for _ in 0..4 {
        store.gc(None).unwrap();
        let mut scope = RootScope::new(&mut store);
        let value = GcEphemeronTable::get(&mut scope, &owner, &key)
            .unwrap()
            .unwrap();
        let back = *value
            .as_struct(&mut scope)
            .unwrap()
            .unwrap()
            .field(&mut scope, 0)
            .unwrap()
            .unwrap_anyref()
            .unwrap();
        assert!(Rooted::ref_eq(&scope, &key, &back).unwrap());
        assert_eq!(values.count(), 0);
    }
    drop(key);
    store.gc(None).unwrap();
    assert_eq!(keys.count(), 1);
    assert_eq!(values.count(), 1, "a value cannot rescue its own weak key");
}

#[test]
fn ephemeron_does_not_retain_value_when_owner_dies_even_with_a_live_key() {
    let (mut store, allocator) = setup();
    let key = owned(&mut store, &allocator, &Deaths::default());
    let owners = Deaths::default();
    let values = Deaths::default();
    {
        let mut scope = RootScope::new(&mut store);
        let owner = object(&mut scope, &allocator, &owners);
        let value = object(&mut scope, &allocator, &values);
        GcEphemeronTable::initialize(&mut scope, &owner).unwrap();
        GcEphemeronTable::set(&mut scope, &owner, &key, &value).unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(owners.count(), 1);
    assert_eq!(values.count(), 1);
}

#[test]
fn ephemeron_chains_reach_a_fixed_point_before_weak_references_clear() {
    let (mut store, allocator) = setup();
    let table = owned(&mut store, &allocator, &Deaths::default());
    let observer = owned(&mut store, &allocator, &Deaths::default());
    GcEphemeronTable::initialize(&mut store, &table).unwrap();
    let deaths = Deaths::default();
    let first = {
        let mut scope = RootScope::new(&mut store);
        let chain = (0..64)
            .map(|_| object(&mut scope, &allocator, &deaths))
            .collect::<Vec<_>>();
        for pair in chain.windows(2).rev() {
            GcEphemeronTable::set(&mut scope, &table, &pair[0], &pair[1]).unwrap();
        }
        GcWeakRef::initialize(&mut scope, &observer, chain.last().unwrap()).unwrap();
        chain[0].to_owned_rooted(&mut scope).unwrap()
    };
    for _ in 0..4 {
        GcWeakHeap::clear_kept_objects(&mut store).unwrap();
        store.gc(None).unwrap();
        assert_eq!(deaths.count(), 0);
        let mut scope = RootScope::new(&mut store);
        assert!(GcWeakRef::get(&mut scope, &observer).unwrap().is_some());
    }
    GcEphemeronTable::delete(&mut store, &table, &first).unwrap();
    GcWeakHeap::clear_kept_objects(&mut store).unwrap();
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 63);
    assert!(GcWeakRef::get(&mut store, &observer).unwrap().is_none());
    drop(first);
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 64);
}

#[test]
fn ephemeron_value_can_activate_a_second_weak_owner() {
    let (mut store, allocator) = setup();
    let first = owned(&mut store, &allocator, &Deaths::default());
    let key_a = owned(&mut store, &allocator, &Deaths::default());
    let key_b = owned(&mut store, &allocator, &Deaths::default());
    GcEphemeronTable::initialize(&mut store, &first).unwrap();
    let owners = Deaths::default();
    let values = Deaths::default();
    {
        let mut scope = RootScope::new(&mut store);
        let second = object(&mut scope, &allocator, &owners);
        let value = object(&mut scope, &allocator, &values);
        GcEphemeronTable::initialize(&mut scope, &second).unwrap();
        GcEphemeronTable::set(&mut scope, &first, &key_a, &second).unwrap();
        GcEphemeronTable::set(&mut scope, &second, &key_b, &value).unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(values.count(), 0);
    {
        let mut scope = RootScope::new(&mut store);
        let second = GcEphemeronTable::get(&mut scope, &first, &key_a)
            .unwrap()
            .unwrap();
        assert!(GcEphemeronTable::get(&mut scope, &second, &key_b)
            .unwrap()
            .is_some());
    }
    drop(key_a);
    store.gc(None).unwrap();
    assert_eq!(owners.count(), 1);
    assert_eq!(values.count(), 1);
}

#[test]
fn ephemeron_replacement_and_delete_release_old_values() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let key = owned(&mut store, &allocator, &Deaths::default());
    let values = Deaths::default();
    GcEphemeronTable::initialize(&mut store, &owner).unwrap();
    for i in 0..3 {
        {
            let mut scope = RootScope::new(&mut store);
            let value = object(&mut scope, &allocator, &values);
            GcEphemeronTable::set(&mut scope, &owner, &key, &value).unwrap();
        }
        store.gc(None).unwrap();
        assert_eq!(values.count(), i);
    }
    assert!(GcEphemeronTable::delete(&mut store, &owner, &key).unwrap());
    assert!(!GcEphemeronTable::delete(&mut store, &owner, &key).unwrap());
    assert!(GcEphemeronTable::get(&mut store, &owner, &key)
        .unwrap()
        .is_none());
    store.gc(None).unwrap();
    assert_eq!(values.count(), 3);
}

#[test]
fn finalization_keeps_holdings_and_queues_cleanup_outside_collection() {
    let (mut store, allocator) = setup();
    let owners = Deaths::default();
    let targets = Deaths::default();
    let holdings = Deaths::default();
    let owner = owned(&mut store, &allocator, &owners);
    GcFinalizationRegistry::initialize(&mut store, &owner).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        let target = object(&mut scope, &allocator, &targets);
        let holding = object(&mut scope, &allocator, &holdings);
        GcFinalizationRegistry::register(&mut scope, &owner, &target, &holding, None).unwrap();
        scope.as_context_mut().gc(None).unwrap();
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_none());
    }
    store.gc(None).unwrap();
    assert_eq!(targets.count(), 1);
    assert_eq!(holdings.count(), 0);
    drop(owner);
    for _ in 0..4 {
        store.gc(None).unwrap();
        assert_eq!(owners.count(), 0, "pending cleanup is a real job root");
        assert_eq!(holdings.count(), 0);
    }
    {
        let mut scope = RootScope::new(&mut store);
        let cleanup = GcWeakHeap::take_cleanup(&mut scope).unwrap().unwrap();
        scope.as_context_mut().gc(None).unwrap();
        assert!(cleanup.holding.is_struct(&scope).unwrap());
        assert!(cleanup.owner.is_struct(&scope).unwrap());
        assert_eq!(holdings.count(), 0);
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_none());
    }
    store.gc(None).unwrap();
    assert_eq!(owners.count(), 1);
    assert_eq!(holdings.count(), 1);
    assert!(GcWeakHeap::take_cleanup(&mut store).unwrap().is_none());
}

#[test]
fn dead_finalization_owner_releases_holdings_without_scheduling_cleanup() {
    let (mut store, allocator) = setup();
    let owners = Deaths::default();
    let targets = Deaths::default();
    let holdings = Deaths::default();
    {
        let mut scope = RootScope::new(&mut store);
        let owner = object(&mut scope, &allocator, &owners);
        let target = object(&mut scope, &allocator, &targets);
        let holding = object(&mut scope, &allocator, &holdings);
        GcFinalizationRegistry::initialize(&mut scope, &owner).unwrap();
        GcFinalizationRegistry::register(&mut scope, &owner, &target, &holding, None).unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(owners.count(), 1);
    assert_eq!(targets.count(), 1);
    assert_eq!(holdings.count(), 1);
    assert!(GcWeakHeap::take_cleanup(&mut store).unwrap().is_none());
}

#[test]
fn finalization_holdings_join_the_fixed_point_before_target_clearing() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let deaths = Deaths::default();
    GcFinalizationRegistry::initialize(&mut store, &owner).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        let first = object(&mut scope, &allocator, &deaths);
        let second = object(&mut scope, &allocator, &deaths);
        let final_holding = AnyRef::from_i31(&mut scope, I31::wrapping_u32(42));
        GcFinalizationRegistry::register(&mut scope, &owner, &first, &second, None).unwrap();
        GcFinalizationRegistry::register(&mut scope, &owner, &second, &final_holding, None)
            .unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(
        deaths.count(),
        1,
        "the first registration keeps the second target alive"
    );
    {
        let mut scope = RootScope::new(&mut store);
        assert!(GcWeakHeap::take_cleanup(&mut scope)
            .unwrap()
            .unwrap()
            .holding
            .is_struct(&scope)
            .unwrap());
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_none());
    }
    store.gc(None).unwrap();
    assert_eq!(deaths.count(), 2);
    {
        let mut scope = RootScope::new(&mut store);
        let cleanup = GcWeakHeap::take_cleanup(&mut scope).unwrap().unwrap();
        assert_eq!(
            cleanup.holding.as_i31(&scope).unwrap().unwrap().get_u32(),
            42
        );
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_none());
    }
}

#[test]
fn unregister_removes_all_cells_and_pending_cleanups_for_a_token() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let token = owned(&mut store, &allocator, &Deaths::default());
    let live_target = owned(&mut store, &allocator, &Deaths::default());
    let holdings = Deaths::default();
    GcFinalizationRegistry::initialize(&mut store, &owner).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        let dead_target = object(&mut scope, &allocator, &Deaths::default());
        for target in [&*live_target, &*dead_target, &*dead_target] {
            let holding = object(&mut scope, &allocator, &holdings);
            GcFinalizationRegistry::register(&mut scope, &owner, target, &holding, Some(&token))
                .unwrap();
        }
    }
    store.gc(None).unwrap();
    assert_eq!(holdings.count(), 0);
    assert!(GcFinalizationRegistry::unregister(&mut store, &owner, &token).unwrap());
    assert!(!GcFinalizationRegistry::unregister(&mut store, &owner, &token).unwrap());
    assert!(GcWeakHeap::take_cleanup(&mut store).unwrap().is_none());
    store.gc(None).unwrap();
    assert_eq!(holdings.count(), 3);
}

#[test]
fn unregister_tokens_are_weak_without_discarding_the_registration() {
    let (mut store, allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let target = owned(&mut store, &allocator, &Deaths::default());
    let tokens = Deaths::default();
    let holdings = Deaths::default();
    GcFinalizationRegistry::initialize(&mut store, &owner).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        let token = object(&mut scope, &allocator, &tokens);
        let holding = object(&mut scope, &allocator, &holdings);
        GcFinalizationRegistry::register(&mut scope, &owner, &target, &holding, Some(&token))
            .unwrap();
    }
    store.gc(None).unwrap();
    assert_eq!(tokens.count(), 1);
    assert_eq!(holdings.count(), 0);
    drop(target);
    store.gc(None).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_some());
    }
    store.gc(None).unwrap();
    assert_eq!(holdings.count(), 1);
}

#[test]
fn cleanup_can_unregister_its_remaining_holdings_without_consuming_another_registry() {
    let (mut store, allocator) = setup();
    let first = owned(&mut store, &allocator, &Deaths::default());
    let second = owned(&mut store, &allocator, &Deaths::default());
    let token = owned(&mut store, &allocator, &Deaths::default());
    let holdings = Deaths::default();
    for registry in [&*first, &*second] {
        GcFinalizationRegistry::initialize(&mut store, registry).unwrap();
        let mut scope = RootScope::new(&mut store);
        let target = object(&mut scope, &allocator, &Deaths::default());
        for _ in 0..3 {
            let holding = object(&mut scope, &allocator, &holdings);
            GcFinalizationRegistry::register(&mut scope, registry, &target, &holding, Some(&token))
                .unwrap();
        }
    }
    store.gc(None).unwrap();
    {
        let mut scope = RootScope::new(&mut store);
        let cleanup = GcWeakHeap::take_cleanup(&mut scope).unwrap().unwrap();
        // This models unregister from within the first cleanup callback. It
        // cancels only that owner's still-pending cells, including after moving.
        scope.as_context_mut().gc(None).unwrap();
        assert!(GcFinalizationRegistry::unregister(&mut scope, &cleanup.owner, &token).unwrap());
        assert!(
            GcFinalizationRegistry::take_holding(&mut scope, &cleanup.owner)
                .unwrap()
                .is_none()
        );
        let other = GcWeakHeap::take_cleanup(&mut scope).unwrap().unwrap();
        assert!(!Rooted::ref_eq(&scope, &cleanup.owner, &other.owner).unwrap());
        for _ in 0..2 {
            assert!(
                GcFinalizationRegistry::take_holding(&mut scope, &other.owner)
                    .unwrap()
                    .is_some()
            );
        }
        assert!(
            GcFinalizationRegistry::take_holding(&mut scope, &other.owner)
                .unwrap()
                .is_none()
        );
        assert!(GcWeakHeap::take_cleanup(&mut scope).unwrap().is_none());
    }
    store.gc(None).unwrap();
    assert_eq!(holdings.count(), 6);
}

#[test]
fn native_weak_api_rejects_foreign_unrooted_and_immediate_targets() {
    let (mut store, allocator) = setup();
    let (mut other_store, other_allocator) = setup();
    let owner = owned(&mut store, &allocator, &Deaths::default());
    let foreign = owned(&mut other_store, &other_allocator, &Deaths::default());
    assert!(GcWeakRef::initialize(&mut store, &owner, &foreign).is_err());
    let unrooted = {
        let mut scope = RootScope::new(&mut store);
        object(&mut scope, &allocator, &Deaths::default())
    };
    assert!(GcWeakRef::initialize(&mut store, &owner, &unrooted).is_err());
    let immediate = AnyRef::from_i31(&mut store, I31::wrapping_u32(7));
    assert!(GcWeakRef::initialize(&mut store, &owner, &immediate).is_err());
    GcEphemeronTable::initialize(&mut store, &owner).unwrap();
    assert!(GcWeakRef::get(&mut store, &owner).is_err());
    assert!(GcWeakRef::initialize(&mut store, &owner, &owner).is_err());
    assert!(GcEphemeronTable::set(&mut store, &owner, &immediate, &immediate).is_err());
    GcEphemeronTable::set(&mut store, &owner, &owner, &immediate).unwrap();
    store.gc(None).unwrap();
    let value = GcEphemeronTable::get(&mut store, &owner, &owner)
        .unwrap()
        .unwrap();
    assert_eq!(value.as_i31(&store).unwrap().unwrap().get_u32(), 7);
}

fn native_wasm_weak_lifetime(allocate_to_collect: bool) {
    use wasm_encoder::{
        BlockType, CodeSection, ConstExpr, EntityType, ExportKind, ExportSection, FieldType,
        Function, FunctionSection, GlobalSection, GlobalType, HeapType, ImportSection, Instruction,
        RefType, StorageType, TypeSection, ValType,
    };
    use wasmtime::{Caller, Linker, Module};

    let mut config =
        crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast).unwrap();
    config.gc_heap_initial_size(65536);
    let engine = Engine::new(&config).unwrap();
    let mut store = Store::new(&engine, ());
    store.set_epoch_deadline(u64::MAX / 2);
    let mut types = TypeSection::new();
    types.ty().function([], []); // 0: collection / job boundary
    types.ty().struct_([
        FieldType {
            element_type: StorageType::Val(ValType::Ref(RefType::ANYREF)),
            mutable: true,
        },
        FieldType {
            element_type: StorageType::Val(ValType::Ref(RefType::EXTERNREF)),
            mutable: false,
        },
    ]); // 1: owner or target, with a finalization witness
    types
        .ty()
        .function([ValType::Ref(RefType::EXTERNREF)], [ValType::I32]); // 2
    let non_null_any = ValType::Ref(RefType {
        nullable: false,
        ..RefType::ANYREF
    });
    types.ty().function([non_null_any; 2], []); // 3: initialize
    types
        .ty()
        .function([non_null_any], [ValType::Ref(RefType::ANYREF)]); // 4: get
    let mut imports = ImportSection::new();
    imports.import(
        crate::WASM_HOST_IMPORT_NAMESPACE,
        "gc",
        EntityType::Function(0),
    );
    imports.import("weak_test", "initialize", EntityType::Function(3));
    imports.import("weak_test", "get", EntityType::Function(4));
    imports.import("weak_test", "end_job", EntityType::Function(0));
    let mut functions = FunctionSection::new();
    functions.function(2);
    let mut exports = ExportSection::new();
    exports.export("collect", ExportKind::Func, 0);
    exports.export("run", ExportKind::Func, 4);
    let mut globals = GlobalSection::new();
    globals.global(
        GlobalType {
            val_type: ValType::Ref(RefType::ANYREF),
            mutable: true,
            shared: false,
        },
        &ConstExpr::ref_null(RefType::ANYREF.heap_type),
    );
    let mut body = Function::new_with_locals_types([
        ValType::Ref(RefType {
            nullable: true,
            heap_type: HeapType::Concrete(1),
        }),
        ValType::Ref(RefType {
            nullable: true,
            heap_type: HeapType::Concrete(1),
        }),
        ValType::I32,
    ]);
    // Both identities originate in Wasm. Only owner and target locals root
    // them when the host's native collector suspends this frame.
    body.instruction(&Instruction::RefNull(RefType::ANYREF.heap_type));
    body.instruction(&Instruction::RefNull(RefType::EXTERNREF.heap_type));
    body.instruction(&Instruction::StructNew(1));
    body.instruction(&Instruction::LocalSet(1));
    body.instruction(&Instruction::RefNull(RefType::ANYREF.heap_type));
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::StructNew(1));
    body.instruction(&Instruction::LocalTee(2));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::StructSet {
        struct_type_index: 1,
        field_index: 0,
    });
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::RefAsNonNull);
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::RefAsNonNull);
    body.instruction(&Instruction::Call(1));
    body.instruction(&Instruction::Call(3));
    body.instruction(&Instruction::Call(0));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::RefAsNonNull);
    body.instruction(&Instruction::Call(2));
    body.instruction(&Instruction::RefCastNullable(RefType::EQREF.heap_type));
    body.instruction(&Instruction::LocalGet(2));
    body.instruction(&Instruction::RefEq);
    body.instruction(&Instruction::I32Eqz);
    body.instruction(&Instruction::If(BlockType::Empty));
    body.instruction(&Instruction::Unreachable);
    body.instruction(&Instruction::End);
    body.instruction(&Instruction::RefNull(HeapType::Concrete(1)));
    body.instruction(&Instruction::LocalSet(2));
    body.instruction(&Instruction::Call(3));
    if allocate_to_collect {
        // Exercise allocation-triggered collection as well as explicit gc().
        body.instruction(&Instruction::I32Const(100_000));
        body.instruction(&Instruction::LocalSet(3));
        body.instruction(&Instruction::Loop(BlockType::Empty));
        body.instruction(&Instruction::RefNull(RefType::ANYREF.heap_type));
        body.instruction(&Instruction::RefNull(RefType::EXTERNREF.heap_type));
        body.instruction(&Instruction::StructNew(1));
        body.instruction(&Instruction::GlobalSet(0));
        body.instruction(&Instruction::LocalGet(3));
        body.instruction(&Instruction::I32Const(1));
        body.instruction(&Instruction::I32Sub);
        body.instruction(&Instruction::LocalTee(3));
        body.instruction(&Instruction::BrIf(0));
        body.instruction(&Instruction::End);
    } else {
        body.instruction(&Instruction::Call(0));
    }
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::RefAsNonNull);
    body.instruction(&Instruction::Call(2));
    body.instruction(&Instruction::RefIsNull);
    body.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&body);
    let mut wasm = wasm_encoder::Module::new();
    wasm.section(&types)
        .section(&imports)
        .section(&functions)
        .section(&globals)
        .section(&exports)
        .section(&code);
    let module = Module::new(&engine, wasm.finish()).unwrap();
    let mut linker = Linker::new(&engine);
    crate::gc_host::register(&mut linker).unwrap();
    linker
        .func_wrap(
            "weak_test",
            "initialize",
            |mut caller: Caller<'_, ()>, owner: Rooted<AnyRef>, target: Rooted<AnyRef>| {
                GcWeakRef::initialize(&mut caller, &owner, &target)
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "weak_test",
            "get",
            |mut caller: Caller<'_, ()>, owner: Rooted<AnyRef>| GcWeakRef::get(&mut caller, &owner),
        )
        .unwrap();
    linker
        .func_wrap("weak_test", "end_job", |mut caller: Caller<'_, ()>| {
            GcWeakHeap::clear_kept_objects(&mut caller)
        })
        .unwrap();
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let run = instance
        .get_typed_func::<Option<Rooted<ExternRef>>, i32>(&mut store, "run")
        .unwrap();
    let collect = instance
        .get_typed_func::<(), ()>(&mut store, "collect")
        .unwrap();
    let deaths = Deaths::default();
    {
        let mut scope = RootScope::new(&mut store);
        let witness = ExternRef::new(&mut scope, Witness(deaths.clone())).unwrap();
        assert_eq!(run.call(&mut scope, Some(witness)).unwrap(), 1);
    }
    collect.call(&mut store, ()).unwrap();
    assert_eq!(deaths.count(), 1);
}

#[test]
fn wasm_stack_roots_and_weak_clearing_share_the_real_host_collection() {
    native_wasm_weak_lifetime(false);
}

#[test]
fn allocation_triggered_collection_clears_unreachable_native_weak_cycles() {
    native_wasm_weak_lifetime(true);
}
