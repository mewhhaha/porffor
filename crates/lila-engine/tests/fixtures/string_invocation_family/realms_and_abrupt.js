function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'mutable public constructor or canonical String method'; }
const foreign = $262.createRealm().global;
const ForeignError = foreign.Error;
const marker = new ForeignError('String family marker');
const localMethods = {substr: String.prototype.substr, normalize: String.prototype.normalize,
  padStart: String.prototype.padStart, split: String.prototype.split, toUpperCase: String.prototype.toUpperCase};
const foreignMethods = {substr: foreign.String.prototype.substr, normalize: foreign.String.prototype.normalize,
  padStart: foreign.String.prototype.padStart, split: foreign.String.prototype.split, toUpperCase: foreign.String.prototype.toUpperCase};
const localTypePrototype = TypeError.prototype;
const foreignTypePrototype = foreign.TypeError.prototype;
const localRangePrototype = RangeError.prototype;
const foreignRangePrototype = foreign.RangeError.prototype;
const localObjectPrototype = Object.prototype;
const foreignObjectPrototype = foreign.Object.prototype;
const getPrototypeOf = Object.getPrototypeOf;
const define = Object.defineProperty;
globalThis.String = poison; foreign.String = poison;
globalThis.TypeError = poison; foreign.TypeError = poison;
globalThis.RangeError = poison; foreign.RangeError = poison;
function expectNative(action, prototype, label) {
  let caught, finalized = 0;
  try { action(); } catch (error) { caught = error; } finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === prototype && finalized === 1, label);
}
let borrowedCases = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? foreignMethods : localMethods;
  const typePrototype = direction === 0 ? foreignTypePrototype : localTypePrototype;
  const rangePrototype = direction === 0 ? foreignRangePrototype : localRangePrototype;
  const oppositePrototype = direction === 0 ? localObjectPrototype : foreignObjectPrototype;
  const receiver = Object.create(oppositePrototype);
  receiver.toString = function() { check(this === receiver, 'borrowed raw receiver'); return 'e\u0301abc'; };
  receiver.saved = methods.substr; receiver.substr = poison;
  check(receiver.saved(-3, 2) === 'ab', 'borrowed substr on opposite-Realm object');
  receiver.saved = methods.normalize; receiver.normalize = poison;
  check(receiver.saved('NFC') === '\u00E9abc', 'borrowed normalization on opposite-Realm object');
  receiver.saved = methods.toUpperCase; receiver.toUpperCase = poison;
  check(receiver.saved() === 'E\u0301ABC', 'borrowed case method on opposite-Realm object');
  let formConversions = 0;
  const form = {toString() { formConversions++; throw 'form after null receiver'; }};
  expectNative(() => methods.normalize.call(null, form, 'ignored'), typePrototype, 'normalization null receiver defining Realm');
  check(formConversions === 0, 'null receiver precedes form coercion');
  expectNative(() => methods.normalize.call('x', 'invalid-form'), rangePrototype, 'normalization form RangeError defining Realm');
  const noncallable = {[Symbol.split]: 0};
  expectNative(() => methods.split.call(receiver, noncallable, {}), typePrototype, 'noncallable split hook defining Realm');

  for (let absent = 0; absent < 2; absent++) {
    const argumentTrace = [];
    let hookGets = 0;
    const pattern = {};
    define(pattern, Symbol.split, {get() { hookGets++; throw 'split hook after null receiver'; }});
    const limit = {valueOf() { throw 'limit after null receiver'; }};
    const ignored = {toString() { throw 'ignored operand conversion'; }};
    function argument(label, value) { argumentTrace.push(label); return value; }
    expectNative(() => methods.split.call(absent === 0 ? null : undefined,
      argument('pattern', pattern), argument('limit', limit), argument('extra', ignored), argument('last', 42)),
      typePrototype, 'split nullish rejection after complete call arguments');
    check(argumentTrace.join(',') === 'pattern,limit,extra,last' && hookGets === 0,
      'split.call preserves ignored argument evaluation before native rejection');
    borrowedCases++;
  }
}
check(borrowedCases === 4, 'both borrowed split directions and both nullish receivers');

