function same(a,b,label) { if (!Object.is(a,b)) throw new Error(label); }
function check(v,label) { if (!v) throw new Error(label); }
var rtf = new Intl.RelativeTimeFormat('en-US');
var log = [];
var value = {[Symbol.toPrimitive](hint) { log.push('number:'+hint); return -1; }};
var unit = {[Symbol.toPrimitive](hint) { log.push('unit:'+hint); return 'days'; }};
same(rtf.format(value,unit), '1 day ago', 'coerced result');
same(log.join(','), 'number:number,unit:string', 'coercion order and hints');
for (var nonfinite of [NaN, Infinity, -Infinity]) {
  log = [];
  try { rtf.format(nonfinite,{toString() { log.push('unit'); return 'day'; }}); throw new Error('missing finite error'); } catch(e) { check(e instanceof RangeError, 'finite RangeError'); }
  same(log.join(','), 'unit', 'unit coerced before finite validation');
}
var sentinel = {marker:2};
log = [];
try { rtf.format({valueOf() { throw sentinel; }}, {toString() { log.push('unit'); return 'day'; }}); throw new Error('missing value abrupt'); } catch(e) { same(e,sentinel,'value abrupt'); }
same(log.length,0,'value abrupt stops unit');
try { rtf.format(NaN, {toString() { throw sentinel; }}); throw new Error('missing unit abrupt'); } catch(e) { same(e,sentinel,'unit abrupt precedes finite error'); }
for (var bad of [1n, Symbol('value')]) {
  log = [];
  try { rtf.format(bad,{toString() { log.push('unit'); return 'day'; }}); throw new Error('missing number TypeError'); } catch(e) { check(e instanceof TypeError,'ToNumber TypeError'); }
  same(log.length,0,'ToNumber TypeError stops unit');
}
for (var badUnit of ['Day','day ','dayss','millisecond',undefined]) {
  try { rtf.format(1,badUnit); throw new Error('missing unit RangeError'); } catch(e) { check(e instanceof RangeError,'unit RangeError'); }
}
try { rtf.format(1,Symbol('unit')); throw new Error('missing unit TypeError'); } catch(e) { check(e instanceof TypeError,'unit ToString TypeError'); }
print('ok scalar_coercion_order_and_finite_validation');
262;
