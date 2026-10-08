const foreign = __lilaCreateRealm().global;
const foreignPrototype = foreign.TypeError.prototype;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
const intrinsicTypePrototype = TypeError.prototype;
const intrinsicReferencePrototype = ReferenceError.prototype;
TypeError = undefined;
ReferenceError = undefined;
const strictMode = (function() { return this === undefined; })();
async function run() {
  const trace = [];
  let rhsCalls = 0;
  let assigned = 'kept';
  function rhs(label, value) { rhsCalls++; trace.push(label); return value; }
  function checkMarker(error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'abrupt marker or assignment';
  }
  let value = 0;
  try { assigned = value ||= await rhs('reject-rhs', Promise.reject(marker)); }
  catch (error) { checkMarker(error); if (value !== 0) throw 'rejection Put'; trace.push('rejected'); }
  finally { await 0; trace.push('reject-finally'); }
  const keyObject = {[Symbol.toPrimitive]() { trace.push('key'); throw marker; }};
  const object = {value: 0};
  try { assigned = object[keyObject] ||= await rhs('bad-key-rhs', 1); }
  catch (error) { checkMarker(error); trace.push('key-thrown'); }
  finally { await 0; trace.push('key-finally'); }
  const getter = {get value() { trace.push('get'); throw marker; }};
  try { assigned = getter.value ??= await rhs('bad-get-rhs', 1); }
  catch (error) { checkMarker(error); trace.push('get-thrown'); }
  finally { await 0; trace.push('get-finally'); }
  const setter = {get value() { return 0; }, set value(value) { if (value !== 1) throw 'setter value'; trace.push('set'); throw marker; }};
  try { assigned = setter.value ||= await rhs('set-rhs', 1); }
  catch (error) { checkMarker(error); trace.push('set-thrown'); }
  finally { await 0; trace.push('set-finally'); }
  const constant = 0;
  try { constant ||= await rhs('const-rhs', 1); throw 'missing immutable error'; }
  catch (error) { if (Object.getPrototypeOf(error) !== intrinsicTypePrototype) throw 'immutable error Realm'; trace.push('const-thrown'); }
  finally { await 0; trace.push('const-finally'); }
  try {
    later ??= await rhs('bad-tdz-rhs', 1);
    let later = 0;
  } catch (error) { if (Object.getPrototypeOf(error) !== intrinsicReferencePrototype) throw 'TDZ error Realm'; trace.push('tdz-thrown'); }
  finally { await 0; trace.push('tdz-finally'); }
  const failedSet = new Proxy({value: 0}, {set() { trace.push('false-set'); return false; }});
  let falseSetCaught = false;
  let falseSetResult;
  try { falseSetResult = failedSet.value ||= await rhs('false-set-rhs', 2); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicTypePrototype) throw 'strict Set error Realm';
    falseSetCaught = true;
  } finally { await 0; trace.push('false-set-finally'); }
  if (falseSetCaught !== strictMode || failedSet.value !== 0 || (!strictMode && falseSetResult !== 2)) throw 'Set strictness or result';
  if (rhsCalls !== 4 || assigned !== 'kept') throw 'abrupt Get precedes RHS or result publication';
  if (trace.join(',') !== 'reject-rhs,rejected,reject-finally,key,key-thrown,key-finally,get,get-thrown,get-finally,set-rhs,set,set-thrown,set-finally,const-rhs,const-thrown,const-finally,tdz-thrown,tdz-finally,false-set-rhs,false-set,false-set-finally') throw 'abrupt operation order';
}
run().then(() => print('logical-assignment-abrupt:ok'), error => print('unexpected:' + error));
