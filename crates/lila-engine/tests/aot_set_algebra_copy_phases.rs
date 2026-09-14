use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_set_algebra(source: &str) {
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
        .unwrap_or_else(|error| panic!("Set algebra failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn difference_visits_the_original_copy_when_has_clears_and_adds_receiver_keys() {
    assert_set_algebra(
        r#"
const receiver = new Set([1, 2, 3, 4]);
const seen = [];
const other = {
  size: 100,
  has(value) {
    seen.push(value);
    if (seen.length === 1) { receiver.clear(); receiver.add(11); receiver.add(22); }
    return true;
  },
  keys() { throw 'unexpected keys'; }
};
const result = receiver.difference(other);
result.size === 0 && Array.from(receiver).join(',') === '11,22' &&
seen.join(',') === '1,2,3,4';
"#,
    );
}

#[test]
fn difference_copies_before_calling_keys_and_reading_the_iterator_next_method() {
    assert_set_algebra(
        r#"
const receiver = new Set([1, 2, 3]);
const order = [];
const other = {size: 0, has() { throw 'unexpected has'; }, keys() {
  order.push('keys'); receiver.clear(); receiver.add(4);
  return {get next() {
    order.push('get next'); receiver.clear(); receiver.add(5);
    let index = 0;
    return function() { return index < 3 ? {done: false, value: ++index} : {done: true}; };
  }};
}};
const result = receiver.difference(other);
result.size === 0 && Array.from(receiver).join(',') === '5' &&
order.join('|') === 'keys|get next';
"#,
    );
}

#[test]
fn symmetric_difference_and_union_copy_after_next_lookup_and_before_first_next_call() {
    assert_set_algebra(
        r#"
for (const method of ['symmetricDifference', 'union']) {
  const receiver = new Set([1]);
  const order = [];
  const other = {
    get size() { order.push('get size'); return 0; },
    get has() { order.push('get has'); return function() { throw 'unexpected has'; }; },
    get keys() {
      order.push('get keys');
      return function() {
        order.push('keys'); receiver.clear(); receiver.add(2);
        return {get next() {
          order.push('get next'); receiver.clear(); receiver.add(3);
          return function() {
            order.push('next'); receiver.clear(); receiver.add(4);
            return {done: true, get value() { throw 'unexpected value'; }};
          };
        }};
      };
    }
  };
  const result = receiver[method](other);
  if (Array.from(result).join(',') !== '3' || Array.from(receiver).join(',') !== '4' ||
      order.join('|') !== 'get size|get has|get keys|keys|get next|next') throw method;
}
true;
"#,
    );
}

#[test]
fn copying_preserves_duplicate_suppression_zero_normalization_and_intrinsic_result_order() {
    assert_set_algebra(
        r#"
for (const method of ['symmetricDifference', 'union']) {
  const receiver = new Set([99]);
  const other = {size: 0, has() { throw 'unexpected has'; }, keys() {
    return {get next() {
      receiver.clear(); receiver.add(1); receiver.add(-0);
      const values = [2, 2, -0, 3, 3]; let index = 0;
      return function() {
        return index < values.length ? {done: false, value: values[index++]} : {done: true};
      };
    }};
  }};
  const result = receiver[method](other);
  const expected = method === 'union' ? '1,0,2,3' : '1,2,3';
  if (Array.from(result).join(',') !== expected || Object.getPrototypeOf(result) !== Set.prototype)
    throw method;
  if (method === 'union' && !Object.is(Array.from(result)[1], 0)) throw 'negative zero';
}
true;
"#,
    );
}

#[test]
fn intersection_keeps_its_live_receiver_traversal_and_does_not_add_reinserted_keys_twice() {
    assert_set_algebra(
        r#"
const receiver = new Set([1, 2, 3]);
const seen = [];
const other = {size: 100, has(value) {
  if (value === 2 && !seen.includes(value)) { receiver.delete(value); receiver.add(value); }
  seen.push(value); return true;
}, keys() { throw 'unexpected keys'; }};
const result = receiver.intersection(other);
Array.from(result).join(',') === '1,2,3' && seen.join(',') === '1,2,3,2';
"#,
    );
}
