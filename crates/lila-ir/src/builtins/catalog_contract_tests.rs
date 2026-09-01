use super::StandardBuiltinId;

#[test]
fn indexed_receiver_mutation_is_owned_by_the_builtin_catalog() {
    let mutating_builtins = StandardBuiltinId::all_functions()
        .iter()
        .copied()
        .filter(|builtin| builtin.mutates_indexed_receiver())
        .collect::<Vec<_>>();

    assert_eq!(
        mutating_builtins,
        vec![
            StandardBuiltinId::ArrayPrototypeSplice,
            StandardBuiltinId::ArrayPrototypeSort,
            StandardBuiltinId::ArrayPrototypeReverse,
            StandardBuiltinId::ArrayPrototypeCopyWithin,
            StandardBuiltinId::ArrayPrototypePop,
            StandardBuiltinId::ArrayPrototypePush,
            StandardBuiltinId::ArrayPrototypeShift,
            StandardBuiltinId::ArrayPrototypeUnshift,
            StandardBuiltinId::ArrayPrototypeFill,
            StandardBuiltinId::TypedArrayPrototypeCopyWithin,
            StandardBuiltinId::TypedArrayPrototypeSet,
            StandardBuiltinId::TypedArrayPrototypeReverse,
            StandardBuiltinId::TypedArrayPrototypeSort,
        ]
    );
}
