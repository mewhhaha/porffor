function check(value, message) { if (!value) throw new Error(message); }
var trace = [];
function* conditional(flag) {
  try {
    if (flag) {
      try { trace.push('try'); throw 'caught'; }
      catch (error) { trace.push(error); }
      finally { trace.push('inner-finally'); }
    } else {
      trace.push('else');
    }
    // A later region must start at the branch's actual join state.
    try { trace.push('after'); } finally { trace.push('after-finally'); }
    return flag ? 'yes' : 'no';
  } finally {
    trace.push('finally');
    yield 'cleanup';
    trace.push('finally-resumed');
  }
}
for (var flag of [true, false]) {
  trace = [];
  var iterator = conditional(flag);
  var cleanup = iterator.next();
  check(!cleanup.done && cleanup.value === 'cleanup', 'selected finalizer suspends');
  var result = iterator.next();
  check(result.done && result.value === (flag ? 'yes' : 'no'), 'held branch return');
  check(trace.join(',') === (flag ? 'try,caught,inner-finally,' : 'else,') +
        'after,after-finally,finally,finally-resumed', 'only the selected branch runs once');
  check(iterator.next().done, 'closed after completion');
}
for (var mode of ['return', 'throw']) {
  trace = [];
  var iterator = conditional(true);
  check(iterator.next().value === 'cleanup', 'injected completion starts at finalizer');
  var marker = {};
  if (mode === 'return') {
    var result = iterator.return(marker);
    check(result.done && result.value === marker, 'injected return identity');
  } else {
    var caught;
    try { iterator.throw(marker); } catch (error) { caught = error; }
    check(caught === marker, 'injected throw identity');
  }
  check(trace.join(',') === 'try,caught,inner-finally,after,after-finally,finally',
        'injection skips the rest of the active finalizer');
  check(iterator.next().done, 'closed after injection');
}
print('generator-conditional-try:ok');
