use super::*;

#[test]
fn changed_facts_share_until_mutated_and_replacement_preserves_previous_values() {
    let (baseline, clones) = counted_baseline();
    let value = |value| Counted {
        value,
        clones: clones.clone(),
    };
    let mut facts = CatalogOverlay::new(baseline);
    assert!(facts.insert("source".to_string(), value(7)).is_none());
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
    let old = facts.insert("source".to_string(), value(9)).unwrap();
    assert_eq!(old.value, 7);
    assert_eq!(clones.load(Ordering::SeqCst), 2);
    assert_eq!(shared.get("source").unwrap().value, 7);
    let old = facts.insert("source".to_string(), value(10)).unwrap();
    assert_eq!(old.value, 9);
    assert_eq!(clones.load(Ordering::SeqCst), 2);
    let old = facts.insert("a".to_string(), value(11)).unwrap();
    assert_eq!(old.value, 0);
    assert_eq!(clones.load(Ordering::SeqCst), 3);
    assert_eq!(facts.remove("a").unwrap().value, 11);
    assert!(facts.insert("a".to_string(), value(12)).is_none());
    assert_eq!(baseline.get("a").unwrap().value, 0);
}
