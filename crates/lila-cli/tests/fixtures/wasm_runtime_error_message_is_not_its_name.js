// A runtime-thrown error must carry a real `message`, distinct from its `name`.
//
// `emit_runtime_error_object` (crates/lila-aot-wasm/src/builtins/errors.rs)
// defines the `message` property from the error's *name* payload and ignores the
// message it was handed, so every error the runtime throws reports
// `e.message === e.name`. Nothing else in this repository observes that: no
// Test262 case reads a runtime-thrown error's message, and before this fixture no
// CLI fixture did either, so the defect could be rediscovered indefinitely while
// every suite stayed green. This fixture is the observer.
//
// Spec-correct output is `string(message-differs)`. Today it is
// `string(message-equals-name)`.

var observed = "no-throw";
try {
  null.x;
} catch (e) {
  if (!(e instanceof TypeError)) {
    observed = "wrong-error-kind";
  } else if (e.message === e.name) {
    observed = "message-equals-name";
  } else {
    observed = "message-differs";
  }
}

observed;
