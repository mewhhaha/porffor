//! Fresh intrinsic properties retain completed values and defining Realm.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_proxy_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_method(
            context.constructor,
            IntrinsicKey::Name("revocable"),
            StandardBuiltinId::ProxyRevocable,
            context.realm,
            true,
            true,
            function,
        )?;
        Ok(())
    }
}
