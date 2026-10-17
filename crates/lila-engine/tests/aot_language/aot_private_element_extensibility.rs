use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, LanguageFeaturePolicy, PrivateElementExtensibility,
    RealmBuilder, RunOptions,
};

fn assert_policy_pair(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for enabled in [false, true] {
        let language_features = LanguageFeaturePolicy::default()
            .with_private_element_extensibility(if enabled {
                PrivateElementExtensibility::NonExtensibleAppliesToPrivate
            } else {
                PrivateElementExtensibility::Ecma262
            });
        let source = format!("const enabled = {enabled};\n{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions {
                    language_features,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("private-element policy executes through actual Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            outcome.note.contains("boolean(true)"),
            "policy={enabled}: {}",
            outcome.note
        );
    }
}

#[test]
fn private_field_proposal_checks_nonextensibility_after_each_initializer() {
    assert_policy_pair(
        r#"
class Base { constructor(receiver) { return receiver; } }
let initializers = 0;
class Stamp extends Base {
  #value = (++initializers, 41);
  static has(receiver) { return #value in receiver; }
  static read(receiver) { return receiver.#value; }
}
for (const receiver of [Object.preventExtensions({}), Object.seal([]), Object.freeze(function () {})]) {
  let caught;
  try { new Stamp(receiver); } catch (error) { caught = error; }
  if (enabled ? !(caught instanceof TypeError) || Stamp.has(receiver)
              : caught !== undefined || !Stamp.has(receiver) || Stamp.read(receiver) !== 41)
    throw 'private field extensibility';
}
if (initializers !== 3) throw 'initializer order';
class Mutable { #value = 1; next() { return ++this.#value; } }
const mutable = Object.freeze(new Mutable());
mutable.next() === 2 && Object.isFrozen(mutable);
"#,
    );
}

#[test]
fn private_method_proposal_checks_once_before_batch_and_again_for_each_field() {
    assert_policy_pair(
        r#"
let log = '', initializers = 0;
const target = {};
const receiver = new Proxy(target, { isExtensible(value) {
  if (value !== target) throw 'wrong target';
  log += 'extensible;';
  return true;
} });
class Base { constructor(value) { return value; } }
class Stamp extends Base {
  #first() { return 1; }
  #second() { return 2; }
  get #accessor() { return 3; }
  #field = (log += 'initializer;', ++initializers, this.#first() + this.#second() + this.#accessor);
  static read(value) { return value.#field; }
}
new Stamp(receiver);
if (initializers !== 1 || Stamp.read(receiver) !== 6 ||
    log !== (enabled ? 'extensible;initializer;extensible;' : 'initializer;'))
  throw 'method batch count or order';
let emptyCalls = 0;
const nonextensible = new Proxy(Object.preventExtensions({}), { isExtensible() { emptyCalls++; return false; } });
class NoPrivate extends Base { constructor(value) { super(value); } publicMethod() {} }
if (new NoPrivate(nonextensible) !== nonextensible || emptyCalls !== 0) throw 'empty private batch';
let rejectedInitializers = 0;
class Reject extends Base {
  #one() {}
  get #two() { return 2; }
  field = ++rejectedInitializers;
  static has(value) { return #one in value || #two in value; }
}
let caught;
try { new Reject(nonextensible); } catch (error) { caught = error; }
(enabled ? caught instanceof TypeError && !Reject.has(nonextensible) && rejectedInitializers === 0 && emptyCalls === 1
         : caught instanceof TypeError && Reject.has(nonextensible) && rejectedInitializers === 1 && emptyCalls === 0);
"#,
    );
}

#[test]
fn private_field_proposal_preserves_arbitrary_proxy_throw_and_revocation() {
    assert_policy_pair(
        r#"
class Base { constructor(value) { return value; } }
const marker = {};
let calls = 0, initializers = 0;
class Stamp extends Base {
  #field = ++initializers;
  static has(value) { return #field in value; }
}
const receiver = new Proxy({}, { isExtensible() { calls++; throw marker; } });
let caught;
try { new Stamp(receiver); } catch (error) { caught = error; }
if (enabled ? caught !== marker || calls !== 1 || Stamp.has(receiver)
            : caught !== undefined || calls !== 0 || !Stamp.has(receiver)) throw 'trap throw';
const revoked = Proxy.revocable({}, {});
revoked.revoke();
let revokedError;
try { new Stamp(revoked.proxy); } catch (error) { revokedError = error; }
initializers === 2 && (enabled ? revokedError instanceof TypeError && !Stamp.has(revoked.proxy)
                              : revokedError === undefined && Stamp.has(revoked.proxy));
"#,
    );
}

#[test]
fn private_field_proposal_rechecks_duplicate_after_reentrant_extensibility() {
    assert_policy_pair(
        r#"
class Base { constructor(value) { return value; } }
let calls = 0, initializers = 0, receiver;
class Stamp extends Base {
  #field = ++initializers;
  static read(value) { return value.#field; }
}
receiver = new Proxy({}, { isExtensible() {
  calls++;
  if (calls === 1) new Stamp(receiver);
  return true;
} });
let caught;
try { new Stamp(receiver); } catch (error) { caught = error; }
const fieldReentry = enabled ? caught instanceof TypeError && calls === 2 && initializers === 2 && Stamp.read(receiver) === 2
                             : caught === undefined && calls === 0 && initializers === 1 && Stamp.read(receiver) === 1;
let methodCalls = 0, methodInitializers = 0, methodReceiver;
class MethodStamp extends Base {
  #one() { return 1; }
  #two() { return 2; }
  #field = ++methodInitializers;
  static read(value) { return value.#one() + value.#two() + value.#field; }
}
methodReceiver = new Proxy({}, { isExtensible() {
  methodCalls++;
  if (methodCalls === 1) new MethodStamp(methodReceiver);
  return true;
} });
let methodError;
try { new MethodStamp(methodReceiver); } catch (error) { methodError = error; }
fieldReentry && methodInitializers === 1 && MethodStamp.read(methodReceiver) === 4 &&
  (enabled ? methodError instanceof TypeError && methodCalls === 3
           : methodError === undefined && methodCalls === 0);
"#,
    );
}

#[test]
fn private_field_proposal_static_and_base_initializers_keep_partial_brand_state() {
    assert_policy_pair(
        r#"
let exposed, later = 0;
class Base {
  #first = (exposed = this, 1);
  #second = (Object.preventExtensions(this), 2);
  #third = ++later;
  static state(value) { return [#first in value, #second in value, #third in value].join('|'); }
}
let caught;
try { new Base(); } catch (error) { caught = error; }
if (enabled ? !(caught instanceof TypeError) || Base.state(exposed) !== 'true|false|false' || later !== 0
            : caught !== undefined || Base.state(exposed) !== 'true|true|true' || later !== 1) throw 'base partial fields';
let ctor, staticLater = 0;
let staticCaught;
try {
  class Static {
    static #first = (ctor = this, 1);
    static #second = (Object.preventExtensions(this), 2);
    static #third = ++staticLater;
    static state(value) { return [#first in value, #second in value, #third in value].join('|'); }
  }
} catch (error) { staticCaught = error; }
enabled ? staticCaught instanceof TypeError && ctor.state(ctor) === 'true|false|false' && staticLater === 0
        : staticCaught === undefined && ctor.state(ctor) === 'true|true|true' && staticLater === 1;
"#,
    );
}

#[test]
fn private_element_proposal_native_and_prepared_sources_inherit_selected_policy() {
    assert_policy_pair(
        r#"
const source = "class Base { constructor(value) { return value; } } class Stamp extends Base { #field; static has(value) { return #field in value; } } const receiver = Object.preventExtensions({}); let caught; try { new Stamp(receiver); } catch (error) { caught = error; } if (enabled ? !(caught instanceof TypeError) || Stamp.has(receiver) : caught !== undefined || !Stamp.has(receiver)) throw 'dynamic policy'; true;";
let direct = eval(source);
let indirect = (0, eval)(source);
const body = "class Base { constructor(value) { return value; } } class Stamp extends Base { #field; static has(value) { return #field in value; } } const receiver = Object.preventExtensions({}); let caught; try { new Stamp(receiver); } catch (error) { caught = error; } return enabled ? caught instanceof TypeError && !Stamp.has(receiver) : caught === undefined && Stamp.has(receiver);";
const dynamic = Function('enabled', body)(enabled);
// Literal prepared candidates share the same P policy as runtime-native sources.
const prepared = Function('enabled', "class Base { constructor(value) { return value; } } class Stamp extends Base { #field; } try { new Stamp(Object.preventExtensions({})); return !enabled; } catch (error) { return enabled && error instanceof TypeError; }")(enabled);
direct && indirect && dynamic && prepared;
"#,
    );
}

#[test]
fn private_element_proposal_cross_program_super_uses_constructor_defining_policy() {
    assert_policy_pair(
        r#"
let receiver;
class Base { constructor() { return receiver; } }
let body = String.fromCharCode(115, 117, 112, 101, 114, 40, 41, 59);
class Derived extends Base {
  #method() { return 1; }
  #field = 2;
  constructor() { eval(body); }
  static has(value) { return #method in value || #field in value; }
}
receiver = Object.preventExtensions({});
let caught;
try { new Derived(); } catch (error) { caught = error; }
enabled ? caught instanceof TypeError && !Derived.has(receiver)
        : caught === undefined && Derived.has(receiver);
"#,
    );
}

#[test]
fn pinned_nonextensible_private_field_fixture_uses_explicit_policy_in_both_script_modes() {
    const SOURCE: &str = include_str!(
        "../../../../test262/vendor/test262/test/language/statements/class/elements/private-class-field-on-nonextensible-objects.js"
    );
    const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{STA}\n{ASSERT}\n{SOURCE}\ntrue;");
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions {
                    language_features: LanguageFeaturePolicy::default()
                        .with_private_element_extensibility(
                            PrivateElementExtensibility::NonExtensibleAppliesToPrivate,
                        ),
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("pinned proposal fixture executes through actual Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            outcome.note.contains("boolean(true)"),
            "directive={directive:?}: {}",
            outcome.note
        );
    }
}
