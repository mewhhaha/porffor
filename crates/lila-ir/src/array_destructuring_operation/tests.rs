use super::*;

fn binding(name: &str, slot: u32) -> OwnedEnvBindingIr {
    OwnedEnvBindingIr {
        name: name.into(),
        slot,
    }
}

#[test]
fn retained_iterator_requires_one_exact_allocation_by_both_name_and_slot() {
    let iterator = binding("iterator", 0);
    let foreign_name = binding("foreign", 0);
    let foreign_slot = binding("iterator", 1);
    for inventory in [
        vec![],
        vec![iterator.clone(), iterator.clone()],
        vec![iterator.clone(), foreign_name],
        vec![iterator.clone(), foreign_slot],
    ] {
        assert_eq!(
            ArrayIteratorStorageIr::new(iterator.clone(), &inventory),
            Err(ArrayIteratorStorageError::MissingOwnedBinding)
        );
    }
    assert_eq!(
        ArrayIteratorStorageIr::new(iterator.clone(), &[iterator.clone()])
            .unwrap()
            .binding(),
        &iterator
    );
}

#[test]
fn step_and_rest_publish_the_actual_owned_result_before_a_pure_identifier_read() {
    let inventory = [
        binding("iterator", 0),
        binding("step", 1),
        binding("rest", 2),
    ];
    let storage = ArrayIteratorStorageIr::new(inventory[0].clone(), &inventory).unwrap();
    let step =
        ArrayDestructuringOperationIr::step_value(&storage, inventory[1].clone(), &inventory)
            .unwrap();
    let rest =
        ArrayDestructuringOperationIr::rest_array(&storage, inventory[2].clone(), &inventory)
            .unwrap();
    for ((statements, read), result, kind) in [
        (step, &inventory[1], ValueKind::Dynamic),
        (rest, &inventory[2], ValueKind::Array),
    ] {
        let [StatementIr::Lexical {
            mode: BindingMode::Let,
            name,
            init,
        }, StatementIr::ArrayDestructuringOperation(operation)] = statements.as_slice()
        else {
            panic!("actual initialization and private publication")
        };
        assert_eq!(name, &result.name);
        assert_eq!(init, &TypedExpr::undefined());
        assert_eq!(operation.result_binding(), Some(result));
        assert_eq!(operation.storage(), &storage);
        assert_eq!(read.expr, ExprIr::Identifier(result.name.clone()));
        assert_eq!(read.kind, kind);
    }
    let StatementIr::ArrayDestructuringOperation(elision) =
        ArrayDestructuringOperationIr::elision(&storage)
    else {
        panic!("statement-only elision")
    };
    assert_eq!(elision.result_binding(), None);
    assert_eq!(elision.kind(), ArrayDestructuringOperationKindIr::Elision);
}

#[test]
fn result_publication_rejects_missing_duplicated_or_iterator_aliased_cells() {
    let inventory = [binding("iterator", 0), binding("result", 1)];
    let storage = ArrayIteratorStorageIr::new(inventory[0].clone(), &inventory).unwrap();
    for factory in [
        ArrayDestructuringOperationIr::step_value,
        ArrayDestructuringOperationIr::rest_array,
    ] {
        assert_eq!(
            factory(&storage, inventory[0].clone(), &inventory),
            Err(ArrayIteratorStorageError::AliasedResultBinding)
        );
        assert_eq!(
            factory(&storage, inventory[1].clone(), &inventory[..1]),
            Err(ArrayIteratorStorageError::MissingOwnedBinding)
        );
        for alias in [
            binding("foreign", 1),
            binding("result", 2),
            inventory[1].clone(),
        ] {
            let mut duplicate = inventory.to_vec();
            duplicate.push(alias);
            assert_eq!(
                factory(&storage, inventory[1].clone(), &duplicate),
                Err(ArrayIteratorStorageError::MissingOwnedBinding)
            );
        }
    }
}
