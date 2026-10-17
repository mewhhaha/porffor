//! Reserve the exact scopes one analysis pass will allocate before it starts.
use crate::analysis::AnalysisAllocationState;
use crate::{PreparedScriptKind, ScriptInstantiation};
use boa_ast::function::{ClassDeclaration, ClassElement, ClassElementName, ClassExpression};
use boa_ast::visitor::{VisitWith, Visitor};
use boa_ast::Script;
use std::ops::ControlFlow;

pub(crate) fn require_capacity(
    allocations: &AnalysisAllocationState,
    script: &Script,
    instantiation: &ScriptInstantiation,
    source_len: usize,
) -> Result<(), &'static str> {
    // Every class scope consumes source syntax. Ordinary programs prove a
    // conservative bound in O(1); only a nearly exhausted foreign ID space
    // needs an extra AST walk to count the actual allocations precisely.
    if let Some(bound) = source_len
        .checked_add(1)
        .and_then(|n| u32::try_from(n).ok())
    {
        if allocations
            .require_private_environment_capacity(bound)
            .is_ok()
        {
            return Ok(());
        }
    }
    let inherited = match instantiation {
        ScriptInstantiation::Prepared(PreparedScriptKind::DirectEval(context)) => {
            u32::from(!context.private_names().is_empty())
        }
        ScriptInstantiation::Prepared(
            PreparedScriptKind::IndirectEval
            | PreparedScriptKind::RealmScript
            | PreparedScriptKind::ShadowRealmEvaluate,
        )
        | ScriptInstantiation::FreshEntry
        | ScriptInstantiation::ModuleAfterGlobalScript
        | ScriptInstantiation::GlobalScriptAfterPrelude => 0,
    };
    let mut scopes = ScopeCount(inherited);
    if script.visit_with(&mut scopes).is_break() {
        return Err("private-scope allocation count overflow");
    }
    allocations.require_private_environment_capacity(scopes.0)
}

struct ScopeCount(u32);
impl ScopeCount {
    fn count(&mut self, elements: &[ClassElement]) -> ControlFlow<()> {
        let required = elements.iter().any(|element| match element {
            ClassElement::MethodDefinition(method) => {
                matches!(method.name(), ClassElementName::PrivateName(_))
            }
            ClassElement::PrivateFieldDefinition(_)
            | ClassElement::PrivateStaticFieldDefinition(_)
            | ClassElement::AccessorFieldDefinition(_)
            | ClassElement::StaticAccessorFieldDefinition(_) => true,
            ClassElement::FieldDefinition(_)
            | ClassElement::StaticFieldDefinition(_)
            | ClassElement::StaticBlock(_) => false,
        });
        if required {
            self.0 = match self.0.checked_add(1) {
                Some(next) => next,
                None => return ControlFlow::Break(()),
            };
        }
        ControlFlow::Continue(())
    }
}
impl<'ast> Visitor<'ast> for ScopeCount {
    type BreakTy = ();
    fn visit_class_declaration(&mut self, class: &'ast ClassDeclaration) -> ControlFlow<()> {
        self.count(class.elements())?;
        class.visit_with(self)
    }
    fn visit_class_expression(&mut self, class: &'ast ClassExpression) -> ControlFlow<()> {
        self.count(class.elements())?;
        class.visit_with(self)
    }
}
