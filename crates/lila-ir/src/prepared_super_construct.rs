//! SuperCall preparation belongs before ArgumentListEvaluation and suspension.
use crate::{
    BindingMode, ExprIr, KindSet, OwnedEnvBindingIr, StatementIr, TypedExpr, ValueInfo, ValueKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSuperConstructIr {
    constructor: Box<TypedExpr>,
    new_target: Box<TypedExpr>,
    arguments: Vec<TypedExpr>,
}
impl PreparedSuperConstructIr {
    pub fn constructor(&self) -> &TypedExpr {
        &self.constructor
    }
    pub fn new_target(&self) -> &TypedExpr {
        &self.new_target
    }
    pub fn arguments(&self) -> &[TypedExpr] {
        &self.arguments
    }
    pub fn operands(&self) -> impl Iterator<Item = &TypedExpr> {
        std::iter::once(self.constructor())
            .chain(std::iter::once(self.new_target()))
            .chain(self.arguments.iter())
    }
}

/// The only mint writes the exact implicit activation reads in spec order.
/// Capturing this does not read the constructor's uninitialized `this` cell.
pub(crate) struct SuperConstructCapturePlan {
    constructor: OwnedEnvBindingIr,
    new_target: OwnedEnvBindingIr,
}
impl SuperConstructCapturePlan {
    pub(crate) fn new(
        constructor: OwnedEnvBindingIr,
        new_target: OwnedEnvBindingIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Option<Self> {
        if constructor.name == new_target.name || constructor.slot == new_target.slot {
            return None;
        }
        for binding in [&constructor, &new_target] {
            if inventory
                .iter()
                .filter(|row| row.name == binding.name || row.slot == binding.slot)
                .count()
                != 1
                || !inventory.iter().any(|row| row == binding)
            {
                return None;
            }
        }
        Some(Self {
            constructor,
            new_target,
        })
    }
    pub(crate) fn into_prefix(self) -> (Vec<StatementIr>, CapturedSuperConstruct) {
        let info = ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: crate::FunctionTargetKnowledge::unknown(),
        };
        let prefix = vec![
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: self.new_target.name.clone(),
                init: TypedExpr::from_info(info.clone(), ExprIr::SuperNewTarget),
            },
            StatementIr::Lexical {
                mode: BindingMode::Let,
                name: self.constructor.name.clone(),
                init: TypedExpr::from_info(info.clone(), ExprIr::SuperConstructor),
            },
        ];
        let retained = CapturedSuperConstruct {
            constructor: TypedExpr::from_info(
                info.clone(),
                ExprIr::Identifier(self.constructor.name),
            ),
            new_target: TypedExpr::from_info(info, ExprIr::Identifier(self.new_target.name)),
        };
        (prefix, retained)
    }
}

#[must_use = "captured super preparation must be consumed by its SuperCall"]
pub(crate) struct CapturedSuperConstruct {
    constructor: TypedExpr,
    new_target: TypedExpr,
}
impl CapturedSuperConstruct {
    pub(crate) fn finish(self, arguments: Vec<TypedExpr>) -> PreparedSuperConstructIr {
        PreparedSuperConstructIr {
            constructor: Box::new(self.constructor),
            new_target: Box::new(self.new_target),
            arguments,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(name: &str, slot: u32) -> OwnedEnvBindingIr {
        OwnedEnvBindingIr {
            name: name.into(),
            slot,
        }
    }

    #[test]
    fn preparation_reads_the_derived_activation_in_spec_order_without_this() {
        let constructor = binding("constructor", 2);
        let target = binding("newTarget", 3);
        let (prefix, captured) = SuperConstructCapturePlan::new(
            constructor.clone(),
            target.clone(),
            &[constructor, target],
        )
        .unwrap()
        .into_prefix();
        assert!(
            matches!(&prefix[0], StatementIr::Lexical { name, init, mode: BindingMode::Let }
            if name == "newTarget" && matches!(init.expr, ExprIr::SuperNewTarget))
        );
        assert!(
            matches!(&prefix[1], StatementIr::Lexical { name, init, mode: BindingMode::Let }
            if name == "constructor" && matches!(init.expr, ExprIr::SuperConstructor))
        );
        let prepared = captured.finish(vec![TypedExpr::undefined()]);
        assert!(
            matches!(&prepared.constructor().expr, ExprIr::Identifier(name) if name == "constructor")
        );
        assert!(
            matches!(&prepared.new_target().expr, ExprIr::Identifier(name) if name == "newTarget")
        );
        assert_eq!(prepared.arguments(), &[TypedExpr::undefined()]);
    }

    #[test]
    fn absent_and_name_or_slot_aliased_activation_allocations_are_refused() {
        let constructor = binding("constructor", 2);
        let target = binding("newTarget", 3);
        for inventory in [
            vec![constructor.clone()],
            vec![
                constructor.clone(),
                target.clone(),
                binding("constructor", 4),
            ],
            vec![constructor.clone(), target.clone(), binding("other", 3)],
        ] {
            assert!(SuperConstructCapturePlan::new(
                constructor.clone(),
                target.clone(),
                &inventory
            )
            .is_none());
        }
        assert!(SuperConstructCapturePlan::new(
            constructor.clone(),
            constructor.clone(),
            &[constructor]
        )
        .is_none());
        let target = binding("newTarget", 2);
        let constructor = binding("constructor", 2);
        assert!(SuperConstructCapturePlan::new(
            constructor.clone(),
            target.clone(),
            &[constructor, target]
        )
        .is_none());
    }
}
