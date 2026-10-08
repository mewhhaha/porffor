function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'canonical or replaced method must not be called'; }
const nativeSubstring = String.prototype.substring;
const nativeSlice = String.prototype.slice;
const swapped = {cut: String.prototype.substring, substring: poison, toString() { return 'abcdef'; }};
check(swapped.cut(4, 1) === 'bcd' && swapped.cut(-3, 2) === 'ab', 'substring swaps and clamps');
const negative = {cut: String.prototype.slice, slice: poison, toString() { return 'abcdef'; }};
check(negative.cut(-3, -1) === 'de' && negative.cut(4, 1) === '', 'slice negative indexes without swapping');
const units = '😀x';
check(units.slice(0, 1) === '\uD83D' && units.substring(1, 2) === '\uDE00' && units.slice(0, 2) === '😀',
  'range algorithms retain UTF-16 code units');

const define = Object.defineProperty;
const trace = [];
const source = {substring: poison, toString() {
  check(this === source, 'receiver ToString original this'); trace.push('receiver'); return 'A😀BC';
}};
let selected = source;
define(source, 'cut', {configurable: true, get() { trace.push('callee'); return nativeSubstring; }});
function base() { trace.push('base'); return selected; }
function key() { trace.push('key'); return 'cut'; }
function startArgument() {
  trace.push('start.arg');
  selected = {toString() { throw 'rebound receiver'; }};
  define(source, 'cut', {value: poison});
  return {valueOf() { trace.push('start.coerce'); return 1; }};
}
const end = {valueOf() { trace.push('end.coerce'); return 4; }};
let position = 0;
const iterator = {};
const next = new Proxy(function() {}, {apply(target, receiver, args) {
  check(receiver === iterator && args.length === 0, 'String arguments next receiver and argc');
  const index = position++;
  check(index < 2, 'bounded String argument spread');
  trace.push('next' + index);
  if (index === 0) define(iterator, 'next', {value: poison});
  return {get done() { trace.push('done' + index); return index === 1; }, get value() {
    check(index === 0, 'String terminal value unread'); trace.push('value0'); return end;
  }};
}});
define(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
define(iterator, 'return', {get() { throw 'String argument spread normal completion does not close'; }});
const spread = [99];
define(spread, '0', {get() { throw 'String numeric spread snapshot'; }});
define(spread, Symbol.iterator, {get() {
  trace.push('iterator.get');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === spread && args.length === 0, 'String argument iterator receiver and argc');
    trace.push('open'); return iterator;
  }});
}});
function tail() { trace.push('tail'); return {valueOf() { throw 'ignored extra coerced'; }}; }
check(base()[key()](startArgument(), ...spread, tail()) === '😀B' && selected !== source,
  'computed alias retains original callee and raw receiver');
check(trace.join(',') === 'base,key,callee,start.arg,iterator.get,open,next.get,next0,done0,value0,next1,done1,tail,' +
  'receiver,start.coerce,end.coerce', 'all arguments before receiver then numeric coercion');

const proxyTrace = [];
const proxySource = {slice: poison, toString() { proxyTrace.push('receiver'); return 'abcdef'; }};
let applyCalls = 0;
proxySource.cut = new Proxy(nativeSlice, {apply(target, receiver, args) {
  check(target === nativeSlice && receiver === proxySource && args.length === 3 && args[2] === 'ignored',
    'String Proxy receives raw object and full arguments');
  proxyTrace.push('apply'); applyCalls++;
  return Reflect.apply(target, receiver, args);
}});
function proxyStart() { proxyTrace.push('start.arg'); return -3; }
function proxyEnd() { proxyTrace.push('end.arg'); return -1; }
function proxyExtra() { proxyTrace.push('extra.arg'); return 'ignored'; }
check(proxySource.cut(proxyStart(), proxyEnd(), proxyExtra()) === 'de' && applyCalls === 1 &&
  proxyTrace.join(',') === 'start.arg,end.arg,extra.arg,apply,receiver', 'callable Proxy and ignored String extras');

let receiverFlow = 1;
const receiverEffects = {cut: String.prototype.substring, substring: poison, toString() {
  receiverFlow = function() { return 41; }; return 'abcdef';
}};
check(receiverEffects.cut(1, 3) === 'bc' && typeof receiverFlow === 'function' && receiverFlow() === 41,
  'substring receiver coercion invalidates captured facts');
