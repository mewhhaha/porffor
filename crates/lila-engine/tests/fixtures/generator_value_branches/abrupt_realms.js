const foreign = __lilaCreateRealm().global;
const foreignType = foreign.TypeError.prototype;
const marker = new foreign.TypeError('generator marker');
foreign.TypeError = null;
const trace = [];
function check(error) {
  if (error !== marker || Object.getPrototypeOf(error) !== foreignType) throw 'abrupt identity or Realm';
}
function result(actual, value, done) {
  if (actual.value !== value || actual.done !== done) throw 'iterator completion';
}
function* caught() {
  try {
    const value = false || (yield 'throw-token');
    trace.push('wrong-publication');
    return value;
  } catch (error) { check(error); trace.push('caught'); return 'caught'; }
  finally { trace.push('caught-finally'); }
}
const throwing = caught();
result(throwing.next(), 'throw-token', false);
result(throwing.throw(marker), 'caught', true);
function* returned() {
  try {
    var value = true && (yield 'return-token');
    trace.push('wrong-return-publication');
    return value;
  } finally { trace.push('return-finally'); }
}
const returning = returned();
result(returning.next(), 'return-token', false);
result(returning.return(41), 41, true);
function* replacement() {
  try { const value = null ?? (yield 'replacement-token'); trace.push('wrong-replacement'); }
  finally { trace.push('replacement-finally'); return 'replaced'; }
}
const replacing = replacement();
result(replacing.next(), 'replacement-token', false);
result(replacing.return(9), 'replaced', true);
function operand() { trace.push('operand'); throw marker; }
function* beforeYield() {
  try { const value = true ? (yield operand()) : 0; trace.push('wrong-operand-publication'); }
  catch (error) { check(error); trace.push('operand-caught'); return 17; }
  finally { trace.push('operand-finally'); }
}
result(beforeYield().next(), 17, true);
const base = {};
const key = {[Symbol.toPrimitive]() { trace.push('key'); throw marker; }};
function* afterYield() {
  try { var value = base?.[(yield 'key-token')]; trace.push('wrong-key-publication'); }
  catch (error) { check(error); trace.push('key-caught'); return 23; }
  finally { trace.push('key-finally'); }
}
const keyed = afterYield();
result(keyed.next(), 'key-token', false);
result(keyed.next(key), 23, true);
if (trace.join(',') !== 'caught,caught-finally,return-finally,replacement-finally,operand,operand-caught,operand-finally,key,key-caught,key-finally') throw 'completion precedence or result publication';
print('generator-abrupt:ok');
