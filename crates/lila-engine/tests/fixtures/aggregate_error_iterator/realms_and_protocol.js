const other = __lilaCreateRealm().global;
const LocalAggregateError = AggregateError;
const ForeignAggregateError = other.AggregateError;
const LocalArray = Array;
const ForeignArray = other.Array;
const localErrorPrototype = LocalAggregateError.prototype;
const foreignErrorPrototype = ForeignAggregateError.prototype;
const localArrayPrototype = LocalArray.prototype;
const foreignArrayPrototype = ForeignArray.prototype;
const hasOwn = Object.prototype.hasOwnProperty;
function wrong() { throw 'public AggregateError or Array constructor'; }
globalThis.AggregateError = wrong; other.AggregateError = wrong;
globalThis.Array = wrong; other.Array = wrong;

function make(Constructor, input, construct) {
  return construct ? new Constructor(input, 'list') : Constructor(input, 'list');
}
function check(error, errorPrototype, arrayPrototype, values) {
  if (Object.getPrototypeOf(error) !== errorPrototype) throw 'NewTarget Error prototype';
  const errors = error.errors;
  if (Object.getPrototypeOf(errors) !== arrayPrototype || !LocalArray.isArray(errors)) throw 'called-Realm errors Array';
  if (errors.length !== values.length) throw 'iterated errors length';
  for (let i = 0; i < values.length; i++) {
    if (!Object.is(errors[i], values[i]) || !hasOwn.call(errors, i)) throw 'own iterated value';
  }
  const descriptor = Object.getOwnPropertyDescriptor(error, 'errors');
  if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable || descriptor.value !== errors) throw 'errors property descriptor';
}
function installIterator(input, values, trace) {
  let cursor = 0;
  let returnGets = 0;
  const truthyDone = {[Symbol.toPrimitive]() { throw 'done ToBoolean coercion'; }};
  const iterator = {get return() { returnGets++; throw 'normal list must not close'; }};
  const nextTarget = function() { throw 'Proxy next target bypassed'; };
  const next = new Proxy(nextTarget, {
    apply(target, receiver, args) {
      if (target !== nextTarget || receiver !== iterator || args.length !== 0) throw 'cached next receiver and argc0';
      trace.push('next.apply');
      const position = cursor++;
      if (position === 0) Object.defineProperty(iterator, 'next', {configurable: true, value() { throw 'next reread'; }});
      return {
        get done() { trace.push('done' + position); return position === values.length ? truthyDone : false; },
        get value() {
          if (position === values.length) throw 'terminal value Get';
          trace.push('value' + position); return values[position];
        }
      };
    }
  });
  Object.defineProperty(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
  const methodTarget = function() { throw 'Proxy iterator target bypassed'; };
  const method = new Proxy(methodTarget, {
    apply(target, receiver, args) {
      if (target !== methodTarget || receiver !== input || args.length !== 0) throw 'iterator receiver and argc0';
      trace.push('iterator.apply'); return iterator;
    }
  });
  Object.defineProperty(input, Symbol.iterator, {configurable: true, get() { trace.push('iterator.get'); return method; }});
  return () => returnGets;
}
function argumentsObject() { return arguments; }

let copies = 0;
for (let direction = 0; direction < 2; direction++) {
  const Constructor = direction === 0 ? LocalAggregateError : ForeignAggregateError;
  const Source = direction === 0 ? ForeignArray : LocalArray;
  const errorPrototype = direction === 0 ? localErrorPrototype : foreignErrorPrototype;
  const arrayPrototype = direction === 0 ? localArrayPrototype : foreignArrayPrototype;
  const OtherNewTarget = direction === 0 ? ForeignAggregateError : LocalAggregateError;
  const otherErrorPrototype = direction === 0 ? foreignErrorPrototype : localErrorPrototype;
  for (let construct = 0; construct < 2; construct++) {
    check(make(Constructor, 'A\u{1F600}B', construct), errorPrototype, arrayPrototype, ['A', '\u{1F600}', 'B']);
    check(make(Constructor, '', construct), errorPrototype, arrayPrototype, []);
    const ordinary = new Source(2);
    ordinary[0] = -0; ordinary[1] = NaN;
    const ordinaryError = make(Constructor, ordinary, construct);
    check(ordinaryError, errorPrototype, arrayPrototype, [-0, NaN]);
    if (ordinaryError.errors === ordinary || ordinary.length !== 2 || !Object.is(ordinary[0], -0)) throw 'fresh ordinary list';
    const token = {};
    const customArray = new Source(2);
    Object.defineProperty(customArray, '0', {configurable: true, get() { throw 'Array index snapshot'; }});
    Object.defineProperty(customArray, '1', {configurable: true, get() { throw 'Array index snapshot'; }});
    const arrayTrace = [];
    const arrayReturnGets = installIterator(customArray, [token, 'array-iterator'], arrayTrace);
    check(make(Constructor, customArray, construct), errorPrototype, arrayPrototype, [token, 'array-iterator']);
    const expected = 'iterator.get,iterator.apply,next.get,next.apply,done0,value0,next.apply,done1,value1,next.apply,done2';
    if (arrayTrace.join(',') !== expected || arrayReturnGets() !== 0) throw 'Array override Get once and cached Proxy next';
    const args = argumentsObject('ignored0', 'ignored1');
    Object.defineProperty(args, '0', {configurable: true, get() { throw 'Arguments index snapshot'; }});
    Object.defineProperty(args, '1', {configurable: true, get() { throw 'Arguments index snapshot'; }});
    const argsTrace = [];
    const argsReturnGets = installIterator(args, ['arguments-iterator', token], argsTrace);
    check(make(Constructor, args, construct), errorPrototype, arrayPrototype, ['arguments-iterator', token]);
    if (argsTrace.join(',') !== expected || argsReturnGets() !== 0) throw 'Arguments override Get once';
    copies += 5;
  }
  const source = new Source(1);
  source[0] = 'cross-new-target';
  const cross = Reflect.construct(Constructor, [source], OtherNewTarget);
  check(cross, otherErrorPrototype, arrayPrototype, ['cross-new-target']);
  if (Object.getPrototypeOf(cross.errors) === (direction === 0 ? foreignArrayPrototype : localArrayPrototype)) throw 'list does not borrow NewTarget Realm';
}
if (copies !== 20) throw 'paired call and Construct iterator cohorts';
print('aggregate-error-iterator-realms:ok');
262;
