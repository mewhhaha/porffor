//! Named Environment Records visible to source-level direct eval.

use crate::BindingMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalVisibleBindingIr {
    pub source_name: String,
    pub slot: u32,
    pub mode: BindingMode,
    pub declaration: EvalBindingDeclarationIr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalBindingDeclarationIr {
    NamedFunctionExpression,
    Parameter,
    Variable,
    Lexical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalDeclarativeEnvironmentKindIr {
    GlobalLexical,
    Parameters,
    Variable,
    Lexical,
    Catch,
    SimpleCatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalEnvironmentRoleIr {
    Declarative {
        kind: EvalDeclarativeEnvironmentKindIr,
        bindings: Vec<EvalVisibleBindingIr>,
    },
    WithObject {
        object_slot: u32,
    },
}
