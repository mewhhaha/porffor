const trace = [];
let skippedCalls = 0;
let nullReads = 0;
function skipped() {
  skippedCalls++;
  Promise.resolve().then(() => trace.push('skipped-job'));
  throw 'nullish key evaluated';
}
function nullBase(value) { nullReads++; return value; }
async function run() {
  const a = nullBase(null)?.[await skipped()].tail;
  const b = nullBase(undefined)?.[await skipped()][await skipped()];
  if (a !== undefined || b !== undefined || nullReads !== 2 || skippedCalls !== 0) throw 'whole suffix skip';
  trace.push('skips');
  const symbol = Symbol('key');
  const target = {};
  let proxy;
  Object.defineProperty(target, symbol, {get() {
    if (this !== proxy) throw 'accessor receiver';
    trace.push('access');
    return 17;
  }});
  proxy = new Proxy(target, {get(object, key, receiver) {
    if (object !== target || key !== symbol || receiver !== proxy) throw 'proxy key or receiver';
    trace.push('get');
    return Reflect.get(object, key, receiver);
  }});
  const replacement = {[symbol]: 'replacement'};
  let current = proxy;
  let baseCalls = 0;
  function base() { baseCalls++; trace.push('base'); return current; }
  const keyObject = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'property key hint';
    trace.push('key');
    return symbol;
  }};
  const thenable = {get then() {
    trace.push('then');
    current = replacement;
    return resolve => { trace.push('resolve'); resolve(keyObject); };
  }};
  function keySource() { trace.push('key-source'); return thenable; }
  const selected = base()?.[await keySource()];
  if (selected !== 17 || baseCalls !== 1 || current !== replacement) throw 'retained base';
  if (trace.join(',') !== 'skips,base,key-source,then,caller,resolve,key,get,access') throw 'selected key order';
  let synchronousGets = 0;
  const synchronousTarget = {get undefined() { synchronousGets++; return 41; }};
  const foldedKey = synchronousTarget?.[undefined?.[await skipped()]];
  const foldedLogical = true && undefined?.[await skipped()];
  const foldedConditional = true ? undefined?.[await skipped()] : 0;
  if (foldedKey !== 41 || foldedLogical !== undefined || foldedConditional !== undefined || synchronousGets !== 1 || skippedCalls !== 0) throw 'zero-suspension branch joins';
  let nonNullishKeys = 0;
  function missing() { nonNullishKeys++; return 'missing'; }
  if ((0)?.[await missing()] !== undefined) throw 'number value';
  if (false?.[await missing()] !== undefined) throw 'boolean value';
  if (''?.[await missing()] !== undefined) throw 'string value';
  const htmlDda = __lilaCreateHTMLDDA();
  Object.defineProperty(htmlDda, 'value', {value: 29});
  function valueKey() { nonNullishKeys++; return 'value'; }
  if (htmlDda?.[await valueKey()] !== 29 || nonNullishKeys !== 4) throw 'nullish selection is not truthiness';
  if (skippedCalls !== 0) throw 'skipped key effects';
}
run().then(() => print('optional-values:ok'), error => print('unexpected:' + error));
trace.push('caller');
