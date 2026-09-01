use std::collections::BTreeSet;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const UNDEFINED_STATEMENT_RESULT: &str =
    "self.emit_statement_result(function, ValueKind::Undefined);";
const KIND_ONLY_RESET: &str = "self.set_completion_kind(CompletionKind::Normal, function);";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TryClause {
    Catch,
    Finally,
}

struct TryClauseEntry {
    function_name: &'static str,
    clause: TryClause,
    start_anchor: &'static str,
    end_anchor: &'static str,
}

const TRY_CLAUSE_ENTRIES: [TryClauseEntry; 12] = [
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
        function_name: "compile_async_try_catch",
        clause: TryClause::Catch,
        start_anchor: "self.write_binding_from_locals(",
        end_anchor: "self.push_scope();",
    },
    TryClauseEntry {
        function_name: "compile_async_try_catch_finally",
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
        function_name: "compile_async_try_catch_finally",
        clause: TryClause::Finally,
        start_anchor: "self.emit_push_async_pending_completion(function)?;",
        end_anchor: "let finalizer_epilogue_frame",
    },
    TryClauseEntry {
        function_name: "compile_async_try_finally",
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
fn try_clause_entry_inventory_is_exactly_six_catch_and_six_finally_paths() {
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
        ("compile_try_catch", 1),
        ("compile_generator_try_catch", 1),
        ("compile_generator_try_finally", 1),
        ("compile_generator_try_catch_finally", 2),
        ("compile_async_try_catch", 1),
        ("compile_async_try_finally", 1),
        ("compile_async_try_catch_finally", 2),
        ("compile_try_finally", 1),
        ("compile_try_catch_finally", 2),
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
