use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_standalone_locale(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("standalone Locale provider call must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn standalone_constructor_resolves_a_provider_language_alias() {
    assert_standalone_locale(
        "new Intl.Locale('iw').language === 'he';",
        HostSurfacePolicy::default(),
    );
}

#[test]
fn standalone_options_resolve_provider_aliases_and_read_stored_fields() {
    assert_standalone_locale(
        "var locale = new Intl.Locale('en', {calendar:'islamicc',numeric:true}); locale.calendar === 'islamic-civil' && locale.numeric === true;",
        HostSurfacePolicy::default(),
    );
}

#[test]
fn standalone_created_realm_constructor_uses_the_provider() {
    assert_standalone_locale(
        "var foreign = __lilaCreateRealm().global; var locale = new foreign.Intl.Locale('iw', {calendar:'islamicc'}); locale.language === 'he' && locale.calendar === 'islamic-civil' && Object.getPrototypeOf(locale) === foreign.Intl.Locale.prototype;",
        HostSurfacePolicy::Test262,
    );
}
