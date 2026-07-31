//! Module early errors (16.2 `Module : ModuleBody`, 16.2.2.1, 16.2.3.1).
//!
//! Two different things arrive here, both of which must end up as a
//! `SyntaxError` reported before anything is compiled:
//!
//! * the static-semantics rules that are decidable from the entry tables
//!   [`super::record`] built — duplicate `ExportedNames`, and an
//!   `ExportedBinding` with no matching declaration; and
//! * boa's own module-goal early errors, which abort the parse before a record
//!   exists and reach us only as a message string.
//!
//! The second half is the one that changes observable behaviour. A dependency
//! module never passes through `porffor_front::parse` — the host loader reads
//! its bytes and [`super::record::parse_module_record`] is the only thing that
//! ever parses it — so without this classification a dependency with a
//! duplicate export would be reported as "unsupported" rather than as the
//! `SyntaxError` a `negative: phase: parse` case expects.
//!
//! # What is deliberately *not* checked here
//!
//! `import` / `export` outside the module goal. boa's script parser rejects
//! both keywords outright (`boa_parser-0.21.1/src/parser/mod.rs`, `ScriptParser`
//! never admits a `ModuleItem`), and `porffor_front::parse` already turns that
//! failure into a parse diagnostic on the script path. Verified, not
//! duplicated: a second check could only produce a second error for a source
//! that already failed.

use crate::*;

/// Early errors of `Module : ModuleBody` that are decidable from the entry
/// tables.
///
/// Every rule here is *also* enforced by boa's module parser, so on a source
/// that reached us through boa this returns an empty list. It runs anyway
/// because the tables it checks are built by [`super::record`], not by boa: if
/// `ExportEntriesForModule` ever disagrees with `ExportedNames`, that
/// disagreement has to surface as a compile error rather than as a namespace
/// object with a silently missing or duplicated key.
pub(crate) fn module_early_errors(record: &SourceTextModuleRecordIr) -> Vec<IrDiagnostic> {
    let mut diagnostics = Vec::new();

    // 16.2.3.1: "It is a Syntax Error if the ExportedNames of ModuleItemList
    // contains any duplicate entries."
    for export_name in record.duplicate_export_names() {
        let error = ModuleLinkErrorIr::DuplicateExport {
            module: record.id,
            export_name,
        };
        diagnostics.push(IrDiagnostic::early_error(
            error.code(),
            "SyntaxError",
            error.message(),
            None,
        ));
    }

    // 16.2: "It is a Syntax Error if any element of the ExportedBindings of
    // ModuleItemList does not also occur in either the VarDeclaredNames of
    // ModuleItemList, or the LexicallyDeclaredNames of ModuleItemList."
    //
    // `record.environment` is exactly that union: import bindings (which
    // LexicallyDeclaredNames of `ModuleItem : ImportDeclaration` contributes),
    // every `var` including the ones nested in blocks, every top-level lexical
    // and hoistable declaration, and the `*default*` binding. Membership in it
    // is therefore the rule, stated once.
    //
    // Only *local* export entries carry a `[[LocalName]]`. `export { x } from
    // "m"` and `export * from "m"` bind nothing in this module, and
    // ExportedBindings excludes them for that reason.
    for entry in &record.local_export_entries {
        if record
            .environment
            .iter()
            .any(|binding| binding.name == entry.local_name)
        {
            continue;
        }
        diagnostics.push(IrDiagnostic::early_error(
            "E_MODULE_UNDECLARED_EXPORT",
            "SyntaxError",
            format!(
                "exported binding {} is not declared in module {}",
                entry.local_name, record.key
            ),
            None,
        ));
    }

    diagnostics
}

/// One boa module-goal early error: the fragments its message always contains,
/// and the stable diagnostic code it maps to.
///
/// Matching on message text is unpleasant but unavoidable: `boa_parser` reports
/// every static-semantics failure as a generic `Error::general`/`Error::lex`
/// with no machine-readable kind. The fragments are chosen to be the invariant
/// part of each `format!` in `boa_parser-0.21.1/src/parser/mod.rs`
/// (`ModuleParser::parse`) and `boa_ast-0.21.1/src/operations/mod.rs`
/// (`CheckLabelsError::message`), never the interpolated identifier.
struct ParseFailureRule {
    /// Every fragment that must appear in boa's message for this rule to fire.
    fragments: &'static [&'static str],
    /// Stable diagnostic code.
    code: &'static str,
}

