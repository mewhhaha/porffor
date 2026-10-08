const foreign = __lilaCreateRealm().global;
const foreignType = foreign.TypeError.prototype;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
async function run() {
  const trace = [];
  let assigned = 'initial';
  let skippedCalls = 0;
  function skipped() { skippedCalls++; throw 'skipped key'; }
  function check(error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignType || assigned !== 'initial') throw 'abrupt identity or result';
  }
  const target = {value: 1};
  try { assigned = target?.[await Promise.reject(marker)]; }
  catch (error) { check(error); trace.push('rejected'); }
  finally { await 0; trace.push('reject-finally'); }
  const throwingKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'key hint';
    trace.push('coerce');
    throw marker;
  }};
  try { assigned = target?.[await throwingKey]; }
  catch (error) { check(error); trace.push('coercion-thrown'); }
  finally { await 0; trace.push('coerce-finally'); }
  const throwingProperty = {get value() { trace.push('get'); throw marker; }};
  try { assigned = throwingProperty?.[await 'value']; }
  catch (error) { check(error); trace.push('getter-thrown'); }
  finally { await 0; trace.push('get-finally'); }
  const throwingBase = {get value() { trace.push('base'); throw marker; }};
  try { assigned = throwingBase.value?.[await skipped()]; }
  catch (error) { check(error); trace.push('base-thrown'); }
  finally { await 0; trace.push('base-finally'); }
  const thenable = {get then() { trace.push('then'); throw marker; }};
  try { assigned = target?.[await thenable]; }
  catch (error) { check(error); trace.push('then-thrown'); }
  finally { await 0; trace.push('then-finally'); }
  const kept = undefined?.[await skipped()].value;
  const recovered = target?.[await 'value'];
  if (kept !== undefined || recovered !== 1 || assigned !== 'initial' || skippedCalls !== 0) throw 'post-abrupt optional join';
  if (trace.join(',') !== 'rejected,reject-finally,coerce,coercion-thrown,coerce-finally,get,getter-thrown,get-finally,base,base-thrown,base-finally,then,then-thrown,then-finally') throw 'abrupt order';
}
run().then(() => print('optional-abrupt:ok'), error => print('unexpected:' + error));
