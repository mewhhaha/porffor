use super::*;

#[test]
fn changed_facts_share_until_mutated_and_replacement_preserves_previous_values() {
    let (baseline, clones) = counted_baseline();
    let value = |value| Counted {
        value,
        clones: clones.clone(),
    };
    let mut facts = CatalogOverlay::new(baseline);
    assert!(facts.get("source").is_none());
    facts.insert("source".to_string(), value(7));
    let mut fork = facts.clone();
    assert_eq!(clones.load(Ordering::SeqCst), 0);
    assert!(std::ptr::eq(
        facts.get("source").unwrap(),
        fork.get("source").unwrap()
    ));
    fork.get_mut("source").unwrap().value = 8;
    assert_eq!(clones.load(Ordering::SeqCst), 1);
    assert_eq!(facts.get("source").unwrap().value, 7);
    assert_eq!(fork.get("source").unwrap().value, 8);
    let shared = facts.clone();
    facts.insert("source".to_string(), value(9));
    assert_eq!(facts.get("source").unwrap().value, 9);
    assert_eq!(clones.load(Ordering::SeqCst), 1);
    assert_eq!(shared.get("source").unwrap().value, 7);
    facts.insert("source".to_string(), value(10));
    assert_eq!(facts.get("source").unwrap().value, 10);
    assert_eq!(clones.load(Ordering::SeqCst), 1);
    facts.insert("a".to_string(), value(11));
    assert_eq!(facts.get("a").unwrap().value, 11);
    assert_eq!(clones.load(Ordering::SeqCst), 1);
    assert_eq!(facts.remove("a").unwrap().value, 11);
    assert!(facts.get("a").is_none());
    facts.insert("a".to_string(), value(12));
    assert_eq!(baseline.get("a").unwrap().value, 0);

    // Insertion must remain available for facts that cannot be cloned at all.
    // A discarded recipe or shared previous row never requires ownership.
    struct NonClone(usize);
    let baseline = Box::leak(Box::new(BTreeMap::from([("x".to_string(), NonClone(1))])));
    let mut facts = CatalogOverlay::new(baseline);
    facts.insert("x".to_string(), NonClone(2));
    facts.insert("x".to_string(), NonClone(3));
    assert_eq!(facts.get("x").unwrap().0, 3);
    assert_eq!(baseline.get("x").unwrap().0, 1);
}
