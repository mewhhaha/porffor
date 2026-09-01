const ANALYSIS_SOURCE: &str = include_str!("../src/analysis.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/language.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/annex-b-direct-function-collection.md");
const TASK: &str = include_str!("../../../tasks/24-globals-errors-annexb-host.md");
const BLOCK_WITNESS: &str = include_str!(
    "../../../test262/vendor/test262/test/annexB/language/global-code/block-decl-global-init.js"
);
const SWITCH_WITNESS: &str = include_str!(
    "../../../test262/vendor/test262/test/annexB/language/global-code/switch-case-global-init.js"
);
const TRY_WITNESS: &str = include_str!(
    "../../../test262/vendor/test262/test/annexB/language/global-code/block-decl-global-no-skip-try.js"
);

fn balanced_block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let marker_offset = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    let body_offset = source[marker_offset..]
        .find('{')
        .map(|offset| marker_offset + offset)
        .unwrap_or_else(|| panic!("missing body after marker: {marker}"));
    let mut depth = 0_u32;
    for (offset, byte) in source.as_bytes()[body_offset..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[marker_offset..=body_offset + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated body after marker: {marker}");
}

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier marker: {earlier}"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later marker: {later}"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn annex_b_direct_function_collection_is_private_non_derived_and_exhaustive() {
    assert!(ANALYSIS_SOURCE.contains(
        "pub(crate) struct AnnexBFunctionPlan {\n    pub(crate) owner_id: String,\n    pub(crate) source_name: String,\n    pub(crate) block_storage_name: String,\n    pub(crate) copy_to_variable_environment: bool,\n}\n\nenum AnnexBDirectFunctionCollection {"
    ));
    let domain = balanced_block_after(ANALYSIS_SOURCE, "enum AnnexBDirectFunctionCollection {");
    assert_eq!(
        code_without_whitespace(domain),
        "enumAnnexBDirectFunctionCollection{Skip,Record,}"
    );
    for forbidden in ["#[derive", "pub ", "pub(crate)", "impl ", "Default"] {
        assert!(
            !domain.contains(forbidden),
            "forbidden domain text: {forbidden}"
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "AnnexBDirectFunctionCollection"),
        10,
        "the declaration, parameter, six producers and two match arms must own every domain mention"
    );
    for forbidden in [
        "impl Copy for AnnexBDirectFunctionCollection",
        "impl Clone for AnnexBDirectFunctionCollection",
        "impl PartialEq for AnnexBDirectFunctionCollection",
        "impl Eq for AnnexBDirectFunctionCollection",
        "impl Default for AnnexBDirectFunctionCollection",
    ] {
        assert!(!ANALYSIS_SOURCE.contains(forbidden));
    }

    let consumer = balanced_block_after(ANALYSIS_SOURCE, "fn collect_annex_b_nested_items(");
    assert!(consumer.contains("direct_function_collection: AnnexBDirectFunctionCollection,"));
    let policy = balanced_block_after(consumer, "match direct_function_collection {");
    assert_eq!(
        code_without_whitespace(policy),
        "matchdirect_function_collection{AnnexBDirectFunctionCollection::Skip=>{}AnnexBDirectFunctionCollection::Record=>{letdirect_functions=items.iter().filter_map(|item|matchitem{StatementListItem::Declaration(declaration)=>matchdeclaration.as_ref(){Declaration::FunctionDeclaration(function)=>Some(function),_=>None,},StatementListItem::Statement(_)=>None,}).collect::<Vec<_>>();self.record_annex_b_direct_functions(owner_id,&direct_functions,eligible_keys,interner,);}}"
    );
    assert_eq!(
        consumer
            .matches("self.record_annex_b_direct_functions(")
            .count(),
        1
    );
    assert_eq!(
        consumer.matches("match direct_function_collection").count(),
        1
    );
    for forbidden in [
        "record_direct_declarations",
        "if direct_function_collection",
        "unreachable!",
        "default",
        "==",
        "!=",
    ] {
        assert!(
            !consumer.contains(forbidden),
            "forbidden consumer fallback: {forbidden}"
        );
    }
    assert_before(
        consumer,
        "match direct_function_collection",
        "for item in items",
    );
}

#[test]
fn exactly_six_named_producers_preserve_owner_switch_and_recursive_order() {
    assert_eq!(
        ANALYSIS_SOURCE
            .matches("collect_annex_b_nested_items(")
            .count(),
        7,
        "one declaration and six calls own the complete policy census"
    );
    assert_eq!(
        ANALYSIS_SOURCE
            .matches("AnnexBDirectFunctionCollection::Skip")
            .count(),
        3,
        "one match arm plus owner and switch producers"
    );
    assert_eq!(
        ANALYSIS_SOURCE
            .matches("AnnexBDirectFunctionCollection::Record")
            .count(),
        5,
        "one match arm plus block, try, catch and finally producers"
    );

    let owner = balanced_block_after(ANALYSIS_SOURCE, "fn collect_owner_annex_b_function_plans(");
    assert!(owner.contains(
        "self.collect_annex_b_nested_items(\n            owner_id,\n            items,\n            &eligible_keys,\n            interner,\n            AnnexBDirectFunctionCollection::Skip,\n        );"
    ));
    assert_before(
        owner,
        "if owner_strict",
        "AnnexBDirectFunctionCollection::Skip",
    );

    let recursive = balanced_block_after(ANALYSIS_SOURCE, "fn collect_annex_b_nested_statement(");
    let normalized = code_without_whitespace(recursive);
    for producer in [
        "Statement::Block(block)=>self.collect_annex_b_nested_items(owner_id,block.statement_list().statements(),eligible_keys,interner,AnnexBDirectFunctionCollection::Record,),",
        "self.collect_annex_b_nested_items(owner_id,case.body().statements(),eligible_keys,interner,AnnexBDirectFunctionCollection::Skip,);",
        "self.collect_annex_b_nested_items(owner_id,statement.block().statement_list().statements(),eligible_keys,interner,AnnexBDirectFunctionCollection::Record,);",
        "self.collect_annex_b_nested_items(owner_id,catch.block().statement_list().statements(),eligible_keys,interner,AnnexBDirectFunctionCollection::Record,);",
        "self.collect_annex_b_nested_items(owner_id,finally.block().statement_list().statements(),eligible_keys,interner,AnnexBDirectFunctionCollection::Record,);",
    ] {
        assert!(normalized.contains(producer), "missing producer: {producer}");
    }
    assert_before(
        recursive,
        "let direct_functions = statement",
        "for case in statement.cases()",
    );
    assert_before(
        recursive,
        "self.record_annex_b_direct_functions(",
        "for case in statement.cases()",
    );
    let aggregation_start = recursive
        .find("let direct_functions = statement")
        .expect("missing switch-wide direct-function aggregation");
    let aggregation_end = recursive[aggregation_start..]
        .find("for case in statement.cases()")
        .map(|offset| aggregation_start + offset)
        .expect("missing switch case recursion after aggregation");
    assert_eq!(
        code_without_whitespace(&recursive[aggregation_start..aggregation_end]),
        "letdirect_functions=statement.cases().iter().flat_map(|case|case.body().statements()).filter_map(|item|matchitem{StatementListItem::Declaration(declaration)=>matchdeclaration.as_ref(){Declaration::FunctionDeclaration(function)=>Some(function),_=>None,},StatementListItem::Statement(_)=>None,}).collect::<Vec<_>>();self.record_annex_b_direct_functions(owner_id,&direct_functions,eligible_keys,interner,);"
    );
    assert_before(
        recursive,
        "statement.block().statement_list().statements()",
        "if let Some(catch)",
    );
    assert_before(recursive, "if let Some(catch)", "if let Some(finally)");
}

#[test]
fn annex_b_collection_contract_names_the_existing_behavioral_witnesses() {
    for test in [
        "annex_b_block_functions_create_undefined_owner_bindings_and_copy_when_selected",
        "annex_b_switch_declarations_share_one_case_block_binding",
        "annex_b_copy_bypasses_a_same_named_catch_binding",
    ] {
        assert!(LIB_SOURCE.contains(&format!("fn {test}()")));
        assert!(CONTRACT.contains(test));
    }
    assert!(CLI_TESTS.contains("fn run_wasm_backend_supports_annex_b_block_functions()"));
    assert!(CONTRACT.contains("run_wasm_backend_supports_annex_b_block_functions"));
    for (path, source) in [
        (
            "annexB/language/global-code/block-decl-global-init.js",
            BLOCK_WITNESS,
        ),
        (
            "annexB/language/global-code/switch-case-global-init.js",
            SWITCH_WITNESS,
        ),
        (
            "annexB/language/global-code/block-decl-global-no-skip-try.js",
            TRY_WITNESS,
        ),
    ] {
        assert!(!source.is_empty(), "empty pinned witness: {path}");
        assert!(CONTRACT.contains(path), "contract omits witness: {path}");
    }
    assert!(CONTRACT.contains("byte-identical"));
    assert!(TASK.contains("`AnnexBDirectFunctionCollection::{Skip, Record}`"));
    assert!(code_without_whitespace(TASK).contains("noemitted-IR,Wasmorconformancechange"));
}
use std::fs;
use std::path::Path;
