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
