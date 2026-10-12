use super::*;
use wasmtime::{GcWeakHeapEpoch, GcWeakKeyFingerprint};

#[test]
fn long_boxed_ephemeron_chains_survive_both_orders_and_release() {
    const LINKS: usize = 20_000;
    for reverse in [false, true] {
        let (mut store, pre) = heap();
        let owner = object(&mut store, &pre, -1);
        let (head, entries, leaf) = {
            let mut roots = RootScope::new(&mut store);
            let keys = (0..=LINKS)
                .map(|index| object(&mut roots, &pre, index as i32))
                .collect::<Vec<_>>();
            let cells = (0..LINKS)
                .map(|index| {
                    let cell = object(&mut roots, &pre, -(index as i32));
                    cell.as_struct(&roots)
                        .unwrap()
                        .unwrap()
                        .set_field(&mut roots, 0, Val::AnyRef(Some(keys[index + 1])))
                        .unwrap();
                    cell
                })
                .collect::<Vec<_>>();
            // Values are cells whose strong child is the next key, as in the
            // actual emitted JS StoredValue representation. Copying a cell
            // alone cannot activate the next conditional edge.
            let mut entries = Vec::with_capacity(LINKS);
            for offset in 0..LINKS {
                let index = if reverse { LINKS - 1 - offset } else { offset };
                entries.push(
                    GcEphemeron::new(&mut roots, &owner, &keys[index], Some(&cells[index]))
                        .unwrap(),
                );
            }
            let leaf = WeakAnyRef::new(&mut roots, &owner, &keys[LINKS]).unwrap();
            (keys[0].to_owned_rooted(&mut roots).unwrap(), entries, leaf)
        };
        for _ in 0..3 {
            store.gc(None).unwrap();
            let mut roots = RootScope::new(&mut store);
            assert!(entries
                .iter()
                .all(|entry| entry.is_live(&mut roots).unwrap()));
            let end = leaf.upgrade(&mut roots).unwrap().unwrap();
            assert_eq!(marker(&mut roots, end), LINKS as i32);
            let first = if reverse {
                entries[LINKS - 1]
            } else {
                entries[0]
            };
            assert!(first.matches_key(&mut roots, &head).unwrap());
        }
        drop(head);
        store.gc(None).unwrap();
        assert!(entries
            .iter()
            .all(|entry| !entry.is_live(&mut store).unwrap()));
        assert!(leaf.upgrade(&mut store).unwrap().is_none());
    }
}

#[test]
fn equal_owner_and_key_activate_each_duplicate_edge_once() {
    let (mut store, pre) = heap();
    let watcher = object(&mut store, &pre, -1);
    let (owner, first, second, membership, weak_a, weak_b) = {
        let mut roots = RootScope::new(&mut store);
        let owner = object(&mut roots, &pre, 10);
        let a = object(&mut roots, &pre, 20);
        let b = object(&mut roots, &pre, 30);
        let first = GcEphemeron::new(&mut roots, &owner, &owner, Some(&a)).unwrap();
        let second = GcEphemeron::new(&mut roots, &owner, &owner, Some(&b)).unwrap();
        let membership = GcEphemeron::new(&mut roots, &owner, &owner, None).unwrap();
        let weak_a = WeakAnyRef::new(&mut roots, &watcher, &a).unwrap();
        let weak_b = WeakAnyRef::new(&mut roots, &watcher, &b).unwrap();
        (
            owner.to_owned_rooted(&mut roots).unwrap(),
            first,
            second,
            membership,
            weak_a,
            weak_b,
        )
    };
    for _ in 0..4 {
        store.gc(None).unwrap();
        let mut roots = RootScope::new(&mut store);
        assert!(first.matches_key(&mut roots, &owner).unwrap());
        assert!(second.matches_key(&mut roots, &owner).unwrap());
        assert!(membership.matches_key(&mut roots, &owner).unwrap());
        assert!(membership.value(&mut roots).unwrap().is_none());
        let a = weak_a.upgrade(&mut roots).unwrap().unwrap();
        let b = weak_b.upgrade(&mut roots).unwrap().unwrap();
        assert_eq!(marker(&mut roots, a), 20);
        assert_eq!(marker(&mut roots, b), 30);
    }
    drop(owner);
    store.gc(None).unwrap();
    for entry in [first, second, membership] {
        assert!(!entry.is_live(&mut store).unwrap());
    }
    assert!(weak_a.upgrade(&mut store).unwrap().is_none());
    assert!(weak_b.upgrade(&mut store).unwrap().is_none());
}

#[test]
fn nonrooting_key_tokens_invalidate_after_every_collection_and_never_alias_stores() {
    let (mut store, pre) = heap();
    assert!(GcWeakHeapEpoch::current(&mut store).is_err());
    let owner = object(&mut store, &pre, 1);
    let key = object(&mut store, &pre, 10);
    let distinct = object(&mut store, &pre, 10);
    let entry = GcEphemeron::new(&mut store, &owner, &key, None).unwrap();
    let token = GcWeakKeyFingerprint::new(&mut store, &key).unwrap();
    assert_eq!(token, GcWeakKeyFingerprint::new(&mut store, &key).unwrap());
    assert_ne!(
        token,
        GcWeakKeyFingerprint::new(&mut store, &distinct).unwrap()
    );
    let mut previous = token;
    for _ in 0..6 {
        store.gc(None).unwrap();
        let current = GcWeakKeyFingerprint::new(&mut store, &key).unwrap();
        assert_ne!(previous.epoch(), current.epoch());
        assert_eq!(entry.key_fingerprint(&mut store).unwrap(), Some(current));
        assert!(entry.matches_key(&mut store, &key).unwrap());
        previous = current;
    }
    let (mut foreign, foreign_pre) = heap();
    let foreign_key = object(&mut foreign, &foreign_pre, 10);
    assert_ne!(
        previous,
        GcWeakKeyFingerprint::new(&mut foreign, &foreign_key).unwrap()
    );
    assert!(GcWeakKeyFingerprint::new(&mut foreign, &key).is_err());
    assert!(entry.key_fingerprint(&mut foreign).is_err());
    let immediate = AnyRef::from_i31(&mut store, I31::wrapping_u32(1));
    assert!(GcWeakKeyFingerprint::new(&mut store, &immediate).is_err());

    let (dead_entry, stale_token) = {
        let mut roots = RootScope::new(&mut store);
        let dead_key = object(&mut roots, &pre, 42);
        let edge = GcEphemeron::new(&mut roots, &owner, &dead_key, None).unwrap();
        (
            edge,
            GcWeakKeyFingerprint::new(&mut roots, &dead_key).unwrap(),
        )
    };
    for _ in 0..4 {
        store.gc(None).unwrap();
        assert!(dead_entry.key_fingerprint(&mut store).unwrap().is_none());
        let mut roots = RootScope::new(&mut store);
        let replacement = object(&mut roots, &pre, 42);
        assert_ne!(
            stale_token,
            GcWeakKeyFingerprint::new(&mut roots, &replacement).unwrap()
        );
    }
    entry.remove(&mut store).unwrap();
    assert!(entry.key_fingerprint(&mut store).unwrap().is_none());
    let before_empty_cycle = GcWeakHeapEpoch::current(&mut store).unwrap();
    store.gc(None).unwrap();
    assert_ne!(
        before_empty_cycle,
        GcWeakHeapEpoch::current(&mut store).unwrap()
    );
}
