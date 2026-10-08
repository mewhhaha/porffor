//! Compiler-owned ArgumentListEvaluation results; these are never JS arrays.
use crate::{
    ExprIr, FunctionTargetKnowledge, KindSet, StatementIr, TypedExpr, ValueInfo, ValueKind,
};

/// The actual iterator/value evaluation that creates a private argv vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentListCaptureIr {
    arguments: Vec<TypedExpr>,
}
impl ArgumentListCaptureIr {
    pub fn arguments(&self) -> &[TypedExpr] {
        &self.arguments
    }
}

/// A stored argument list produced by the paired capture, not a JS iterable.
/// Private fields prevent a caller from supplying an ordinary JS array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedArgumentListIr {
    binding: Box<TypedExpr>,
}
impl CapturedArgumentListIr {
    pub fn binding(&self) -> &TypedExpr {
        &self.binding
    }
    pub fn storage_name(&self) -> &str {
        let ExprIr::Identifier(name) = &self.binding.expr else {
            unreachable!("private argument-list constructor owns an identifier")
        };
        name
    }
}

#[must_use = "ArgumentListEvaluation must be stored before its captured list is consumed"]
pub(crate) struct ArgumentListCapturePlan {
    arguments: Vec<TypedExpr>,
}
impl ArgumentListCapturePlan {
    pub(crate) fn new(arguments: Vec<TypedExpr>) -> Self {
        Self { arguments }
    }
    pub(crate) fn into_binding(self, name: String) -> (StatementIr, CapturedArgumentListIr) {
        let info = private_list_info();
        let binding = TypedExpr::from_info(info.clone(), ExprIr::Identifier(name.clone()));
        let capture = TypedExpr::from_info(
            info,
            ExprIr::CaptureArgumentList(ArgumentListCaptureIr {
                arguments: self.arguments,
            }),
        );
        (
            StatementIr::Lexical {
                mode: crate::BindingMode::Let,
                name,
                init: capture,
            },
            CapturedArgumentListIr {
                binding: Box::new(binding),
            },
        )
    }
}

pub(crate) fn private_list_info() -> ValueInfo {
    ValueInfo {
        // Planning metadata for the private paired witness only. It never
        // becomes a JavaScript value or an Array instance.
        kind: ValueKind::Dynamic,
        possible_kinds: KindSet::all_runtime_tags(),
        heap_shape: None,
        function_targets: FunctionTargetKnowledge::none(),
    }
}

/// The activation-owned receiver written by runtime ResolveBinding's call base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedCallReceiverIr {
    binding: Box<TypedExpr>,
}
impl CapturedCallReceiverIr {
    pub(crate) fn new(name: String) -> Self {
        let info = ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::all_runtime_tags(),
            heap_shape: None,
            function_targets: FunctionTargetKnowledge::unknown(),
        };
        Self {
            binding: Box::new(TypedExpr::from_info(info, ExprIr::Identifier(name))),
        }
    }
    pub fn binding(&self) -> &TypedExpr {
        &self.binding
    }
    pub fn storage_name(&self) -> &str {
        let ExprIr::Identifier(name) = &self.binding.expr else {
            unreachable!("private receiver constructor owns an identifier")
        };
        name
    }
}

/// Complete the existing optional-chain Reference once, including its
/// conditional ReferenceOrUndefined receiver. Private fields require a real
/// lowered chain and an activation-owned receiver rather than an arbitrary value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalCallReferenceCaptureIr {
    chain_expression: Box<TypedExpr>,
    receiver: CapturedCallReceiverIr,
}
impl OptionalCallReferenceCaptureIr {
    pub(crate) fn new(
        info: ValueInfo,
        target: Box<TypedExpr>,
        chain: Vec<crate::OptionalChainOperationIr>,
        receiver: CapturedCallReceiverIr,
    ) -> Self {
        Self {
            chain_expression: Box::new(TypedExpr::from_info(
                info,
                ExprIr::OptionalPropertyChain { target, chain },
            )),
            receiver,
        }
    }
    pub fn chain_expression(&self) -> &TypedExpr {
        &self.chain_expression
    }
    pub fn receiver(&self) -> &CapturedCallReceiverIr {
        &self.receiver
    }
    pub fn operands(&self) -> [&TypedExpr; 2] {
        [self.chain_expression(), self.receiver.binding()]
    }
}
