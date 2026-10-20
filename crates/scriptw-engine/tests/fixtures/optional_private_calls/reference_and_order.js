const trace = [];
let argumentsRead = 0;
let selected;
let original;
function argument() {
  argumentsRead++;
  trace.push('argument');
  selected = replacement;
  return 41;
}
function base() { trace.push('base'); return selected; }
class C {
  #method(value) {
    if (this !== original) throw 'lost private Reference receiver';
    trace.push('call');
    return value + 1;
  }
  #missing;
  #null = null;
  #noncallable = 17;
  static invoke() { return base().#method?.(argument()); }
  static grouped(value) { return ((value.#method))?.(argument()); }
  static missing(value) { return value.#missing?.(argument()); }
  static nullCallee(value) { return value.#null?.(argument()); }
  static noncallable(value) { return value.#noncallable?.(argument()); }
  static argumentThrow(value, marker) {
    return value.#noncallable?.((function() { throw marker; })());
  }
}
const replacement = new C();
original = new C();
selected = original;
if (C.invoke() !== 42 || trace.join(',') !== 'base,argument,call') {
  throw 'private base/Get/call ordering';
}
trace.length = 0;
if (C.grouped(original) !== 42 || trace.join(',') !== 'argument,call') {
  throw 'parenthesized private Reference receiver';
}
const beforeSkipped = argumentsRead;
if (C.missing(original) !== undefined || C.nullCallee(original) !== undefined ||
    argumentsRead !== beforeSkipped) {
  throw 'nullish private callee did not skip arguments';
}
for (const wrongBrand of [{}, null, new Proxy(original, {})]) {
  let caught = false;
  try { C.missing(wrongBrand); }
  catch (error) { caught = error instanceof TypeError; }
  if (!caught || argumentsRead !== beforeSkipped) {
    throw 'private brand failure must precede optional callee and arguments';
  }
}
let caught = false;
try { C.noncallable(original); }
catch (error) { caught = error instanceof TypeError; }
if (!caught || argumentsRead !== beforeSkipped + 1) {
  throw 'non-nullish private call must evaluate arguments before callability';
}
const marker = Symbol('argument throw');
let observed;
try { C.argumentThrow(original, marker); }
catch (error) { observed = error; }
if (observed !== marker) throw 'argument Throw replaced by callability error';
class StaticReceiver {
  static #method() { return this; }
  static invoke() { return this.#method?.(); }
}
if (StaticReceiver.invoke() !== StaticReceiver) throw 'static private receiver';
print('optional-private-reference:ok');
262;
