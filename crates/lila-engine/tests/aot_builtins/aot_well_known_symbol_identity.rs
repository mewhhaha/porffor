use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};
use lila_ir::WellKnownSymbol;

#[test]
fn lowered_known_symbols_match_dynamic_and_foreign_keys_without_string_collisions() {
    let members = WellKnownSymbol::ALL
        .iter()
        .map(|symbol| format!("Symbol.{}", symbol.member_name()))
        .collect::<Vec<_>>()
        .join(",");
    let names = WellKnownSymbol::ALL
        .iter()
        .map(|symbol| format!("'{0}'", symbol.member_name()))
        .collect::<Vec<_>>()
        .join(",");
    let descriptions = WellKnownSymbol::ALL
        .iter()
        .map(|symbol| format!("'{0}'", symbol.description()))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "function known() {{ return [{members}]; }}\n\
         function require(value, stage, index) {{ if (!value) throw stage + ':' + index; }}\n\
         const retained = known(); const object = {{}};\n\
         const foreign = __lilaCreateRealm().global;\n\
         const names = [{names}], descriptions = [{descriptions}];\n\
         for (let index = 0; index < names.length; index++) {{\n\
             const member = names[index], description = descriptions[index];\n\
             require(typeof retained[index] === 'symbol', 'type', index);\n\
             require(retained[index] === Reflect.get(Symbol, member), 'dynamic', index);\n\
             require(retained[index] === Reflect.get(foreign.Symbol, member), 'realm', index);\n\
             object[retained[index]] = index; object[description] = 'string';\n\
             require(Reflect.get(object, Reflect.get(Symbol, member)) === index, 'key', index);\n\
             require(Reflect.get(object, description) === 'string', 'string', index);\n\
         }}\n\
         require(Reflect.ownKeys(object).length === {}, 'own-keys', 0);\n\
         print('known-symbols:ok');",
        WellKnownSymbol::ALL.len() * 2,
    );
    lila_engine::configure_compilation_jobs(1).unwrap();
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            &source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{observed:?}"
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("known-symbols:ok".into())]
    );
}
