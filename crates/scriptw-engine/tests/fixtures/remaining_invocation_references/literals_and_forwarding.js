function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'reacquired canonical property or unintended coercion'; }
const define = Object.defineProperty;
const finiteDescriptor = Object.getOwnPropertyDescriptor(Number, 'isFinite');
const nanDescriptor = Object.getOwnPropertyDescriptor(Number, 'isNaN');
const safeDescriptor = Object.getOwnPropertyDescriptor(Number, 'isSafeInteger');
const unescapeDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'unescape');
const nativeUnescape = unescape;
const nativeCall = Function.prototype.call;
const nativeApply = Function.prototype.apply;
const nativeBind = Function.prototype.bind;
const arraySpecies = Object.getOwnPropertyDescriptor(Array, Symbol.species).get;
const typedSpecies = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array), Symbol.species).get;
const bufferSpecies = Object.getOwnPropertyDescriptor(ArrayBuffer, Symbol.species).get;
const regexpSpecies = Object.getOwnPropertyDescriptor(RegExp, Symbol.species).get;
const speciesCallDescriptor = Object.getOwnPropertyDescriptor(bufferSpecies, 'call');
function restore(target, key, descriptor) {
  if (descriptor === undefined) delete target[key];
  else define(target, key, descriptor);
}
try {
  const literalTrace = [];
  function ignored(label) { literalTrace.push(label); return {valueOf: poison, toString: poison}; }
  check(Number.isFinite(3, ignored('finite.extra')) === true, 'finite literal result');
  check(Number.isNaN(3, ignored('nan.extra')) === false, 'NaN literal result');
  check(Number.isSafeInteger(3, ignored('safe.extra')) === true, 'safeInteger literal result');
  check(unescape('%41%u0042%zz', ignored('unescape.extra')) === 'AB%zz', 'unescape literal and malformed spelling');
  check(literalTrace.join(',') === 'finite.extra,nan.extra,safe.extra,unescape.extra', 'literal native calls retain ignored operand effects');

  const finiteReceiver = {saved: Number.isFinite, isFinite: poison};
  const nanReceiver = {saved: Number.isNaN, isNaN: poison};
  const safeReceiver = {saved: Number.isSafeInteger, isSafeInteger: poison};
  const decoder = {saved: unescape, unescape: poison};
  check(finiteReceiver.saved(3, ignored('alias.finite')) === true && nanReceiver.saved(3, ignored('alias.nan')) === false &&
    safeReceiver.saved(3, ignored('alias.safe')) === true && decoder.saved('%41', ignored('alias.unescape')) === 'A',
    'transferred native literals retain acquired properties');
  const pureTrace = [];
  const noCoercion = {valueOf: poison, toString: poison};
  function firstArgument() { pureTrace.push('first'); return noCoercion; }
  function lastArgument() { pureTrace.push('extra'); return 2; }
  check(Number.isFinite(firstArgument(), lastArgument()) === false &&
    Number.isNaN(firstArgument(), lastArgument()) === false && Number.isSafeInteger(firstArgument(), lastArgument()) === false &&
    pureTrace.join(',') === 'first,extra,first,extra,first,extra', 'Number predicates evaluate operands without ToPrimitive');

  let replacements = 0;
  function replaceFinite() { replacements++; Number.isFinite = poison; return 2; }
  check(Number.isFinite(3, replaceFinite()) === true && replacements === 1, 'finite callee acquired before ignored replacement');
  restore(Number, 'isFinite', finiteDescriptor);
  function replaceNaN() { replacements++; Number.isNaN = poison; return 2; }
  check(Number.isNaN(3, replaceNaN()) === false && replacements === 2, 'NaN callee acquired before replacement');
  restore(Number, 'isNaN', nanDescriptor);
  function replaceSafe() { replacements++; Number.isSafeInteger = poison; return 2; }
  check(Number.isSafeInteger(3, replaceSafe()) === true && replacements === 3, 'safeInteger callee acquired before replacement');
  restore(Number, 'isSafeInteger', safeDescriptor);
  function replaceUnescape() { replacements++; globalThis.unescape = poison; return 2; }
  check(unescape('%41', replaceUnescape()) === 'A' && replacements === 4, 'global unescape Reference acquired before replacement');
  restore(globalThis, 'unescape', unescapeDescriptor);
  const transferredDecode = nativeUnescape;
  check(transferredDecode('%u0041', ignored('direct.alias')) === 'A', 'transferred non-property unescape call');
  check(Number.isFinite(Infinity) === false && Number.isNaN(NaN) === true && Number.isSafeInteger(1.5) === false &&
    Number.isSafeInteger(9007199254740991) === true && Number.isSafeInteger(9007199254740992) === false,
    'retained native predicate boundary results');

  let kindFlow = 1;
  const shapeFlow = {value: 1};
  const elementFlow = [1];
  const hookSymbol = Symbol('unescape effect');
  function effectFunction() { return 73; }
  const coercible = {toString() {
    kindFlow = effectFunction; shapeFlow.value = hookSymbol; elementFlow[0] = effectFunction;
    return '%41%u0042';
  }};
  const unescapeTrace = [];
  function decodeFirst() { unescapeTrace.push('first'); return coercible; }
  function decodeExtra() { unescapeTrace.push('extra'); return {toString: poison}; }
  check(nativeUnescape(decodeFirst(), decodeExtra()) === 'AB' && unescapeTrace.join(',') === 'first,extra' &&
    typeof kindFlow === 'function' && kindFlow() === 73 && typeof shapeFlow.value === 'symbol' && shapeFlow.value === hookSymbol &&
    typeof elementFlow[0] === 'function' && elementFlow[0]() === 73, 'unescape ToString invalidates captured kind/shape/element facts');

  const rawThis = Symbol('strict forwarding this');
  function target(first, second) { 'use strict'; return [this, arguments.length, first, second]; }
  target.invoke = Function.prototype.call; target.call = poison;
  const forwarded = target.invoke(rawThis, ...[7, 9], 'last');
  check(forwarded[0] === rawThis && forwarded[1] === 3 && forwarded[2] === 7 && forwarded[3] === 9,
    'acquired Function.call alias/raw this/full spread forwarding');
  target.applySaved = Function.prototype.apply; target.apply = poison;
  const applyTrace = [];
  const applyArguments = {get length() { applyTrace.push('length'); return 2; },
    get 0() { applyTrace.push('get0'); return 11; }, get 1() { applyTrace.push('get1'); return 13; },
    get [Symbol.iterator]() { throw 'apply must use array-like Get, not iteration'; }};
  const applied = target.applySaved(null, applyArguments, (applyTrace.push('extra'), 99));
  check(applied[0] === null && applied[1] === 2 && applied[2] === 11 && applied[3] === 13 &&
    applyTrace.join(',') === 'extra,length,get0,get1', 'acquired Function.apply/array-like/ignored operands');
  target.bindSaved = Function.prototype.bind; target.bind = poison;
  const bound = target.bindSaved(undefined, ...[17]);
  const boundResult = bound(19);
  check(boundResult[0] === undefined && boundResult[1] === 2 && boundResult[2] === 17 && boundResult[3] === 19,
    'acquired Function.bind baseline forwarding');

  let proxyFlow = 1;
  const proxyTarget = new Proxy(target, {apply(callee, receiver, args) {
    check(callee === target && receiver === rawThis && args.length === 2, 'Function.call actual Proxy target Reference');
    proxyFlow = effectFunction; return Reflect.apply(callee, receiver, args);
  }});
  const calledProxy = nativeCall.call(proxyTarget, rawThis, 23, 29);
  check(calledProxy[0] === rawThis && calledProxy[2] === 23 && calledProxy[3] === 29 &&
    typeof proxyFlow === 'function' && proxyFlow() === 73, 'Function.call Proxy invocation effects');
  let applyFlow = 1;
  const observedArguments = {get length() { applyFlow = effectFunction; return 1; }, get 0() { return 31; }};
  const applyEffect = nativeApply.call(target, rawThis, observedArguments);
  check(applyEffect[2] === 31 && typeof applyFlow === 'function' && applyFlow() === 73, 'Function.apply argument Get effects');
  let bindFlow = 1;
  const bindTrace = [];
  const metadataTarget = new Proxy(target, {get(callee, key, receiver) {
    if (key === 'length') { bindTrace.push('length'); bindFlow = effectFunction; }
    if (key === 'name') bindTrace.push('name');
    return Reflect.get(callee, key, receiver);
  }});
  const metadataBound = nativeBind.call(metadataTarget, rawThis, 37);
  check(typeof bindFlow === 'function' && bindFlow() === 73 && bindTrace.join(',') === 'length,name' &&
    metadataBound(41)[2] === 37, 'Function.bind Proxy metadata Get effects');

  const objectValue = {value: 1};
  const functionValue = function() { return 79; };
  const symbolValue = Symbol('species raw result');
  const rawValues = [3, 'text', true, 5n, symbolValue, null, undefined, objectValue, functionValue, []];
  for (let index = 0; index < rawValues.length; index++) {
    const raw = rawValues[index];
    check(arraySpecies.call(raw) === raw && typedSpecies.call(raw) === raw && bufferSpecies.call(raw) === raw &&
      regexpSpecies.call(raw) === raw, 'all four generic species getters return arbitrary raw this');
  }
  const speciesTrace = [];
  bufferSpecies.call = Function.prototype.call;
  function replaceCall() { speciesTrace.push('argument'); bufferSpecies.call = poison; return {toString: poison}; }
  check(bufferSpecies.call(symbolValue, replaceCall()) === symbolValue && speciesTrace.join(',') === 'argument',
    'species getter retains acquired Function.call and unboxed Symbol this');
  restore(bufferSpecies, 'call', speciesCallDescriptor);
  bufferSpecies.call = new Proxy(nativeCall, {apply(callee, receiver, args) {
    speciesTrace.push('apply');
    check(callee === nativeCall && receiver === bufferSpecies && args.length === 2 && args[0] === null,
      'species actual acquired Proxy call method');
    return Reflect.apply(callee, receiver, args);
  }});
  check(bufferSpecies.call(null, (speciesTrace.push('extra'), 1)) === null &&
    speciesTrace.join(',') === 'argument,extra,apply', 'species call method Proxy preserves raw null and ignored extra');
} finally {
  restore(Number, 'isFinite', finiteDescriptor);
  restore(Number, 'isNaN', nanDescriptor);
  restore(Number, 'isSafeInteger', safeDescriptor);
  restore(globalThis, 'unescape', unescapeDescriptor);
  restore(bufferSpecies, 'call', speciesCallDescriptor);
}
print('remaining-invocation-forwarding:ok');
262;
