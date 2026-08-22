use crate::{ClassFunctionKind, FunctionExecutionKind, FunctionFlavor};

/// The lexical `super` capability provided by a function owner.
///
/// Arrow owners are deliberately `None`: their capability is selected by
/// walking to the first non-arrow owner, never copied onto the arrow itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexicalSuperOwnerRole {
    None,
    HomeObject,
    DerivedConstructorActivation,
}

/// The reachable combinations of function execution, lexical flavor,
/// constructability and class role.
///
/// See `docs/rust-rewrite/contracts/function-protocol.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionProtocolIr {
    OrdinaryCallOnly,
    OrdinaryCallAndConstruct,
    Arrow,
    Generator,
    Async,
    AsyncArrow,
    AsyncGenerator,
    ObjectMethod(FunctionExecutionKind),
    ObjectGetter,
    ObjectSetter,
    ClassConstructor,
    ClassMethod(FunctionExecutionKind),
    ClassGetter,
    ClassSetter,
}

impl FunctionProtocolIr {
    #[must_use]
    pub(crate) const fn lexical_super_owner_role(self) -> LexicalSuperOwnerRole {
        match self {
            Self::ObjectMethod(_)
            | Self::ObjectGetter
            | Self::ObjectSetter
            | Self::ClassMethod(_)
            | Self::ClassGetter
            | Self::ClassSetter => LexicalSuperOwnerRole::HomeObject,
            Self::OrdinaryCallOnly
            | Self::OrdinaryCallAndConstruct
            | Self::Arrow
            | Self::Generator
            | Self::Async
            | Self::AsyncArrow
            | Self::AsyncGenerator
            | Self::ClassConstructor => LexicalSuperOwnerRole::None,
        }
    }

    #[must_use]
    pub const fn flavor(self) -> FunctionFlavor {
        match self {
            Self::Arrow | Self::AsyncArrow => FunctionFlavor::Arrow,
            Self::OrdinaryCallOnly
            | Self::OrdinaryCallAndConstruct
            | Self::Generator
            | Self::Async
            | Self::AsyncGenerator
            | Self::ObjectMethod(_)
            | Self::ObjectGetter
            | Self::ObjectSetter
            | Self::ClassConstructor
            | Self::ClassMethod(_)
            | Self::ClassGetter
            | Self::ClassSetter => FunctionFlavor::Ordinary,
        }
    }

    #[must_use]
    pub const fn execution_kind(self) -> FunctionExecutionKind {
        match self {
            Self::OrdinaryCallOnly
            | Self::OrdinaryCallAndConstruct
            | Self::Arrow
            | Self::ClassConstructor
            | Self::ObjectGetter
            | Self::ObjectSetter
            | Self::ClassGetter
            | Self::ClassSetter => FunctionExecutionKind::Ordinary,
            Self::Generator => FunctionExecutionKind::Generator,
            Self::Async | Self::AsyncArrow => FunctionExecutionKind::Async,
            Self::AsyncGenerator => FunctionExecutionKind::AsyncGenerator,
            Self::ObjectMethod(kind) => kind,
            Self::ClassMethod(kind) => kind,
        }
    }

    #[must_use]
    pub const fn is_constructable(self) -> bool {
        matches!(
            self,
            Self::OrdinaryCallAndConstruct | Self::ClassConstructor
        )
    }

    #[must_use]
    pub const fn class_kind(self) -> ClassFunctionKind {
        match self {
            Self::ClassConstructor => ClassFunctionKind::Constructor,
            Self::ClassMethod(_) => ClassFunctionKind::Method,
            Self::ClassGetter => ClassFunctionKind::Getter,
            Self::ClassSetter => ClassFunctionKind::Setter,
            Self::OrdinaryCallOnly
            | Self::OrdinaryCallAndConstruct
            | Self::Arrow
            | Self::Generator
            | Self::Async
            | Self::AsyncArrow
            | Self::AsyncGenerator
            | Self::ObjectMethod(_)
            | Self::ObjectGetter
            | Self::ObjectSetter => ClassFunctionKind::None,
        }
    }

    #[must_use]
    pub const fn is_object_literal_method(self) -> bool {
        matches!(
            self,
            Self::ObjectMethod(_) | Self::ObjectGetter | Self::ObjectSetter
        )
    }
}
