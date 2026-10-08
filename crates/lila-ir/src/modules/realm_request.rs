//! Host-declared Realm requests are distinct from requests by source records.

use crate::*;

pub(super) const REALM_IMPORT_DISPATCHER: &str = "$lila$module$realmImport";

pub fn scan_script_realm_module_requests(
    source: &lila_front::ParsedScript,
) -> Vec<ModuleRequestKeyIr> {
    source.with_compiler_session(literal_realm_requests)
}

impl ModuleGraphIr {
    pub(super) fn realm_import_dispatcher_source(&self) -> String {
        let mut text =
            format!("async function {REALM_IMPORT_DISPATCHER}(specifier) {{ await void 0;");
        for (request, resolution) in &self.realm_requests {
            if !request.attributes().is_empty() {
                continue;
            }
            text.push_str(" if (specifier === ");
            super::namespace::push_js_string_literal(&mut text, request.specifier());
            text.push_str(") {");
            match resolution {
                RealmModuleResolutionIr::Loaded(module) => {
                    text.push_str(" await ");
                    text.push_str(&super::dynamic::module_evaluator_name(*module));
                    // Namespace is captured by importValue's intrinsic export
                    // continuation. Resolving it here would observe export `then`.
                    text.push_str("; return; }");
                }
                RealmModuleResolutionIr::Rejected(message) => {
                    text.push_str(" throw new $lila$module$SyntaxError(");
                    super::namespace::push_js_string_literal(&mut text, message);
                    text.push_str("); }");
                }
            }
        }
        text.push_str(" throw new $lila$module$TypeError(\"Cannot find module \" + specifier); }");
        text
    }
}

/// Original host/load/link rejection remains data in the compiled catalog;
/// importValue's outer promise decides how it crosses the Realm boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealmModuleResolutionIr {
    Loaded(ModuleUnitId),
    Rejected(String),
}

pub(super) fn literal_realm_requests<'ast>(
    source: &'ast impl VisitWith,
    interner: &'ast Interner,
) -> Vec<ModuleRequestKeyIr> {
    struct Requests<'a> {
        interner: &'a Interner,
        requests: BTreeSet<ModuleRequestKeyIr>,
    }
    impl<'ast> Visitor<'ast> for Requests<'_> {
        type BreakTy = ();
        fn visit_expression(&mut self, expression: &'ast Expression) -> ControlFlow<()> {
            let Expression::Call(call) = expression else {
                return expression.visit_with(self);
            };
            if let Expression::PropertyAccess(PropertyAccess::Simple(access)) =
                call.function().flatten()
            {
                let name = match access.field() {
                    PropertyAccessField::Const(name) => {
                        Some(self.interner.resolve_expect(name.sym()).to_string())
                    }
                    PropertyAccessField::Expr(expression) => match expression.flatten() {
                        Expression::Literal(literal) => match literal.kind() {
                            LiteralKind::String(name) => {
                                Some(self.interner.resolve_expect(*name).to_string())
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                };
                if name.as_deref() == Some("importValue") {
                    if let Some(Expression::Literal(literal)) =
                        call.args().first().map(Expression::flatten)
                    {
                        if let LiteralKind::String(specifier) = literal.kind() {
                            self.requests.insert(ModuleRequestKeyIr::plain(
                                self.interner.resolve_expect(*specifier).to_string(),
                            ));
                        }
                    }
                }
            }
            expression.visit_with(self)
        }
    }
    let mut requests = Requests {
        interner,
        requests: BTreeSet::new(),
    };
    let _ = source.visit_with(&mut requests);
    requests.requests.into_iter().collect()
}
