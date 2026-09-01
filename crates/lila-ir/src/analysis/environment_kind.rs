#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvironmentKind {
    Activation,
    Block,
    WithObject,
    ClassName,
    SwitchCaseBlock,
    CatchParameter,
    ForLexicalHead,
    ForInOfTdzHead,
    ForInOfIteration,
}

impl EnvironmentKind {
    pub(crate) const fn is_materialized_in_stage_a(self) -> bool {
        matches!(
            self,
            Self::Block | Self::SwitchCaseBlock | Self::CatchParameter
        )
    }

    pub(crate) const fn is_materialized(self) -> bool {
        matches!(
            self,
            Self::Block
                | Self::ClassName
                | Self::WithObject
                | Self::SwitchCaseBlock
                | Self::CatchParameter
                | Self::ForLexicalHead
                | Self::ForInOfTdzHead
                | Self::ForInOfIteration
        )
    }
}
