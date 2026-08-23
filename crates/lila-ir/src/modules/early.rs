//! Module early errors (16.2 `Module : ModuleBody`, 16.2.2.1, 16.2.3.1).
//!
//! Two different things arrive here, both of which must end up as a
//! `SyntaxError` reported before anything is compiled:
//!
//! * the static-semantics rules that are decidable from the entry tables
//!   [`super::record`] built — duplicate `ExportedNames`, and an
//!   `ExportedBinding` with no matching declaration; and
//! * boa's own module-goal early errors, which abort the parse before a record
//!   exists and reach us as `lila-front`'s structured parse rejection.
//!
//! The second half is the one that changes observable behaviour. The host
//! loader retains `lila_front::parse`'s result beside each dependency, so
//! record construction never parses the same bytes again and a dependency
//! rejection keeps the same code as an entry rejection.
//!
//! Classification happens once in `lila-front`; this layer only maps the
//! resulting closed `ParseCode` into the corresponding IR diagnostic.
//!
//! # What is deliberately *not* checked here
//!
//! `import` / `export` outside the module goal. boa's script parser rejects
//! both keywords outright (`boa_parser-0.21.1/src/parser/mod.rs`, `ScriptParser`
//! never admits a `ModuleItem`), and `lila_front::parse` already turns that
//! failure into a parse diagnostic on the script path. Verified, not
//! duplicated: a second check could only produce a second error for a source
//! that already failed.

use crate::*;

