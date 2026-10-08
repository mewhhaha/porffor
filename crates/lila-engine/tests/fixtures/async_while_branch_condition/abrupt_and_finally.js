const foreign = __lilaCreateRealm().global;
const foreignType = foreign.TypeError.prototype;
const marker = new foreign.TypeError('condition marker');
foreign.TypeError = null;
async function run() {
  const trace = [];
  function check(error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignType) {
      throw 'condition abrupt identity or Realm';
    }
  }
  function skipped() { throw 'skipped condition operand'; }
  function throwMarker() { trace.push('throw'); throw marker; }
  try {
    while (true ? (null ?? await Promise.reject(marker)) : await skipped()) {
      throw 'rejected condition entered';
    }
  } catch (error) { check(error); trace.push('reject'); }
  finally { await 0; trace.push('reject-finally'); }
  try {
    while (undefined ?? await throwMarker()) { throw 'throwing condition entered'; }
  } catch (error) { check(error); trace.push('operand'); }
  finally { await 0; trace.push('operand-finally'); }
  const throwingThenable = {get then() { trace.push('then-get'); throw marker; }};
  try {
    while (false || await throwingThenable) { throw 'throwing then entered'; }
  } catch (error) { check(error); trace.push('then'); }
  finally { await 0; trace.push('then-finally'); }
  const throwingLeft = {get value() { trace.push('left-get'); throw marker; }};
  try {
    while (throwingLeft.value && await skipped()) { throw 'throwing left entered'; }
  } catch (error) { check(error); trace.push('left'); }
  finally { await 0; trace.push('left-finally'); }
  try {
    while (false || await true) {
      try { break; }
      finally { trace.push('break-finally'); throw marker; }
    }
  } catch (error) { check(error); trace.push('replaced-break'); }
  finally { await 0; trace.push('break-outer-finally'); }
  async function replaceContinue() {
    while (true && await true) {
      try { continue; }
      finally { return 'returned'; }
    }
    throw 'continue finalizer fell through';
  }
  if (await replaceContinue() !== 'returned') throw 'continue replacement';
  trace.push('returned');
  let n = 0;
  while ((++n < 2) && (true ? await true : await skipped())) { trace.push('recovered'); }
  if (n !== 2) throw 'post-abrupt back edge';
  await 0;
  trace.push('post');
  if (trace.join(',') !== 'reject,reject-finally,throw,operand,operand-finally,then-get,then,then-finally,left-get,left,left-finally,break-finally,replaced-break,break-outer-finally,returned,recovered,post') {
    throw 'condition abrupt completion order';
  }
}
run().then(() => print('while-abrupt:ok'), error => print('unexpected:' + error));
