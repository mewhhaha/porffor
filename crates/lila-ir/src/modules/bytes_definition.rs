//! Intrinsic owner of a host-created bytes module in the retained source driver.
//!
//! The canonical private activation path carries a typed module unit directly
//! into its AST owner. The source-phase driver merges text first, so it keeps
//! the generated IIFE arrow's parser-owned UTF-16 span across that merge.
//! Source identifiers and filenames cannot grant this privilege.

use super::evaluation_mode::ModuleMaterializationModeIr;
use crate::*;

#[derive(Debug, Default)]
pub(super) struct BytesDefinitions(BTreeMap<(usize, usize), usize>);

impl BytesDefinitions {
    pub(super) fn record_body(
        &mut self,
        body: &str,
        preceding_source: &str,
        mode: ModuleMaterializationModeIr,
        bytes_len: usize,
    ) -> Result<(), String> {
        let parsed = lila_front::parse(body, lila_front::ParseOptions::script())
            .map_err(|error| format!("rewritten bytes module did not parse: {error}"))?;
        let ParsedSource::Script(parsed) = parsed else {
            unreachable!("Script parse options produce Script syntax")
        };
        struct Owners(Vec<boa_ast::LinearSpan>);
        impl<'a> Visitor<'a> for Owners {
            type BreakTy = ();
            fn visit_expression(&mut self, expression: &'a Expression) -> ControlFlow<()> {
                if let Expression::Call(call) = expression {
                    if let Expression::ArrowFunction(owner) = call.function().flatten() {
                        self.0.push(owner.linear_span());
                    }
                }
                expression.visit_with(self)
            }
        }
        let owner = parsed.with_compiler_session(|script, _| {
            let mut owners = Owners(Vec::new());
            match mode {
                ModuleMaterializationModeIr::Eager => {
                    let _ = script.visit_with(&mut owners);
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
                    let _ = execute.body().visit_with(&mut owners);
                }
            }
            let [owner] = owners.0.as_slice() else {
                panic!("bytes module has exactly one generated IIFE arrow")
            };
            *owner
        });
        let offset = preceding_source.encode_utf16().count();
        assert!(
            self.0
                .insert(
                    (owner.start().pos() + offset, owner.end().pos() + offset),
                    bytes_len,
                )
                .is_none(),
            "bytes module owners have distinct spans"
        );
        Ok(())
    }

    pub(super) fn prepend(&mut self, prefix: &str) {
        let offset = prefix.encode_utf16().count();
        self.0 = self
            .0
            .iter()
            .map(|(&(start, end), &bytes_len)| ((start + offset, end + offset), bytes_len))
            .collect();
    }

    pub(super) fn apply<'a>(&self, script: &'a Script, analysis: &mut Analysis<'a>) {
        if self.0.is_empty() {
            return;
        }
        struct Owners<'a, 'b> {
            remaining: BTreeMap<(usize, usize), usize>,
            analysis: &'b mut Analysis<'a>,
        }
        impl<'a> Visitor<'a> for Owners<'a, '_> {
            type BreakTy = ();
            fn visit_arrow_function(&mut self, owner: &'a ArrowFunction) -> ControlFlow<()> {
                let span = owner.linear_span();
                if let Some(bytes_len) = self
                    .remaining
                    .remove(&(span.start().pos(), span.end().pos()))
                {
                    super::synchronous_definition::mark_bytes_intrinsics(
                        owner.body(),
                        bytes_len,
                        self.analysis,
                    );
                }
                owner.visit_with(self)
            }
        }
        let mut owners = Owners {
            remaining: self.0.clone(),
            analysis,
        };
        let _ = script.visit_with(&mut owners);
        assert!(
            owners.remaining.is_empty(),
            "trusted bytes IIFE spans survive Script merge"
        );
    }
}
