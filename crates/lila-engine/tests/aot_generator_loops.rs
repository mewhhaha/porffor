use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_generator_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("generator loop must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(&outcome.completion, ObservedCompletion::Normal(_)),
        "completion: {:?}\nsource:\n{source}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn generator_loop_lexicals_survive_resume_and_remain_per_activation() {
    assert_generator_trace(
        r#"
function* values() {
  for (let index = 0; index < 4; index++) {
    let value = index * 2;
    yield value;
    print("resumed:" + value);
  }
}
function report(result) { print(result.value + ":" + result.done); }
var left = values();
var right = values();
report(left.next());
report(left.next());
report(right.next());
report(left.next());
report(left.next());
report(left.next());
report(right.next());
"#,
        &[
            "0:false",
            "resumed:0",
            "2:false",
            "0:false",
            "resumed:2",
            "4:false",
            "resumed:4",
            "6:false",
            "resumed:6",
            "undefined:true",
            "resumed:0",
            "2:false",
        ],
    );
}

#[test]
fn conditional_generator_loop_skips_iterations_and_resumes_after_selected_branch_yield() {
    assert_generator_trace(
        r#"
function* oddValues() {
  for (var index = 0; index < 4; index++) {
    let value = index * 10;
    print("before:" + index);
    if (index % 2) {
      yield value;
      print("branch:" + value);
    }
    print("after:" + value);
  }
  yield "tail";
}
function report(result) { print(result.value + ":" + result.done); }
var iterator = oddValues();
report(iterator.next());
report(iterator.next());
report(iterator.next());
report(iterator.next());
"#,
        &[
            "before:0",
            "after:0",
            "before:1",
            "10:false",
            "branch:10",
            "after:10",
            "before:2",
            "after:20",
            "before:3",
            "30:false",
            "branch:30",
            "after:30",
            "tail:false",
            "undefined:true",
        ],
    );
}

#[test]
fn generator_branch_lexicals_keep_their_shadowed_slots_across_resume() {
    assert_generator_trace(
        r#"
function* values(chooseLeft) {
  let value = 99;
  if (chooseLeft) {
    const value = 7;
    yield value;
    print("left:" + value);
  } else {
    let value = 8;
    yield value;
    value++;
    print("right:" + value);
  }
  return value;
}
function report(result) { print(result.value + ":" + result.done); }
var left = values(true);
var right = values(false);
report(left.next());
report(right.next());
report(left.next());
report(right.next());
"#,
        &[
            "7:false", "8:false", "left:7", "99:true", "right:9", "99:true",
        ],
    );
}

#[test]
fn generator_branches_resume_nested_yields_and_declarations() {
    assert_generator_trace(
        r#"
function* nestedBlock(flag) { if (flag) { let value = 1; { yield value; } } }
function* multiple(flag) { if (flag) { let value = 1; yield value; yield 2; } }
function* captured(flag) { if (flag) { let value = 1; const read = () => value; yield read(); } }
function* klass(flag) { if (flag) { class Value {} yield Value; } }
function report(label, result) { print(label + ':' + result.value + ':' + result.done); }

let iterator = nestedBlock(true);
report('block', iterator.next());
report('block', iterator.next());
iterator = multiple(true);
report('multiple', iterator.next());
report('multiple', iterator.next());
report('multiple', iterator.next());
iterator = captured(true);
report('captured', iterator.next());
report('captured', iterator.next());
iterator = klass(true);
let result = iterator.next();
print('class:' + result.value.name + ':' + result.done);
report('class', iterator.next());
report('false', klass(false).next());
"#,
        &[
            "block:1:false",
            "block:undefined:true",
            "multiple:1:false",
            "multiple:2:false",
            "multiple:undefined:true",
            "captured:1:false",
            "captured:undefined:true",
            "class:Value:false",
            "class:undefined:true",
            "false:undefined:true",
        ],
    );
}

