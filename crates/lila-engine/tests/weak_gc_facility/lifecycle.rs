use super::*;

#[test]
fn stale_slot_generations_and_foreign_stores_cannot_alias_new_entries() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let a = object(&mut store, &pre, 10);
    let b = object(&mut store, &pre, 20);
    let old = WeakAnyRef::new(&mut store, &owner, &a).unwrap();
    old.remove(&mut store).unwrap();
    let current = WeakAnyRef::new(&mut store, &owner, &b).unwrap();
    old.remove(&mut store).unwrap();
    assert!(old.upgrade(&mut store).unwrap().is_none());
    let upgraded = current.upgrade(&mut store).unwrap().unwrap();
    assert!(Rooted::ref_eq(&store, &upgraded, &b).unwrap());
    let (mut foreign, _) = heap();
    assert!(current.upgrade(&mut foreign).is_err());
    assert!(WeakAnyRef::new(&mut foreign, &owner, &a).is_err());
    let immediate = AnyRef::from_i31(&mut store, I31::wrapping_u32(1));
    assert!(WeakAnyRef::new(&mut store, &owner, &immediate).is_err());
    assert!(GcEphemeron::new(&mut store, &immediate, &a, Some(&b)).is_err());
}

#[test]
fn job_held_roots_survive_until_the_keep_set_is_cleared() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let (weak, mut kept) = {
        let mut roots = RootScope::new(&mut store);
        let target = object(&mut roots, &pre, 10);
        let weak = WeakAnyRef::new(&mut roots, &owner, &target).unwrap();
        let upgraded = weak.upgrade(&mut roots).unwrap().unwrap();
        (weak, vec![upgraded.to_owned_rooted(&mut roots).unwrap()])
    };
    for _ in 0..4 {
        store.gc(None).unwrap();
        let mut roots = RootScope::new(&mut store);
        assert!(weak.upgrade(&mut roots).unwrap().is_some());
    }
    kept.clear();
    store.gc(None).unwrap();
    assert!(weak.upgrade(&mut store).unwrap().is_none());
}

struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn ephemeron_marking_precedes_externref_host_data_sweeping() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let drops = Arc::new(AtomicUsize::new(0));
    let (key, entry, weak) = {
        let mut roots = RootScope::new(&mut store);
        let key = object(&mut roots, &pre, 10);
        let native = ExternRef::new(&mut roots, Dropped(Arc::clone(&drops))).unwrap();
        let value = AnyRef::convert_extern(&mut roots, native).unwrap();
        let entry = GcEphemeron::new(&mut roots, &owner, &key, Some(&value)).unwrap();
        let weak = WeakAnyRef::new(&mut roots, &owner, &value).unwrap();
        (key.to_owned_rooted(&mut roots).unwrap(), entry, weak)
    };
    store.gc(None).unwrap();
    assert!(entry.is_live(&mut store).unwrap());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(key);
    store.gc(None).unwrap();
    assert!(weak.upgrade(&mut store).unwrap().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn membership_and_nullable_values_have_distinct_lifetimes() {
    let (mut store, pre) = heap();
    let owner = object(&mut store, &pre, 1);
    let key = object(&mut store, &pre, 10);
    let entry = GcEphemeron::new(&mut store, &owner, &key, None).unwrap();
    store.gc(None).unwrap();
    assert!(entry.matches_key(&mut store, &key).unwrap());
    assert!(entry.value(&mut store).unwrap().is_none());
    let weak = {
        let mut roots = RootScope::new(&mut store);
        let value = object(&mut roots, &pre, 20);
        entry.set_value(&mut roots, Some(&value)).unwrap();
        WeakAnyRef::new(&mut roots, &owner, &value).unwrap()
    };
    store.gc(None).unwrap();
    {
        let mut roots = RootScope::new(&mut store);
        let value = weak.upgrade(&mut roots).unwrap().unwrap();
        assert_eq!(marker(&mut roots, value), 20);
    }
    entry.set_value(&mut store, None).unwrap();
    store.gc(None).unwrap();
    assert!(entry.matches_key(&mut store, &key).unwrap());
    assert!(weak.upgrade(&mut store).unwrap().is_none());
}