/// Order matters only in that the first match wins; the patterns are disjoint.
const PARSE_FAILURE_RULES: &[ParseFailureRule] = &[
    // "exported name `x` declared multiple times"
    ParseFailureRule {
        fragments: &["exported name", "declared multiple times"],
        code: "E_MODULE_DUPLICATE_EXPORT",
    },
    // "could not find the exported binding `x` in the declared names of the module"
    ParseFailureRule {
        fragments: &["could not find the exported binding"],
        code: "E_MODULE_UNDECLARED_EXPORT",
    },
    // "lexical name `x` declared multiple times"
    ParseFailureRule {
        fragments: &["lexical name", "declared multiple times"],
        code: "E_DUPLICATE_LEXICAL_DECLARATION",
    },
    // "module cannot contain `super` on the top-level"
    ParseFailureRule {
        fragments: &["module cannot contain", "super"],
        code: "E_MODULE_TOP_LEVEL_SUPER",
    },
    // "module cannot contain `new.target` on the top-level"
    ParseFailureRule {
        fragments: &["module cannot contain", "new.target"],
        code: "E_MODULE_TOP_LEVEL_NEW_TARGET",
    },
    ParseFailureRule {
        fragments: &["invalid private identifier usage"],
        code: "E_INVALID_PRIVATE_IDENTIFIER",
    },
    // The four `CheckLabelsError` messages. Codes match the ones
    // `porffor_front::parser_static_semantics_error_code` already uses for the
    // same failures on the script path, so one test262 case cannot be reported
    // under two different codes depending on whether it was an entry or a
    // dependency.
    ParseFailureRule {
        fragments: &["duplicate label"],
        code: "E_DUPLICATE_LABEL",
    },
    ParseFailureRule {
        fragments: &["undefined break target"],
        code: "E_UNDEFINED_BREAK_TARGET",
    },
    ParseFailureRule {
        fragments: &["undefined continue target"],
        code: "E_UNDEFINED_CONTINUE_TARGET",
    },
    ParseFailureRule {
        fragments: &["illegal break statement"],
        code: "E_ILLEGAL_BREAK",
    },
    ParseFailureRule {
        fragments: &["illegal continue statement"],
        code: "E_ILLEGAL_CONTINUE",
    },
];

