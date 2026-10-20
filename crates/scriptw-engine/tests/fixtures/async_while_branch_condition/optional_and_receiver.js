const trace = [];
async function run() {
  let baseReads = 0;
  let keyCalls = 0;
  let valueReads = 0;
  let bodies = 0;
  const replacement = {get ready() { throw 'replacement base read'; }};
  const original = {get ready() {
    valueReads++;
    trace.push('original-read');
    source = null;
    return true;
  }};
  let source = original;
  const holder = {get base() {
    baseReads++;
    trace.push('base' + baseReads);
    return source;
  }};
  function key() {
    keyCalls++;
    trace.push('key');
    return {get then() {
      trace.push('key-then');
      source = replacement;
      return function(resolve) {
        Promise.resolve().then(() => { trace.push('key-resolve'); resolve('ready'); });
      };
    }};
  }
  while (holder.base?.[await key()] ?? false) {
    bodies++;
    trace.push('optional-body');
    continue;
  }
  trace.push('optional-done');
  if (baseReads !== 2 || keyCalls !== 1 || valueReads !== 1 || bodies !== 1) {
    throw 'optional base retention or back edge';
  }

  let getterCalls = 0;
  let calls = 0;
  let argumentsRead = 0;
  let n = 0;
  const otherApi = {get check() { throw 'replacement callee read'; }};
  const originalApi = {get check() {
    getterCalls++;
    trace.push('callee');
    return function(value) {
      calls++;
      if (this !== originalApi) throw 'condition call receiver';
      trace.push('call-' + value);
      return value;
    };
  }};
  let api = originalApi;
  function argument() {
    argumentsRead++;
    trace.push('argument');
    return {get then() {
      trace.push('argument-then');
      api = otherApi;
      return function(resolve) {
        Promise.resolve().then(() => { trace.push('argument-resolve'); resolve(true); });
      };
    }};
  }
  while (api.check((n++ < 1) && await argument())) {
    trace.push('call-body');
    api = originalApi;
  }
  trace.push('call-done');
  if (getterCalls !== 2 || calls !== 2 || argumentsRead !== 1 || n !== 2) {
    throw 'callee acquisition repeated on reaction';
  }

  let erased = 0;
  let erasedBodies = 0;
  let erasedKeys = 0;
  function erasedKey() { erasedKeys++; throw 'statically skipped key'; }
  while ((++erased < 3) && (null?.[await erasedKey()] === undefined)) {
    erasedBodies++;
    trace.push('eager' + erased);
  }
  trace.push('erased' + erased);
  if (erased !== 3 || erasedBodies !== 2 || erasedKeys !== 0) {
    throw 'eager prefix escaped the loop';
  }
  await 0;
  trace.push('post');
  if (trace.join(',') !== 'base1,key,key-then,caller,key-resolve,original-read,optional-body,base2,optional-done,callee,argument,argument-then,argument-resolve,call-true,call-body,callee,call-false,call-done,eager1,eager2,erased3,post') {
    throw 'optional/invocation ordering';
  }
}
run().then(() => print('while-references:ok'), error => print('unexpected:' + error));
trace.push('caller');
