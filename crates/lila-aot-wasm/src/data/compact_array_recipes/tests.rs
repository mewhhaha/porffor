use super::*;
fn literal(expr: ExprIr) -> TypedExpr {
    TypedExpr::from_info(lila_ir::ValueInfo::new(lila_ir::ValueKind::Dynamic), expr)
}
#[test]
fn recipes_keep_parent_first_structure_holes_exact_number_bits_and_string_slots() {
    let text = "Ā\0𝄞";
    let child = literal(ExprIr::ArrayLiteral(vec![
        literal(ExprIr::Number((-0.0f64).to_bits())),
        literal(ExprIr::Number(f64::NAN.to_bits())),
        literal(ExprIr::ArrayHole),
        literal(ExprIr::String(text.into())),
    ]));
    let values = vec![child; 64];
    let mut pool = StringPool::default();
    pool.collect_expr(&literal(ExprIr::ArrayLiteral(values.clone())));
    let recipe = pool.compact_array_recipe(&values).unwrap().unwrap();
    assert_eq!(recipe.depth(), 2);
    let data = &pool.compact_array_recipes.bytes()
        [recipe.offset() as usize..(recipe.offset() + recipe.bytes()) as usize];
    let rows = data
        .chunks_exact(COMPACT_ARRAY_RECIPE_ROW_BYTES as usize)
        .map(|row| {
            (
                u32::from_le_bytes(row[0..4].try_into().unwrap()),
                u64::from_le_bytes(row[8..16].try_into().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(rows[0], (CompactArrayOpcode::Begin.word(), 64));
    assert_eq!(rows[1], (CompactArrayOpcode::Begin.word(), 4));
    assert_eq!(
        rows[2],
        (CompactArrayOpcode::Number.word(), (-0.0f64).to_bits())
    );
    assert_eq!(
        rows[3],
        (CompactArrayOpcode::Number.word(), f64::NAN.to_bits())
    );
    assert_eq!(rows[4], (CompactArrayOpcode::Hole.word(), 0));
    assert_eq!(
        rows[5],
        (
            CompactArrayOpcode::String.word(),
            u64::from(pool.pooled_string_index(text).unwrap().ordinal())
        )
    );
    assert_eq!(rows.last(), Some(&(CompactArrayOpcode::End.word(), 0)));
    assert_eq!(
        pool.runtime_regexp_candidate_literals,
        BTreeSet::from([text.to_string()])
    );
    assert_eq!(
        pool.script_string_literals,
        BTreeSet::from([text.to_string()])
    );
}
#[test]
fn reads_bigints_and_symbols_keep_the_existing_path_and_deep_pure_recipes_are_iterative() {
    for boundary in [
        ExprIr::Identifier("live".into()),
        ExprIr::BigInt(lila_ir::BigIntLiteralIr::from_i64(1)),
        ExprIr::WellKnownSymbol(lila_ir::WellKnownSymbol::Iterator),
    ] {
        let mut values = vec![literal(ExprIr::Number(1.0f64.to_bits())); 128];
        values.push(literal(boundary));
        assert!(CompactArrayPlan::of(&values).is_none());
    }
    let mut child = literal(ExprIr::Number(1.0f64.to_bits()));
    for _ in 0..256 {
        child = literal(ExprIr::ArrayLiteral(vec![child]));
    }
    let values = [child];
    let plan = CompactArrayPlan::of(&values).unwrap();
    assert_eq!(plan.depth, 257);
    assert_eq!(plan.nodes.len(), 515);
}