/// Classifies a failed module reparse.
///
/// A recognised static-semantics failure becomes an `EarlyError` diagnostic
/// (`SyntaxError`, phase `parse`). Anything else — a genuine syntax error whose
/// wording we do not model, or a parser abort — stays `Unsupported`, because
/// claiming `SyntaxError` for a source we simply failed to read would turn a
/// compiler gap into a spec claim.
pub(crate) fn module_parse_failure_diagnostic(message: &str) -> IrDiagnostic {
    for rule in PARSE_FAILURE_RULES {
        if rule
            .fragments
            .iter()
            .all(|fragment| message.contains(fragment))
        {
            return IrDiagnostic::early_error(rule.code, "SyntaxError", message, None);
        }
    }
    IrDiagnostic::unsupported(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(name: &str, kind: ModuleBindingKindIr) -> ModuleEnvBindingIr {
        ModuleEnvBindingIr {
            name: name.to_string(),
            kind,
            mutable: kind != ModuleBindingKindIr::Const,
            initialized_before_evaluation: kind == ModuleBindingKindIr::Function,
            in_tdz_until_evaluated: kind != ModuleBindingKindIr::Function,
            indirect: None,
        }
    }

    fn record() -> SourceTextModuleRecordIr {
        SourceTextModuleRecordIr {
            id: 0,
            key: "main.mjs".to_string(),
            source_len: 0,
            has_top_level_await: false,
            requested_modules: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
            environment: Vec::new(),
            import_meta_sites: Vec::new(),
            dynamic_import_sites: Vec::new(),
        }
    }

    #[test]
    fn a_well_formed_record_has_no_early_errors() {
        let mut record = record();
        record
            .environment
            .push(binding("x", ModuleBindingKindIr::Let));
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: "x".to_string(),
            export_name: "x".to_string(),
        });
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name("y".to_string()),
            export_name: "y".to_string(),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    #[test]
    fn duplicate_export_name_across_entry_kinds_is_an_early_error() {
        let mut record = record();
        record
            .environment
            .push(binding("x", ModuleBindingKindIr::Let));
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: "x".to_string(),
            export_name: "shared".to_string(),
        });
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name("y".to_string()),
            export_name: "shared".to_string(),
        });

        let diagnostics = module_early_errors(&record);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostics[0].phase, IrDiagnosticPhase::Early);
        assert_eq!(diagnostics[0].code, Some("E_MODULE_DUPLICATE_EXPORT"));
        assert_eq!(diagnostics[0].error_type, Some("SyntaxError"));
    }

    #[test]
    fn star_exports_never_participate_in_the_duplicate_check() {
        // `export * from "a"; export * from "b"` can legitimately reach the same
        // name twice; that is an *ambiguity* resolved at link time, not a
        // duplicate `ExportedName`.
        let mut record = record();
        record.star_export_entries.push(StarExportEntryIr {
            request: ModuleRequestIr::plain("./a.mjs"),
        });
        record.star_export_entries.push(StarExportEntryIr {
            request: ModuleRequestIr::plain("./b.mjs"),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    #[test]
    fn exported_binding_without_a_declaration_is_an_early_error() {
        let mut record = record();
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: "missing".to_string(),
            export_name: "missing".to_string(),
        });

        let diagnostics = module_early_errors(&record);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, Some("E_MODULE_UNDECLARED_EXPORT"));
        assert_eq!(diagnostics[0].error_type, Some("SyntaxError"));
        assert!(diagnostics[0].message.contains("missing"));
    }

    #[test]
    fn an_import_binding_satisfies_an_exported_binding() {
        // `import * as ns from "m"; export { ns };` keeps a *local* entry whose
        // local name is only ever declared by the import.
        let mut record = record();
        record.environment.push(ModuleEnvBindingIr {
            name: "ns".to_string(),
            kind: ModuleBindingKindIr::Import,
            mutable: false,
            initialized_before_evaluation: true,
            in_tdz_until_evaluated: false,
            indirect: Some((ModuleRequestIr::plain("./m.mjs"), ImportNameIr::Namespace)),
        });
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: "ns".to_string(),
            export_name: "ns".to_string(),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    #[test]
    fn indirect_entries_are_exempt_from_the_declaration_check() {
        let mut record = record();
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name("nothing-local".to_string()),
            export_name: "nothing-local".to_string(),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    #[test]
    fn boa_static_semantics_messages_classify_as_syntax_errors() {
        let cases = [
            (
                "lowering module reparse failed: exported name `x` declared multiple times",
                "E_MODULE_DUPLICATE_EXPORT",
            ),
            (
                "lowering module reparse failed: could not find the exported binding `x` in the \
                 declared names of the module",
                "E_MODULE_UNDECLARED_EXPORT",
            ),
            (
                "lowering module reparse failed: lexical name `x` declared multiple times",
                "E_DUPLICATE_LEXICAL_DECLARATION",
            ),
            (
                "lowering module reparse failed: module cannot contain `super` on the top-level",
                "E_MODULE_TOP_LEVEL_SUPER",
            ),
            (
                "lowering module reparse failed: module cannot contain `new.target` on the \
                 top-level",
                "E_MODULE_TOP_LEVEL_NEW_TARGET",
            ),
            (
                "lowering module reparse failed: invalid private identifier usage",
                "E_INVALID_PRIVATE_IDENTIFIER",
            ),
            (
                "lowering module reparse failed: undefined break target: loop",
                "E_UNDEFINED_BREAK_TARGET",
            ),
        ];
        for (message, code) in cases {
            let diagnostic = module_parse_failure_diagnostic(message);
            assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError, "{message}");
            assert_eq!(diagnostic.phase, IrDiagnosticPhase::Early, "{message}");
            assert_eq!(diagnostic.code, Some(code), "{message}");
            assert_eq!(diagnostic.error_type, Some("SyntaxError"), "{message}");
        }
    }

    #[test]
    fn an_unmodelled_parse_failure_stays_unsupported() {
        // Claiming `SyntaxError` for a failure we do not model would dress a
        // compiler gap up as a spec claim.
        let diagnostic =
            module_parse_failure_diagnostic("lowering module reparse failed: unexpected token ')'");
        assert_eq!(diagnostic.kind, IrDiagnosticKind::Unsupported);
        assert_eq!(diagnostic.error_type, None);
    }
}