/// The two conditions this module names directly, as `ParseClassified`
/// witnesses.
///
/// `ParseClassified::from_parse_table` is a `const fn` whose `None` arm is a
/// `panic!`, so these two `const` items are *evaluated by rustc*: naming a
/// link-only code here (`ModuleMissingExport`, say) fails to build rather than
/// reporting a `resolution`-kind condition from a `ParseModule`-stage producer.
/// That closes the call-site half of MC4 — assertion P7 constrains what the
/// fragment table can yield, not what a producer can say.
const DUPLICATE_EXPORT: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ModuleDuplicateExport);
const UNDECLARED_EXPORT: ParseClassified =
    ParseClassified::from_parse_table(EarlyErrorCode::ModuleUndeclaredExport);

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
        // The code is named directly. It used to be read back out of a
        // `ModuleLinkErrorIr`, which was the round trip that let this producer
        // and `graph.rs`'s disagree about the *phase* of one condition: 16.2.3.1
        // makes a duplicate `ExportedName` an early error, and
        // `early-dup-export-id.js` is `phase: parse`.
        let error = ModuleLinkErrorIr::DuplicateExport {
            module: record.id,
            export_name,
        };
        diagnostics.push(IrDiagnostic::rejected_at_parse(
            DUPLICATE_EXPORT,
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
    //
    // The comparison is `[[LocalName]]` against `[[LocalName]]`, in one domain.
    // Writing `entry.export_name` here compiled before this area was typed and
    // produced a spurious `SyntaxError` for `export { x as y }`; it is now
    // `E0308: expected LocalName, found ExportName`.
    for entry in &record.local_export_entries {
        if record
            .environment
            .iter()
            .any(|binding| binding.name == entry.local_name)
        {
            continue;
        }
        diagnostics.push(IrDiagnostic::rejected_at_parse(
            UNDECLARED_EXPORT,
            format!(
                "exported binding {} is not declared in module {}",
                entry.local_name.spec_name(),
                record.key.as_str()
            ),
            None,
        ));
    }

    diagnostics
}

/// Maps a retained failed module parse into an IR diagnostic.
///
/// The classification itself is `lila_front::classify_parse_failure`, the
/// **one** fragment table in this workspace. This module used to carry a second
/// copy of it, keyed to the same boa messages and maintained by hand alongside
/// the copy in `lila-front`; the two had drifted in both directions by the
/// time they were merged. See
/// `docs/rust-rewrite/contracts/early-error-taxonomy.md`.
///
/// A recognised static-semantics failure becomes a coded rejection
/// (`SyntaxError`, phase `parse`, via `IrDiagnostic::rejected`). Anything else —
/// a genuine syntax error whose wording we do not model, or a parser abort —
/// stays `Unsupported`, because claiming `SyntaxError` for a source we simply
/// failed to read would turn a compiler gap into a spec claim.
pub(crate) fn module_parse_failure_diagnostic(error: &lila_front::ParseError) -> IrDiagnostic {
    let diagnostic = error.diagnostic();
    match diagnostic.code {
        lila_front::ParseCode::Early(code) => {
            IrDiagnostic::rejected_at_parse(code, diagnostic.message.clone(), diagnostic.span)
        }
        lila_front::ParseCode::Malformed | lila_front::ParseCode::UnsupportedParserFeature => {
            IrDiagnostic::unsupported(diagnostic.message.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(name: &str, kind: ModuleBindingKindIr) -> ModuleEnvBindingIr {
        ModuleEnvBindingIr {
            name: LocalName::from_bound_name(name),
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
            key: ModuleKey::from_host("main.mjs"),
            source_len: 0,
            has_top_level_await: false,
            requested_modules: Vec::new(),
            module_resolution_requests: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
            environment: Vec::new(),
            import_meta_sites: Vec::new(),
            dynamic_import_sites: Vec::new(),
        }
    }

    fn classified_parse_error(message: &str) -> lila_front::ParseError {
        let code = lila_front::classify_parse_failure(message)
            .expect("fixture must name a classified Boa static-semantics failure");
        lila_front::ParseError::early_error(code, message, None)
    }

    #[test]
    fn a_well_formed_record_has_no_early_errors() {
        let mut record = record();
        record
            .environment
            .push(binding("x", ModuleBindingKindIr::Let));
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: LocalName::from_bound_name("x"),
            export_name: ExportName::new("x"),
        });
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name(ExportName::new("y")),
            export_name: ExportName::new("y"),
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
            local_name: LocalName::from_bound_name("x"),
            export_name: ExportName::new("shared"),
        });
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name(ExportName::new("y")),
            export_name: ExportName::new("shared"),
        });

        let diagnostics = module_early_errors(&record);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostics[0].phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostics[0].code(),
            Some(EarlyErrorCode::ModuleDuplicateExport)
        );
        assert_eq!(
            diagnostics[0].error_type(),
            Some(NativeErrorKind::SyntaxError)
        );
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
            local_name: LocalName::from_bound_name("missing"),
            export_name: ExportName::new("missing"),
        });

        let diagnostics = module_early_errors(&record);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code(),
            Some(EarlyErrorCode::ModuleUndeclaredExport)
        );
        assert_eq!(
            diagnostics[0].error_type(),
            Some(NativeErrorKind::SyntaxError)
        );
        assert!(diagnostics[0].message.contains("missing"));
    }

    #[test]
    fn an_import_binding_satisfies_an_exported_binding() {
        // `import * as ns from "m"; export { ns };` keeps a *local* entry whose
        // local name is only ever declared by the import.
        let mut record = record();
        record.environment.push(ModuleEnvBindingIr {
            name: LocalName::from_bound_name("ns"),
            kind: ModuleBindingKindIr::Import,
            mutable: false,
            initialized_before_evaluation: true,
            in_tdz_until_evaluated: false,
            indirect: Some((ModuleRequestIr::plain("./m.mjs"), ImportNameIr::Namespace)),
        });
        record.local_export_entries.push(LocalExportEntryIr {
            local_name: LocalName::from_bound_name("ns"),
            export_name: ExportName::new("ns"),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    #[test]
    fn indirect_entries_are_exempt_from_the_declaration_check() {
        let mut record = record();
        record.indirect_export_entries.push(IndirectExportEntryIr {
            request: ModuleRequestIr::plain("./dep.mjs"),
            import_name: ImportNameIr::Name(ExportName::new("nothing-local")),
            export_name: ExportName::new("nothing-local"),
        });
        assert_eq!(module_early_errors(&record), Vec::new());
    }

    /// Every witness of every row of the one fragment table crosses the typed
    /// front-end-to-IR diagnostic boundary without changing its condition.
    #[test]
    fn boa_static_semantics_messages_classify_as_syntax_errors() {
        let cases = [
            (
                "Duplicate __proto__ fields are not allowed in object literals.",
                EarlyErrorCode::ObjectDuplicateProto,
            ),
            (
                "exported name `x` declared multiple times",
                EarlyErrorCode::ModuleDuplicateExport,
            ),
            (
                "could not find the exported binding `x` in the declared names of the module",
                EarlyErrorCode::ModuleUndeclaredExport,
            ),
            (
                "lexical name `x` declared multiple times",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "lexical name declared multiple times",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "lexical name declared in var names",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "lexical name declared in var declared names",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "invalid scope analysis: duplicate lexical declaration",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "formal parameter `x` declared in lexically declared names",
                EarlyErrorCode::DuplicateLexicalDeclaration,
            ),
            (
                "Duplicate parameter name not allowed in this context",
                EarlyErrorCode::DuplicateFormalParameter,
            ),
            (
                "duplicate parameter name not allowed in unique formal parameters",
                EarlyErrorCode::DuplicateFormalParameter,
            ),
            (
                "duplicate catch parameter identifier",
                EarlyErrorCode::DuplicateCatchParameter,
            ),
            (
                "catch parameter identifier declared in catch body",
                EarlyErrorCode::CatchBodyDeclarationConflict,
            ),
            (
                "a class may only have one constructor",
                EarlyErrorCode::DuplicateClassConstructor,
            ),
            (
                "class constructor may not be a generator method",
                EarlyErrorCode::ClassConstructorGeneratorMethod,
            ),
            (
                "class constructor may not be an async method",
                EarlyErrorCode::ClassConstructorAsyncMethod,
            ),
            (
                "class constructor may not be a getter method",
                EarlyErrorCode::ClassConstructorGetter,
            ),
            (
                "class constructor may not be a setter method",
                EarlyErrorCode::ClassConstructorSetter,
            ),
            (
                "class constructor may not be a private method",
                EarlyErrorCode::ClassPrivateConstructorName,
            ),
            (
                "private identifier has already been declared",
                EarlyErrorCode::ClassDuplicatePrivateName,
            ),
            (
                "class may not have field definitions named 'constructor'",
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                "class may not have static field definitions named 'constructor' or 'prototype'",
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
            (
                "'arguments' not allowed in class static block",
                EarlyErrorCode::ClassStaticBlockContainsArguments,
            ),
            (
                "invalid await usage at line 1, col 1",
                EarlyErrorCode::ClassStaticBlockContainsAwait,
            ),
            (
                "'arguments' not allowed in class field definition",
                EarlyErrorCode::ClassFieldContainsArguments,
            ),
            (
                "with statement not allowed in strict mode",
                EarlyErrorCode::StrictModeWithStatement,
            ),
            (
                "module cannot contain `super` on the top-level",
                EarlyErrorCode::ModuleTopLevelSuper,
            ),
            (
                "module cannot contain `new.target` on the top-level",
                EarlyErrorCode::ModuleTopLevelNewTarget,
            ),
            (
                "invalid private identifier usage",
                EarlyErrorCode::InvalidPrivateIdentifier,
            ),
            ("duplicate label: lbl", EarlyErrorCode::DuplicateLabel),
            (
                "undefined break target: lbl",
                EarlyErrorCode::UndefinedBreakTarget,
            ),
            (
                "undefined continue target: lbl",
                EarlyErrorCode::UndefinedContinueTarget,
            ),
            ("illegal break statement", EarlyErrorCode::IllegalBreak),
            (
                "illegal continue statement",
                EarlyErrorCode::IllegalContinue,
            ),
        ];
        for (boa_message, code) in cases {
            let error = classified_parse_error(boa_message);
            let diagnostic = module_parse_failure_diagnostic(&error);
            assert_eq!(
                diagnostic.kind,
                IrDiagnosticKind::EarlyError,
                "{boa_message}"
            );
            assert_eq!(
                diagnostic.phase(),
                IrDiagnosticPhase::Early,
                "{boa_message}"
            );
            assert_eq!(diagnostic.code(), Some(code), "{boa_message}");
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{boa_message}"
            );
        }
    }

    #[test]
    fn duplicate_formal_parameter_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "function duplicate(a, a) {}",
            lila_front::ParseOptions::module(),
        )
        .expect_err("module code is strict, so duplicate formal parameters should fail");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::DuplicateFormalParameter)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn duplicate_catch_parameter_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "try {} catch ({ a, b: a }) {}",
            lila_front::ParseOptions::module(),
        )
        .expect_err("duplicate BoundNames in a catch parameter should fail");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::DuplicateCatchParameter)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn catch_body_declaration_conflicts_map_to_early_syntax_errors() {
        for source in [
            "try {} catch (a) { let a; }",
            "try {} catch ({ a }) { var a; }",
        ] {
            let error = lila_front::parse(source, lila_front::ParseOptions::module())
                .expect_err("catch parameter/body declaration conflict should fail");
            let diagnostic = module_parse_failure_diagnostic(&error);

            assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError, "{source:?}");
            assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early, "{source:?}");
            assert_eq!(
                diagnostic.code(),
                Some(EarlyErrorCode::CatchBodyDeclarationConflict),
                "{source:?}"
            );
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{source:?}"
            );
            assert!(diagnostic.span.is_some(), "{source:?}: {diagnostic:?}");
        }
    }

    #[test]
    fn duplicate_class_constructor_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "class C { constructor() {} constructor() {} }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("a class may not contain two ordinary constructors");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::DuplicateClassConstructor)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn class_constructor_generator_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "class C { async *constructor() {} }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("a non-static class constructor may not be a generator method");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ClassConstructorGeneratorMethod)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn remaining_class_constructor_restrictions_map_to_early_syntax_errors() {
        for (source, code) in [
            (
                "class C { async constructor() {} }",
                EarlyErrorCode::ClassConstructorAsyncMethod,
            ),
            (
                "class C { get constructor() {} }",
                EarlyErrorCode::ClassConstructorGetter,
            ),
            (
                "class C { set constructor(value) {} }",
                EarlyErrorCode::ClassConstructorSetter,
            ),
            (
                "class C { static async *#constructor() {} }",
                EarlyErrorCode::ClassPrivateConstructorName,
            ),
        ] {
            let error = lila_front::parse(source, lila_front::ParseOptions::module())
                .expect_err("a forbidden class constructor form should fail before evaluation");
            let diagnostic = module_parse_failure_diagnostic(&error);

            assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError, "{source:?}");
            assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early, "{source:?}");
            assert_eq!(diagnostic.code(), Some(code), "{source:?}");
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{source:?}"
            );
            assert!(diagnostic.span.is_some(), "{source:?}: {diagnostic:?}");
        }
    }

    #[test]
    fn duplicate_class_private_name_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "export default class { #x; static #x() {} }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("a class may not declare the same private name twice");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ClassDuplicatePrivateName)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn class_field_literal_name_module_parse_maps_to_early_syntax_errors() {
        for (source, code) in [
            (
                r#"export default class { accessor "constructor"; }"#,
                EarlyErrorCode::ClassFieldConstructorName,
            ),
            (
                r#"export default class { static accessor "prototype" = 1; }"#,
                EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName,
            ),
        ] {
            let error = lila_front::parse(source, lila_front::ParseOptions::module())
                .expect_err("a forbidden literal class-field name should fail");
            let diagnostic = module_parse_failure_diagnostic(&error);

            assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError, "{source:?}");
            assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early, "{source:?}");
            assert_eq!(diagnostic.code(), Some(code), "{source:?}");
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{source:?}"
            );
            assert!(diagnostic.span.is_some(), "{source:?}: {diagnostic:?}");
        }
    }

    #[test]
    fn strict_mode_with_statement_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse("with ({}) {}", lila_front::ParseOptions::module())
            .expect_err("Module code is strict without a directive");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::StrictModeWithStatement)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn class_static_block_arguments_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "class C { static { arguments; } }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("lexical arguments use in a class static block should fail");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ClassStaticBlockContainsArguments)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn class_static_block_await_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "export async function outer() { class C { static { await 0; } } }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("an AwaitExpression in a class static block should fail");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ClassStaticBlockContainsAwait)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    #[test]
    fn class_field_arguments_module_parse_maps_to_an_early_syntax_error() {
        let error = lila_front::parse(
            "export default class { static accessor #value = () => arguments; }",
            lila_front::ParseOptions::module(),
        )
        .expect_err("lexical arguments use in a class field should fail");
        let diagnostic = module_parse_failure_diagnostic(&error);

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ClassFieldContainsArguments)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        assert!(diagnostic.span.is_some(), "{diagnostic:?}");
    }

    /// Drift B1, closed. A duplicate `__proto__` inside a *dependency* module
    /// used to reach `IrDiagnostic::unsupported` — `code: None`,
    /// `error_type: None` — because this crate's copy of the table had no row
    /// for it at all, while the entry path's copy did.
    #[test]
    fn duplicate_proto_in_a_dependency_module_is_a_syntax_error() {
        let error = classified_parse_error(
            "Duplicate __proto__ fields are not allowed in object literals.",
        );
        let diagnostic = module_parse_failure_diagnostic(&error);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ObjectDuplicateProto),
            "{diagnostic:?}"
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
    }

    /// Drift B2, closed. The block, switch and scope-analysis wordings for a
    /// lexical redeclaration used to miss this crate's copy of the table
    /// entirely and be reported as unsupported, which
    /// `compile_negative_error_matches` rejects outright.
    #[test]
    fn every_lexical_redeclaration_wording_reaches_one_code() {
        for boa_message in [
            "lexical name declared multiple times",
            "lexical name `x` declared multiple times",
            "lexical name declared in var names",
            "lexical name declared in var declared names",
            "invalid scope analysis: duplicate lexical declaration",
            "formal parameter `x` declared in lexically declared names",
        ] {
            let error = classified_parse_error(boa_message);
            let diagnostic = module_parse_failure_diagnostic(&error);
            assert_eq!(
                diagnostic.code(),
                Some(EarlyErrorCode::DuplicateLexicalDeclaration),
                "{boa_message}"
            );
        }
    }

    #[test]
    fn an_unmodelled_parse_failure_stays_unsupported() {
        // Claiming `SyntaxError` for a failure we do not model would dress a
        // compiler gap up as a spec claim.
        let error = lila_front::ParseError::malformed("unexpected token ')'", None);
        let diagnostic = module_parse_failure_diagnostic(&error);
        assert_eq!(diagnostic.kind, IrDiagnosticKind::Unsupported);
        assert_eq!(diagnostic.code(), None);
        assert_eq!(diagnostic.error_type(), None);
    }
}
