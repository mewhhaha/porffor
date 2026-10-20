//! Nonempty accessor definitions preserve getter and setter roles through
//! builtin materialization and GC property publication.

use super::*;

pub(crate) struct AccessorGetter<T>(T);

impl<T> AccessorGetter<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self(value)
    }
}

pub(crate) struct AccessorSetter<T>(T);

impl<T> AccessorSetter<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self(value)
    }
}

#[must_use]
pub(crate) enum AccessorDescriptor<T> {
    Getter(AccessorGetter<T>),
    Setter(AccessorSetter<T>),
    GetterAndSetter {
        getter: AccessorGetter<T>,
        setter: AccessorSetter<T>,
    },
}

pub(crate) type AccessorGetterLocals<'v> = AccessorGetter<&'v ValueLocals>;
pub(crate) type AccessorSetterLocals<'v> = AccessorSetter<&'v ValueLocals>;
pub(crate) type AccessorDescriptorLocals<'v> = AccessorDescriptor<&'v ValueLocals>;

impl<T> AccessorDescriptor<T> {
    /// Materialize the getter before the setter, retaining their roles.
    pub(crate) fn try_map<U, E>(
        self,
        mut materialize: impl FnMut(T) -> Result<U, E>,
    ) -> Result<AccessorDescriptor<U>, E> {
        Ok(match self {
            Self::Getter(AccessorGetter(getter)) => {
                AccessorDescriptor::Getter(AccessorGetter::new(materialize(getter)?))
            }
            Self::Setter(AccessorSetter(setter)) => {
                AccessorDescriptor::Setter(AccessorSetter::new(materialize(setter)?))
            }
            Self::GetterAndSetter {
                getter: AccessorGetter(getter),
                setter: AccessorSetter(setter),
            } => AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(materialize(getter)?),
                setter: AccessorSetter::new(materialize(setter)?),
            },
        })
    }

    /// Only the GC object owner can project a definition into lattice fields.
    pub(super) fn into_fields<R>(self) -> (Presence<T, R>, Presence<T, R>) {
        match self {
            Self::Getter(AccessorGetter(getter)) => (Presence::Present(getter), Presence::Absent),
            Self::Setter(AccessorSetter(setter)) => (Presence::Absent, Presence::Present(setter)),
            Self::GetterAndSetter {
                getter: AccessorGetter(getter),
                setter: AccessorSetter(setter),
            } => (Presence::Present(getter), Presence::Present(setter)),
        }
    }
}

impl AccessorDescriptor<ValueLocals> {
    pub(crate) fn borrowed(&self) -> AccessorDescriptorLocals<'_> {
        match self {
            Self::Getter(AccessorGetter(getter)) => {
                AccessorDescriptor::Getter(AccessorGetter::new(getter))
            }
            Self::Setter(AccessorSetter(setter)) => {
                AccessorDescriptor::Setter(AccessorSetter::new(setter))
            }
            Self::GetterAndSetter {
                getter: AccessorGetter(getter),
                setter: AccessorSetter(setter),
            } => AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(getter),
                setter: AccessorSetter::new(setter),
            },
        }
    }

    pub(crate) fn clear(self, function: &mut Function) {
        match self {
            Self::Getter(AccessorGetter(getter)) => getter.clear(function),
            Self::Setter(AccessorSetter(setter)) => setter.clear(function),
            Self::GetterAndSetter {
                getter: AccessorGetter(getter),
                setter: AccessorSetter(setter),
            } => {
                setter.clear(function);
                getter.clear(function);
            }
        }
    }
}
