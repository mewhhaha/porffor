const SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-iterator-receiver-policy.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn fnv1a(source: &str) -> u64 {
    source.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[test]
fn array_iterator_receiver_policy_is_a_private_two_variant_domain() {
    let variants = bounded(
        SOURCE,
        "enum ArrayIteratorReceiverPolicy {",
        "\n}\n\n/// A standard builtin whose algorithm observes",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();

    assert_eq!(variants, ["GenericArrayLike,", "TypedArray,"]);
    let declaration_offset = SOURCE
        .find("enum ArrayIteratorReceiverPolicy {")
        .expect("Array iterator receiver policy declaration");
    assert_eq!(
        SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!SOURCE.contains(&format!(
            "impl {capability} for ArrayIteratorReceiverPolicy"
        )));
    }
    assert!(!SOURCE.contains("pub enum ArrayIteratorReceiverPolicy"));
    assert!(!SOURCE.contains("pub(crate) enum ArrayIteratorReceiverPolicy"));
    assert_eq!(SOURCE.matches("ArrayIteratorReceiverPolicy").count(), 12);
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("capability-free `ArrayIteratorReceiverPolicy`"));
    }
}

#[test]
fn receiver_validation_and_materialization_project_the_policy_directly() {
    let consumer = bounded(
        SOURCE,
        "    fn compile_array_iterator_method_builtin(",
        "    pub(crate) fn compile_standard_builtin(",
    );

    assert!(consumer.contains("receiver_policy: ArrayIteratorReceiverPolicy,"));
    assert_eq!(consumer.matches("match &receiver_policy {").count(), 2);
    assert_eq!(
        consumer
            .matches("ArrayIteratorReceiverPolicy::GenericArrayLike")
            .count(),
        2
    );
    assert_eq!(
        consumer
            .matches("ArrayIteratorReceiverPolicy::TypedArray")
            .count(),
        2
    );
    assert!(!consumer.contains("validates_typed_array"));
    assert!(!consumer.contains("receiver_policy: bool"));
    assert!(!consumer.contains("matches!(receiver_policy"));
    assert!(!consumer.contains("receiver_policy.clone()"));
    assert!(!consumer.contains("receiver_policy =="));
    assert!(!consumer.contains("receiver_policy !="));
    assert!(!consumer.contains("if receiver_policy"));
    assert!(!consumer.contains("=> true"));
    assert!(!consumer.contains("=> false"));
    assert!(!consumer.contains("_ =>"));
    assert!(!consumer.contains("unreachable!"));

    let validation_match = consumer
        .find("match &receiver_policy {")
        .expect("missing receiver validation projection");
    let typed_array_witness = consumer
        .find("self.emit_typed_array_witness(")
        .expect("missing strict TypedArray validation");
    let materialization_match = consumer
        .rfind("match &receiver_policy {")
        .expect("missing iterator materialization projection");
    assert!(validation_match < typed_array_witness);
    assert!(typed_array_witness < materialization_match);
    assert_eq!(
        consumer
            .matches("self.emit_typed_array_iterator_create_from_locals(")
            .count(),
        2
    );
    assert_eq!(
        consumer
            .matches("self.emit_array_iterator_create_from_locals(")
            .count(),
        1
    );

    let normalized_consumer =
        consumer.replace("match &receiver_policy {", "match receiver_policy {");
    assert_eq!(
        (normalized_consumer.len(), fnv1a(&normalized_consumer)),
        (5741, 0xa6d9_f530_766c_a36a)
    );
}

#[test]
fn exactly_six_builtins_select_three_producers_per_policy() {
    let producer_start = SOURCE
        .find("            StandardBuiltinId::ArrayPrototypeKeys => {")
        .expect("first Array iterator receiver-policy producer");
    let producers = SOURCE[producer_start..]
        .split_once("            StandardBuiltinId::ArrayIteratorIdentity => {")
        .expect("end of Array iterator receiver-policy producers")
        .0;
    let producer_body = bounded(
        SOURCE,
        "            StandardBuiltinId::ArrayPrototypeKeys => {",
        "            StandardBuiltinId::ArrayIteratorIdentity => {",
    );

    assert_eq!(
        producers
            .matches("self.compile_array_iterator_method_builtin(")
            .count(),
        6
    );
    assert_eq!(
        producers
            .matches("ArrayIteratorReceiverPolicy::GenericArrayLike")
            .count(),
        3
    );
    assert_eq!(
        producers
            .matches("ArrayIteratorReceiverPolicy::TypedArray")
            .count(),
        3
    );
    for builtin in [
        "StandardBuiltinId::ArrayPrototypeKeys",
        "StandardBuiltinId::ArrayPrototypeEntries",
        "StandardBuiltinId::ArrayPrototypeValues",
        "StandardBuiltinId::TypedArrayPrototypeKeys",
        "StandardBuiltinId::TypedArrayPrototypeEntries",
        "StandardBuiltinId::TypedArrayPrototypeValues",
    ] {
        assert_eq!(producers.matches(builtin).count(), 1, "builtin `{builtin}`");
    }
    assert!(!producers.contains("validates_typed_array"));
    assert!(!producers.contains("_ =>"));
    assert!(!producers.contains("unreachable!"));
    assert_eq!(
        (producer_body.len(), fnv1a(producer_body)),
        (2008, 0xe19e_e2fc_8619_5a59)
    );
}
