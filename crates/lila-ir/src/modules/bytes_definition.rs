//! Intrinsic sites of host-created bytes modules in the retained source driver.
//!
//! The canonical private activation path carries a typed module unit into its
//! AST owner. Source-phase graphs retain the older merged Script driver, so
//! their bytes sites are identified by trusted spans of the generated module
//! body, never by a filename or a user-spellable identifier.

use super::evaluation_mode::ModuleMaterializationModeIr;
use crate::*;

#[derive(Debug, Default)]
pub(super) struct BytesDefinitions(BTreeMap<(usize, usize), BytesSite>);

#[derive(Debug, Clone, Copy)]
enum BytesSite {
    Constructor,
    Apply(StandardBuiltinId),
}

impl BytesDefinitions {
    pub(super) fn record_body(
        &mut self,
        body: &str,
        preceding_source: &str,
        mode: ModuleMaterializationModeIr,
    ) -> Result<(), String> {
        let parsed = lila_front::parse(body, lila_front::ParseOptions::script())
            .map_err(|error| format!("rewritten bytes module did not parse: {error}"))?;
        let ParsedSource::Script(parsed) = parsed else {
            unreachable!("Script parse options produce Script syntax")
        };
        let offset = preceding_source.encode_utf16().count();
        struct Sites<'a> {
            offset: usize,
            sites: &'a mut BTreeMap<(usize, usize), BytesSite>,
            constructors: usize,
            calls: usize,
        }
        impl<'a> Visitor<'a> for Sites<'_> {
            type BreakTy = ();
            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                let site = match expression {
                    Expression::New(_) => {
                        self.constructors += 1;
                        Some(BytesSite::Constructor)
                    }
                    Expression::Call(call)
                        if matches!(call.function().flatten(), Expression::ArrowFunction(_)) =>
                    {
                        None
                    }
                    Expression::Call(_) => {
                        let getter = match self.calls {
                            0 => StandardBuiltinId::TypedArrayPrototypeBufferGetter,
                            1 => StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable,
                            _ => panic!("bytes module has exactly two intrinsic calls"),
                        };
                        self.calls += 1;
                        Some(BytesSite::Apply(getter))
                    }
                    _ => None,
                };
                if let Some(site) = site {
                    let span = expression.span();
                    assert!(
                        self.sites
                            .insert(
                                (
                                    span.start().pos() + self.offset,
                                    span.end().pos() + self.offset
                                ),
                                site,
                            )
                            .is_none(),
                        "bytes intrinsic spans are distinct"
                    );
                }
                expression.visit_with(self)
            }
        }
        parsed.with_compiler_session(|script, _| {
            let mut sites = Sites {
                offset,
                sites: &mut self.0,
                constructors: 0,
                calls: 0,
            };
            match mode {
                ModuleMaterializationModeIr::Eager => {
                    let _ = script.visit_with(&mut sites);
                }
                ModuleMaterializationModeIr::Deferred => {
                    let execute = script
                        .statements()
                        .statements()
                        .iter()
                        .filter_map(|item| match item {
                            StatementListItem::Declaration(declaration) => {
                                match declaration.as_ref() {
                                    Declaration::FunctionDeclaration(function) => Some(function),
                                    _ => None,
                                }
                            }
                            _ => None,
                        })
                        .nth(1)
                        .expect("deferred bytes module has an execute function");
                    let _ = execute.body().visit_with(&mut sites);
                }
            }
            assert_eq!(sites.constructors, 2, "bytes module has two constructors");
            assert_eq!(sites.calls, 2, "bytes module has two intrinsic calls");
        });
        Ok(())
    }

    pub(super) fn prepend(&mut self, prefix: &str) {
        let offset = prefix.encode_utf16().count();
        self.0 = self
            .0
            .iter()
            .map(|(&(start, end), &site)| ((start + offset, end + offset), site))
            .collect();
    }

    pub(super) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        if self.0.is_empty() {
            return;
        }
        struct Sites<'a, 'b> {
            remaining: BTreeMap<(usize, usize), BytesSite>,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for Sites<'a, '_> {
            type BreakTy = ();
            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                let span = expression.span();
                if let Some(site) = self
                    .remaining
                    .remove(&(span.start().pos(), span.end().pos()))
                {
                    match (site, expression) {
                        (BytesSite::Constructor, Expression::New(new)) => {
                            self.analysis.module_execution.intrinsics.insert(
                                std::ptr::from_ref(new.constructor().flatten()) as usize,
                                StandardBuiltinId::Uint8ArrayConstructor,
                            );
                        }
                        (BytesSite::Apply(getter), Expression::Call(call)) => {
                            self.analysis.module_execution.intrinsics.insert(
                                std::ptr::from_ref(call.function().flatten()) as usize,
                                StandardBuiltinId::ReflectApply,
                            );
                            self.analysis.module_execution.intrinsics.insert(
                                std::ptr::from_ref(
                                    call.args().first().expect("getter argument").flatten(),
                                ) as usize,
                                getter,
                            );
                        }
                        _ => panic!("trusted bytes expression shape changed after Script merge"),
                    }
                }
                expression.visit_with(self)
            }
        }
        let mut sites = Sites {
            remaining: self.0.clone(),
            analysis,
        };
        let _ = script.visit_with(&mut sites);
        assert!(
            sites.remaining.is_empty(),
            "trusted bytes spans survive Script merge"
        );
    }
}
