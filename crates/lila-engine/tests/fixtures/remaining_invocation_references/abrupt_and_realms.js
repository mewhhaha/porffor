function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'public constructor, canonical method or unreachable operand'; }
const define = Object.defineProperty;
const getPrototypeOf = Object.getPrototypeOf;
const Float = Float64Array;
const nativeOf = Uint8Array.of;
const nativeApply = Function.prototype.apply;
const nativeCall = Function.prototype.call;
const nativeBind = Function.prototype.bind;
const nativeUnescape = unescape;
const nativeNext = getPrototypeOf([][Symbol.iterator]()).next;
const bufferSpecies = Object.getOwnPropertyDescriptor(ArrayBuffer, Symbol.species).get;
const speciesCallDescriptor = Object.getOwnPropertyDescriptor(bufferSpecies, 'call');
const localTypePrototype = TypeError.prototype;
const foreign = $262.createRealm().global;
const ForeignObject = foreign.Object;
const foreignApply = foreign.Function.prototype.apply;
const foreignNext = getPrototypeOf(new foreign.Array()[Symbol.iterator]()).next;
const foreignTypePrototype = foreign.TypeError.prototype;
const foreignErrorPrototype = foreign.Error.prototype;
const localArrayDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'Array');
const localTypeDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'TypeError');
const foreignArrayDescriptor = Object.getOwnPropertyDescriptor(foreign, 'Array');
const foreignTypeDescriptor = Object.getOwnPropertyDescriptor(foreign, 'TypeError');
const marker = new foreign.Error('remaining invocation marker');
function restore(target, key, descriptor) {
  if (descriptor === undefined) delete target[key];
  else define(target, key, descriptor);
}
function expectMarker(action, expected, trace) {
  let result = 'unpublished', caught;
  try { const value = action(); result = value; }
  catch (error) { caught = error; trace.push('catch'); }
  finally { trace.push('finally'); }
  check(caught === marker && getPrototypeOf(caught) === foreignErrorPrototype && result === 'unpublished' &&
    trace.join(',') === expected + ',catch,finally', 'original foreign marker/publication cutoff/finally');
}
function expectNative(action, prototype) {
  let caught, finalized = 0;
  try { action(); } catch (error) { caught = error; } finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === prototype && finalized === 1,
    'native error uses called intrinsic Realm');
}
try {
  const acquisitionTrace = [];
  function AcquisitionConstructor(length) { throw 'construct after acquisition failure'; }
  AcquisitionConstructor.from = poison;
  define(AcquisitionConstructor, 'saved', {get() { acquisitionTrace.push('callee'); throw marker; }});
  expectMarker(() => AcquisitionConstructor.saved((acquisitionTrace.push('unreached.argument'), [])), 'callee', acquisitionTrace);

  const argumentTrace = [];
  function ArgumentConstructor(length) { throw 'construct after argument failure'; }
  ArgumentConstructor.saved = Int8Array.from; ArgumentConstructor.from = poison;
  function failingArgument() { argumentTrace.push('argument'); throw marker; }
  expectMarker(() => ArgumentConstructor.saved(failingArgument(), (argumentTrace.push('unreached.extra'), 2)),
    'argument', argumentTrace);

  const constructTrace = [];
  function ThrowingConstructor(length) { constructTrace.push('construct'); throw marker; }
  ThrowingConstructor.saved = Uint8Array.of; ThrowingConstructor.of = poison;
  expectMarker(() => ThrowingConstructor.saved((constructTrace.push('argument'), 7)), 'argument,construct', constructTrace);

  const coercionTrace = [];
  let constructedOf;
  function PartialOf(length) { coercionTrace.push('construct'); constructedOf = new Float(length); return constructedOf; }
  PartialOf.saved = Uint8Array.of; PartialOf.of = poison;
  const badElement = {valueOf() { coercionTrace.push('coerce'); throw marker; }};
  expectMarker(() => PartialOf.saved(7, badElement, 9), 'construct,coerce', coercionTrace);
  check(constructedOf.length === 3 && constructedOf[0] === 7 && constructedOf[1] === 0 && constructedOf[2] === 0,
    'of preserves prior element write and stops remaining conversion');

  const lengthTrace = [];
  function NoLengthConstructor(length) { lengthTrace.push('unreached.construct'); return new Float(length); }
  NoLengthConstructor.saved = Int8Array.from; NoLengthConstructor.from = poison;
  const noLength = {get [Symbol.iterator]() { lengthTrace.push('iterator.get'); return undefined; },
    get length() { lengthTrace.push('length'); throw marker; }};
  expectMarker(() => NoLengthConstructor.saved(noLength), 'iterator.get,length', lengthTrace);

  const elementTrace = [];
  let constructedFrom;
  function PartialFrom(length) { elementTrace.push('construct'); constructedFrom = new Float(length); return constructedFrom; }
  PartialFrom.saved = Int8Array.from; PartialFrom.from = poison;
  const brokenSource = {get [Symbol.iterator]() { elementTrace.push('iterator.get'); return undefined; },
    get length() { elementTrace.push('length'); return 2; },
    get 0() { elementTrace.push('get0'); return 7; }, get 1() { elementTrace.push('get1'); throw marker; }};
  expectMarker(() => PartialFrom.saved(brokenSource), 'iterator.get,length,construct,get0,get1', elementTrace);
  check(constructedFrom[0] === 7 && constructedFrom[1] === 0, 'from preserves prior array-like write');

  let closes = 0, spreadConstructs = 0;
  function SpreadConstructor(length) { spreadConstructs++; return new Float(length); }
  SpreadConstructor.saved = nativeOf; SpreadConstructor.of = poison;
  for (let stage = 0; stage < 4; stage++) {
    const trace = [];
    const iterator = {};
    define(iterator, 'return', {get() { closes++; throw 'argument spread must not close'; }});
    define(iterator, 'next', {get() {
      trace.push('next.get'); if (stage === 0) throw marker;
      return new Proxy(function() {}, {apply(target, receiver, args) {
        check(receiver === iterator && args.length === 0, 'spread abrupt cached-next Reference');
        trace.push('next'); if (stage === 1) throw marker;
        return {get done() { trace.push('done'); if (stage === 2) throw marker; return false; },
          get value() { trace.push('value'); throw marker; }};
      }});
    }});
    const iterable = {[Symbol.iterator]() { trace.push('open'); return iterator; }};
    const expected = ['open,next.get', 'open,next.get,next', 'open,next.get,next,done', 'open,next.get,next,done,value'][stage];
    expectMarker(() => SpreadConstructor.saved(...iterable, (trace.push('unreached.argument'), 9)), expected, trace);
  }
  check(closes === 0 && spreadConstructs === 0, 'no argument spread Close or factory invocation after iterator abrupt');

  const literalTrace = [];
  function ignoredFailure() { literalTrace.push('ignored.argument'); throw marker; }
  expectMarker(() => Number.isFinite(3, ignoredFailure()), 'ignored.argument', literalTrace);
  const unescapeTrace = [];
  const failingString = {toString() { unescapeTrace.push('toString'); throw marker; }};
  function firstString() { unescapeTrace.push('first.argument'); return failingString; }
  expectMarker(() => nativeUnescape(firstString(), (unescapeTrace.push('ignored.argument'), 2)),
    'first.argument,ignored.argument,toString', unescapeTrace);

  const applyTrace = [];
  function unapplied() { applyTrace.push('unreached.target'); }
  unapplied.saved = Function.prototype.apply; unapplied.apply = poison;
  const brokenArguments = {get length() { applyTrace.push('length'); return 2; },
    get 0() { applyTrace.push('get0'); return 7; }, get 1() { applyTrace.push('get1'); throw marker; }};
  expectMarker(() => unapplied.saved(null, brokenArguments, (applyTrace.push('ignored.argument'), 9)),
    'ignored.argument,length,get0,get1', applyTrace);

  const callTrace = [];
  const callTarget = new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === null && args.length === 2 && args[0] === 7 && args[1] === 9, 'abrupt Function.call Proxy operands');
    callTrace.push('apply'); throw marker;
  }});
  expectMarker(() => nativeCall.call(callTarget, null, (callTrace.push('first.argument'), 7),
    (callTrace.push('last.argument'), 9)), 'first.argument,last.argument,apply', callTrace);

  const bindTrace = [];
  const bindTarget = new Proxy(function() {}, {get(target, key, receiver) {
    if (key === 'length') { bindTrace.push('length'); throw marker; }
    if (key === 'name') bindTrace.push('unreached.name');
    return Reflect.get(target, key, receiver);
  }});
  expectMarker(() => nativeBind.call(bindTarget, (bindTrace.push('this.argument'), null),
    (bindTrace.push('bound.argument'), 7)), 'this.argument,bound.argument,length', bindTrace);

  const speciesTrace = [];
  bufferSpecies.call = new Proxy(nativeCall, {apply(target, receiver, args) {
    check(receiver === bufferSpecies && args[0] === undefined && args.length === 2, 'abrupt acquired species call method');
    speciesTrace.push('apply'); throw marker;
  }});
  expectMarker(() => bufferSpecies.call(undefined, (speciesTrace.push('ignored.argument'), 9)),
    'ignored.argument,apply', speciesTrace);

  // Both called-Realm identities are saved before public names are poisoned.
  globalThis.TypeError = poison; foreign.TypeError = poison;
  globalThis.Array = poison; foreign.Array = poison;
  const localInvalid = {saved: foreignApply, apply: poison};
  const foreignInvalid = new ForeignObject();
  foreignInvalid.saved = nativeApply; foreignInvalid.apply = poison;
  expectNative(() => localInvalid.saved(null, []), foreignTypePrototype);
  expectNative(() => foreignInvalid.saved(undefined, []), localTypePrototype);
  const localBadIterator = {advance: foreignNext, next: poison};
  const foreignBadIterator = new ForeignObject();
  foreignBadIterator.advance = nativeNext; foreignBadIterator.next = poison;
  expectNative(() => localBadIterator.advance('ignored'), foreignTypePrototype);
  expectNative(() => foreignBadIterator.advance('ignored', 2), localTypePrototype);
} finally {
  restore(bufferSpecies, 'call', speciesCallDescriptor);
  restore(globalThis, 'Array', localArrayDescriptor);
  restore(globalThis, 'TypeError', localTypeDescriptor);
  restore(foreign, 'Array', foreignArrayDescriptor);
  restore(foreign, 'TypeError', foreignTypeDescriptor);
}
print('remaining-invocation-abrupt:ok');
262;
