//! Fresh intrinsic properties retain completed values and defining Realm.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter, AccessorSetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_error_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("toString"),
            StandardBuiltinId::ErrorPrototypeToString,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            context.constructor,
            IntrinsicKey::Name("isError"),
            StandardBuiltinId::ErrorIsError,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("stack"),
            AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(StandardBuiltinId::ErrorPrototypeStackGetter),
                setter: AccessorSetter::new(StandardBuiltinId::ErrorPrototypeStackSetter),
            },
            context.realm,
            true,
            function,
        )?;
        Ok(())
    }
}
