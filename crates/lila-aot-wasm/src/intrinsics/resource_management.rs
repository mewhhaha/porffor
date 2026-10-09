//! DisposableStack disposal aliases share one completed native function.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_disposable_stack_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("use", StandardBuiltinId::DisposableStackPrototypeUse),
            ("adopt", StandardBuiltinId::DisposableStackPrototypeAdopt),
            ("defer", StandardBuiltinId::DisposableStackPrototypeDefer),
            ("move", StandardBuiltinId::DisposableStackPrototypeMove),
        ] {
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Name(name),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        self.emit_install_intrinsic_method_aliases(
            context.prototype,
            &[
                IntrinsicKey::Name("dispose"),
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Dispose),
            ],
            StandardBuiltinId::DisposableStackPrototypeDispose,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("disposed"),
            AccessorDescriptor::Getter(AccessorGetter::new(
                StandardBuiltinId::DisposableStackPrototypeDisposedGetter,
            )),
            context.realm,
            true,
            function,
        )?;
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "DisposableStack",
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }
}
