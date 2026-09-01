use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

#[test]
fn reflect_define_property_trap_descriptor_uses_the_method_realm_object_prototype() {
    let source = r#"
var mainDescriptorPrototype;
var mainProxy = new Proxy({}, {
  defineProperty: function (target, key, descriptor) {
    mainDescriptorPrototype = Object.getPrototypeOf(descriptor);
    return true;
  }
});
var mainResult = Reflect.defineProperty(mainProxy, "value", { value: 1 });

var other = __lilaCreateRealm().global;
var otherDescriptorPrototype;
var otherProxy = new Proxy({}, {
  defineProperty: function (target, key, descriptor) {
    otherDescriptorPrototype = Object.getPrototypeOf(descriptor);
    return true;
  }
});
var otherResult = other.Reflect.defineProperty(otherProxy, "value", { value: 2 });

mainResult
  && otherResult
  && mainDescriptorPrototype === Object.prototype
  && otherDescriptorPrototype === other.Object.prototype
  && otherDescriptorPrototype !== Object.prototype;
"#;
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Reflect.defineProperty should expose both trap descriptors");

    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}
