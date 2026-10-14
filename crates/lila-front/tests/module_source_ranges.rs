use boa_ast::ModuleItemSourceSyntax;
use lila_front::{parse, ParseOptions};

fn source_slice(source: &str, span: boa_ast::LinearSpan) -> String {
    String::from_utf16(
        &source
            .encode_utf16()
            .skip(span.start().pos())
            .take(span.end().pos() - span.start().pos())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

#[test]
fn module_ranges_come_from_grammar_and_preserve_asi_boundaries() {
    let source = "// 🦀\r\nimport data from 'm' with {type: 'json'}\nconst value = 1;\nexport {value}\nexport default function /* original */ () {}\n[1].forEach(print);\nexport const last = /export/;";
    let parsed = parse(source, ParseOptions::module()).unwrap();
    parsed
        .as_module()
        .unwrap()
        .with_compiler_session(|module, _| {
            let syntax = module.items().source_syntax();
            assert_eq!(syntax.len(), module.items().items().len());
            let ModuleItemSourceSyntax::Declaration(import) = syntax[0] else {
                panic!("import range");
            };
            assert_eq!(
                source_slice(source, import),
                "import data from 'm' with {type: 'json'}\n"
            );
            let ModuleItemSourceSyntax::Declaration(export) = syntax[2] else {
                panic!("export-list range");
            };
            // The parser consumes the LineTerminator which supplies ASI.
            assert_eq!(source_slice(source, export), "export {value}\n");
            let ModuleItemSourceSyntax::DefaultExport {
                keywords,
                declaration_end: Some(end),
            } = syntax[3]
            else {
                panic!("default range");
            };
            assert_eq!(source_slice(source, keywords), "export default");
            assert_eq!(
                source_slice(source, boa_ast::LinearSpan::new(keywords.end(), end)),
                " function /* original */ () {}"
            );
            let ModuleItemSourceSyntax::ExportKeyword(keyword) = syntax[5] else {
                panic!("export keyword");
            };
            assert_eq!(source_slice(source, keyword), "export");
        });
}

#[test]
fn expression_default_has_no_declaration_termination() {
    let source = "export default (function () {})\nexport const value = 2;";
    let parsed = parse(source, ParseOptions::module()).unwrap();
    parsed
        .as_module()
        .unwrap()
        .with_compiler_session(|module, _| {
            assert!(matches!(
                module.items().source_syntax()[0],
                ModuleItemSourceSyntax::DefaultExport {
                    declaration_end: None,
                    ..
                }
            ));
        });
}

#[test]
fn every_import_form_requires_a_semicolon_or_an_asi_boundary() {
    for declaration in [
        "import 'm'",
        "import value from 'm'",
        "import {value} from 'm'",
        "import * as namespace from 'm'",
        "import value, {other} from 'm'",
        "import source value from 'm'",
        "import defer * as namespace from 'm'",
        "import value from 'm' with {type: 'json'}",
    ] {
        let invalid = format!("{declaration} const next = 1;");
        assert!(
            parse(&invalid, ParseOptions::module()).is_err(),
            "{invalid}"
        );
        for separator in [";", "\n", "\r\n", "/*\u{2028}*/"] {
            let valid = format!("{declaration}{separator}const next = 1;");
            parse(&valid, ParseOptions::module())
                .unwrap_or_else(|error| panic!("{valid}: {error}"));
        }
        parse(declaration, ParseOptions::module())
            .unwrap_or_else(|error| panic!("{declaration}: {error}"));
    }
}

#[test]
fn dynamic_import_heads_come_from_grammar_with_utf16_and_phase_trivia() {
    use boa_ast::{
        expression::ImportCall,
        visitor::{VisitWith, Visitor},
    };
    use std::ops::ControlFlow;
    struct Heads(Vec<boa_ast::LinearSpan>);
    impl<'ast> Visitor<'ast> for Heads {
        type BreakTy = std::convert::Infallible;
        fn visit_import_call(&mut self, call: &'ast ImportCall) -> ControlFlow<Self::BreakTy> {
            self.0.push(call.head_span().expect("parsed head"));
            call.visit_with(self)
        }
    }
    let source = "// 🦀\nif (true) {} /import/.test('import'); import /*a*/ . /*b*/ defer(import.source('m')); const object = { import() {} };";
    let parsed = parse(source, ParseOptions::script()).unwrap();
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            let mut heads = Heads(Vec::new());
            let _ = script.visit_with(&mut heads);
            assert_eq!(
                heads
                    .0
                    .into_iter()
                    .map(|span| source_slice(source, span))
                    .collect::<Vec<_>>(),
                ["import /*a*/ . /*b*/ defer", "import.source"]
            );
        });
}

#[test]
fn source_modifier_lookahead_retains_from_bindings_and_ordinary_default_imports() {
    use boa_ast::{declaration::ImportPhase, ModuleItem};
    for (source, phase, local) in [
        ("import source from from 'm';", ImportPhase::Source, "from"),
        (
            "import source source from 'm';",
            ImportPhase::Source,
            "source",
        ),
        (
            r"import source \u0066rom from 'm';",
            ImportPhase::Source,
            "from",
        ),
        (
            r"import source \u0073ource from 'm';",
            ImportPhase::Source,
            "source",
        ),
        (
            "import source\r\nfrom\u{2028}from 'm';",
            ImportPhase::Source,
            "from",
        ),
        (
            "import source /* binding */ from /* delimiter */ from 'm';",
            ImportPhase::Source,
            "from",
        ),
        ("import source from 'm';", ImportPhase::Evaluation, "source"),
        (
            r"import sour\u0063e from 'm';",
            ImportPhase::Evaluation,
            "source",
        ),
        (
            "import source, {named as value} from 'm';",
            ImportPhase::Evaluation,
            "source",
        ),
        (
            "import source, * as namespace from 'm';",
            ImportPhase::Evaluation,
            "source",
        ),
        ("import from from 'm';", ImportPhase::Evaluation, "from"),
    ] {
        let parsed = parse(source, ParseOptions::module())
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        parsed
            .as_module()
            .unwrap()
            .with_compiler_session(|module, interner| {
                let [ModuleItem::ImportDeclaration(import)] = module.items().items() else {
                    panic!("one original import: {source}");
                };
                assert_eq!(import.request().phase(), phase, "{source}");
                assert_eq!(
                    interner
                        .resolve_expect(import.default().expect("actual local binding").sym())
                        .utf8(),
                    Some(local),
                    "{source}"
                );
                let ModuleItemSourceSyntax::Declaration(span) = module.items().source_syntax()[0]
                else {
                    panic!("original declaration range: {source}");
                };
                assert_eq!(source_slice(source, span), source, "unchanged source range");
            });
    }
}

#[test]
fn source_import_contextual_modifiers_and_delimiters_reject_escapes_and_reserved_bindings() {
    for source in [
        r"import sour\u0063e from from 'm';",
        r"import source from fr\u006fm 'm';",
        r"import source value fr\u006fm 'm';",
        r"import source fr\u006fm 'm';",
        "import source await from 'm';",
        "import source yield from 'm';",
    ] {
        assert!(parse(source, ParseOptions::module()).is_err(), "{source}");
    }
}
