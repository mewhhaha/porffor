use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_private_update(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("private numeric update failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn prefix_and_postfix_private_fields_preserve_numeric_types_and_values() {
    assert_private_update(
        r#"
class Counter {
  #value;
  constructor(value) { this.#value = value; }
  read() { return this.#value; }
  postIncrement() { return this.#value++; }
  preIncrement() { return ++this.#value; }
  postDecrement() { return this.#value--; }
  preDecrement() { return --this.#value; }
}
var text = new Counter('7');
if (text.postIncrement() !== 7 || text.read() !== 8 || text.preIncrement() !== 9 ||
    text.postDecrement() !== 9 || text.preDecrement() !== 7) throw 'Number update';
var zero = new Counter(-0);
if (!Object.is(zero.postIncrement(), -0) || zero.read() !== 1) throw 'signed zero';
var positive = new Counter(9223372036854775807n);
if (positive.postIncrement() !== 9223372036854775807n || positive.read() !== 9223372036854775808n ||
    positive.postDecrement() !== 9223372036854775808n || positive.read() !== 9223372036854775807n) {
  throw 'inline heap transition';
}
var negative = new Counter(-9223372036854775808n);
if (negative.preDecrement() !== -9223372036854775809n ||
    negative.preIncrement() !== -9223372036854775808n) throw 'negative heap transition';
var wide = new Counter(18446744073709551615n);
wide.preIncrement() === 18446744073709551616n && wide.preDecrement() === 18446744073709551615n;
"#,
    );
}

#[test]
fn private_accessors_evaluate_the_base_once_and_keep_it_across_rebinding() {
    assert_private_update(
        r#"
var trace = [], current, replacement;
class Counter {
  #stored;
  constructor(value) { this.#stored = value; }
  read() { return this.#stored; }
  get #value() {
    trace.push('get');
    current = replacement;
    var value = this.#stored;
    return { [Symbol.toPrimitive](hint) { trace.push(hint); current = replacement; return value; } };
  }
  set #value(value) { trace.push('set'); this.#stored = value; return 'ignored setter return'; }
  static updateIdentifier() { return current.#value++; }
  static updateCall() { return ++base().#value; }
}
function base() { trace.push('base'); return current; }
var original = new Counter(9223372036854775807n);
replacement = new Counter(10n);
current = original;
var old = Counter.updateIdentifier();
if (old !== 9223372036854775807n || original.read() !== 9223372036854775808n ||
    replacement.read() !== 10n || current !== replacement || trace.join(',') !== 'get,number,set') {
  throw 'captured identifier base';
}
trace = [];
current = original;
var next = Counter.updateCall();
next === 9223372036854775809n && original.read() === next && replacement.read() === 10n &&
  trace.join(',') === 'base,get,number,set';
"#,
    );
}

#[test]
fn private_update_abrupt_completions_preserve_identity_and_stop_later_steps() {
    assert_private_update(
        r#"
var marker = {}, received, reads = 0, conversions = 0, writes = 0;
class GetterThrows {
  get #value() { reads++; throw marker; }
  set #value(value) { writes++; }
  update() { return this.#value++; }
}
try { new GetterThrows().update(); } catch (error) { received = error; }
if (received !== marker || reads !== 1 || writes !== 0) throw 'getter abrupt';
class CoercionThrows {
  get #value() { return { valueOf() { conversions++; throw marker; } }; }
  set #value(value) { writes++; }
  update() { return --this.#value; }
}
try { new CoercionThrows().update(); } catch (error) { received = error; }
if (received !== marker || conversions !== 1 || writes !== 0) throw 'coercion abrupt';
class SetterThrows {
  get #value() { return { valueOf() { conversions++; return 9223372036854775807n; } }; }
  set #value(value) { writes++; if (value !== 9223372036854775808n) throw 'wrong new value'; throw marker; }
  update() { return ++this.#value; }
}
try { new SetterThrows().update(); } catch (error) { received = error; }
received === marker && conversions === 2 && writes === 1;
"#,
    );
}

#[test]
fn private_brand_and_missing_accessor_checks_keep_their_required_order() {
    assert_private_update(
        r#"
var caught = 0, reads = 0, conversions = 0, writes = 0;
class Counter {
  #value = 1;
  static update(base) { return base.#value++; }
}
for (const base of [{}, null, 1]) {
  try { Counter.update(base); } catch (error) { if (error instanceof TypeError) caught++; }
}
class MissingGetter {
  set #value(value) { writes++; }
  update() { return this.#value++; }
}
try { new MissingGetter().update(); } catch (error) { if (error instanceof TypeError) caught++; }
class MissingSetter {
  get #value() { reads++; return { valueOf() { conversions++; return 1n; } }; }
  update() { return this.#value--; }
}
try { new MissingSetter().update(); } catch (error) { if (error instanceof TypeError) caught++; }
class SymbolValue {
  get #value() { reads++; return Symbol(); }
  set #value(value) { writes++; }
  update() { return ++this.#value; }
}
try { new SymbolValue().update(); } catch (error) { if (error instanceof TypeError) caught++; }
caught === 6 && reads === 2 && conversions === 1 && writes === 0;
"#,
    );
}

#[test]
fn existing_private_fields_remain_mutable_on_sealed_frozen_and_stamped_objects() {
    assert_private_update(
        r#"
class Counter {
  #value = 1;
  next() { return ++this.#value; }
}
var counter = new Counter();
Object.seal(counter);
if (counter.next() !== 2) throw 'sealed private field';
Object.freeze(counter);
if (counter.next() !== 3 || !Object.isFrozen(counter)) throw 'frozen private field';
class Override { constructor(base) { return base; } }
class Stamp extends Override {
  #value = 10n;
  static next(base) { return base.#value++; }
  static read(base) { return base.#value; }
}
var receiver = {}, proxy = new Proxy({}, {});
new Stamp(receiver);
new Stamp(proxy);
Object.freeze(receiver);
Object.freeze(proxy);
Stamp.next(receiver) === 10n && Stamp.read(receiver) === 11n && Object.isFrozen(receiver) &&
  Stamp.next(proxy) === 10n && Stamp.read(proxy) === 11n && Object.isFrozen(proxy);
"#,
    );
}

#[test]
fn private_updates_retain_class_brands_across_generator_resumption_and_closures() {
    assert_private_update(
        r#"
function make() {
  return class Counter {
    #value = 0n;
    static #count = 0;
    static next() { return ++this.#count; }
    *steps() { yield this.#value++; return ++this.#value; }
    closure() { return () => this.#value++; }
  };
}
var First = make(), Second = make(), first = new First(), second = new Second();
var steps = first.steps(), a = steps.next(), b = steps.next(), next = first.closure();
a.value === 0n && !a.done && b.value === 2n && b.done && next() === 2n && next() === 3n &&
  second.closure()() === 0n && First.next() === 1 && First.next() === 2 && Second.next() === 1;
"#,
    );
}

#[test]
fn private_hooks_invalidate_flow_facts_and_keep_arbitrary_catch_values() {
    assert_private_update(
        r#"
function outer() {
  let label = 1;
  class Counter {
    #value = { valueOf() { label = 'changed'; return 1; } };
    update() { return this.#value++; }
  }
  new Counter().update();
  return label + 1;
}
class Throwing {
  get #value() { throw 'private'; }
  update(flag) {
    try { if (flag) throw 1; this.#value++; }
    catch (caught) { return caught + 1; }
  }
}
outer() === 'changed1' && new Throwing().update(false) === 'private1';
"#,
    );
}
