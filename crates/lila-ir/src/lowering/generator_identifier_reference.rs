use super::*;
use crate::environment_identifier::IdentifierReferenceFallbackIr;

/// The selected Record or cell remains rooted separately from any old JS value.
#[must_use = "the retained Identifier Reference must supply the later PutValue or be released"]
pub(super) struct RetainedGeneratorIdentifierTarget {
    name: String,
    reference: CapturedIdentifierReferenceIr,
}

/// Compound/logical assignment owns GetValue; a write-only target cannot read it.
#[must_use = "the retained Identifier Reference must supply the later PutValue or be released"]
pub(super) struct RetainedGeneratorIdentifierReference {
    target: RetainedGeneratorIdentifierTarget,
    old_value: String,
}

impl RetainedGeneratorIdentifierTarget {
    fn locate(
        lowerer: &mut ScriptLowerer<'_>,
        prefix: &mut Vec<StatementIr>,
        name: String,
        access: IdentifierReferenceCaptureAccess,
    ) -> (Self, TypedExpr) {
        let storage = lowerer.alloc_suspension_owned_binding(
            "generator.identifier.reference.",
            ValueInfo::undefined(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: storage.clone(),
            init: TypedExpr::undefined(),
        });
        let reference = CapturedIdentifierReferenceIr::new(storage);
        let capture = if lowerer.uses_runtime_identifier_environment() {
            IdentifierReferenceCaptureIr::runtime(reference.clone(), access)
        } else {
            let located = lowerer
                .lookup_binding_with_location(&name)
                .filter(|_| !lowerer.is_unshadowed_script_global_binding(&name));
            let position = located.as_ref().map(|(binding, location)| {
                lowerer.declarative_binding_position(binding, *location)
            });
            // ResolveBinding retains the actual cell without testing its TDZ.
            // ReadBeforeRhs tests it at GetValue; WriteOnly waits for PutValue.
            let fallback = match located {
                Some((binding, _)) => {
                    IdentifierReferenceFallbackIr::declarative(binding.storage_name)
                }
                None => IdentifierReferenceFallbackIr::global(),
            };
            if let Some(objects) = lowerer.with_environment_chain.select_preceding(position) {
                let selection = lowerer
                    .with_environment_reference_plan(name.clone(), objects)
                    .select_binding_object();
                IdentifierReferenceCaptureIr::with_object(
                    reference.clone(),
                    selection,
                    fallback,
                    access,
                )
            } else {
                IdentifierReferenceCaptureIr::located(reference.clone(), fallback, access)
            }
        };
        let value = lowerer.environment_identifier(
            name.clone(),
            EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture },
        );
        (Self { name, reference }, value)
    }

    pub(super) fn capture_write_only(
        lowerer: &mut ScriptLowerer<'_>,
        prefix: &mut Vec<StatementIr>,
        name: String,
    ) -> Self {
        let (target, capture) = Self::locate(
            lowerer,
            prefix,
            name,
            IdentifierReferenceCaptureAccess::WriteOnly,
        );
        prefix.push(StatementIr::Expression(capture));
        target
    }

    pub(super) fn put_value(self, lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> TypedExpr {
        let mut info = value.value_info();
        // An Object Environment setter may mutate the returned RHS object.
        // PutValue preserves its whole value and kind, not its heap facts.
        info.heap_shape = None;
        let write = lowerer.environment_identifier(
            self.name,
            EnvironmentIdentifierOperationIr::PutCapturedReference {
                reference: self.reference,
                value: Box::new(value),
            },
        );
        TypedExpr::from_info(info, write.expr)
    }

    fn release_operation(&self) -> (String, EnvironmentIdentifierOperationIr) {
        (
            self.name.clone(),
            EnvironmentIdentifierOperationIr::ReleaseCapturedReference {
                reference: self.reference.clone(),
            },
        )
    }
}

impl RetainedGeneratorIdentifierReference {
    pub(super) fn capture(
        lowerer: &mut ScriptLowerer<'_>,
        prefix: &mut Vec<StatementIr>,
        name: String,
    ) -> Self {
        let (target, value) = RetainedGeneratorIdentifierTarget::locate(
            lowerer,
            prefix,
            name,
            IdentifierReferenceCaptureAccess::ReadBeforeRhs,
        );
        let old_value = lowerer.alloc_suspension_owned_binding(
            "generator.compound.old.",
            unknown_runtime_value_info(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: old_value.clone(),
            init: value,
        });
        Self { target, old_value }
    }

    pub(super) fn old_value(&self) -> TypedExpr {
        TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::Identifier(self.old_value.clone()),
        )
    }

    pub(super) fn put_value(self, lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> TypedExpr {
        self.target.put_value(lowerer, value)
    }

    pub(super) fn release_operation(&self) -> (String, EnvironmentIdentifierOperationIr) {
        self.target.release_operation()
    }
}
