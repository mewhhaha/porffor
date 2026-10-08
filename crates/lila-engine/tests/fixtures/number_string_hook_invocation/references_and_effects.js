function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'Number method reacquired or ignored argument coerced'; }
const numberPrototype = Number.prototype;
const matchDescriptor = Object.getOwnPropertyDescriptor(numberPrototype, 'match');
const splitDescriptor = Object.getOwnPropertyDescriptor(numberPrototype, 'split');
const define = Object.defineProperty;
const nativeSplit = String.prototype.split;
function restore(key, descriptor) {
  if (descriptor === undefined) delete numberPrototype[key];
  else define(numberPrototype, key, descriptor);
}
try {
  Number.prototype.match = String.prototype.match;
  const matchTrace = [];
  const matchPattern = {[Symbol.match](raw) {
    matchTrace.push('hook');
    check(this === matchPattern && raw === 123 && typeof raw === 'number' && arguments.length === 1,
      'borrowed match retains raw primitive and pattern receiver');
    return 37;
  }};
  function matchArgument() { matchTrace.push('argument'); Number.prototype.match = poison; return matchPattern; }
  const matched = (123).match(matchArgument());
  check(matched === 37 && matched + 1 === 38 && matchTrace.join(',') === 'argument,hook',
    'former one-argument match gate retains callee before replacement');

  Number.prototype.split = String.prototype.split;
  const splitTrace = [];
  const originalLimit = {valueOf: poison};
  const splitPattern = {[Symbol.split](raw, limit) {
    splitTrace.push('hook');
    check(this === splitPattern && raw === 321 && typeof raw === 'number' && limit === originalLimit && arguments.length === 2,
      'borrowed split retains raw primitive and uncoerced limit');
    return Symbol.for('number-split-result');
  }};
  function splitLimit() { splitTrace.push('limit.argument'); Number.prototype.split = poison; return originalLimit; }
  const split = (321).split(splitPattern, splitLimit());
  check(split === Symbol.for('number-split-result') && typeof split === 'symbol' &&
    splitTrace.join(',') === 'limit.argument,hook', 'former two-argument split gate retains callee');

  Number.prototype.split = String.prototype.split;
  let staticEffects = 0;
  const staticTrace = [];
  // This exact function-expression/RegExp return is recognized by the former
  // synthetic-throw gate. The real @@split makes that toString unreachable.
  var recognizedSeparator = {toString: function() { return /x/; }, [Symbol.split](raw, limit) {
    staticTrace.push('hook');
    check(this === recognizedSeparator && raw === 456 && limit === 2, 'recognized separator raw hook operands');
    return 41;
  }};
  function staticLimit() { staticTrace.push('limit.argument'); staticEffects++; return 2; }
  check((456).split(recognizedSeparator, staticLimit()) === 41 && staticEffects === 1 &&
    staticTrace.join(',') === 'limit.argument,hook', 'recognized separator preserves argument effect and normal result');

  const callableResult = function() { return 43; };
  const symbolResult = Symbol('arbitrary borrowed hook');
  const results = [47, callableResult, symbolResult];
  Number.prototype.match = String.prototype.match;
  Number.prototype.split = String.prototype.split;
  for (let index = 0; index < results.length; index++) {
    const result = results[index];
    const pattern = {[Symbol.match](raw) {
      check(this === pattern && raw === 987 && arguments.length === 1, 'match arbitrary result Reference'); return result;
    }, [Symbol.split](raw, limit) {
      check(this === pattern && raw === 987 && limit === originalLimit && arguments.length === 2,
        'split arbitrary result Reference'); return result;
    }};
    const matchValue = (987).match(pattern);
    const splitValue = (987).split(pattern, originalLimit);
    check(matchValue === result && splitValue === result, 'arbitrary hook result identity');
    if (index === 0) check(matchValue + 1 === 48 && splitValue + 1 === 48, 'Number hook result');
    if (index === 1) check(matchValue() === 43 && splitValue() === 43, 'Function hook result');
    if (index === 2) check(typeof matchValue === 'symbol' && typeof splitValue === 'symbol', 'Symbol hook result');
  }

  Number.prototype.match = String.prototype.match;
  const spreadTrace = [];
  let position = 0;
  const spreadPattern = {[Symbol.match](raw) {
    check(this === spreadPattern && raw === 654 && arguments.length === 1, 'spread native hook raw receiver');
    spreadTrace.push('hook'); return callableResult;
  }};
  const iterator = {};
  const next = new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === iterator && args.length === 0, 'cached next raw Reference');
    const index = position++; check(index < 2, 'bounded argument spread'); spreadTrace.push('next' + index);
    if (index === 0) define(iterator, 'next', {value: poison});
    return {get done() { spreadTrace.push('done' + index); return index === 1; }, get value() {
      check(index === 0, 'terminal value Get omitted'); spreadTrace.push('value0'); return spreadPattern;
    }};
  }});
  define(iterator, 'next', {configurable: true, get() { spreadTrace.push('next.get'); return next; }});
  define(iterator, 'return', {get() { throw 'argument spread must not close'; }});
  const iterable = [99];
  define(iterable, '0', {get() { throw 'argument spread numeric snapshot'; }});
  define(iterable, Symbol.iterator, {get() {
    spreadTrace.push('iterator.get'); Number.prototype.match = poison;
    return new Proxy(function() {}, {apply(target, receiver, args) {
      check(receiver === iterable && args.length === 0, 'iterator acquisition raw Reference');
      spreadTrace.push('open'); return iterator;
    }});
  }});
  function extra(label) { spreadTrace.push(label); return {toString: poison, valueOf: poison}; }
  check((654).match(...iterable, extra('extra'), extra('last')) === callableResult &&
    spreadTrace.join(',') === 'iterator.get,open,next.get,next0,done0,value0,next1,done1,extra,last,hook',
    'acquired Number hook survives cached-next spread and ignored operands');

  const proxyTrace = [];
  const proxyPattern = {[Symbol.split](raw, limit) {
    check(this === proxyPattern && raw === 765 && limit === originalLimit, 'Proxy-forwarded hook operands'); return symbolResult;
  }};
  Number.prototype.split = new Proxy(nativeSplit, {apply(target, receiver, args) {
    proxyTrace.push('apply');
    check(target === nativeSplit && receiver === 765 && typeof receiver === 'number' && args.length === 4 &&
      args[0] === proxyPattern && args[1] === originalLimit && args[2] === 'ignored' && args[3] === 42,
      'callable Proxy preserves all supplied arguments');
    return Reflect.apply(target, receiver, args);
  }});
  check((765).split(proxyPattern, ...[originalLimit], (proxyTrace.push('extra'), 'ignored'),
    (proxyTrace.push('last'), 42)) === symbolResult && proxyTrace.join(',') === 'extra,last,apply',
    'Proxy Call after complete arguments');

  Number.prototype.match = String.prototype.match;
  let callableFlow = 1;
  const effectMatch = {};
  define(effectMatch, Symbol.match, {get() {
    callableFlow = callableResult;
    return function(raw) { check(raw === 222 && this === effectMatch, 'effect match Reference'); return 53; };
  }});
  check((222).match(effectMatch) === 53 && typeof callableFlow === 'function' && callableFlow() === 43,
    'match getter invalidates captured kind');
  Number.prototype.split = String.prototype.split;
  const shapeFlow = {value: 1};
  const elementFlow = [1];
  const effectSplit = {[Symbol.split](raw, limit) {
    check(raw === 333 && limit === originalLimit && this === effectSplit, 'effect split Reference');
    shapeFlow.value = symbolResult; elementFlow[0] = callableResult; return 59;
  }};
  check((333).split(effectSplit, originalLimit) === 59 && typeof shapeFlow.value === 'symbol' &&
    typeof elementFlow[0] === 'function' && elementFlow[0]() === 43, 'split hook invalidates captured shape and element facts');
} finally {
  restore('match', matchDescriptor);
  restore('split', splitDescriptor);
}
check(Object.getOwnPropertyDescriptor(numberPrototype, 'match') === undefined ? matchDescriptor === undefined :
  Object.getOwnPropertyDescriptor(numberPrototype, 'match').value === matchDescriptor.value, 'match descriptor restored');
check(Object.getOwnPropertyDescriptor(numberPrototype, 'split') === undefined ? splitDescriptor === undefined :
  Object.getOwnPropertyDescriptor(numberPrototype, 'split').value === splitDescriptor.value, 'split descriptor restored');
print('number-string-hooks-references:ok');
262;
