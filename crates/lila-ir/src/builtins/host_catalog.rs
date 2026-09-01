// This registry is the sole source for HostBuiltinId and its host-visible
// surface. Row order is declaration order and therefore the derived `Ord`
// contract. Every row must classify its exposure, which determines realm scope.
use super::*;

host_builtin_catalog! {
    Print {
        name: PRINT_NAME,
        function: HOST_PRINT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::ProductExtension),
    }
    Gc {
        name: GC_NAME,
        function: HOST_GC_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::ProductExtension),
    }
    AssertThrows {
        name: ASSERT_THROWS_NAME,
        function: HOST_ASSERT_THROWS_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    IsConstructor {
        name: IS_CONSTRUCTOR_NAME,
        function: HOST_IS_CONSTRUCTOR_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    CreateRealm {
        name: CREATE_REALM_NAME,
        function: HOST_CREATE_REALM_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    RealmEvalScript {
        name: REALM_EVAL_SCRIPT_NAME,
        function: DYNAMIC_REALM_EVAL_SCRIPT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    CreateHTMLDDA {
        name: CREATE_HTMLDDA_NAME,
        function: HOST_CREATE_HTMLDDA_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    HTMLDDA {
        name: "IsHTMLDDA",
        function: HOST_HTMLDDA_FUNCTION_ID,
        surface: HostBuiltinSurface::InternalCallable,
    }
    ParseInt {
        name: PARSE_INT_NAME,
        function: HOST_PARSE_INT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::EcmaGlobal),
    }
    ParseFloat {
        name: PARSE_FLOAT_NAME,
        function: HOST_PARSE_FLOAT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::EcmaGlobal),
    }
    DetachArrayBuffer {
        name: DETACH_ARRAY_BUFFER_NAME,
        function: HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentStart {
        name: AGENT_START_NAME,
        function: HOST_AGENT_START_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentBroadcast {
        name: AGENT_BROADCAST_NAME,
        function: HOST_AGENT_BROADCAST_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentReceiveBroadcast {
        name: AGENT_RECEIVE_BROADCAST_NAME,
        function: HOST_AGENT_RECEIVE_BROADCAST_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentReport {
        name: AGENT_REPORT_NAME,
        function: HOST_AGENT_REPORT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentGetReport {
        name: AGENT_GET_REPORT_NAME,
        function: HOST_AGENT_GET_REPORT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentSleep {
        name: AGENT_SLEEP_NAME,
        function: HOST_AGENT_SLEEP_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentMonotonicNow {
        name: AGENT_MONOTONIC_NOW_NAME,
        function: HOST_AGENT_MONOTONIC_NOW_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
    AgentLeaving {
        name: AGENT_LEAVING_NAME,
        function: HOST_AGENT_LEAVING_FUNCTION_ID,
        surface: HostBuiltinSurface::global(HostBuiltinExposure::Test262Capability),
    }
}

impl HostBuiltinId {
    pub(crate) const fn may_invalidate_caller_flow(self) -> bool {
        match self {
            Self::CreateRealm => false,
            Self::Print
            | Self::Gc
            | Self::AssertThrows
            | Self::IsConstructor
            | Self::RealmEvalScript
            | Self::CreateHTMLDDA
            | Self::HTMLDDA
            | Self::ParseInt
            | Self::ParseFloat
            | Self::DetachArrayBuffer
            | Self::AgentStart
            | Self::AgentBroadcast
            | Self::AgentReceiveBroadcast
            | Self::AgentReport
            | Self::AgentGetReport
            | Self::AgentSleep
            | Self::AgentMonotonicNow
            | Self::AgentLeaving => true,
        }
    }

    pub const fn may_run_user_code_synchronously(self) -> bool {
        match self {
            Self::AssertThrows | Self::ParseInt | Self::ParseFloat => true,
            Self::Print
            | Self::Gc
            | Self::IsConstructor
            | Self::CreateRealm
            | Self::RealmEvalScript
            | Self::CreateHTMLDDA
            | Self::HTMLDDA
            | Self::DetachArrayBuffer
            | Self::AgentStart
            | Self::AgentBroadcast
            | Self::AgentReceiveBroadcast
            | Self::AgentReport
            | Self::AgentGetReport
            | Self::AgentSleep
            | Self::AgentMonotonicNow
            | Self::AgentLeaving => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_realm_is_the_only_host_builtin_that_preserves_caller_flow() {
        for builtin in HostBuiltinId::ALL.iter().copied() {
            assert_eq!(
                builtin.may_invalidate_caller_flow(),
                builtin != HostBuiltinId::CreateRealm,
                "{builtin:?}"
            );
        }
    }

    #[test]
    fn host_caller_flow_classification_is_distinct_from_synchronous_user_code() {
        assert!(!HostBuiltinId::DetachArrayBuffer.may_run_user_code_synchronously());
        assert!(HostBuiltinId::DetachArrayBuffer.may_invalidate_caller_flow());
    }
}