const numericFlow = {value: 1};
const numericEffects = {cut: String.prototype.slice, slice: poison, toString() { return 'abcdef'; }};
const changingStart = {valueOf() { numericFlow.value = 'number-hook'; return 1; }};
check(numericEffects.cut(changingStart, 3) === 'bc' && typeof numericFlow.value === 'string' && numericFlow.value.charAt(0) === 'n',
  'slice numeric coercion invalidates shape facts');

const foreign = $262.createRealm().global;
const ForeignString = foreign.String;
const localTypeErrorPrototype = TypeError.prototype;
const foreignTypeErrorPrototype = foreign.TypeError.prototype;
const foreignSubstring = ForeignString.prototype.substring;
const foreignSlice = ForeignString.prototype.slice;
const getPrototypeOf = Object.getPrototypeOf;
const marker = new foreign.Error('String range marker');
String = poison; foreign.String = poison;
TypeError = poison; foreign.TypeError = poison;
const opposite = Object.create(foreign.Object.prototype);
opposite.toString = function() { return 'abcdef'; };
opposite.cut = nativeSubstring; opposite.substring = poison;
check(opposite.cut(4, 1) === 'bcd', 'local range method on foreign object after globals poison');
const local = {cut: foreignSlice, slice: poison, toString() { return 'abcdef'; }};
check(local.cut(-3, -1) === 'de', 'foreign range method on local object after globals poison');
function nativeError(method, errorPrototype) {
  let caught;
  let finalized = 0;
  try { Reflect.apply(method, null, [0, 1]); }
  catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === errorPrototype && finalized === 1, 'range native error defining Realm');
}
nativeError(nativeSubstring, localTypeErrorPrototype);
nativeError(nativeSlice, localTypeErrorPrototype);
nativeError(foreignSubstring, foreignTypeErrorPrototype);
nativeError(foreignSlice, foreignTypeErrorPrototype);

function coercionFailure(method, canonical, stage, expected) {
  const effects = [];
  const receiver = {cut: method, toString() {
    effects.push('receiver'); if (stage === 'receiver') throw marker; return 'abcdef';
  }};
  receiver[canonical] = poison;
  const start = {valueOf() { effects.push('start'); if (stage === 'start') throw marker; return 1; }};
  const end = {valueOf() { effects.push('end'); throw marker; }};
  let result = 'unpublished';
  let caught;
  let prior = 17;
  try { prior = 18; result = receiver.cut(start, end); }
  catch (error) { caught = error; }
  finally { effects.push('finally'); }
  check(caught === marker && result === 'unpublished' && prior === 18 && effects.join(',') === expected + ',finally',
    'original range coercion throw and exact cutoff');
}
coercionFailure(nativeSubstring, 'substring', 'receiver', 'receiver');
coercionFailure(foreignSlice, 'slice', 'start', 'receiver,start');
coercionFailure(nativeSubstring, 'substring', 'end', 'receiver,start,end');

const abruptTrace = [];
const inaccessible = {get cut() { abruptTrace.push('callee'); throw marker; }, substring: poison};
let caught;
let result = 'unpublished';
function argument() { abruptTrace.push('argument'); return 1; }
try { result = inaccessible.cut(argument()); }
catch (error) { caught = error; }
finally { abruptTrace.push('finally'); }
check(caught === marker && result === 'unpublished' && abruptTrace.join(',') === 'callee,finally', 'range acquisition skips arguments');
const argumentFailure = {cut: nativeSlice, slice: poison, toString() { throw 'receiver coercion before arguments complete'; }};
function failingArgument() { throw marker; }
caught = undefined; result = 'unpublished';
try { result = argumentFailure.cut(failingArgument()); } catch (error) { caught = error; }
check(caught === marker && result === 'unpublished', 'range argument throw skips receiver coercion');
let closes = 0;
const failingIterator = {next() { return {get done() { return false; }, get value() { throw marker; }}; }};
define(failingIterator, 'return', {get() { closes++; throw 'range argument spread must not close'; }});
const failingSpread = {[Symbol.iterator]() { return failingIterator; }};
caught = undefined; result = 'unpublished';
let finalized = 0;
try { result = argumentFailure.cut(...failingSpread); }
catch (error) { caught = error; }
finally { finalized++; }
check(caught === marker && result === 'unpublished' && closes === 0 && finalized === 1, 'range real spread failure before coercion');
print('invocation-shortcut-strings:ok');
262;
