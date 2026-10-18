use super::*;
use lila_ir::{HostSurfacePolicy, RuntimeGlobalScriptKind};

fn request(source: &str) -> lila_ir::RuntimeGlobalScriptProgram {
    lila_ir::lower_runtime_global_script(
        RuntimeGlobalScriptKind::RealmScript,
        &source.encode_utf16().collect::<Vec<_>>(),
        HostSurfacePolicy::default(),
    )
    .unwrap()
}

fn words(pool: &StringPool, names: GlobalVarDeclarationNames) -> Vec<u64> {
    let span = names.span();
    let start = pool.compiler_owned_boundary().static_len() + span.start() as usize;
    pool.bytes[start..start + span.length() as usize]
        .chunks_exact(8)
        .map(|word| u64::from_le_bytes(word.try_into().unwrap()))
        .collect()
}

#[test]
fn name_plans_keep_source_order_runtime_domain_and_function_overlap() {
    let program = request("var first, undefined, overlap; function overlap() {}");
    let unit = program
        .program()
        .script
        .as_ref()
        .unwrap()
        .prepared_script_units()
        .find(|unit| unit.id == program.script_id())
        .unwrap();
    let mut pool = StringPool::default();
    pool.intern_string("undefined");
    pool.compiler_owned = pool.boundary();
    assert!(pool.global_var_declaration_tables(unit).is_err());
    pool.collect_global_var_declarations(unit).unwrap();
    let table = pool.global_var_declaration_tables(unit).unwrap();
    assert_eq!(
        words(&pool, table.lexical_checks),
        vec![1_u64 << 32, 0, (1_u64 << 32) | 1]
    );
    assert_eq!(words(&pool, table.variable_bindings), vec![1_u64 << 32, 0]);
    let bytes = pool.bytes.len();
    assert!(pool.collect_global_var_declarations(unit).is_err());
    assert_eq!(
        pool.bytes.len(),
        bytes,
        "duplicate ownership fails before mutation"
    );
}

#[test]
fn a_collected_empty_plan_is_distinct_from_a_missing_plan() {
    let program = request(";");
    let unit = program
        .program()
        .script
        .as_ref()
        .unwrap()
        .prepared_script_units()
        .find(|unit| unit.id == program.script_id())
        .unwrap();
    let mut pool = StringPool::default();
    assert!(pool.global_var_declaration_tables(unit).is_err());
    pool.collect_global_var_declarations(unit).unwrap();
    let table = pool.global_var_declaration_tables(unit).unwrap();
    assert!(words(&pool, table.lexical_checks).is_empty());
    assert!(words(&pool, table.variable_bindings).is_empty());
}

#[test]
fn sloppy_direct_eval_collects_global_branch_names_without_becoming_a_global_frame() {
    let mut descriptor = b"LILADE01".to_vec();
    descriptor.extend_from_slice(&[0, 1, 0, 0]);
    descriptor.extend_from_slice(&0_u64.to_le_bytes());
    let caller = lila_ir::RuntimeDirectEvalCaller::validate(
        lila_ir::RuntimeDirectEvalCallerDescriptor::decode(&descriptor).unwrap(),
        [],
    )
    .unwrap();
    for (source, strict) in [("var direct;", false), ("'use strict'; var direct;", true)] {
        let program = lila_ir::lower_runtime_direct_eval(
            &caller,
            &source.encode_utf16().collect::<Vec<_>>(),
            HostSurfacePolicy::default(),
        )
        .unwrap();
        let unit = program
            .program()
            .script
            .as_ref()
            .unwrap()
            .prepared_script_units()
            .find(|unit| unit.id == program.script_id())
            .unwrap();
        assert!(matches!(&unit.kind, PreparedScriptKind::DirectEval(_)));
        assert_eq!(unit.strict, strict);
        assert!(!unit.has_global_variable_environment());
        let mut pool = StringPool::default();
        pool.collect_global_var_declarations(unit).unwrap();
        if strict {
            assert!(pool.global_var_declaration_tables(unit).is_err());
        } else {
            let table = pool.global_var_declaration_tables(unit).unwrap();
            assert_eq!(words(&pool, table.lexical_checks), vec![1_u64 << 32]);
            assert_eq!(words(&pool, table.variable_bindings), vec![1_u64 << 32]);
        }
    }
}
