const trace = [];
function record(name, value) { trace.push(name); return value; }
let skippedCalls = 0;
function skipped() { skippedCalls++; throw 'skipped suffix evaluated'; }
async function run() {
  const leaf = {value: 31};
  const child = {get ordinary() { trace.push('get-ordinary'); return leaf; }};
  const root = {
    get first() { trace.push('get-first'); return child; },
    get empty() { trace.push('get-empty'); return undefined; },
    get nullish() { trace.push('get-nullish'); return null; }
  };
  const value = root?.[await record('first-key', 'first')].ordinary[await record('second-key', 'value')];
  if (value !== 31 || trace.join(',') !== 'first-key,caller,get-first,get-ordinary,second-key') throw 'each Get before later key';
  trace.length = 0;
  const early = null?.[await skipped()].ordinary[await skipped()];
  const later = root?.[await record('nullish-key', 'nullish')]?.[await skipped()].ordinary;
  if (early !== undefined || later !== undefined || skippedCalls !== 0) throw 'shorted suffix value';
  if (trace.join(',') !== 'nullish-key,get-nullish') throw 'later optional guard';
  trace.length = 0;
  let coerced = 0;
  const keyObject = {[Symbol.toPrimitive]() { coerced++; throw 'key coerced before nullish error'; }};
  function ordinaryKey() { trace.push('ordinary-key'); return keyObject; }
  let caught = false;
  try { root?.[await record('empty-key', 'empty')][await ordinaryKey()]; }
  catch (error) {
    caught = Object.getPrototypeOf(error) === TypeError.prototype;
    trace.push('type-error');
  }
  if (!caught || coerced !== 0 || trace.join(',') !== 'empty-key,get-empty,ordinary-key,type-error') throw 'ordinary nullish key order';
  trace.length = 0;
  const holder = {base: 40, method(value) {
    if (this !== holder) throw 'outer property Reference';
    return this.base + value;
  }};
  const table = {holder};
  const called = (table?.[await record('holder-key', 'holder')]).method(2);
  if (called !== 42 || trace.join(',') !== 'holder-key') throw 'grouped value keeps outer Reference';
  function C(value) { this.value = value; }
  table.C = C;
  const constructed = new (table?.[await 'C'])(43);
  if (constructed.value !== 43 || Object.getPrototypeOf(constructed) !== C.prototype) throw 'constructor value';
  function take(value) { if (value !== leaf) throw 'argument value'; return value.value; }
  table.leaf = leaf;
  if (take(table?.[await 'leaf']) !== 31) throw 'call argument value';
  const combined = false || (null ?? (true ? root?.[await 'first']?.[await 'ordinary'] : await skipped()));
  if (combined !== leaf || skippedCalls !== 0) throw 'nested branch owners';
}
run().then(() => print('optional-suffixes:ok'), error => print('unexpected:' + error));
trace.push('caller');
