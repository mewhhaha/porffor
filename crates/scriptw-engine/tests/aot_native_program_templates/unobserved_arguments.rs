use super::succeeds;

#[test]
fn strict_empty_methods_and_observed_arguments_keep_their_actual_semantics() {
    succeeds(
        r#"
      var E = eval('(class { method() {} get value() {} set value(v) {} static method() {} })');
      var e = new E();
      if (e.method(1, 2) !== undefined || E.method(3) !== undefined || e.value !== undefined)
        throw new Error('empty strict body');
      e.value = 7;
      var setter = Object.getOwnPropertyDescriptor(E.prototype, 'value').set;
      if (setter.call(e, 8, 9) !== undefined) throw new Error('empty setter');
      for (var repeat = 0; repeat < 520; repeat++) {
        e.method();
        setter.call(e, repeat);
      }
      var empty = (function () { 'use strict'; return function Empty(value) {}; })();
      var effects = 0;
      if (empty(++effects) !== undefined || effects !== 1 || empty.length !== 1)
        throw new Error('inert parameters retain caller argument effects and length');
      var instance = new empty();
      if (!(instance instanceof empty) || Object.getPrototypeOf(instance) !== empty.prototype)
        throw new Error('ordinary empty constructor result');
      function inner() { return inner.caller; }
      function outer() { e.method(); return inner(); }
      if (outer() !== outer) throw new Error('active source callable was not restored');
      for (var key of ['arguments', 'caller']) {
        var rejected = false;
        try { e.method[key]; } catch (error) { rejected = error instanceof TypeError; }
        if (!rejected) throw new Error('strict reflection');
      }
      class Observed {
        method(value) { value = 99; return arguments; }
        capture(value) { return () => arguments; }
        evaluate(value) { return eval('arguments'); }
        defaults(value = arguments.length) { return value; }
      }
      var o = new Observed();
      var a = o.method(11), b = o.method(12);
      if (a === b || a[0] !== 11 || b[0] !== 12) throw new Error('unmapped identity');
      if (o.capture(13)()[0] !== 13 || o.evaluate(14)[0] !== 14)
        throw new Error('retained arguments owner');
      if (o.defaults(undefined, 15) !== 2) throw new Error('parameter observation');
      class Parameters {
        defaults(value = ++effects) {}
        destructure({ value }) {}
        constructor() { this.initialized = true; }
      }
      var parameters = new Parameters();
      if (!parameters.initialized) throw new Error('constructor body retained');
      parameters.defaults();
      parameters.destructure({ get value() { effects++; return 1; } });
      if (effects !== 3) throw new Error('parameter effects retained');
      1;
    "#,
    );
}
