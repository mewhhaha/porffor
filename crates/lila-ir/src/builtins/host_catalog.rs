// This registry is the sole source for HostBuiltinId and its host-visible
// surface. Row order is declaration order and therefore the derived `Ord`
// contract. Every row must classify its exposure and realm installation scope.
use super::*;

host_builtin_catalog! {
    Print {
        name: PRINT_NAME,
        function: HOST_PRINT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::ProductExtension,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    Gc {
        name: GC_NAME,
        function: HOST_GC_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::ProductExtension,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AssertThrows {
        name: ASSERT_THROWS_NAME,
        function: HOST_ASSERT_THROWS_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    IsConstructor {
        name: IS_CONSTRUCTOR_NAME,
        function: HOST_IS_CONSTRUCTOR_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    CreateRealm {
        name: CREATE_REALM_NAME,
        function: HOST_CREATE_REALM_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    RealmEvalScript {
        name: REALM_EVAL_SCRIPT_NAME,
        function: DYNAMIC_REALM_EVAL_SCRIPT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    CreateHTMLDDA {
        name: CREATE_HTMLDDA_NAME,
        function: HOST_CREATE_HTMLDDA_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    HTMLDDA {
        name: "IsHTMLDDA",
        function: HOST_HTMLDDA_FUNCTION_ID,
        surface: HostBuiltinSurface::InternalCallable,
    }
    ParseInt {
        name: PARSE_INT_NAME,
        function: HOST_PARSE_INT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::EcmaGlobal,
            HostBuiltinRealmScope::EveryRealm,
        ),
    }
    ParseFloat {
        name: PARSE_FLOAT_NAME,
        function: HOST_PARSE_FLOAT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::EcmaGlobal,
            HostBuiltinRealmScope::EveryRealm,
        ),
    }
    DetachArrayBuffer {
        name: DETACH_ARRAY_BUFFER_NAME,
        function: HOST_DETACH_ARRAY_BUFFER_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentStart {
        name: AGENT_START_NAME,
        function: HOST_AGENT_START_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentBroadcast {
        name: AGENT_BROADCAST_NAME,
        function: HOST_AGENT_BROADCAST_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentReceiveBroadcast {
        name: AGENT_RECEIVE_BROADCAST_NAME,
        function: HOST_AGENT_RECEIVE_BROADCAST_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentReport {
        name: AGENT_REPORT_NAME,
        function: HOST_AGENT_REPORT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentGetReport {
        name: AGENT_GET_REPORT_NAME,
        function: HOST_AGENT_GET_REPORT_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentSleep {
        name: AGENT_SLEEP_NAME,
        function: HOST_AGENT_SLEEP_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentMonotonicNow {
        name: AGENT_MONOTONIC_NOW_NAME,
        function: HOST_AGENT_MONOTONIC_NOW_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
    AgentLeaving {
        name: AGENT_LEAVING_NAME,
        function: HOST_AGENT_LEAVING_FUNCTION_ID,
        surface: HostBuiltinSurface::global(
            HostBuiltinExposure::Test262Capability,
            HostBuiltinRealmScope::EntryRealmOnly,
        ),
    }
}
