//! Actual ForIn source identity, head classification and original binding evidence.

use super::*;
use boa_ast::statement::iteration::ForInLoop;

/// Only the actual ForIn AST can mint this source-identity domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ForInSourceIdentity(usize);

impl ForInSourceIdentity {
    pub(crate) fn from_source(source: &ForInLoop) -> Self {
        Self(source as *const ForInLoop as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GeneratorForInHeadKind {
    Binding,
    BindingPattern,
    AssignmentIdentifier,
    AssignmentPattern,
    Property,
    SuperProperty,
}

/// The names come from this actual initializer, with the same scoped spelling
/// consumed by analysis. A foreign, same-shaped AST cannot mint this evidence.
pub(crate) struct GeneratorForInHeadProof {
    source: ForInSourceIdentity,
    mode: BindingMode,
    kind: GeneratorForInHeadKind,
    bound_names: Vec<String>,
    lexical_storage_names: BTreeMap<String, String>,
    identifier_name: Option<String>,
}

impl GeneratorForInHeadProof {
    pub(crate) fn matches_source(
        &self,
        source: ForInSourceIdentity,
        mode: BindingMode,
        kind: GeneratorForInHeadKind,
    ) -> bool {
        self.source == source && self.mode == mode && self.kind == kind
    }

    pub(crate) const fn mode(&self) -> BindingMode {
        self.mode
    }
    pub(crate) const fn kind(&self) -> GeneratorForInHeadKind {
        self.kind
    }
    pub(crate) fn bound_names(&self) -> &[String] {
        &self.bound_names
    }
    pub(crate) fn lexical_storage_names(&self) -> &BTreeMap<String, String> {
        &self.lexical_storage_names
    }
    pub(crate) fn validate_identifier_write(
        self,
        source: &ForInLoop,
        ignored: Option<crate::reference::IgnoredIterationIdentifierWriteIr>,
    ) -> Option<Self> {
        if self.source != ForInSourceIdentity::from_source(source) {
            return None;
        }
        if let Some(ignored) = &ignored {
            if self.kind != GeneratorForInHeadKind::AssignmentIdentifier
                || self.identifier_name.as_deref() != Some(ignored.source_name())
                || !ignored.is_ignored()
            {
                return None;
            }
        }
        Some(self)
    }
}

/// Every complete protocol uses this actual head classifier before lowering
/// its selected-key initialization region.
pub(crate) fn mixed_for_in_head_mode_and_kind(
    source: &ForInLoop,
) -> Option<(BindingMode, GeneratorForInHeadKind)> {
    classify_for_in_head(source)
}

fn classify_for_in_head(source: &ForInLoop) -> Option<(BindingMode, GeneratorForInHeadKind)> {
    let classified = match source.initializer() {
        IterableLoopInitializer::Var(variable) => (
            BindingMode::Var,
            match variable.binding() {
                Binding::Identifier(_) => GeneratorForInHeadKind::Binding,
                Binding::Pattern(_) => GeneratorForInHeadKind::BindingPattern,
            },
        ),
        IterableLoopInitializer::Let(binding) => (
            BindingMode::Let,
            match binding {
                Binding::Identifier(_) => GeneratorForInHeadKind::Binding,
                Binding::Pattern(_) => GeneratorForInHeadKind::BindingPattern,
            },
        ),
        IterableLoopInitializer::Const(binding) => (
            BindingMode::Const,
            match binding {
                Binding::Identifier(_) => GeneratorForInHeadKind::Binding,
                Binding::Pattern(_) => GeneratorForInHeadKind::BindingPattern,
            },
        ),
        IterableLoopInitializer::Identifier(_) => (
            BindingMode::Var,
            GeneratorForInHeadKind::AssignmentIdentifier,
        ),
        IterableLoopInitializer::Pattern(_) => {
            (BindingMode::Var, GeneratorForInHeadKind::AssignmentPattern)
        }
        IterableLoopInitializer::Access(PropertyAccess::Simple(_) | PropertyAccess::Private(_)) => {
            (BindingMode::Var, GeneratorForInHeadKind::Property)
        }
        IterableLoopInitializer::Access(PropertyAccess::Super(_)) => {
            (BindingMode::Var, GeneratorForInHeadKind::SuperProperty)
        }
        IterableLoopInitializer::Using(_)
        | IterableLoopInitializer::AwaitUsing(_)
        | IterableLoopInitializer::WebCompatCall(_) => return None,
    };
    Some(classified)
}

impl GeneratorForInHeadProof {
    pub(crate) fn from_mixed_source(source: &ForInLoop, interner: &Interner) -> Option<Self> {
        let (mode, kind) = mixed_for_in_head_mode_and_kind(source)?;
        Self::from_classified_source(source, interner, mode, kind)
    }

    fn from_classified_source(
        source: &ForInLoop,
        interner: &Interner,
        mode: BindingMode,
        kind: GeneratorForInHeadKind,
    ) -> Option<Self> {
        let binding = match source.initializer() {
            IterableLoopInitializer::Var(variable) => Some(variable.binding()),
            IterableLoopInitializer::Let(binding) | IterableLoopInitializer::Const(binding) => {
                Some(binding)
            }
            IterableLoopInitializer::Identifier(_)
            | IterableLoopInitializer::Pattern(_)
            | IterableLoopInitializer::Access(_)
            | IterableLoopInitializer::Using(_)
            | IterableLoopInitializer::AwaitUsing(_)
            | IterableLoopInitializer::WebCompatCall(_) => None,
        };
        let bound_names: Vec<String> = match binding {
            Some(binding) => supported_bound_names(interner, binding)?
                .into_iter()
                .map(|name| name.source_name)
                .collect(),
            None => Vec::new(),
        };
        let lexical_storage_names = if matches!(mode, BindingMode::Let | BindingMode::Const) {
            bound_names
                .iter()
                .map(|name| (name.clone(), for_in_loop_binding_storage_name(source, name)))
                .collect()
        } else {
            BTreeMap::new()
        };
        Some(GeneratorForInHeadProof {
            source: ForInSourceIdentity::from_source(source),
            mode,
            kind,
            bound_names,
            lexical_storage_names,
            identifier_name: match source.initializer() {
                IterableLoopInitializer::Identifier(identifier) => {
                    Some(interner.resolve_expect(identifier.sym()).to_string())
                }
                IterableLoopInitializer::Var(_)
                | IterableLoopInitializer::Let(_)
                | IterableLoopInitializer::Const(_)
                | IterableLoopInitializer::Pattern(_)
                | IterableLoopInitializer::Access(_)
                | IterableLoopInitializer::Using(_)
                | IterableLoopInitializer::AwaitUsing(_)
                | IterableLoopInitializer::WebCompatCall(_) => None,
            },
        })
    }
}
