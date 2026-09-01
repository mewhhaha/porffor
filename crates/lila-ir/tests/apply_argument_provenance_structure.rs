use std::fs;
use std::path::Path;

const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const CALL_EXPRESSION_SOURCE: &str = include_str!("../src/lowering/call_expression.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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
fn apply_parameter_observations_use_the_dense_argument_authority() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "forwarded_apply_arg_infos"),
        0
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "fn forwarded_apply_args("),
        1
    );

    let authority = bounded(
        LOWERING_SOURCE,
        "    fn forwarded_apply_args(&self, apply_arg: Option<&TypedExpr>) -> Option<Vec<TypedExpr>> {",
        "\n    fn merge_array_species_constructor_this_info",
    );
    assert_eq!(
        normalized(authority),
        normalized(
            r#"
        let Some(apply_arg) = apply_arg else {
            return Some(Vec::new());
        };
        if apply_arg.possible_kinds.is_subset_of(
            KindSet::from_kind(ValueKind::Undefined).union(KindSet::from_kind(ValueKind::Null)),
        ) {
            return Some(Vec::new());
        }
        let ExprIr::ArrayLiteral(elements) = &apply_arg.expr else {
            return None;
        };
        (!elements
            .iter()
            .any(|element| matches!(element.expr, ExprIr::ArrayHole)))
        .then(|| elements.clone())
    }
"#,
        )
    );

    let parameter_observation = bounded(
        CALL_EXPRESSION_SOURCE,
        "                                            let forwarded_args = if matches!(",
        "                                            if let Some(forwarded_args) = forwarded_args {",
    );
    let apply_projection = normalized(
        r#"
        } else {
            self.forwarded_apply_args(args.get(1)).map(
                |forwarded_args| {
                    forwarded_args
                        .iter()
                        .map(TypedExpr::value_info)
                        .collect()
                },
            )
        };
"#,
    );
    assert_eq!(
        normalized(parameter_observation)
            .matches(apply_projection.as_str())
            .count(),
        1
    );
}
