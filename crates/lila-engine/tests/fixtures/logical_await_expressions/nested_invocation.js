const trace = [];
function record(name, value) { trace.push(name); return value; }
function skipped() { throw 'skipped operand ran'; }
async function run() {
  const object = {base: 10};
  const original = function(first, second, third) {
    if (this !== object) throw 'call receiver changed';
    trace.push('call');
    return first + second + third + this.base;
  };
  Object.defineProperty(object, 'method', {
    get() { trace.push('get'); return original; }, configurable: true
  });
  const thenable = {get then() {
    trace.push('then');
    Object.defineProperty(object, 'method', {value() { throw 'replacement called'; }, configurable: true});
    return resolve => { trace.push('resolve'); resolve(2); };
  }};
  const value = object.method(record('first', 1),
    record('left', false) || (record('gate', true) &&
      (record('condition', false) ? await skipped() : await thenable)), record('last', 3));
  if (value !== 16 || trace.join(',') !== 'get,first,left,gate,condition,then,caller,resolve,last,call') throw 'call order';
  trace.length = 0;
  function C(value) { trace.push('construct'); this.value = value; }
  let Current = C;
  const constructorThenable = {get then() {
    trace.push('new-then');
    Current = function() { throw 'replacement constructed'; };
    return resolve => { trace.push('new-resolve'); resolve(4); };
  }};
  const made = new Current(record('new-left', null) ??
    (record('new-gate', true) ? await constructorThenable : await skipped()));
  if (made.value !== 4 || Object.getPrototypeOf(made) !== C.prototype) throw 'constructor reference';
  if (trace.join(',') !== 'new-left,new-gate,new-then,new-resolve,construct') throw 'constructor order';
  trace.length = 0;
  const tag = {base: 20};
  const originalTag = function(strings, first, second) {
    if (this !== tag || strings[0] !== '') throw 'tag reference';
    trace.push('tag');
    return this.base + first + second;
  };
  Object.defineProperty(tag, 'method', {
    get() { trace.push('tag-get'); return originalTag; }, configurable: true
  });
  const tagThenable = {get then() {
    trace.push('tag-then');
    Object.defineProperty(tag, 'method', {value() { throw 'replacement tagged'; }, configurable: true});
    return resolve => { trace.push('tag-resolve'); resolve(5); };
  }};
  const tagged = tag.method`${record('tag-left', 0) ||
    (record('tag-gate', null) ?? await tagThenable)}${record('tag-last', 6)}`;
  if (tagged !== 31 || trace.join(',') !== 'tag-get,tag-left,tag-gate,tag-then,tag-resolve,tag-last,tag') throw 'tag order';
  trace.length = 0;
  const destination = {value: 0};
  const replacement = {changed: 'untouched'};
  let baseSource = destination;
  let keySource = 'value';
  let baseGets = 0;
  let keyGets = 0;
  const references = {
    get base() { baseGets++; trace.push('property-base'); return baseSource; },
    get key() { keyGets++; trace.push('property-key'); return keySource; }
  };
  const propertyThenable = {get then() {
    trace.push('property-then');
    baseSource = replacement;
    keySource = 'changed';
    return resolve => { trace.push('property-resolve'); resolve(12); };
  }};
  const written = references.base[references.key] = record('property-left', true) && await propertyThenable;
  if (written !== 12 || destination.value !== 12 || replacement.changed !== 'untouched') throw 'retained property Reference';
  if (baseGets !== 1 || keyGets !== 1 || baseSource !== replacement || keySource !== 'changed') throw 'property operands once';
  if (trace.join(',') !== 'property-base,property-key,property-left,property-then,property-resolve') throw 'property assignment order';
  trace.length = 0;
  baseSource = destination;
  keySource = 'value';
  const kept = references.base[references.key] = record('property-skipped', destination.value) || await skipped();
  if (kept !== 12 || destination.value !== 12 || replacement.changed !== 'untouched') throw 'skipped property assignment value';
  if (baseGets !== 2 || keyGets !== 2 || trace.join(',') !== 'property-base,property-key,property-skipped') throw 'skipped property operands once';
  let leftThenGets = 0;
  const retained = (await {get then() {
    leftThenGets++;
    return resolve => resolve(null);
  }}) ?? (false || (true ? await {value: 9} : await skipped()));
  await 0;
  if (retained.value !== 9 || leftThenGets !== 1) throw 'awaited left and retained result';
  const nested = (false && await skipped()) ||
    ((undefined ?? await 0) ? await skipped() : (true && await 11));
  if (nested !== 11) throw 'nested join selection';
  let tdzObserved = 0;
  const initialized = null ?? await {get then() {
    try { (() => initialized)(); } catch (error) {
      if (Object.getPrototypeOf(error) !== ReferenceError.prototype) throw 'initialization Realm';
      tdzObserved++;
    }
    return resolve => resolve(7);
  }};
  if (initialized !== 7 || tdzObserved !== 1) throw 'logical lexical initialization';
}
run().then(() => print('logical-invocation:ok'), error => print('unexpected:' + error));
trace.push('caller');
