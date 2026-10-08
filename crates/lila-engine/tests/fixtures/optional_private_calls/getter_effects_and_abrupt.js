const trace = [];
let flag = false;
let captured = {value: 1};
let acquired;
let selected;
let getterReads = 0;
let argumentReads = 0;
let original;
const getterMarker = Symbol('getter throw');
let mode = 'undefined';
class C {
  get #callee() {
    getterReads++;
    trace.push('get');
    flag = true;
    captured.value = 2;
    if (mode === 'throw') throw getterMarker;
    return acquired;
  }
  static invoke(value) { return value.#callee?.(argument()); }
  static read(value) { return value?.#callee; }
}
function argument() {
  argumentReads++;
  trace.push('argument');
  selected = null;
  acquired = function() { throw 'replacement callee'; };
  return 41;
}
original = new C();
selected = original;
if (C.invoke(selected) !== undefined || getterReads !== 1 || argumentReads !== 0 ||
    !flag || captured.value !== 2 || trace.join(',') !== 'get') {
  throw 'nullish getter result skipped getter effects';
}
flag = false;
captured = {value: 1};
trace.length = 0;
if (C.read(original) !== undefined || !flag || captured.value !== 2 ||
    trace.join(',') !== 'get') {
  throw 'optional private Get kept stale caller facts';
}
const body = function(value) {
  trace.push('call');
  if (this !== original) throw 'getter-returned callable receiver';
  return value + 1;
};
acquired = new Proxy(body, {
  apply(target, receiver, args) {
    trace.push('apply');
    if (receiver !== original) throw 'getter-returned Proxy receiver';
    return Reflect.apply(target, receiver, args);
  }
});
selected = original;
trace.length = 0;
if (C.invoke(selected) !== 42 ||
    trace.join(',') !== 'get,argument,apply,call' || getterReads !== 3 || argumentReads !== 1) {
  throw 'private getter/callee acquired again after argument effects';
}
mode = 'throw';
trace.length = 0;
let observed;
try { C.invoke(original); }
catch (error) { observed = error; }
if (observed !== getterMarker || argumentReads !== 1 || trace.join(',') !== 'get') {
  throw 'private getter Throw identity or abrupt cutoff';
}
const beforeNull = getterReads;
if (C.read(null) !== undefined || getterReads !== beforeNull) {
  throw 'optional private base did not skip getter';
}
let brandError;
try { C.invoke(null); }
catch (error) { brandError = error; }
if (!(brandError instanceof TypeError) || getterReads !== beforeNull || argumentReads !== 1) {
  throw 'private Reference brand check replaced by optional base skipping';
}
print('optional-private-getters:ok');
262;
