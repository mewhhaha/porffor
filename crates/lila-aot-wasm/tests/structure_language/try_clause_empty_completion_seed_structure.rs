use std::collections::BTreeSet;

const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const STATEMENT_COMPLETION_SOURCE: &str =
    include_str!("../../src/control_flow/statement_completion.rs");
const UNDEFINED_STATEMENT_RESULT: &str = "self.emit_statement_result(function);";
const KIND_ONLY_RESET: &str = "self.set_completion_kind(CompletionKind::Normal, function);";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TryClause {
    Try,
    Catch,
    Finally,
}

struct TryClauseEntry {
    function_name: &'static str,
    clause: TryClause,
    start_anchor: &'static str,
    end_anchor: &'static str,
}

const TRY_CLAUSE_ENTRIES: [TryClauseEntry; 15] = [
    TryClauseEntry {
        function_name: "compile_try_catch",
        clause: TryClause::Try,
        start_anchor: ") -> Result<(), EmitError> {",
        end_anchor: "let outer_frame",
    },
    TryClauseEntry {
        function_name: "compile_try_finally",
        clause: TryClause::Try,
        start_anchor: ") -> Result<(), EmitError> {",
        end_anchor: "let _outer_frame",
    },
    TryClauseEntry {
        function_name: "compile_try_catch_finally",
        clause: TryClause::Try,
        start_anchor: ") -> Result<(), EmitError> {",
        end_anchor: "let _outer_frame",
    },
    TryClauseEntry {
        function_name: "compile_try_catch",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_generator_try_catch",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_generator_try_catch_finally",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_async_try_catch_in_source_environment",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_async_try_catch_finally_in_source_environment",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_try_catch_finally",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_generator_try_finally",
        clause: TryClause::Finally,
        start_anchor: "self.emit_push_generator_pending_completion(function)?;",
        end_anchor: "let finalizer_epilogue_frame",
    },
    TryClauseEntry {
        function_name: "compile_generator_try_catch_finally",
        clause: TryClause::Finally,
        start_anchor: "self.emit_push_generator_pending_completion(function)?;",
        end_anchor: "let finalizer_epilogue_frame",
    },
    TryClauseEntry {
        function_name: "compile_async_try_catch_finally_in_source_environment",
        clause: TryClause::Finally,
        start_anchor: "self.emit_push_async_pending_completion(function)?;",
        end_anchor: "let finalizer_epilogue_frame",
    },
    TryClauseEntry {
        function_name: "compile_async_try_finally_in_source_environment",
        clause: TryClause::Finally,
        start_anchor: "self.emit_push_async_pending_completion(function)?;",
        end_anchor: "let finalizer_epilogue_frame",
    },
    TryClauseEntry {
        function_name: "compile_try_finally",
        clause: TryClause::Finally,
        start_anchor: "self.save_current_completion(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_try_catch_finally",
        clause: TryClause::Finally,
        start_anchor: "self.save_current_completion(",
        end_anchor: "self.push_scope();",
    },
];

fn function_source(function_name: &str) -> &str {
    let declaration = format!("fn {function_name}(");
    let declaration_start = CONTROL_FLOW_SOURCE
        .find(&declaration)
        .unwrap_or_else(|| panic!("missing function declaration: {function_name}"));
    let source_after_declaration = &CONTROL_FLOW_SOURCE[declaration_start..];
    let body_start = source_after_declaration
        .find('{')
        .unwrap_or_else(|| panic!("missing function body: {function_name}"));
    let mut depth = 0;

    for (offset, character) in source_after_declaration[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source_after_declaration[..=body_start + offset];
                }
            }
            _ => {}
        }
    }

    panic!("unterminated function body: {function_name}");
}

fn entry_source(entry: &TryClauseEntry) -> &str {
    let function = function_source(entry.function_name);
    function
        .split_once(entry.start_anchor)
        .unwrap_or_else(|| {
            panic!(
                "{} is missing entry anchor: {}",
                entry.function_name, entry.start_anchor
            )
        })
        .1
        .split_once(entry.end_anchor)
        .unwrap_or_else(|| {
            panic!(
                "{} is missing entry boundary after {}: {}",
                entry.function_name, entry.start_anchor, entry.end_anchor
            )
        })
        .0
}

#[test]
fn try_clause_entry_inventory_covers_three_try_six_catch_and_six_finally_paths() {
    assert_eq!(
        TRY_CLAUSE_ENTRIES
            .iter()
            .filter(|entry| entry.clause == TryClause::Try)
            .count(),
        3
    );
    assert_eq!(
        TRY_CLAUSE_ENTRIES
            .iter()
            .filter(|entry| entry.clause == TryClause::Catch)
            .count(),
        6
    );
    assert_eq!(
        TRY_CLAUSE_ENTRIES
            .iter()
            .filter(|entry| entry.clause == TryClause::Finally)
            .count(),
        6
    );

    let identities = TRY_CLAUSE_ENTRIES
        .iter()
        .map(|entry| (entry.function_name, entry.clause))
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), TRY_CLAUSE_ENTRIES.len());

    for (function_name, expected_entries) in [
        ("compile_try_catch", 2),
        ("compile_generator_try_catch", 1),
        ("compile_generator_try_finally", 1),
        ("compile_generator_try_catch_finally", 2),
        ("compile_async_try_catch_in_source_environment", 1),
        ("compile_async_try_finally_in_source_environment", 1),
        ("compile_async_try_catch_finally_in_source_environment", 2),
        ("compile_try_finally", 2),
        ("compile_try_catch_finally", 3),
    ] {
        assert_eq!(
            function_source(function_name)
                .matches(UNDEFINED_STATEMENT_RESULT)
                .count(),
            expected_entries,
            "unexpected undefined statement-result count in {function_name}"
        );
    }
}

#[test]
fn every_try_clause_entry_seeds_an_undefined_statement_result() {
    let seed = STATEMENT_COMPLETION_SOURCE
        .split_once("pub(crate) fn emit_statement_result(")
        .expect("seed owner")
        .1
        .split_once("pub(crate) fn save_current_completion(")
        .expect("seed boundary")
        .0;
    assert_eq!(
        seed.matches("self.completion().value().set_undefined(function);")
            .count(),
        1
    );
    assert_eq!(seed.matches(KIND_ONLY_RESET).count(), 1);
    assert!(seed.find("set_undefined").unwrap() < seed.find(KIND_ONLY_RESET).unwrap());
    for wrapper in [
        "compile_async_try_catch",
        "compile_async_try_finally",
        "compile_async_try_catch_finally",
    ] {
        let source = function_source(wrapper);
        assert!(source.contains("with_checked_async_generator_source_environment"));
        assert_eq!(
            source
                .matches(&format!("builder.{wrapper}_in_source_environment("))
                .count(),
            1
        );
    }
    for entry in &TRY_CLAUSE_ENTRIES {
        let entry = entry_source(entry);
        assert_eq!(
            entry.matches(UNDEFINED_STATEMENT_RESULT).count(),
            1,
            "try-clause entry must publish exactly one undefined statement result"
        );
        assert_eq!(
            entry.matches(KIND_ONLY_RESET).count(),
            0,
            "try-clause entry must not retain the previous payload and tag"
        );
    }
}
