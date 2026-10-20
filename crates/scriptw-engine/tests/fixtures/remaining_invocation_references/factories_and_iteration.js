function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'canonical property, replaced callee or ignored operand coerced'; }
const define = Object.defineProperty;
const getPrototypeOf = Object.getPrototypeOf;
const Float = Float64Array;
const floatPrototype = Float.prototype;
const fromTrace = [];
let fromConstructs = 0, getterFlow = 1;
function DifferentFrom(length) {
  check(new.target === DifferentFrom && arguments.length === 1 && length === 2, 'from actual constructor operands');
  fromTrace.push('construct'); fromConstructs++; return new Float(length);
}
DifferentFrom.saved = Int8Array.from;
DifferentFrom.from = poison;
const arrayLike = {
  get [Symbol.iterator]() { fromTrace.push('iterator.get'); return undefined; },
  get length() { fromTrace.push('length'); return 2; },
  get 0() { fromTrace.push('get0'); getterFlow = function() { return 61; }; return 1.25; },
  get 1() { fromTrace.push('get1'); return 2.5; }
};
const mapperThis = {name: 'mapper receiver'};
function mapper(value, index) {
  check(this === mapperThis && arguments.length === 2, 'from mapping raw this/argc');
  fromTrace.push('map' + index); return value + 0.5;
}
function fromArgument() { fromTrace.push('argument'); DifferentFrom.saved = poison; return arrayLike; }
function fromExtra() { fromTrace.push('extra'); return {toString: poison, valueOf: poison}; }
const fromResult = DifferentFrom.saved(fromArgument(), ...[mapper, mapperThis], fromExtra());
check(getPrototypeOf(fromResult) === floatPrototype && fromResult.length === 2 && fromResult[0] === 1.75 && fromResult[1] === 3 &&
  fromConstructs === 1 && typeof getterFlow === 'function' && getterFlow() === 61 &&
  fromTrace.join(',') === 'argument,extra,iterator.get,length,construct,get0,map0,get1,map1',
  'from original alias/custom typed-array result/array-like Getter effects');

const ofTrace = [];
let coercionFlow = 1;
function DifferentOf(length) {
  check(new.target === DifferentOf && arguments.length === 1 && length === 3, 'of actual constructor length');
  ofTrace.push('construct'); return new Float(length);
}
DifferentOf.saved = Uint8Array.of;
DifferentOf.of = poison;
const firstValue = {valueOf() { ofTrace.push('coerce'); coercionFlow = function() { return 67; }; return 1.25; }};
function ofArgument() { ofTrace.push('first.argument'); DifferentOf.saved = poison; return firstValue; }
let position = 0;
const iterator = {};
const next = new Proxy(function() {}, {apply(target, receiver, args) {
  check(receiver === iterator && args.length === 0, 'of argument cached next raw Reference');
  const index = position++; check(index < 2, 'bounded of argument spread'); ofTrace.push('next' + index);
  if (index === 0) define(iterator, 'next', {value: poison});
  return {get done() { ofTrace.push('done' + index); return index === 1; }, get value() {
    check(index === 0, 'terminal argument value Get omitted'); ofTrace.push('value0'); return 2.5;
  }};
}});
define(iterator, 'next', {configurable: true, get() { ofTrace.push('next.get'); return next; }});
define(iterator, 'return', {get() { throw 'normal argument spread close'; }});
const spread = [99];
define(spread, '0', {get() { throw 'argument numeric snapshot'; }});
define(spread, Symbol.iterator, {get() {
  ofTrace.push('iterator.get'); return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === spread && args.length === 0, 'of iterator acquisition raw Reference');
    ofTrace.push('open'); return iterator;
  }});
}});
const ofResult = DifferentOf.saved(ofArgument(), ...spread, (ofTrace.push('last.argument'), 3.75));
check(getPrototypeOf(ofResult) === floatPrototype && ofResult.length === 3 && ofResult[0] === 1.25 && ofResult[1] === 2.5 && ofResult[2] === 3.75 &&
  typeof coercionFlow === 'function' && coercionFlow() === 67 && ofTrace.join(',') ===
  'first.argument,iterator.get,open,next.get,next0,done0,value0,next1,done1,last.argument,construct,coerce',
  'of retained alias/full real spread/custom result/element coercion effects');

const proxyConstructorTrace = [];
function ConstructorTarget() { throw 'Proxy construct trap bypassed'; }
const constructor = new Proxy(ConstructorTarget, {construct(target, args, newTarget) {
  check(target === ConstructorTarget && newTarget === constructor && args.length === 1 && args[0] === 2,
    'borrowed factory Proxy Construct identities');
  proxyConstructorTrace.push('construct'); return new Float(args[0]);
}});
constructor.saved = Int8Array.of; constructor.of = poison;
const proxyResult = constructor.saved(1.5, 2.75);
check(getPrototypeOf(proxyResult) === floatPrototype && proxyResult[0] === 1.5 && proxyResult[1] === 2.75 &&
  proxyConstructorTrace.join(',') === 'construct', 'factory actual Proxy constructor/result');

const iterationTrace = [];
const iterationFlow = [1];
const source = {get length() { iterationTrace.push('length'); return 2; },
  get 0() { iterationTrace.push('get0'); iterationFlow[0] = function() { return 71; }; return 7; },
  get 1() { iterationTrace.push('get1'); return 9; }};
const values = Array.prototype.values.call(source);
const nativeNext = values.next;
values.advance = nativeNext; values.next = poison;
function nextArgument() { iterationTrace.push('argument'); values.advance = poison; return {valueOf: poison, toString: poison}; }
const first = values.advance(nextArgument(), ...[1, 2], (iterationTrace.push('extra'), 3));
check(first.done === false && first.value === 7 && iterationTrace.join(',') === 'argument,extra,length,get0' &&
  typeof iterationFlow[0] === 'function' && iterationFlow[0]() === 71, 'iterator alias/raw receiver/full ignored argv and element Getter effects');
values.advance = nativeNext;
const second = values.advance('ignored');
const terminal = values.advance('ignored', 2);
check(second.done === false && second.value === 9 && terminal.done === true && terminal.value === undefined &&
  iterationTrace.join(',') === 'argument,extra,length,get0,length,get1,length', 'iterator current array-like length and terminal cutoff');
print('remaining-invocation-factories:ok');
262;
