//! `Proxy` intrinsic installation.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_proxy_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Unused bindings are expected: most families touch only a
        // few of these.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta: _,
            prototype_global_index: _,
            constructor_global_index: _,
            object_local,
            key_local: _,
            payload_local: _,
            tag_local: _,
            prototype_object_local: _,
        } = *context;

        let revocable_meta = self
            .functions
            .get(&StandardBuiltinId::ProxyRevocable.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Proxy.revocable`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "revocable", revocable_meta, function)?;

        Ok(())
    }
}
