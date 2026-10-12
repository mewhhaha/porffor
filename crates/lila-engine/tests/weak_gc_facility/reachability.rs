use super::*;

#[test]
fn a_dead_target_is_cleared_without_retaining_it_strongly() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let weak = {
        let mut roots = RootScope::new(&mut store);
        let target = object(&mut roots, &pre, 42);
        WeakAnyRef::new(&mut roots, &owner, &target).unwrap()
    };
    store.gc(None).unwrap();
    assert!(weak.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn surviving_targets_move_without_losing_identity_or_fields() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let (weak, target) = {
        let mut roots = RootScope::new(&mut store);
        let target = object(&mut roots, &pre, 42);
        let weak = WeakAnyRef::new(&mut roots, &owner, &target).unwrap();
        (weak, target.to_owned_rooted(&mut roots).unwrap())
    };
    for _ in 0..6 {
        store.gc(None).unwrap();
        let mut roots = RootScope::new(&mut store);
        let upgraded = weak.upgrade(&mut roots).unwrap().unwrap();
        assert!(Rooted::ref_eq(&roots, &target, &upgraded).unwrap());
        assert_eq!(marker(&mut roots, upgraded), 42);
    }
    drop(target);
    store.gc(None).unwrap();
    assert!(weak.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn reverse_order_ephemeron_chains_reach_a_fixed_point() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let (a, first, second, weak_c) = {
        let mut roots = RootScope::new(&mut store);
        let a = object(&mut roots, &pre, 10);
        let b = object(&mut roots, &pre, 20);
        let c = object(&mut roots, &pre, 30);
        let second = GcEphemeron::new(&mut roots, &owner, &b, Some(&c)).unwrap();
        let first = GcEphemeron::new(&mut roots, &owner, &a, Some(&b)).unwrap();
        let weak_c = WeakAnyRef::new(&mut roots, &owner, &c).unwrap();
        (
            a.to_owned_rooted(&mut roots).unwrap(),
            first,
            second,
            weak_c,
        )
    };
    for _ in 0..4 {
        store.gc(None).unwrap();
        let mut roots = RootScope::new(&mut store);
        assert!(first.matches_key(&mut roots, &a).unwrap());
        let b = first.value(&mut roots).unwrap().unwrap();
        assert_eq!(marker(&mut roots, b), 20);
        assert!(second.matches_key(&mut roots, &b).unwrap());
        let c = weak_c.upgrade(&mut roots).unwrap().unwrap();
        assert_eq!(marker(&mut roots, c), 30);
    }
    drop(a);
    store.gc(None).unwrap();
    assert!(!first.is_live(&mut store).unwrap());
    assert!(!second.is_live(&mut store).unwrap());
    assert!(weak_c.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn ephemeron_passes_and_weak_updates_cross_increment_boundaries() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let (a, first, second, weak_c) = {
        let mut roots = RootScope::new(&mut store);
        let a = object(&mut roots, &pre, 10);
        let b = object(&mut roots, &pre, 20);
        let c = object(&mut roots, &pre, 30);
        let weak_c = WeakAnyRef::new(&mut roots, &owner, &c).unwrap();
        let second = GcEphemeron::new(&mut roots, &owner, &b, Some(&c)).unwrap();
        // Put the activating edge beyond a whole collector scan increment.
        // Dropped handle copies neither deregister nor strongly retain targets.
        for _ in 0..20_000 {
            WeakAnyRef::new(&mut roots, &owner, &c).unwrap();
        }
        let first = GcEphemeron::new(&mut roots, &owner, &a, Some(&b)).unwrap();
        (
            a.to_owned_rooted(&mut roots).unwrap(),
            first,
            second,
            weak_c,
        )
    };
    store.gc(None).unwrap();
    {
        let mut roots = RootScope::new(&mut store);
        assert!(first.matches_key(&mut roots, &a).unwrap());
        assert!(second.is_live(&mut roots).unwrap());
        let c = weak_c.upgrade(&mut roots).unwrap().unwrap();
        assert_eq!(marker(&mut roots, c), 30);
    }
    drop(a);
    store.gc(None).unwrap();
    assert!(!first.is_live(&mut store).unwrap());
    assert!(!second.is_live(&mut store).unwrap());
    assert!(weak_c.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn an_unreachable_collection_does_not_keep_values_for_live_keys() {
    let (mut store, pre) = heap();
    let watcher = object(&mut store, &pre, 1);
    let key = object(&mut store, &pre, 10);
    let (entry, weak) = {
        let mut roots = RootScope::new(&mut store);
        let owner = object(&mut roots, &pre, 2);
        let value = object(&mut roots, &pre, 30);
        let entry = GcEphemeron::new(&mut roots, &owner, &key, Some(&value)).unwrap();
        let weak = WeakAnyRef::new(&mut roots, &watcher, &value).unwrap();
        (entry, weak)
    };
    store.gc(None).unwrap();
    assert!(!entry.is_live(&mut store).unwrap());
    assert!(weak.upgrade(&mut store).unwrap().is_none());
    assert_eq!(marker(&mut store, key), 10);
}

#[test]
fn a_value_to_key_cycle_cannot_activate_its_own_ephemeron() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let (entry, weak_key, weak_value) = {
        let mut roots = RootScope::new(&mut store);
        let key = object(&mut roots, &pre, 10);
        let value = object(&mut roots, &pre, 20);
        value
            .as_struct(&roots)
            .unwrap()
            .unwrap()
            .set_field(&mut roots, 0, Val::AnyRef(Some(key)))
            .unwrap();
        let entry = GcEphemeron::new(&mut roots, &owner, &key, Some(&value)).unwrap();
        let weak_key = WeakAnyRef::new(&mut roots, &owner, &key).unwrap();
        let weak_value = WeakAnyRef::new(&mut roots, &owner, &value).unwrap();
        (entry, weak_key, weak_value)
    };
    store.gc(None).unwrap();
    assert!(!entry.is_live(&mut store).unwrap());
    assert!(weak_key.upgrade(&mut store).unwrap().is_none());
    assert!(weak_value.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn repeated_strong_and_ephemeron_cycles_are_reclaimed_under_a_fixed_heap_cap() {
    use wasmtime::{StoreLimits, StoreLimitsBuilder};

    const HEAP_CAP: usize = 64 * 1024;
    const ROUNDS: usize = 128;
    const CYCLES_PER_ROUND: usize = 32;

    fn allocate(
        mut store: impl AsContextMut<Data = StoreLimits>,
        pre: &StructRefPre,
        marker: i32,
    ) -> Rooted<AnyRef> {
        StructRef::new(&mut store, pre, &[Val::AnyRef(None), Val::I32(marker)])
            .unwrap()
            .to_anyref()
    }

    fn connect(
        mut store: impl AsContextMut<Data = StoreLimits>,
        from: Rooted<AnyRef>,
        to: Option<Rooted<AnyRef>>,
    ) {
        from.as_struct(&store)
            .unwrap()
            .unwrap()
            .set_field(&mut store, 0, Val::AnyRef(to))
            .unwrap();
    }

    let mut config = Config::new();
    config
        .wasm_gc(true)
        .wasm_function_references(true)
        .collector(Collector::Copying)
        .gc_heap_initial_size(HEAP_CAP as u64);
    let engine = Engine::new(&config).unwrap();
    let ty = StructType::new(
        &engine,
        [
            FieldType::new(Mutability::Var, StorageType::ValType(ValType::EQREF)),
            FieldType::new(Mutability::Const, StorageType::ValType(ValType::I32)),
        ],
    )
    .unwrap();
    let mut store = Store::new(
        &engine,
        StoreLimitsBuilder::new().memory_size(HEAP_CAP).build(),
    );
    store.limiter(|limits| limits);
    let pre = StructRefPre::new(&mut store, ty);
    let owner = allocate(&mut store, &pre, 1);
    let live = {
        let mut roots = RootScope::new(&mut store);
        let a = allocate(&mut roots, &pre, 10);
        let b = allocate(&mut roots, &pre, 20);
        connect(&mut roots, a, Some(b));
        connect(&mut roots, b, Some(a));
        connect(&mut roots, owner, Some(a));
        WeakAnyRef::new(&mut roots, &owner, &a).unwrap()
    };

    // More than one heap's worth of cyclic records must be reclaimed. The
    // live owner/cycle supplies the opposite witness across every movement.
    for round in 0..ROUNDS {
        let dead = {
            let mut roots = RootScope::new(&mut store);
            (0..CYCLES_PER_ROUND)
                .map(|index| {
                    let a = allocate(&mut roots, &pre, index as i32);
                    let b = allocate(&mut roots, &pre, -(index as i32));
                    connect(&mut roots, a, Some(b));
                    connect(&mut roots, b, Some(a));
                    let entry = GcEphemeron::new(&mut roots, &owner, &a, Some(&b)).unwrap();
                    let weak = WeakAnyRef::new(&mut roots, &owner, &a).unwrap();
                    (entry, weak)
                })
                .collect::<Vec<_>>()
        };
        store.gc(None).unwrap();
        assert!(store.gc_heap_capacity() <= HEAP_CAP, "round {round}");
        for (entry, weak) in dead {
            assert!(!entry.is_live(&mut store).unwrap(), "round {round}");
            assert!(weak.upgrade(&mut store).unwrap().is_none(), "round {round}");
            entry.remove(&mut store).unwrap();
            weak.remove(&mut store).unwrap();
        }
        let mut roots = RootScope::new(&mut store);
        let a = live.upgrade(&mut roots).unwrap().unwrap();
        let structure = a.as_struct(&roots).unwrap().unwrap();
        assert_eq!(structure.field(&mut roots, 1).unwrap().unwrap_i32(), 10);
        let b = *structure
            .field(&mut roots, 0)
            .unwrap()
            .unwrap_anyref()
            .unwrap();
        let back = *b
            .as_struct(&roots)
            .unwrap()
            .unwrap()
            .field(&mut roots, 0)
            .unwrap()
            .unwrap_anyref()
            .unwrap();
        assert!(Rooted::ref_eq(&roots, &a, &back).unwrap());
    }
    connect(&mut store, owner, None);
    store.gc(None).unwrap();
    assert!(live.upgrade(&mut store).unwrap().is_none());
    assert!(store.gc_heap_capacity() <= HEAP_CAP);
    live.remove(&mut store).unwrap();
}