function expectMarker(action, expected, trace) {
  let result = 'unpublished', caught;
  try { const value = action(); result = value; }
  catch (error) { caught = error; trace.push('catch'); }
  finally { trace.push('finally'); }
  check(caught === marker && getPrototypeOf(caught) === ForeignError.prototype && result === 'unpublished' &&
    trace.join(',') === expected + ',catch,finally', 'original foreign marker and precise cutoff');
}
const acquisitionTrace = [];
const inaccessible = {get saved() { acquisitionTrace.push('callee'); throw marker; }, substr: poison};
expectMarker(() => inaccessible.saved((acquisitionTrace.push('argument'), 1)), 'callee', acquisitionTrace);
const argumentTrace = [];
const argumentReceiver = {substr: poison, toString() { throw 'coercion before arguments finish'; }};
define(argumentReceiver, 'saved', {get() { argumentTrace.push('callee'); return localMethods.substr; }});
function failingArgument() { argumentTrace.push('first'); throw marker; }
expectMarker(() => argumentReceiver.saved(failingArgument(), (argumentTrace.push('unreached'), 2)),
  'callee,first', argumentTrace);
const spreadTrace = [];
let closes = 0;
const failingIterator = {next() { spreadTrace.push('next'); return {
  get done() { spreadTrace.push('done'); return false; },
  get value() { spreadTrace.push('value'); throw marker; }
}; }};
define(failingIterator, 'return', {get() { closes++; throw 'argument spread must not close'; }});
const failingSpread = {[Symbol.iterator]() { spreadTrace.push('open'); return failingIterator; }};
const spreadReceiver = {saved: foreignMethods.substr, substr: poison, toString() { throw 'coercion after spread error'; }};
expectMarker(() => spreadReceiver.saved(...failingSpread, (spreadTrace.push('unreached'), 2)),
  'open,next,done,value', spreadTrace);
check(closes === 0, 'spread step abrupt has no IteratorClose');
const receiverTrace = [];
const receiverFailure = {saved: localMethods.substr, substr: poison, toString() { receiverTrace.push('receiver'); throw marker; }};
expectMarker(() => receiverFailure.saved((receiverTrace.push('first'), 1), (receiverTrace.push('second'), 2),
  (receiverTrace.push('extra'), 'ignored')), 'first,second,extra,receiver', receiverTrace);
const numericTrace = [];
const numericFailure = {saved: foreignMethods.substr, substr: poison, toString() { numericTrace.push('receiver'); return 'abcdef'; }};
const start = {valueOf() { numericTrace.push('start'); throw marker; }};
const count = {valueOf() { numericTrace.push('unreached'); return 2; }};
expectMarker(() => numericFailure.saved(start, count), 'receiver,start', numericTrace);
const paddingTrace = [];
const paddingFailure = {saved: foreignMethods.padStart, padStart: poison, toString() { paddingTrace.push('receiver'); return 'x'; }};
const filler = {toString() { paddingTrace.push('filler'); throw marker; }};
expectMarker(() => paddingFailure.saved(3, filler), 'receiver,filler', paddingTrace);
const hookGetTrace = [];
const hookReceiver = {saved: localMethods.split, split: poison, toString() { throw 'raw split receiver coercion'; }};
const getPattern = {};
define(getPattern, Symbol.split, {get() { hookGetTrace.push('hook'); throw marker; }});
expectMarker(() => hookReceiver.saved((hookGetTrace.push('argument'), getPattern)), 'argument,hook', hookGetTrace);
const hookCallTrace = [];
const callPattern = {};
define(callPattern, Symbol.split, {get() {
  hookCallTrace.push('hook');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === callPattern && args[0] === hookReceiver && args.length === 2, 'abrupt Proxy hook Reference');
    hookCallTrace.push('apply'); throw marker;
  }});
}});
expectMarker(() => hookReceiver.saved((hookCallTrace.push('first'), callPattern), (hookCallTrace.push('second'), 7),
  (hookCallTrace.push('extra'), 42)), 'first,second,extra,hook,apply', hookCallTrace);
print('string-invocation-realms:ok');
262;
