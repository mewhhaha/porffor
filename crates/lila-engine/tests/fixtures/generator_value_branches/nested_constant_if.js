function check(value, message) { if (!value) throw new Error(message); }
let trace = [];
function* conditional(flag) {
  yield 'start';
  try {
    yield 'body';
    if (flag) {
      try {
        if (false) { throw 'unselected false arm'; }
        else { trace.push('false-else'); }
        if (true) { trace.push('true-then'); }
        else { throw 'unselected true arm'; }
      } finally { trace.push('inner-finally'); }
    } else { trace.push('outer-else'); }
    return 'normal';
  } finally {
    trace.push('outer-finally');
    const received = yield 'cleanup';
    trace.push('received:' + received);
  }
}
for (const flag of [false, true]) {
  for (const mode of ['normal', 'return', 'throw']) {
    trace = [];
    const iterator = conditional(flag);
    const start = iterator.next();
    check(!start.done && start.value === 'start', 'initial suspension');
    const body = iterator.next();
    check(!body.done && body.value === 'body', 'body suspension');
    const marker = {mode, flag};
    const cleanup = mode === 'normal' ? iterator.next() :
      mode === 'return' ? iterator.return(marker) : iterator.throw(marker);
    check(!cleanup.done && cleanup.value === 'cleanup', 'finalizer suspension');
    const selected = mode === 'normal' ?
      (flag ? 'false-else,true-then,inner-finally,' : 'outer-else,') : '';
    check(trace.join(',') === selected + 'outer-finally', 'selected body and finalizer');
    if (mode === 'throw') {
      let caught;
      try { iterator.next(19); } catch (error) { caught = error; }
      check(caught === marker, 'injected whole throw');
    } else {
      const done = iterator.next(19);
      check(done.done && done.value === (mode === 'normal' ? 'normal' : marker),
        'normal and injected return completion');
    }
    check(trace.join(',') === selected + 'outer-finally,received:19', 'resume exactly once');
    check(iterator.next().done, 'closed after completion');
  }
}
function* eager() { if (false) throw 'eager false'; if (true) yield 42; return 19; }
const ordinary = eager();
check(ordinary.next().value === 42 && ordinary.next().value === 19 && ordinary.next().done,
  'ordinary constant branch outside a complete region');
print('generator-nested-constant-if:ok');