#[test]
fn conditional_while_generator_consumes_return_and_throw_before_continuing_iteration() {
    assert_generator_trace(
        r#"
function* values() {
  var index = 0;
  while (index < 4) {
    let value = index++;
    if (value % 2 === 0) { print("skip:" + value); }
    else { yield value; print("resumed:" + value); }
    print("end:" + value);
  }
}
function report(result) { print(result.value + ":" + result.done); }
var returned = values();
report(returned.next());
report(returned.return(91));
report(returned.next());
var thrown = values();
report(thrown.next());
var marker = {};
try { thrown.throw(marker); } catch (error) { print("same:" + (error === marker)); }
report(thrown.next());
"#,
        &[
            "skip:0",
            "end:0",
            "1:false",
            "91:true",
            "undefined:true",
            "skip:0",
            "end:0",
            "1:false",
            "same:true",
            "undefined:true",
        ],
    );
}

#[test]
fn array_from_async_consumes_and_maps_every_synchronous_generator_iteration() {
    assert_generator_trace(
        r#"
function* values() {
  for (let index = 0; index < 4; index++) { yield index * 2; }
}
Array.fromAsync({ [Symbol.iterator]: values }, function (value, index) {
  return value * index;
}).then(function (result) {
  print(result.join(","));
  return Array.fromAsync({ [Symbol.asyncIterator]: values }, function (value, index) {
    return Promise.resolve(value + index);
  });
}).then(function (result) { print(result.join(",")); });
void 0;
"#,
        &["0,2,8,18", "0,3,6,9"],
    );
}

#[test]
fn array_from_async_array_like_path_preserves_try_block_lexicals_across_await() {
    assert_generator_trace(
        r#"
(async function () {
  const iteratorPrototype = Object.getPrototypeOf([].values());
  const originalNext = iteratorPrototype.next;
  try {
    iteratorPrototype.next = function () { throw "array iterator was used"; };
    const expected = [0, 1, 2];
    const input = { length: 3, 0: 0, 1: 1, 2: 2 };
    const output = await Array.fromAsync(input);
    print(output.join(",") + ":" + (output.join(",") === expected.join(",")));
  } finally {
    iteratorPrototype.next = originalNext;
  }
  print("restored:" + (iteratorPrototype.next === originalNext));
})();
void 0;
"#,
        &["0,1,2:true", "restored:true"],
    );
}

#[test]
fn nested_for_of_generator_resumes_without_restarting_either_iterable() {
    assert_generator_trace(
        r#"
let outerReads = 0;
let innerReads = 0;
function* product(xs, ys) {
  for (let x of xs) {
    for (let y of ys) {
      yield x + ':' + y;
    }
  }
}
const xs = { [Symbol.iterator]() { outerReads++; return [1, 2][Symbol.iterator](); } };
const ys = { [Symbol.iterator]() { innerReads++; return ['a', 'b'][Symbol.iterator](); } };
const p = product(xs, ys);
for (let i = 0; i < 5; i++) {
  const result = p.next();
  print(result.value + ':' + result.done);
}
print('reads:' + outerReads + ':' + innerReads);
"#,
        &[
            "1:a:false",
            "1:b:false",
            "2:a:false",
            "2:b:false",
            "undefined:true",
            "reads:1:2",
        ],
    );
}

#[test]
fn nested_if_for_and_for_of_generator_keeps_branch_and_loop_progress() {
    assert_generator_trace(
        r#"
function* permutations(items) {
  if (items.length === 0) {
    yield [];
  } else {
    for (let i = 0; i < items.length; i++) {
      let tail = items.slice();
      let head = tail.splice(i, 1);
      for (let rest of permutations(tail)) {
        yield head.concat(rest);
      }
    }
  }
}
print(Array.from(permutations([1, 2, 3]), x => x.join('')).join(','));
"#,
        &["123,132,213,231,312,321"],
    );
}

#[test]
fn nested_generator_for_of_closes_each_active_iterator_on_return_and_throw() {
    assert_generator_trace(
        r#"
const log = [];
function iter(label) {
  let step = 0;
  return {
    [Symbol.iterator]() { return this; },
    next() { return step++ < 2 ? { value: label + step, done: false } : { done: true }; },
    return() { log.push(label); return { done: true }; }
  };
}
function* values() {
  for (let x of iter('outer')) {
    for (let y of iter('inner')) yield x + y;
  }
}
const returned = values();
print(returned.next().value);
print(returned.return(9).value + ':' + returned.next().done);
print(log.join(','));
log.length = 0;
const thrown = values();
print(thrown.next().value);
const marker = {};
try { thrown.throw(marker); } catch (error) { print('same:' + (error === marker)); }
print(log.join(','));
"#,
        &[
            "outer1inner1",
            "9:true",
            "inner,outer",
            "outer1inner1",
            "same:true",
            "inner,outer",
        ],
    );
}

