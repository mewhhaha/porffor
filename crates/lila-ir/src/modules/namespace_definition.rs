//! Trusted linker definitions that survive parsing the merged Script.

use super::default_export_definition::DefaultExportDefinitions;
use super::evaluation_mode::ModuleMaterializationModeIr;
use super::record::DefaultExportFormIr;
use crate::*;

#[derive(Debug, Default)]
pub(crate) struct LinkedScriptDefinitions {
    defaults: DefaultExportDefinitions,
    namespaces: BTreeMap<(boa_ast::Position, boa_ast::Position), ModuleNamespaceModeIr>,
}

impl LinkedScriptDefinitions {
    pub(super) fn rewrite_body(
        source: &str,
        rewrite: super::source::DefaultExportRewrite<'_>,
    ) -> Result<String, String> {
        DefaultExportDefinitions::rewrite_body(source, rewrite)
    }

    pub(super) fn record_body(
        &mut self,
        body: &str,
        module: ModuleUnitId,
        mode: ModuleMaterializationModeIr,
        form: DefaultExportFormIr,
        preceding_source: &str,
    ) -> Result<(), String> {
        self.defaults
            .record_body(body, module, mode, form, preceding_source)
    }

    pub(super) fn record_namespaces(&mut self, prelude: &str, graph: &ModuleGraphIr) {
        let mut expected = graph
            .materialized_units()
            .filter_map(|(_, mode, unit)| {
                unit.namespace.as_ref().map(|namespace| {
                    (
                        namespace.cell.as_str().to_string(),
                        match mode {
                            ModuleMaterializationModeIr::Eager => ModuleNamespaceModeIr::Eager,
                            ModuleMaterializationModeIr::Deferred => {
                                ModuleNamespaceModeIr::Deferred
                            }
                        },
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        if expected.is_empty() {
            return;
        }
        let ParsedSource::Script(parsed) =
            lila_front::parse(prelude, lila_front::ParseOptions::script())
                .expect("trusted namespace prelude parses")
        else {
            unreachable!("Script parser yields Script")
        };
        parsed.with_compiler_session(|script, interner| {
            for statement in script.statements().statements() {
                let StatementListItem::Declaration(declaration) = statement else {
                    continue;
                };
                let Declaration::Lexical(declaration) = declaration.as_ref() else {
                    continue;
                };
                for variable in declaration.variable_list().as_ref() {
                    let Binding::Identifier(binding) = variable.binding() else {
                        continue;
                    };
                    let name = interner.resolve_expect(binding.sym()).to_string();
                    let Some(mode) = expected.remove(&name) else {
                        continue;
                    };
                    let Some(Expression::ArrayLiteral(array)) =
                        variable.init().map(Expression::flatten)
                    else {
                        panic!("trusted namespace initializer must be an export-reader array")
                    };
                    assert!(
                        array.as_ref().len() % 2 == 1,
                        "namespace table has evaluator and key/reader pairs"
                    );
                    let span = array.span();
                    self.namespaces.insert((span.start(), span.end()), mode);
                }
            }
        });
        assert!(
            expected.is_empty(),
            "every observed namespace has a trusted initializer"
        );
    }

    pub(super) fn prepend(&mut self, prefix: &str) {
        self.defaults.prepend(prefix);
        // Linker wrappers are ASCII and terminate in LF. A non-terminated
        // prefix would need to adjust first-line columns as well.
        assert!(prefix.is_ascii() && prefix.ends_with('\n'));
        let lines = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
        let shift = |position: boa_ast::Position| {
            boa_ast::Position::new(position.line_number() + lines, position.column_number())
        };
        self.namespaces = self
            .namespaces
            .iter()
            .map(|(&(start, end), &mode)| ((shift(start), shift(end)), mode))
            .collect();
    }

    pub(crate) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        self.defaults.apply(script, analysis);
        struct Initializers<'a, 'b> {
            remaining: BTreeMap<(boa_ast::Position, boa_ast::Position), ModuleNamespaceModeIr>,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for Initializers<'a, '_> {
            type BreakTy = ();
            fn visit_array_literal(&mut self, array: &'a ArrayLiteral) -> ControlFlow<()> {
                let span = array.span();
                if let Some(mode) = self.remaining.remove(&(span.start(), span.end())) {
                    self.analysis
                        .namespace_initializers
                        .insert(std::ptr::from_ref(array) as usize, mode);
                }
                array.visit_with(self)
            }
        }
        let mut visitor = Initializers {
            remaining: self.namespaces.clone(),
            analysis,
        };
        let _ = script.visit_with(&mut visitor);
        assert!(
            visitor.remaining.is_empty(),
            "trusted namespace spans survive merged Script parsing"
        );
    }
}
