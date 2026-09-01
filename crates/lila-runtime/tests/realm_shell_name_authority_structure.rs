const RUNTIME_SOURCE: &str = include_str!("../src/lib.rs");
const ENGINE_SOURCE: &str = include_str!("../../lila-engine/src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn realm_retains_host_hooks_without_caching_a_shell_name() {
    let realm = bounded(
        RUNTIME_SOURCE,
        "pub struct Realm {",
        "impl core::fmt::Debug for Realm",
    );
    assert!(realm.contains("host_hooks: Arc<dyn HostHooks>,"));
    assert!(!realm.contains("shell_name:"));

    let build = bounded(
        RUNTIME_SOURCE,
        "pub fn build(self) -> Realm {",
        "impl Realm {",
    );
    assert!(!build.contains("shell_name:"));
    assert!(!build.contains("shell_name().to_string()"));
}

#[test]
fn runtime_and_engine_project_the_authoritative_host_hook_name() {
    let realm_methods = bounded(RUNTIME_SOURCE, "impl Realm {", "#[cfg(test)]");
    let shell_name = bounded(
        realm_methods,
        "pub fn shell_name(&self) -> &'static str {",
        "pub fn host_hooks(&self)",
    );
    assert!(shell_name.contains("self.host_hooks.shell_name()"));

    let engine = bounded(ENGINE_SOURCE, "impl Engine {", "pub fn compile_script(");
    assert!(engine.contains("pub fn shell_name(&self) -> &str {"));
    assert!(engine.contains("self.realm.shell_name()"));
    assert!(!engine.contains("&self.realm.shell_name"));
}