#[test]
fn generator_for_of_captured_lexicals_keep_distinct_iteration_cells() {
    assert_generator_trace(
        r#"
const reads = [];
function* values() {
  for (let x of [3, 4]) {
    reads.push(() => x);
    yield x;
  }
}

const g = values();
print(g.next().value);
print(g.next().value);
print(g.next().done);
print(reads.map(read => read()).join(','));
"#,
        &["3", "4", "true", "3,4"],
    );
}

#[test]
fn generator_for_of_iterator_errors_follow_close_precedence() {
    assert_generator_trace(
        r#"
function* values(iterable) { for (const value of iterable) yield value; }
function iterator(next, close) {
  return { [Symbol.iterator]() { return this; }, next, return: close };
}

const nextError = {};
let nextCloses = 0;
const badNext = values(iterator(
  () => { throw nextError; },
  () => { nextCloses++; return { done: true }; }
));
try { badNext.next(); } catch (error) { print('next:' + (error === nextError)); }
print('next-close:' + nextCloses);

const valueError = {};
let valueCloses = 0;
const badValue = values(iterator(
  () => ({ done: false, get value() { throw valueError; } }),
  () => { valueCloses++; return { done: true }; }
));
try { badValue.next(); } catch (error) { print('value:' + (error === valueError)); }
print('value-close:' + valueCloses);

const thrown = {};
const closeError = {};
let throwCloses = 0;
const badThrow = values(iterator(
  () => ({ value: 1, done: false }),
  () => { throwCloses++; throw closeError; }
));
badThrow.next();
try { badThrow.throw(thrown); } catch (error) { print('throw:' + (error === thrown)); }
print('throw-close:' + throwCloses);

let returnCloses = 0;
const badReturn = values(iterator(
  () => ({ value: 1, done: false }),
  () => { returnCloses++; throw closeError; }
));
badReturn.next();
try { badReturn.return(9); } catch (error) { print('return:' + (error === closeError)); }
print('return-close:' + returnCloses);

let primitiveCloses = 0;
const primitiveReturn = values(iterator(
  () => ({ value: 1, done: false }),
  () => { primitiveCloses++; return 7; }
));
primitiveReturn.next();
try { primitiveReturn.return(9); } catch (error) { print('primitive:' + (error instanceof TypeError)); }
print('primitive-close:' + primitiveCloses);
"#,
        &[
            "next:true",
            "next-close:0",
            "value:true",
            "value-close:0",
            "throw:true",
            "throw-close:1",
            "return:true",
            "return-close:1",
            "primitive:true",
            "primitive-close:1",
        ],
    );
}

#[test]
fn nested_generator_mixes_inert_and_direct_yield_children() {
    assert_generator_trace(
        r#"
function* values(xs) {
  for (let x of xs) {
    if (x === 1) print('inert-if');
    for (let j = 0; j < 1; j++) print('inert-for:' + j);
    for (let v of [x]) print('inert-of:' + v);
    if (x > 0) yield 'if:' + x;
    for (let k = 0; k < 1; k++) yield 'for:' + x + ':' + k;
    if (true) { for (let z of [x]) yield 'constant:' + z; }
    yield 'tail:' + x;
  }
}
const g = values([1, 2]);
for (let n = 0; n < 9; n++) {
  const result = g.next();
  print(result.value + ':' + result.done);
}
"#,
        &[
            "inert-if",
            "inert-for:0",
            "inert-of:1",
            "if:1:false",
            "for:1:0:false",
            "constant:1:false",
            "tail:1:false",
            "inert-for:0",
            "inert-of:2",
            "if:2:false",
            "for:2:0:false",
            "constant:2:false",
            "tail:2:false",
            "undefined:true",
        ],
    );
}
