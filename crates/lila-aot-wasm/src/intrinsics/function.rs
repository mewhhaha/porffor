//! Fresh intrinsic properties retain completed values and defining Realm.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_function_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("call", StandardBuiltinId::FunctionPrototypeCall),
            ("apply", StandardBuiltinId::FunctionPrototypeApply),
            ("bind", StandardBuiltinId::FunctionPrototypeBind),
            ("toString", StandardBuiltinId::FunctionPrototypeToString),
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
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::HasInstance),
            StandardBuiltinId::FunctionPrototypeSymbolHasInstance,
            context.realm,
            false,
            false,
            function,
        )?;
        Ok(())
    }
}
