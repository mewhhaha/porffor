use boa_ast::{
    function::{ClassDeclaration, ClassExpression},
    Span, Spanned,
};
use boa_interner::{Interner, Sym};

/// A class display label does not by itself establish a source name binding.
pub(crate) struct SourceClassName(ClassNameSource);

enum ClassNameSource {
    Anonymous,
    InferredLabel(String),
    Binding { name: String, span: Span },
}

impl SourceClassName {
    pub(crate) fn from_declaration(class: &ClassDeclaration, interner: &Interner) -> Self {
        let identifier = class.name();
        let interned = interner.resolve_expect(identifier.sym());
        let label = interned.to_string();
        // Anonymous default exports carry the parser's synthetic `default`
        // label; that keyword is never a source BindingIdentifier.
        if identifier.sym() != Sym::DEFAULT
            && class.name_scope().has_binding(&interned.utf16().into())
        {
            Self(ClassNameSource::Binding {
                name: label,
                span: identifier.span(),
            })
        } else {
            Self(ClassNameSource::InferredLabel(label))
        }
    }

    pub(crate) fn from_expression(class: &ClassExpression, interner: &Interner) -> Self {
        let Some(identifier) = class.name() else {
            return Self(ClassNameSource::Anonymous);
        };
        let label = interner.resolve_expect(identifier.sym()).to_string();
        // The parser records this scope only for a source BindingIdentifier.
        // NamedEvaluation changes the display name while leaving it absent.
        if class.name_scope().is_some() {
            Self(ClassNameSource::Binding {
                name: label,
                span: identifier.span(),
            })
        } else {
            Self(ClassNameSource::InferredLabel(label))
        }
    }

    pub(crate) fn binding_name(&self) -> Option<&str> {
        match &self.0 {
            ClassNameSource::Binding { name, .. } => Some(name),
            ClassNameSource::Anonymous | ClassNameSource::InferredLabel(_) => None,
        }
    }

    pub(crate) fn binding_span(&self) -> Option<Span> {
        match &self.0 {
            ClassNameSource::Binding { span, .. } => Some(*span),
            ClassNameSource::Anonymous | ClassNameSource::InferredLabel(_) => None,
        }
    }

    pub(crate) fn into_label(self) -> Option<String> {
        match self.0 {
            ClassNameSource::Anonymous => None,
            ClassNameSource::InferredLabel(label)
            | ClassNameSource::Binding { name: label, .. } => Some(label),
        }
    }
}
