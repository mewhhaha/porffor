#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvironmentKind {
    Activation,
    FunctionParameters,
    FunctionBody,
    ParameterEvalVariable,
    Block,
    WithObject,
    ClassName,
    NamedFunctionExpression,
    SwitchCaseBlock,
    CatchParameter,
    SimpleCatchParameter,
    ForLexicalHead,
    ForInOfTdzHead,
    ForInOfIteration,
}

impl EnvironmentKind {
    pub(crate) const fn is_materialized_in_stage_a(self) -> bool {
        matches!(
            self,
            Self::FunctionParameters
                | Self::FunctionBody
                | Self::Block
                | Self::SwitchCaseBlock
                | Self::CatchParameter
                | Self::SimpleCatchParameter
        )
    }

    pub(crate) const fn is_materialized(self) -> bool {
        matches!(
            self,
            Self::FunctionParameters
                | Self::FunctionBody
                | Self::ParameterEvalVariable
                | Self::Block
                | Self::NamedFunctionExpression
                | Self::ClassName
                | Self::WithObject
                | Self::SwitchCaseBlock
                | Self::CatchParameter
                | Self::SimpleCatchParameter
                | Self::ForLexicalHead
                | Self::ForInOfTdzHead
                | Self::ForInOfIteration
        )
    }
}
