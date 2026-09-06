use super::*;
use crate::emit::ProxyExecutionRealmSource;

enum ProxyExecutionRealmAccess {
    TrustedCurrentEnvironment,
    MainRealmFallback,
}

const fn proxy_execution_realm_access(
    source: ProxyExecutionRealmSource,
) -> ProxyExecutionRealmAccess {
    match source {
        ProxyExecutionRealmSource::MainRealmFallback => {
            ProxyExecutionRealmAccess::MainRealmFallback
        }
        ProxyExecutionRealmSource::StandardBuiltinEnvironment
        | ProxyExecutionRealmSource::ObjectReadHelperArgument
        | ProxyExecutionRealmSource::ProxyDispatchHelperArgument => {
            ProxyExecutionRealmAccess::TrustedCurrentEnvironment
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_proxy_execution_realm_argument(&self, function: &mut Function) {
        match proxy_execution_realm_access(self.proxy_execution_realm_source()) {
            ProxyExecutionRealmAccess::TrustedCurrentEnvironment => {
                function.instruction(&Instruction::LocalGet(self.current_env_local));
            }
            ProxyExecutionRealmAccess::MainRealmFallback => {
                function.instruction(&Instruction::I64Const(0));
            }
        }
    }

    pub(crate) fn emit_proxy_execution_realm_type_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match proxy_execution_realm_access(self.proxy_execution_realm_source()) {
            ProxyExecutionRealmAccess::TrustedCurrentEnvironment => self
                .emit_throw_current_function_realm_type_error(
                    message,
                    payload_local,
                    tag_local,
                    function,
                ),
            ProxyExecutionRealmAccess::MainRealmFallback => self.emit_throw_runtime_error(
                TYPE_ERROR_NAME,
                message,
                payload_local,
                tag_local,
                function,
            ),
        }
    }

    pub(crate) fn emit_install_proxy_execution_realm_array_prototype(
        &mut self,
        array_payload_local: u32,
        function: &mut Function,
    ) {
        match proxy_execution_realm_access(self.proxy_execution_realm_source()) {
            ProxyExecutionRealmAccess::TrustedCurrentEnvironment => {
                let prototype = self.emit_load_current_function_realm_array_prototype(function);
                self.emit_install_current_function_realm_array_prototype(
                    array_payload_local,
                    prototype,
                    function,
                );
            }
            ProxyExecutionRealmAccess::MainRealmFallback => {
                let prototype_local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_local));
                self.store_i64_local_at_offset(
                    array_payload_local,
                    HEAP_PROTOTYPE_OFFSET,
                    prototype_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    array_payload_local,
                    HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Array.tag() as u64,
                    function,
                );
                self.release_temp_local(prototype_local);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_execution_realm_excludes_ordinary_lexical_environments() {
        let proxy_dispatch_helpers = RuntimeHelperId::ALL
            .iter()
            .copied()
            .filter(|helper| {
                ProxyExecutionRealmSource::for_runtime_helper(*helper)
                    == ProxyExecutionRealmSource::ProxyDispatchHelperArgument
            })
            .collect::<Vec<_>>();
        assert_eq!(
            proxy_dispatch_helpers,
            vec![RuntimeHelperId::ProxyCall, RuntimeHelperId::ProxyConstruct]
        );

        let object_read_helpers = RuntimeHelperId::ALL
            .iter()
            .copied()
            .filter(|helper| {
                ProxyExecutionRealmSource::for_runtime_helper(*helper)
                    == ProxyExecutionRealmSource::ObjectReadHelperArgument
            })
            .collect::<Vec<_>>();
        assert_eq!(
            object_read_helpers,
            vec![
                RuntimeHelperId::ObjectRead,
                RuntimeHelperId::ObjectReadProxy,
                RuntimeHelperId::IndexedElementRead
            ]
        );

        for source in [
            ProxyExecutionRealmSource::StandardBuiltinEnvironment,
            ProxyExecutionRealmSource::ObjectReadHelperArgument,
            ProxyExecutionRealmSource::ProxyDispatchHelperArgument,
        ] {
            match proxy_execution_realm_access(source) {
                ProxyExecutionRealmAccess::TrustedCurrentEnvironment => {}
                ProxyExecutionRealmAccess::MainRealmFallback => {
                    panic!("trusted Proxy dispatch source lost its execution Realm")
                }
            }
        }

        match proxy_execution_realm_access(ProxyExecutionRealmSource::MainRealmFallback) {
            ProxyExecutionRealmAccess::MainRealmFallback => {}
            ProxyExecutionRealmAccess::TrustedCurrentEnvironment => {
                panic!("ordinary lexical environment became Proxy execution Realm metadata")
            }
        }
    }
}
