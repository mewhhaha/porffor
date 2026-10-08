const foreign = __lilaCreateRealm().global;
const foreignType = foreign.TypeError.prototype;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
async function run() {
  const trace = [];
  let assigned = 'initial';
  function skipped() { trace.push('skipped'); throw 'skipped operand ran'; }
  function throwMarker() { trace.push('throw'); throw marker; }
  function check(error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignType) throw 'marker identity or Realm';
    if (assigned !== 'initial') throw 'abrupt result written';
  }
  try {
    assigned = true && (false ? await skipped() : (null ?? await Promise.reject(marker)));
    trace.push('wrong-rejection-join');
  } catch (error) {
    check(error);
    trace.push('rejected');
  } finally { await 0; trace.push('rejection-finally'); }
  try {
    assigned = undefined ?? await throwMarker();
    trace.push('wrong-throw-join');
  } catch (error) {
    check(error);
    trace.push('thrown');
  } finally { await 0; trace.push('throw-finally'); }
  const throwingThenable = {get then() { trace.push('then-get'); throw marker; }};
  try {
    assigned = false || await throwingThenable;
    trace.push('wrong-then-join');
  } catch (error) {
    check(error);
    trace.push('then-rejected');
  } finally { await 0; trace.push('then-finally'); }
  const throwingLeft = {get value() { trace.push('left-get'); throw marker; }};
  try {
    assigned = throwingLeft.value ?? await skipped();
    trace.push('wrong-left-join');
  } catch (error) {
    check(error);
    trace.push('left-thrown');
  } finally { await 0; trace.push('left-finally'); }
  const recovered = false || (null ?? (true ? await 13 : await skipped()));
  if (recovered !== 13 || assigned !== 'initial') throw 'post-abrupt join';
  if (trace.join(',') !== 'rejected,rejection-finally,throw,thrown,throw-finally,then-get,then-rejected,then-finally,left-get,left-thrown,left-finally') throw 'abrupt order';
}
run().then(() => print('logical-abrupt:ok'), error => print('unexpected:' + error));
